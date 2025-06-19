# Advanced Confluence Trading Strategy

This document outlines a complex, multi-indicator confluence trading strategy designed to generate nuanced trading signals. The strategy integrates trend, momentum, and overbought/oversold indicators to make decisions, categorizing them into `StrongBuy`, `Buy`, `Sell`, `StrongSell`, and `Hold` actions.

## Core Components

The strategy is built upon the following technical indicators:

1.  **Simple Moving Averages (SMA)**:

    - `SMA(50)`: A medium-term trend indicator.
    - `SMA(200)`: A long-term trend indicator.
      The relationship between the current price and these SMAs helps determine the overall trend direction.

2.  **Relative Strength Index (RSI)**:

    - `RSI(14)`: A momentum oscillator used to measure the speed and change of price movements. It helps identify bullish or bearish momentum.

3.  **Williams %R**:
    - `Williams %R(14)`: An oscillator that identifies overbought and oversold levels.

## Strategy Logic

The strategy is divided into two main logical blocks: Buy-side and Sell-side.

### Buy Logic

The goal of the buy-side logic is to identify opportunities to enter a long position.

#### Base Buy Conditions (`Action::Buy`)

A `Buy` signal is generated when all of the following conditions are met, indicating a stable uptrend with good momentum:

1.  **Price > SMA(50)**: The price is above the medium-term trend.
2.  **Price > SMA(200)**: The price is above the long-term trend.
3.  **RSI(14) > 55**: Bullish momentum is present.
4.  **Williams %R(14) < -20**: The asset is not in overbought territory, suggesting there is still room for upward movement.

#### Strong Buy Condition (`Action::StrongBuy`)

A `Buy` signal is upgraded to a `StrongBuy` if, in addition to the base conditions, the following is true:

1.  **RSI(14) > 65**: Indicates very strong bullish momentum.

### Sell Logic

The sell-side logic mirrors the buy-side, aiming to identify opportunities to enter a short position or exit a long one.

#### Base Sell Conditions (`Action::Sell`)

A `Sell` signal is generated when all of the following conditions are met, indicating a clear downtrend:

1.  **Price < SMA(50)**: The price is below the medium-term trend.
2.  **Price < SMA(200)**: The price is below the long-term trend.
3.  **RSI(14) < 45**: Bearish momentum is present.
4.  **Williams %R(14) > -80**: The asset is not in oversold territory, suggesting further downside is possible.

#### Strong Sell Condition (`Action::StrongSell`)

A `Sell` signal is upgraded to a `StrongSell` if, in addition to the base sell conditions, the following is true:

1.  **RSI(14) < 35**: Indicates very strong bearish momentum.

### Final Strategy Assembly

The buy and sell logic are combined into a final strategy using a `Sequence` node with `SequenceMode::First`. This means the strategy evaluates the `buy_strategy` first. If it results in a `Buy` or `StrongBuy`, that action is taken. If not, it proceeds to the `sell_strategy`. If neither the buy nor sell conditions are met, the default action is `Hold`.

## Rust Implementation

Here is the Rust code that implements the strategy within a test function.

```rust
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
    let strong_buy_condition = Condition::GreaterThan {
        indicator: Indicator::rsi(14)?,
        value: OutputType::Single(65.0),
    };

    // Base Buy conditions:
    // 1. Price is above the medium-term trend (SMA50).
    // 2. Price is above the long-term trend (SMA200).
    // 3. Momentum is bullish (RSI > 55).
    // 4. Not in overbought territory (Williams %R < -20).
    let buy_conditions = Condition::And(vec![
        // Using LessThan because we want to check: indicator < value, which is SMA(50) < Close
        Condition::LessThan {
            indicator: Indicator::sma(50)?,
            value: OutputType::Close,
        },
        Condition::LessThan {
            indicator: Indicator::sma(200)?,
            value: OutputType::Close,
        },
        Condition::GreaterThan {
            indicator: Indicator::rsi(14)?,
            value: OutputType::Single(55.0),
        },
        Condition::LessThan {
            indicator: Indicator::williams_r(14)?,
            value: OutputType::Single(-20.0),
        },
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
    let strong_sell_condition = Condition::LessThan {
        indicator: Indicator::rsi(14)?,
        value: OutputType::Single(35.0),
    };

    // Base Sell conditions:
    // 1. Price is below the medium-term trend (SMA50).
    // 2. Price is below the long-term trend (SMA200).
    // 3. Momentum is bearish (RSI < 45).
    // 4. Not in oversold territory (Williams %R > -80).
    let sell_conditions = Condition::And(vec![
        // Using GreaterThan because we want to check: indicator > value, which is SMA(50) > Close
        Condition::GreaterThan {
            indicator: Indicator::sma(50)?,
            value: OutputType::Close,
        },
        Condition::GreaterThan {
            indicator: Indicator::sma(200)?,
            value: OutputType::Close,
        },
        Condition::LessThan {
            indicator: Indicator::rsi(14)?,
            value: OutputType::Single(45.0),
        },
        Condition::GreaterThan {
            indicator: Indicator::williams_r(14)?,
            value: OutputType::Single(-80.0),
        },
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
    assert!(strategy.validate().is_ok());

    // Print the strategy's structure as JSON for inspection.
    let json = serde_json::to_string_pretty(&strategy)?;
    dbg!(json);

    // We can't easily test the outcome without a full market data series,
    // but we can validate its construction and serialization.
    Ok(())
}
```
