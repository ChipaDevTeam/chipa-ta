use crate::error::TaResult;
use crate::preprocessing::PreprocessingStep;
use crate::strategy::error::StrategyError;
use crate::strategy::{Action, Condition, MarketData};
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
#[derive(Serialize, Deserialize, Debug, Clone)]
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
        }
    }

    /// Validates that every execution path in the strategy ends with an Action.
    /// Returns Ok(()) if valid, or Err(String) describing the first violation.
    pub fn validate(&self) -> Result<(), StrategyError> {
        match self {
            StrategyNode::Preprocess(_) => Ok(()),
            StrategyNode::Action(_) => Ok(()),
            StrategyNode::If {
                then_branch,
                else_branch,
                ..
            } => {
                // Then branch must be valid
                then_branch.validate()?;
                // Else branch must exist and be valid
                if let Some(else_node) = else_branch {
                    else_node.validate()?;
                } else {
                    return Err(StrategyError::MissingElseBranch);
                }
                Ok(())
            }
            StrategyNode::Sequence { nodes, .. } => {
                if nodes.is_empty() {
                    return Err(StrategyError::EmptySequence);
                }
                for node in nodes {
                    node.validate()?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::OutputType;
    use crate::Indicator;
    use serde_json;

    #[test]
    fn test_rsi_strategy_action() {
        // RSI default period 14: first call returns 50 > 40 => Sell
        let mut strategy = StrategyNode::If {
            condition: Condition::GreaterThan {
                indicator: Indicator::rsi(14).unwrap(),
                value: OutputType::from(40.0),
            },
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
        let cond1 = Condition::GreaterThan {
            indicator: Indicator::rsi(10).unwrap(),
            value: OutputType::from(0.0),
        };
        let cond2 = Condition::LessThan {
            indicator: Indicator::rsi(20).unwrap(),
            value: OutputType::from(0.0),
        };
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
            condition: Condition::GreaterThan {
                indicator: Indicator::rsi(5).unwrap(),
                value: crate::types::OutputType::from(0.0),
            },
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
            condition: Condition::LessThan {
                indicator: Indicator::rsi(3).unwrap(),
                value: crate::types::OutputType::from(0.0),
            },
            then_branch: Box::new(StrategyNode::Action(Action::Hold)),
            else_branch: None,
        };
        let err = node.validate().unwrap_err();
        assert_eq!(err, StrategyError::MissingElseBranch);
    }

    #[test]
    fn test_validate_empty_sequence() {
        // Sequence with no nodes should error
        let seq = StrategyNode::Sequence {
            mode: SequenceMode::Any,
            nodes: vec![],
        };
        let err = seq.validate().unwrap_err();
        assert_eq!(err, StrategyError::EmptySequence);
    }
}
