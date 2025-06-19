# Advanced Strategy V2: Multi-Indicator Confluence

This document outlines a sophisticated trading strategy that leverages a combination of trend-following, momentum, and oscillator indicators to generate high-probability trading signals. The core principle is **confluence**, meaning that a trade is only initiated when multiple, diverse indicators align to support the same directional bias.

## I. Strategy Overview

- **Strategy Type**: Trend-following and momentum-based.
- **Objective**: To capture significant market trends by entering on pullbacks or breakouts that are confirmed by multiple layers of analysis.
- **Timeframe**: Suitable for intermediate-term trading (e.g., 4-hour to daily charts), but adaptable to other timeframes.
- **Core Indicators**:
    - **Trend**: Exponential Moving Averages (EMA) - 50-period and 200-period.
    - **Momentum**: Relative Strength Index (RSI) - 14-period.
    - **Oscillator**: Awesome Oscillator (AO) - default periods (5, 34).

## II. Strategy Logic – Entry and Exit Conditions

### A. Long Entry Conditions (Strong Buy Signal)

All of the following conditions must be met simultaneously:

1.  **Primary Trend Confirmation (Bullish)**:
    - The current price is **above** the 200-period EMA, indicating a long-term uptrend.
    - The 50-period EMA is **above** the 200-period EMA (a "golden cross" formation), confirming sustained bullish momentum.

2.  **Momentum Confirmation (Bullish)**:
    - The RSI is **above 55**, signaling strong bullish momentum and that the asset is not yet overbought.

3.  **Oscillator Confirmation (Bullish)**:
    - The Awesome Oscillator (AO) is **above the zero line**, indicating that short-term momentum is greater than long-term momentum.

### B. Short Entry Conditions (Strong Sell Signal)

All of the following conditions must be met simultaneously:

1.  **Primary Trend Confirmation (Bearish)**:
    - The current price is **below** the 200-period EMA, indicating a long-term downtrend.
    - The 50-period EMA is **below** the 200-period EMA (a "death cross" formation), confirming sustained bearish momentum.

2.  **Momentum Confirmation (Bearish)**:
    - The RSI is **below 45**, signaling strong bearish momentum and that the asset is not yet oversold.

3.  **Oscillator Confirmation (Bearish)**:
    - The Awesome Oscillator (AO) is **below the zero line**, indicating that short-term momentum is weaker than long-term momentum.

### C. Exit Conditions

- **For a Long Position**: The position is held until the "Short Entry Conditions" are met, signaling a complete reversal of the market structure.
- **For a Short Position**: The position is held until the "Long Entry Conditions" are met.
- **No Signal**: If neither the long nor short conditions are met, the strategy remains in a **Hold** state, taking no new action.

## III. Implementation in `chipa-ta`

The strategy is implemented as a `StrategyNode` tree, which provides a clear and composable structure.

### Rust Code Representation:

```rust
use chipa_ta::{
    error::TaResult,
    strategy::{Action, Condition, MarketData, StrategyNode},
    types::OutputType,
    Indicator,
};

fn advanced_strategy_v2() -> TaResult<StrategyNode> {
    // --- INDICATORS ---
    // Trend
    let ema_long = Indicator::ema(200)?;
    let ema_short = Indicator::ema(50)?;

    // Momentum
    let rsi = Indicator::rsi(14)?;
    let ao = Indicator::ao(5, 34)?; // Default periods for Awesome Oscillator

    // --- LONG CONDITIONS (BUY) ---
    let long_conditions = Condition::And(vec![
        // 1. Major trend is up: Price is above the long-term moving average.
        Condition::LessThan {
            indicator: ema_long.clone(),
            value: OutputType::Close, // equivalent to Close > EMA(200)
        },
        // 2. Short-term trend is also up: Price is above the short-term moving average.
        Condition::LessThan {
            indicator: ema_short.clone(),
            value: OutputType::Close, // equivalent to Close > EMA(50)
        },
        // 3. Bullish momentum is confirmed: RSI is in the bullish zone.
        Condition::GreaterThan {
            indicator: rsi.clone(),
            value: OutputType::Single(55.0),
        },
        // 4. Awesome Oscillator confirms bullish momentum.
        Condition::GreaterThan {
            indicator: ao.clone(),
            value: OutputType::Single(0.0),
        },
    ]);

    // --- SHORT CONDITIONS (SELL) ---
    let short_conditions = Condition::And(vec![
        // 1. Major trend is down: Price is below the long-term moving average.
        Condition::GreaterThan {
            indicator: ema_long,
            value: OutputType::Close, // equivalent to Close < EMA(200)
        },
        // 2. Short-term trend is also down: Price is below the short-term moving average.
        Condition::GreaterThan {
            indicator: ema_short,
            value: OutputType::Close, // equivalent to Close < EMA(50)
        },
        // 3. Bearish momentum is confirmed: RSI is in the bearish zone.
        Condition::LessThan {
            indicator: rsi,
            value: OutputType::Single(45.0),
        },
        // 4. Awesome Oscillator confirms bearish momentum.
        Condition::LessThan {
            indicator: ao,
            value: OutputType::Single(0.0),
        },
    ]);

    // --- STRATEGY TREE ---
    let strategy = StrategyNode::If {
        condition: long_conditions,
        then_branch: Box::new(StrategyNode::Action(Action::StrongBuy)),
        else_branch: Some(Box::new(StrategyNode::If {
            condition: short_conditions,
            then_branch: Box::new(StrategyNode::Action(Action::StrongSell)),
            else_branch: Some(Box::new(StrategyNode::Action(Action::Hold))),
        })),
    };

    Ok(strategy)
}
```

This implementation defines a nested `If` structure. The outer `If` checks for the `long_conditions`. If they are met, it produces a `StrongBuy` action. If not, it proceeds to the `else_branch`, which contains another `If` node to check for the `short_conditions`. This ensures a clear, hierarchical evaluation of market conditions.
