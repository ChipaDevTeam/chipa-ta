use crate::error::{TaError, TaResult};
use crate::preprocessing::PreprocessingStep;
use crate::strategy::error::StrategyError;
use crate::strategy::{Action, Condition, MarketData};
use crate::traits::{Period, Reset};
use serde::{Deserialize, Serialize};

/// Aggregation modes for `Sequence` nodes.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SequenceMode {
    /// Return the first action-producing node's result (default behavior).
    First,
    /// Return the first non-Hold action encountered.
    Any,
    /// Return an action only if all action-producing nodes agree; otherwise Hold.
    All,
    /// Return the action with the highest occurrence; on tie defaults to Hold.
    Majority,
    /// Return an action if at least the specified percentage of nodes agree; otherwise Hold.
    Percentage(u8), // 0-100 representing percentage threshold
}

/// AST node for composable trading strategies.
///
/// StrategyNode represents the minimal schema for building trading strategies:
/// - Preprocess: apply data transformations.
/// - If: conditional branching based on market data.
/// - Action: buy/sell/hold signals.
/// - Sequence: chain multiple nodes in order.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum StrategyNode {
    /// Apply a preprocessing step to the market data.
    Preprocess(PreprocessingStep),

    /// Conditional node: if `condition` is true, execute `then_branch`,
    /// otherwise execute `else_branch` if present.
    If {
        /// Condition to evaluate.
        condition: Condition,
        /// Node to execute when condition is true.
        then_branch: Box<StrategyNode>,
        /// Optional node to execute when condition is false.
        else_branch: Option<Box<StrategyNode>>,
    },

    Timeout {
        cooldown: usize,           // Cooldown period in candles
        remaining: usize,          // Remaining cooldown candles
        action: Box<StrategyNode>, // Action to execute after cooldown
    },

    /// Action node: produce a trading action (Buy, Sell, Hold).
    Action(Action),

    /// Sequence of strategy nodes executed under an aggregation mode.
    Sequence {
        mode: SequenceMode,
        nodes: Vec<StrategyNode>,
    },
}

impl StrategyNode {
    /// Evaluate the strategy node against market data, returning a trading `Action`.
    pub fn evaluate(&mut self, data: &mut MarketData) -> TaResult<Action> {
        match self {
            StrategyNode::Preprocess(step) => {
                // Apply preprocessing step, then return Hold by default.
                step.apply(data);
                Ok(Action::Hold)
            }
            StrategyNode::If {
                condition,
                then_branch,
                else_branch,
            } => {
                // Evaluate condition; on true, evaluate then_branch, else else_branch or Hold.
                if condition.evaluate(data)? {
                    then_branch.evaluate(data)
                } else if let Some(else_node) = else_branch {
                    else_node.evaluate(data)
                } else {
                    Ok(Action::Hold)
                }
            }
            StrategyNode::Action(action) => Ok(*action),
            StrategyNode::Sequence { mode, nodes } => {
                // Collect non-Hold actions from sub-nodes, respecting mode.
                let mut actions = Vec::new();
                for node in nodes {
                    let res = node.evaluate(data)?;
                    if res != Action::Hold {
                        actions.push(res);
                        if mode == &SequenceMode::First || mode == &SequenceMode::Any {
                            break;
                        }
                    }
                }
                // Aggregate actions per mode
                let chosen = match mode {
                    SequenceMode::First | SequenceMode::Any => {
                        actions.into_iter().next().unwrap_or(Action::Hold)
                    }
                    SequenceMode::All => {
                        if let Some(first) = actions.first() {
                            if actions.iter().all(|a| a == first) {
                                *first
                            } else {
                                Action::Hold
                            }
                        } else {
                            Action::Hold
                        }
                    }
                    SequenceMode::Majority => {
                        use std::collections::HashMap;
                        let mut counts: HashMap<Action, usize> = HashMap::new();
                        for a in &actions {
                            *counts.entry(*a).or_insert(0) += 1;
                        }
                        counts
                            .into_iter()
                            .max_by_key(|&(_, c)| c)
                            .map(|(a, _)| a)
                            .unwrap_or(Action::Hold)
                    }
                    SequenceMode::Percentage(percentage) => {
                        use std::collections::HashMap;
                        let mut counts: HashMap<Action, usize> = HashMap::new();
                        for a in &actions {
                            *counts.entry(*a).or_insert(0) += 1;
                        }
                        let total = actions.len();
                        counts
                            .into_iter()
                            .filter_map(|(a, c)| {
                                if c * 100 / total >= *percentage as usize {
                                    Some(a)
                                } else {
                                    None
                                }
                            })
                            .next()
                            .unwrap_or(Action::Hold)
                    }
                };
                Ok(chosen)
            }
            StrategyNode::Timeout {
                cooldown,
                remaining,
                action,
            } => {
                if *remaining > 0 {
                    *remaining -= 1; // Decrement cooldown
                    Ok(Action::Hold) // Still in cooldown
                } else {
                    // Execute action after cooldown
                    let result = action.evaluate(data)?;
                    if result != Action::Hold {
                        *remaining = *cooldown; // Reset cooldown
                    }
                    Ok(result)
                }
            }
        }
    }

    /// Returns the maximum indicator period required by this strategy tree, or `None` if no indicators.
    pub fn max_period(&self) -> Option<usize> {
        match self {
            StrategyNode::Preprocess(_) => None,
            StrategyNode::Action(_) => None,
            StrategyNode::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let mut periods = Vec::new();
                if let Some(p) = condition.max_period() {
                    periods.push(p);
                }
                if let Some(p) = then_branch.max_period() {
                    periods.push(p);
                }
                if let Some(node) = else_branch {
                    if let Some(p) = node.max_period() {
                        periods.push(p);
                    }
                }
                periods.into_iter().max()
            }
            StrategyNode::Sequence { nodes, .. } => {
                nodes.iter().filter_map(|n| n.max_period()).max()
            }
            StrategyNode::Timeout { action, .. } => action.max_period(),
        }
    }

    /// Validates that every execution path in the strategy ends with an Action.
    /// Returns Ok(()) if valid, or Err(String) describing the first violation.
    pub fn validate(&self) -> Result<(), TaError> {
        match self {
            StrategyNode::Preprocess(_) => Ok(()),
            StrategyNode::Action(_) => Ok(()),
            StrategyNode::If {
                then_branch,
                else_branch,
                condition,
            } => {
                condition.validate()?;
                // Then branch must be valid
                then_branch.validate()?;
                // Else branch must exist and be valid
                if let Some(else_node) = else_branch {
                    else_node.validate()?;
                } else {
                    return Err(TaError::from(StrategyError::MissingElseBranch));
                }
                Ok(())
            }
            StrategyNode::Sequence { nodes, .. } => {
                if nodes.is_empty() {
                    return Err(TaError::from(StrategyError::EmptySequence));
                }
                for node in nodes {
                    node.validate()?;
                }
                Ok(())
            }
            StrategyNode::Timeout { action, .. } => {
                // Action must be valid
                action.validate()?;
                Ok(())
            }
        }
    }
}

impl Period for StrategyNode {
    /// Returns the maximum period of any contained indicators.
    fn period(&self) -> usize {
        self.max_period().unwrap_or(0)
    }
}

impl Reset for StrategyNode {
    fn reset(&mut self) {
        match self {
            StrategyNode::Preprocess(step) => step.reset(),
            StrategyNode::If {
                then_branch,
                else_branch,
                condition,
            } => {
                then_branch.reset();
                if let Some(else_node) = else_branch {
                    else_node.reset();
                }
                condition.reset();
            }
            StrategyNode::Action(_) => {}
            StrategyNode::Sequence { nodes, .. } => {
                for node in nodes {
                    node.reset();
                }
            }
            StrategyNode::Timeout {
                action, remaining, ..
            } => {
                action.reset();
                *remaining = 0; // Reset cooldown
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helper_types::Bar;
    use crate::types::{OutputType, Statics};
    use crate::Indicator;
    use serde_json;

    #[test]
    fn test_rsi_strategy_action() {
        // RSI default period 14: first call returns 50 > 40 => Sell
        let mut strategy = StrategyNode::If {
            condition: Condition::greater_than(Indicator::rsi(14).unwrap(), OutputType::from(40.0)),
            then_branch: Box::new(StrategyNode::Action(Action::Sell)),
            else_branch: Some(Box::new(StrategyNode::Action(Action::Hold))),
        };
        let mut data = MarketData::Float(100.0);
        let action = strategy.evaluate(&mut data).unwrap();
        assert_eq!(action, Action::Sell);
    }

    #[test]
    fn test_serde_strategynode() {
        // External tagging: enum variant name as key
        let node = StrategyNode::Action(Action::Buy);
        let s = serde_json::to_string(&node).unwrap();
        assert_eq!(s, r#"{"Action":"Buy"}"#);
        let de: StrategyNode = serde_json::from_str(&s).unwrap();
        // We can only compare structure by de-serializing a new Action
        if let StrategyNode::Action(a) = de {
            assert_eq!(a, Action::Buy);
        } else {
            panic!("Deserialized to wrong variant");
        }
    }

    #[test]
    fn test_max_period_composed() {
        // Compose two If nodes with different RSI periods 10 and 20
        let cond1 = Condition::greater_than(Indicator::rsi(10).unwrap(), OutputType::from(0.0));
        let cond2 = Condition::less_than(Indicator::rsi(20).unwrap(), OutputType::from(0.0));
        let node1 = StrategyNode::If {
            condition: cond1,
            then_branch: Box::new(StrategyNode::Action(Action::Hold)),
            else_branch: None,
        };
        let node2 = StrategyNode::If {
            condition: cond2,
            then_branch: Box::new(StrategyNode::Action(Action::Hold)),
            else_branch: None,
        };
        let seq = StrategyNode::Sequence {
            mode: SequenceMode::All,
            nodes: vec![node1, node2],
        };
        assert_eq!(seq.max_period(), Some(20));
        dbg!(&seq);
        dbg!(serde_json::to_string(&seq).unwrap());
    }

    #[test]
    fn test_validate_valid_strategy() {
        // Simple action node is always valid
        let node = StrategyNode::Action(Action::Buy);
        assert!(node.validate().is_ok());

        // If with both branches
        let node = StrategyNode::If {
            condition: Condition::greater_than(Indicator::rsi(5).unwrap(), OutputType::from(0.0)),
            then_branch: Box::new(StrategyNode::Action(Action::Sell)),
            else_branch: Some(Box::new(StrategyNode::Action(Action::Hold))),
        };
        assert!(node.validate().is_ok());

        // Sequence of two actions
        let seq = StrategyNode::Sequence {
            mode: SequenceMode::First,
            nodes: vec![
                StrategyNode::Action(Action::Buy),
                StrategyNode::Action(Action::Sell),
            ],
        };
        assert!(seq.validate().is_ok());
    }

    #[test]
    fn test_validate_missing_else_branch() {
        // If without else should error
        let node = StrategyNode::If {
            condition: Condition::less_than(Indicator::rsi(3).unwrap(), OutputType::from(0.0)),
            then_branch: Box::new(StrategyNode::Action(Action::Hold)),
            else_branch: None,
        };
        let err = node.validate().unwrap_err();
        assert_eq!(err, TaError::from(StrategyError::MissingElseBranch));
    }

    #[test]
    fn test_validate_empty_sequence() {
        // Sequence with no nodes should error
        let seq = StrategyNode::Sequence {
            mode: SequenceMode::Any,
            nodes: vec![],
        };
        let err = seq.validate().unwrap_err();
        assert_eq!(err, TaError::from(StrategyError::EmptySequence));
    }

    #[test]
    fn test_multi_indicator_confluence_strategy() -> TaResult<()> {
        fn multi_indicator_confluence_strategy() -> TaResult<StrategyNode> {
            // --- Long Entry ---
            let long_entry = StrategyNode::If {
                condition: Condition::And(vec![
                    // RSI crosses above 30
                    Condition::cross_over(Indicator::rsi(14)?, OutputType::from(30.0)),
                    // Close > SMA(50)
                    Condition::greater_than(Indicator::sma(50)?, OutputType::Close),
                    // NOTE: Close < Previous Close not supported in Condition
                    // NOTE: No open position not supported in Condition
                ]),
                then_branch: Box::new(StrategyNode::Action(Action::Buy)),
                else_branch: Some(Box::new(StrategyNode::Action(Action::Hold))),
            };

            // --- Short Entry ---
            let short_entry = StrategyNode::If {
                condition: Condition::And(vec![
                    // RSI crosses below 70
                    Condition::cross_under(Indicator::rsi(14)?, OutputType::from(70.0)),
                    // Close < SMA(50)
                    Condition::less_than(Indicator::sma(50)?, OutputType::Close),
                    // NOTE: Close > Previous Close not supported in Condition
                    // NOTE: No open position not supported in Condition
                ]),
                then_branch: Box::new(StrategyNode::Action(Action::Sell)),
                else_branch: Some(Box::new(StrategyNode::Action(Action::Hold))),
            };

            // --- Combine Long and Short Entries ---
            Ok(StrategyNode::Sequence {
                mode: crate::strategy::node::SequenceMode::First,
                nodes: vec![long_entry, short_entry],
            })
        }
        let mut strategy = multi_indicator_confluence_strategy()?;
        assert!(strategy.validate().is_ok());
        dbg!(strategy.max_period());
        let mut data = MarketData::Float(100.0);
        let action = strategy.evaluate(&mut data)?;
        dbg!(action);
        dbg!(serde_json::to_string(&strategy)?);
        Ok(())
    }

    #[test]
    fn test_load_multi_indicator_confluence_strategy() -> TaResult<()> {
        // Load the strategy from JSON
        let json = r#"
            {
        "Sequence": {
            "mode": "First",
            "nodes": [
            {
                "If": {
                "condition": {
                    "And": [
                    {
                        "CrossOver": {
                        "indicator": { "type": "Rsi", "period": 14 },
                        "value": { "Single": 30.0 }
                        }
                    },
                    {
                        "GreaterThan": {
                        "indicator": { "type": "Sma", "period": 50 },
                        "value": "Close"
                        }
                    }
                    ]
                },
                "then_branch": { "Action": "Buy" },
                "else_branch": { "Action": "Hold" }
                }
            },
            {
                "If": {
                "condition": {
                    "And": [
                    {
                        "CrossUnder": {
                        "indicator": { "type": "Rsi", "period": 14 },
                        "value": { "Single": 70.0 }
                        }
                    },
                    {
                        "LessThan": {
                        "indicator": { "type": "Sma", "period": 50 },
                        "value": "Close"
                        }
                    }
                    ]
                },
                "then_branch": { "Action": "Sell" },
                "else_branch": { "Action": "Hold" }
                }
            }
            ]
        }
        }
        "#;

        let mut strategy: StrategyNode = serde_json::from_str(json)?;
        assert!(strategy.validate().is_ok());
        dbg!(strategy.max_period());
        let mut data = MarketData::Float(100.0);
        let action = strategy.evaluate(&mut data)?;
        dbg!(action);
        dbg!(strategy);
        Ok(())
    }

    #[test]
    fn test_advanced_confluence_strategy() -> TaResult<()> {
        // This strategy combines multiple indicators to generate trading signals with
        // varying levels of conviction (Buy/StrongBuy, Sell/StrongSell).
        //
        // It uses:
        // - SMA(200) and SMA(50) for trend direction.
        // - RSI(14) for momentum.
        // - Williams %R(14) for overbought/oversold conditions.

        // ************************************************************************
        //                          BUY LOGIC
        // ************************************************************************

        // Strong Buy condition: RSI is showing very strong momentum.
        let strong_buy_condition = Condition::greater_than(Indicator::rsi(14)?, OutputType::Single(65.0));

        // Base Buy conditions:
        // 1. Price is above the medium-term trend (SMA50).
        // 2. Price is above the long-term trend (SMA200).
        // 3. Momentum is bullish (RSI > 55).
        // 4. Not in overbought territory (Williams %R < -20).
        let buy_conditions = Condition::And(vec![
            // Using LessThan because we want to check: indicator < value, which is SMA(50) < Close
            Condition::less_than(Indicator::sma(50)?, OutputType::Close),
            Condition::less_than(Indicator::sma(200)?, OutputType::Close),
            Condition::greater_than(Indicator::rsi(14)?, OutputType::Single(55.0)),
            Condition::less_than(Indicator::williams_r(14)?, OutputType::Single(-20.0)),
        ]);

        // Buy-side strategy tree:
        // If base conditions are met, check for strong buy conditions.
        let buy_strategy = StrategyNode::If {
            condition: buy_conditions,
            then_branch: Box::new(StrategyNode::If {
                condition: strong_buy_condition,
                then_branch: Box::new(StrategyNode::Action(Action::StrongBuy)),
                else_branch: Some(Box::new(StrategyNode::Action(Action::Buy))),
            }),
            else_branch: Some(Box::new(StrategyNode::Action(Action::Hold))),
        };

        // ************************************************************************
        //                          SELL LOGIC
        // ************************************************************************

        // Strong Sell condition: RSI is showing very strong bearish momentum.
        let strong_sell_condition = Condition::less_than(Indicator::rsi(14)?, OutputType::Single(35.0));

        // Base Sell conditions:
        // 1. Price is below the medium-term trend (SMA50).
        // 2. Price is below the long-term trend (SMA200).
        // 3. Momentum is bearish (RSI < 45).
        // 4. Not in oversold territory (Williams %R > -80).
        let sell_conditions = Condition::And(vec![
            // Using GreaterThan because we want to check: indicator > value, which is SMA(50) > Close
            Condition::greater_than(Indicator::sma(50)?, OutputType::Close),
            Condition::greater_than(Indicator::sma(200)?, OutputType::Close),
            Condition::less_than(Indicator::rsi(14)?, OutputType::Single(45.0)),
            Condition::greater_than(Indicator::williams_r(14)?, OutputType::Single(-80.0)),
        ]);

        // Sell-side strategy tree:
        // If base conditions are met, check for strong sell conditions.
        let sell_strategy = StrategyNode::If {
            condition: sell_conditions,
            then_branch: Box::new(StrategyNode::If {
                condition: strong_sell_condition,
                then_branch: Box::new(StrategyNode::Action(Action::StrongSell)),
                else_branch: Some(Box::new(StrategyNode::Action(Action::Sell))),
            }),
            else_branch: Some(Box::new(StrategyNode::Action(Action::Hold))),
        };

        // ************************************************************************
        //                          FINAL STRATEGY
        // ************************************************************************

        // The final strategy is a sequence that evaluates the buy and sell logic.
        // `SequenceMode::First` ensures that the first non-Hold action is returned.
        let strategy = StrategyNode::Sequence {
            mode: SequenceMode::First,
            nodes: vec![
                buy_strategy,
                sell_strategy,
                // Fallback action if no other conditions are met.
                StrategyNode::Action(Action::Hold),
            ],
        };

        // Validate that the strategy is well-formed (e.g., all paths lead to an action).
        strategy.validate().unwrap();

        // Print the strategy's structure as JSON for inspection.
        let json = serde_json::to_string(&strategy)?;
        dbg!(json);

        // We can't easily test the outcome without a full market data series,
        // but we can validate its construction and serialization.
        Ok(())
    }

    #[test]
    fn test_advanced_confluence_strategy_v2() -> TaResult<()> {
        // This strategy is a complex confluence model using trend, momentum, and oscillator indicators.
        // It aims to generate high-conviction signals by requiring agreement across multiple timeframes and indicator types.

        // --- INDICATORS ---
        // Trend
        let ema_long = Indicator::ema(200)?;
        let ema_short = Indicator::ema(50)?;

        // Momentum
        let rsi = Indicator::rsi(14)?;
        let ao = Indicator::ao(5, 34)?; // Default periods for Awesome Oscillator
        let macd = Indicator::macd(12, 26, 9)?; // MACD with standard periods

        // Volatility
        let bb = Indicator::bb(20, 2.0)?; // Bollinger Bands with standard settings
        let kc = Indicator::kc(20, 2.0)?; // Keltner Channel with standard settings

        // SuperTrend
        let super_trend = Indicator::super_trend(3.0, 10)?;

        // --- LONG CONDITIONS (BUY) ---
        let long_conditions = Condition::And(vec![
            // 1. Major trend is up: Price is above the long-term moving average.
            Condition::less_than(ema_long.clone(), OutputType::Close),
            // 2. Short-term trend is also up: Price is above the short-term moving average.
            Condition::less_than(ema_short.clone(), OutputType::Close),
            // 3. Bullish momentum is confirmed: RSI is in the bullish zone.
            Condition::greater_than(rsi.clone(), OutputType::Single(55.0)),
            // 4. Awesome Oscillator confirms bullish momentum.
            Condition::greater_than(ao.clone(), OutputType::Single(0.0)),
            // 5. MACD line is above its signal line.
            Condition::greater_than(macd.clone(), OutputType::Custom(vec![
                OutputType::Single(0.0),
                OutputType::Static(Statics::True),
                OutputType::Static(Statics::True),
            ])), // equivalent to MACD Line > Signal Line
            // 6. Price is near the lower Bollinger Band.
            Condition::less_than(bb.clone(), OutputType::Custom(vec![
                OutputType::Static(Statics::True),
                OutputType::Static(Statics::True),
                OutputType::Close,
            ])), // equivalent to Close < BB Lower Band
            // 7. SuperTrend is bullish.
            Condition::less_than(super_trend.clone(), OutputType::Custom(vec![OutputType::Close, OutputType::Close])), // equivalent to Close > SuperTrend
        ]);

        // --- SHORT CONDITIONS (SELL) ---
        let short_conditions = Condition::And(vec![
            // 1. Major trend is down: Price is below the long-term moving average.
            Condition::greater_than(ema_long, OutputType::Close), // equivalent to Close < EMA(200)
            // 2. Short-term trend is also down: Price is below the short-term moving average.
            Condition::greater_than(ema_short, OutputType::Close), // equivalent to Close < EMA(50)
            // 3. Bearish momentum is confirmed: RSI is in the bearish zone.
            Condition::less_than(rsi, OutputType::Single(45.0)),
            // 4. Awesome Oscillator confirms bearish momentum.
            Condition::less_than(ao, OutputType::Single(0.0)),
            // 5. MACD line is below its signal line.
            Condition::less_than(macd, OutputType::Array(vec![0.0, 0.0, 0.0])),
            // 6. Price is near the upper Bollinger Band.
            Condition::greater_than(bb, OutputType::Custom(vec![
                    OutputType::Static(Statics::True),
                    OutputType::Close,
                    OutputType::Static(Statics::True),
                ])), // equivalent to Close > BB Upper Band
            // 7. SuperTrend is bearish.
            Condition::greater_than(super_trend, OutputType::Custom(vec![OutputType::Close, OutputType::Close])), // equivalent to Close < SuperTrend
            // 8. Price is near the Keltner Channel upper band.
            Condition::greater_than(kc, OutputType::Custom(vec![
                OutputType::Close,
                OutputType::Static(Statics::True),
                OutputType::Static(Statics::True),
            ])), // equivalent to Close > KC Upper Band
        ]);

        // --- STRATEGY TREE ---
        let mut strategy = StrategyNode::If {
            condition: long_conditions,
            then_branch: Box::new(StrategyNode::Action(Action::Buy)),
            else_branch: Some(Box::new(StrategyNode::If {
                condition: short_conditions,
                then_branch: Box::new(StrategyNode::Action(Action::Sell)),
                else_branch: Some(Box::new(StrategyNode::Action(Action::Hold))),
            })),
        };

        // --- VALIDATION & EXECUTION ---
        strategy.validate().unwrap();

        // We can print the strategy structure and its required period.
        println!(
            "Advanced Strategy V2 JSON: {}",
            serde_json::to_string_pretty(&strategy)?
        );


        println!(
            "Advanced Strategy V2 Max Period: {:?}",
            strategy.max_period()
        );

        // To properly test this, we would need a series of market data points.
        // For this test, we'll just run one evaluation with a sample data point.
        let mut data = MarketData::Bar(Bar {
            open: 100.0,
            high: 105.0,
            low: 98.0,
            close: 102.0,
            price: 102.0,
            volume: 1000.0,
        }); // O, H, L, C, V

        // Note: The first `period` evaluations will not be reliable as indicators warm up.
        // A full test would involve iterating over a historical dataset.
        let action = strategy.evaluate(&mut data)?;
        println!("Action for first data point: {:?}", action);

        // A simple assertion that it runs without error.
        // The actual action depends on the warm-up state of the indicators.
        assert!(matches!(
            action,
            Action::StrongBuy | Action::StrongSell | Action::Hold
        ));

        let files = [
            ("tests/formats/advanced_confluence_strategy_v2.json", serde_json::to_string_pretty(&strategy)?.into_bytes()),
            ("tests/formats/advanced_confluence_strategy_v2.msgpack", rmp_serde::to_vec(&strategy).unwrap()),
            ("tests/formats/advanced_confluence_strategy_v2.ron", ron::to_string(&strategy).unwrap().into_bytes()),
            ("tests/formats/advanced_confluence_strategy_v2.yaml", serde_yaml::to_string(&strategy).unwrap().into_bytes()),
            ("tests/formats/advanced_confluence_strategy_v2.toml", toml::to_string(&strategy).unwrap().into_bytes()),
            // ("tests/formats/advanced_confluence_strategy_v2.xml", quick_xml::se::to_string(&strategy).unwrap().into_bytes()),
            ("tests/formats/advanced_confluence_strategy_v2.cbor", serde_cbor::to_vec(&strategy).unwrap()),
            ("tests/formats/advanced_confluence_strategy_v2.pickle", serde_pickle::to_vec(&strategy, serde_pickle::SerOptions::new().proto_v2()).unwrap()),
            // ("tests/formats/advanced_confluence_strategy_v2.starlark", serde_starlark::to_string(&strategy).unwrap().into_bytes()),
            // ("tests/formats/advanced_confluence_strategy_v2.msg", serde_rosmsg::to_vec(&strategy).unwrap()),
        ];
        for (path, content) in files {
            std::fs::write(path, content)
                .unwrap_or_else(|_| panic!("Failed to write strategy to {}", path));
        }
        Ok(())
    }

    #[test]
    fn test_multi_indicator_crossover_strategy() -> TaResult<()> {
        // This strategy uses RSI crossovers, Awesome Oscillator, and an EMA to generate signals.

        // --- INDICATORS ---
        let rsi = Indicator::rsi(14)?;
        let ao = Indicator::ao(5, 34)?;
        let ema = Indicator::ema(50)?;

        // --- LONG CONDITIONS (BUY) ---
        let long_conditions = Condition::And(vec![
            // 1. RSI crosses above the oversold level (30).
            Condition::cross_over(rsi.clone(), OutputType::Single(30.0)),
            // 2. Awesome Oscillator is positive, indicating bullish momentum.
            Condition::greater_than(ao.clone(), OutputType::Single(0.0)),
            // 3. Price is above the 50-period EMA, confirming an uptrend.
            Condition::less_than(ema.clone(), OutputType::Close),
        ]);

        // --- SHORT CONDITIONS (SELL) ---
        let short_conditions = Condition::And(vec![
            // 1. RSI crosses below the overbought level (70).
            Condition::cross_under(rsi, OutputType::Single(70.0)),
            // 2. Awesome Oscillator is negative, indicating bearish momentum.
            Condition::less_than(ao, OutputType::Single(0.0)),
            // 3. Price is below the 50-period EMA, confirming a downtrend.
            Condition::greater_than(ema, OutputType::Close),
        ]);

        // --- STRATEGY TREE ---
        let mut strategy = StrategyNode::If {
            condition: long_conditions,
            then_branch: Box::new(StrategyNode::Action(Action::StrongBuy)),
            else_branch: Some(Box::new(StrategyNode::If {
                condition: short_conditions,
                then_branch: Box::new(StrategyNode::Action(Action::StrongSell)),
                else_branch: Some(Box::new(StrategyNode::Action(Action::Hold))),
            })),
        };

        // --- VALIDATION & EXECUTION ---
        assert!(strategy.validate().is_ok());

        println!(
            "Strategy JSON: {}",
            serde_json::to_string_pretty(&strategy)?
        );

        // This test requires multiple data points to test crossovers.
        let bar = Bar {
            open: 100.0,
            price: 102.0,
            high: 105.0,
            low: 98.0,
            close: 102.0,
            volume: 1000.0,
        };
        let mut data = MarketData::Bar(bar);
        // First run, no crossover yet.
        let action1 = strategy.evaluate(&mut data)?;
        println!("Action 1: {:?}", action1);
        assert_eq!(action1, Action::Hold); // Should be Hold as crossover cannot happen on first tick.

        // To properly test, we need a sequence of data that triggers a crossover.
        // This is complex to set up in a unit test without a data feed.
        // The main purpose here is to ensure the strategy structure is valid and runs.

        Ok(())
    }

    #[test]
    fn test_supertrend_rsi_ema_strategy() -> TaResult<()> {
        // --- INDICATORS ---
        let super_trend = Indicator::super_trend(3.0, 10)?;
        let rsi = Indicator::rsi(14)?;
        let ema_long = Indicator::ema(200)?;

        // --- LONG CONDITIONS ---
        let long_conditions = Condition::And(vec![
            // 1. SuperTrend is bullish (Close > SuperTrend value)
            Condition::less_than(super_trend.clone(), OutputType::Close),
            // 2. RSI confirms bullish momentum
            Condition::greater_than(rsi.clone(), OutputType::Single(50.0)),
            // 3. Long-term trend is bullish
            Condition::less_than(ema_long.clone(), OutputType::Close),
        ]);

        // --- SHORT CONDITIONS ---
        let short_conditions = Condition::And(vec![
            // 1. SuperTrend is bearish (Close < SuperTrend value)
            Condition::greater_than(super_trend, OutputType::Close),
            // 2. RSI confirms bearish momentum
            Condition::less_than(rsi, OutputType::Single(50.0)),
            // 3. Long-term trend is bearish
            Condition::greater_than(ema_long, OutputType::Close),
        ]);

        // --- STRATEGY TREE ---
        let mut strategy = StrategyNode::If {
            condition: long_conditions,
            then_branch: Box::new(StrategyNode::Action(Action::Buy)),
            else_branch: Some(Box::new(StrategyNode::If {
                condition: short_conditions,
                then_branch: Box::new(StrategyNode::Action(Action::Sell)),
                else_branch: Some(Box::new(StrategyNode::Action(Action::Hold))),
            })),
        };

        // --- VALIDATION & EXECUTION ---
        assert!(strategy.validate().is_ok());
        let bar = Bar {
            open: 100.0,
            price: 102.0,
            high: 105.0,
            low: 98.0,
            close: 102.0,
            volume: 1000.0,
        };
        let mut data = MarketData::Bar(bar);
        let action = strategy.evaluate(&mut data)?;

        // Just a basic check that it runs.
        assert!(matches!(action, Action::Buy | Action::Sell | Action::Hold));

        Ok(())
    }
}
