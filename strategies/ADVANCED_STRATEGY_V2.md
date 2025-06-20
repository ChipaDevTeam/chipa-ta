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
  - **MACD**, **Bollinger Bands**, **Keltner Channel**, **SuperTrend**.

## II. Strategy Logic – Entry and Exit Conditions

### A. Long Entry Conditions (Buy Signal)

All of the following conditions must be met simultaneously:

1. **Major trend is up**: Price is above the long-term EMA (Close > EMA(200)).
2. **Short-term trend is up**: Price is above the short-term EMA (Close > EMA(50)).
3. **Bullish momentum**: RSI > 55.
4. **AO confirms bullishness**: AO > 0.
5. **MACD confirms bullishness**: MACD line > Signal line (see note below).
6. **Price is near the lower Bollinger Band**: Close < BB lower band (see note below).
7. **SuperTrend is bullish**: Close > SuperTrend value.

### B. Short Entry Conditions (Sell Signal)

All of the following conditions must be met simultaneously:

1. **Major trend is down**: Price is below the long-term EMA (Close < EMA(200)).
2. **Short-term trend is down**: Price is below the short-term EMA (Close < EMA(50)).
3. **Bearish momentum**: RSI < 45.
4. **AO confirms bearishness**: AO < 0.
5. **MACD confirms bearishness**: MACD line < Signal line (see note below).
6. **Price is near the upper Bollinger Band**: Close > BB upper band (see note below).
7. **SuperTrend is bearish**: Close < SuperTrend value.
8. **Price is near the Keltner Channel upper band**: Close > KC upper band (see note below).

**Note:**

- For indicators with multiple outputs (e.g., MACD, BB, KC), the actual implementation uses `OutputType::Custom` to select the correct output index or combination for comparison.
- The code below reflects the actual output shapes and how the strategy is implemented in Rust.

### C. Exit Conditions

- **For a Long Position**: The position is held until the "Short Entry Conditions" are met.
- **For a Short Position**: The position is held until the "Long Entry Conditions" are met.
- **No Signal**: If neither the long nor short conditions are met, the strategy remains in a **Hold** state.

## III. Strategy Implementation

Below is the Rust code for the strategy, matching the implementation in `test_advanced_confluence_strategy_v2`:

```rust
use chipa_ta::{
    error::TaResult,
    strategy::{Action, Condition, MarketData, StrategyNode},
    types::OutputType,
    Indicator,
};

fn advanced_strategy_v2() -> TaResult<StrategyNode> {
    // --- INDICATORS ---
    let ema_long = Indicator::ema(200)?;
    let ema_short = Indicator::ema(50)?;
    let rsi = Indicator::rsi(14)?;
    let ao = Indicator::ao(5, 34)?;
    let macd = Indicator::macd(12, 26, 9)?;
    let bb = Indicator::bb(20, 2.0)?;
    let kc = Indicator::kc(20, 2.0)?;
    let super_trend = Indicator::super_trend(3.0, 10)?;

    // --- LONG CONDITIONS (BUY) ---
    let long_conditions = Condition::And(vec![
        // 1. Major trend is up: Price is above the long-term EMA.
        Condition::LessThan {
            indicator: ema_long.clone(),
            value: OutputType::Close,
        },
        // 2. Short-term trend is up: Price is above the short-term EMA.
        Condition::LessThan {
            indicator: ema_short.clone(),
            value: OutputType::Close,
        },
        // 3. Bullish momentum: RSI > 55.
        Condition::GreaterThan {
            indicator: rsi.clone(),
            value: OutputType::Single(55.0),
        },
        // 4. AO > 0.
        Condition::GreaterThan {
            indicator: ao.clone(),
            value: OutputType::Single(0.0),
        },
        // 5. MACD line > Signal line.
        Condition::GreaterThan {
            indicator: macd.clone(),
            value: OutputType::Custom(vec![
                OutputType::Single(0.0),
                OutputType::Static(Statics::True),
                OutputType::Static(Statics::True),
            ]), // Custom output shape for MACD
        },
        // 6. Close < BB lower band.
        Condition::LessThan {
            indicator: bb.clone(),
            value: OutputType::Custom(vec![
                OutputType::Static(Statics::True),
                OutputType::Static(Statics::True),
                OutputType::Close,
            ]), // Custom output shape for BB
        },
        // 7. SuperTrend is bullish: Close > SuperTrend value.
        Condition::LessThan {
            indicator: super_trend.clone(),
            value: OutputType::Custom(vec![
                OutputType::Close,
                OutputType::Close,
            ]), // Custom output shape for SuperTrend
        },
    ]);

    // --- SHORT CONDITIONS (SELL) ---
    let short_conditions = Condition::And(vec![
        // 1. Major trend is down: Price is below the long-term EMA.
        Condition::GreaterThan {
            indicator: ema_long,
            value: OutputType::Close,
        },
        // 2. Short-term trend is down: Price is below the short-term EMA.
        Condition::GreaterThan {
            indicator: ema_short,
            value: OutputType::Close,
        },
        // 3. Bearish momentum: RSI < 45.
        Condition::LessThan {
            indicator: rsi,
            value: OutputType::Single(45.0),
        },
        // 4. AO < 0.
        Condition::LessThan {
            indicator: ao,
            value: OutputType::Single(0.0),
        },
        // 5. MACD line < Signal line.
        Condition::LessThan {
            indicator: macd,
            value: OutputType::Array(vec![0.0, 0.0, 0.0]), // Custom output shape for MACD
        },
        // 6. Close > BB upper band.
        Condition::GreaterThan {
            indicator: bb,
            value: OutputType::Custom(vec![
                OutputType::Static(Statics::True),
                OutputType::Close,
                OutputType::Static(Statics::True),
            ]), // Custom output shape for BB
        },
        // 7. SuperTrend is bearish: Close < SuperTrend value.
        Condition::GreaterThan {
            indicator: super_trend,
            value: OutputType::Custom(vec![
                OutputType::Close,
                OutputType::Close,
            ]), // Custom output shape for SuperTrend
        },
        // 8. Close > KC upper band.
        Condition::GreaterThan {
            indicator: kc,
            value: OutputType::Custom(vec![
                OutputType::Close,
                OutputType::Static(Statics::True),
                OutputType::Static(Statics::True),
            ]), // Custom output shape for KC
        },
    ]);

    // --- STRATEGY TREE ---
    let strategy = StrategyNode::If {
        condition: long_conditions,
        then_branch: Box::new(StrategyNode::Action(Action::Buy)),
        else_branch: Some(Box::new(StrategyNode::If {
            condition: short_conditions,
            then_branch: Box::new(StrategyNode::Action(Action::Sell)),
            else_branch: Some(Box::new(StrategyNode::Action(Action::Hold))),
        })),
    };

    Ok(strategy)
}

#[test]
fn test_advanced_confluence_strategy_v2() -> TaResult<()> {
    // --- INDICATORS ---
    let ema_long = Indicator::ema(200)?;
    let ema_short = Indicator::ema(50)?;
    let rsi = Indicator::rsi(14)?;
    let ao = Indicator::ao(5, 34)?;
    let macd = Indicator::macd(12, 26, 9)?;
    let bb = Indicator::bb(20, 2.0)?;
    let kc = Indicator::kc(20, 2.0)?;
    let super_trend = Indicator::super_trend(3.0, 10)?;

    // --- LONG CONDITIONS ---
    let long_conditions = Condition::And(vec![
        Condition::LessThan { indicator: ema_long.clone(), value: OutputType::Close },
        Condition::LessThan { indicator: ema_short.clone(), value: OutputType::Close },
        Condition::GreaterThan { indicator: rsi.clone(), value: OutputType::Single(55.0) },
        Condition::GreaterThan { indicator: ao.clone(), value: OutputType::Single(0.0) },
        Condition::GreaterThan { indicator: macd.clone(), value: OutputType::Single(0.0) },
        Condition::LessThan { indicator: bb.clone(), value: OutputType::Close },
        Condition::LessThan { indicator: super_trend.clone(), value: OutputType::Close },
    ]);

    // --- SHORT CONDITIONS ---
    let short_conditions = Condition::And(vec![
        Condition::GreaterThan { indicator: ema_long, value: OutputType::Close },
        Condition::GreaterThan { indicator: ema_short, value: OutputType::Close },
        Condition::LessThan { indicator: rsi, value: OutputType::Single(45.0) },
        Condition::LessThan { indicator: ao, value: OutputType::Single(0.0) },
        Condition::LessThan { indicator: macd, value: OutputType::Single(0.0) },
        Condition::GreaterThan { indicator: bb, value: OutputType::Close },
        Condition::GreaterThan { indicator: super_trend, value: OutputType::Close },
        Condition::GreaterThan { indicator: kc, value: OutputType::Close },
    ]);

    let strategy = StrategyNode::If {
        condition: long_conditions,
        then_branch: Box::new(StrategyNode::Action(Action::Buy)),
        else_branch: Some(Box::new(StrategyNode::If {
            condition: short_conditions,
            then_branch: Box::new(StrategyNode::Action(Action::Sell)),
            else_branch: Some(Box::new(StrategyNode::Action(Action::Hold))),
        })),
    };

    let json = serde_json::to_string(&strategy)?;
    dbg!(json);

    Ok(())
}
```

**Key Implementation Notes:**

- For multi-output indicators (MACD, BB, KC, SuperTrend), use `OutputType::Custom` or `OutputType::Array` to select the correct output for comparison.
- The strategy is a nested `If` tree: first checks long conditions, then short, else Hold.
- This matches the actual test and implementation in your Rust codebase.
