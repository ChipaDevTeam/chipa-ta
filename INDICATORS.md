# Technical Analysis Indicators

This document provides a comprehensive list of all technical analysis indicators implemented in the chipa-ta library.

## Overview

The library provides a unified `Indicator` enum that wraps all individual indicator implementations, allowing for:

- Type-safe indicator usage
- Serialization/deserialization support
- Consistent API across all indicators
- Runtime indicator selection

## Implemented Indicators

### 1. None Indicator

**Type**: Utility  
**Purpose**: Pass-through indicator for testing and placeholder usage  
**Period**: 0  
**Output**: Single value (unchanged input)  
**Input**: Any numeric value or candle data

```rust
let indicator = Indicator::none();
```

---

### 2. Simple Moving Average (SMA)

**Type**: Trend Following  
**Purpose**: Calculates arithmetic mean of prices over specified period  
**Period**: User-defined (commonly 10, 20, 50, 200)  
**Output**: Single value  
**Input**: Numeric values or candle close prices

**Common Use Cases:**

- Trend identification
- Support/resistance levels
- Moving average crossovers

```rust
let sma = Indicator::sma(20)?; // 20-period SMA
```

---

### 3. Exponential Moving Average (EMA)

**Type**: Trend Following  
**Purpose**: Weighted average giving more importance to recent prices  
**Period**: User-defined (commonly 12, 26, 50, 200)  
**Output**: Single value  
**Input**: Numeric values or candle close prices

**Common Use Cases:**

- Responsive trend analysis
- MACD calculation
- Signal generation

```rust
let ema = Indicator::ema(14)?; // 14-period EMA
```

---

### 4. Relative Strength Index (RSI)

**Type**: Momentum Oscillator  
**Purpose**: Measures speed and change of price movements  
**Period**: User-defined (commonly 14)  
**Output**: Single value (0-100 range)  
**Input**: Numeric values or candle close prices

**Interpretation:**

- RSI > 70: Potentially overbought
- RSI < 30: Potentially oversold

```rust
let rsi = Indicator::rsi(14)?; // 14-period RSI
```

---

### 5. Moving Average Convergence Divergence (MACD)

**Type**: Momentum & Trend Following  
**Purpose**: Shows relationship between two moving averages  
**Parameters**: Fast period (12), Slow period (26), Signal period (9)  
**Output**: Array [MACD Line, Signal Line, Histogram]  
**Input**: Numeric values or candle close prices

**Components:**

- MACD Line: Fast EMA - Slow EMA
- Signal Line: EMA of MACD Line
- Histogram: MACD Line - Signal Line

```rust
let macd = Indicator::macd(12, 26, 9)?; // Standard MACD
```

---

### 6. True Range (TR)

**Type**: Volatility  
**Purpose**: Measures single-period volatility  
**Period**: 1 (current period only)  
**Output**: Single value  
**Input**: OHLC candle data required

**Calculation**: Maximum of:

- High - Low
- |High - Previous Close|
- |Low - Previous Close|

```rust
let tr = Indicator::tr(); // True Range
```

---

### 7. Average True Range (ATR)

**Type**: Volatility  
**Purpose**: Smoothed measure of volatility over time  
**Period**: User-defined (commonly 14)  
**Output**: Single value  
**Input**: OHLC candle data required

**Common Use Cases:**

- Position sizing
- Stop-loss placement
- Volatility analysis

```rust
let atr = Indicator::atr(14)?; // 14-period ATR
```

---

### 8. SuperTrend

**Type**: Trend Following Overlay  
**Purpose**: Dynamic support/resistance based on ATR  
**Parameters**: Multiplier (2.0-3.0), ATR period (10-14)  
**Output**: Array [SuperTrend Value, Trend Direction]  
**Input**: OHLC candle data required

**Common Use Cases:**

- Trend identification
- Entry/exit signals
- Dynamic stop-loss

```rust
let supertrend = Indicator::super_trend(3.0, 10)?; // SuperTrend(3.0, 10)
```

---

### 9. Bollinger Bands (BB)

**Type**: Volatility & Mean Reversion  
**Purpose**: Volatility bands around moving average  
**Parameters**: Period (20), Standard deviation multiplier (2.0)  
**Output**: Array [Middle Band, Upper Band, Lower Band]  
**Input**: Numeric values or candle close prices

**Components:**

- Middle Band: Simple Moving Average
- Upper Band: SMA + (StdDev × Multiplier)
- Lower Band: SMA - (StdDev × Multiplier)

```rust
let bb = Indicator::bb(20, 2.0)?; // Bollinger Bands(20, 2.0)
```

---

### 10. Stochastic Oscillator (STOCH)

**Type**: Momentum Oscillator  
**Purpose**: Compares closing price to price range  
**Parameters**: Period (14), Smoothing period (3)  
**Output**: Array [%K Value, %D Value]  
**Input**: OHLC candle data required

**Formula**: %K = ((Close - Lowest Low) / (Highest High - Lowest Low)) × 100

**Interpretation:**

- %K, %D > 80: Potentially overbought
- %K, %D < 20: Potentially oversold

```rust
let stoch = Indicator::stoch(14, 3)?; // Stochastic(14, 3)
```

---

### 11. Standard Deviation (SD)

**Type**: Statistical/Volatility  
**Purpose**: Measures price dispersion from average  
**Period**: User-defined (commonly 20)  
**Output**: Single value  
**Input**: Numeric values or candle close prices

**Common Use Cases:**

- Volatility measurement
- Bollinger Bands calculation
- Risk assessment

```rust
let sd = Indicator::sd(20)?; // 20-period Standard Deviation
```

---

### 12. Mean Absolute Error (MAE)

**Type**: Statistical/Error Measurement  
**Purpose**: Measures average prediction error magnitude  
**Period**: User-defined window for error calculation  
**Output**: Single value  
**Input**: Numeric values

**Common Use Cases:**

- Model validation
- Indicator accuracy assessment
- Performance measurement

```rust
let mae = Indicator::mae(10)?; // 10-period MAE
```

## Usage Patterns

### Basic Usage

```rust
// Create an indicator
let mut rsi = Indicator::rsi(14)?;

// Process single values
let result = rsi.next(100.0)?;

// Reset indicator state
rsi.reset();
```

### With Candle Data

```rust
let mut atr = Indicator::atr(14)?;

// Process candle data
let candle = SomeCandle {
    open: 100.0,
    high: 105.0,
    low: 95.0,
    close: 102.0,
    volume: 1000.0
};
let result = atr.next(&candle)?;
```

### Serialization

```rust
// Serialize to JSON
let json = serde_json::to_string(&indicator)?;

// Deserialize from JSON
let restored: Indicator = serde_json::from_str(&json)?;
```

## Output Types

Indicators return different output types based on their nature:

- **Single Value**: SMA, EMA, RSI, ATR, TR, SD, MAE
- **Array Values**:
  - MACD: [MACD Line, Signal Line, Histogram]
  - Bollinger Bands: [Middle, Upper, Lower]
  - SuperTrend: [Value, Direction]
  - Stochastic: [%K, %D]

## Input Requirements

### Numeric Input Only

- SMA, EMA, RSI, MACD, SD, MAE, None

### OHLC Candle Data Required

- TR, ATR, SuperTrend, Stochastic

### Either Numeric or Candle

- Bollinger Bands (uses close price from candles)

## Common Periods

| Indicator       | Common Periods  | Notes                                         |
| --------------- | --------------- | --------------------------------------------- |
| SMA/EMA         | 10, 20, 50, 200 | Longer periods = smoother, less responsive    |
| RSI             | 14              | Standard period, some use 9 or 21             |
| MACD            | 12, 26, 9       | Fast, Slow, Signal - widely accepted standard |
| ATR             | 14              | Standard period for volatility measurement    |
| Bollinger Bands | 20, 2.0         | Period and standard deviation multiplier      |
| Stochastic      | 14, 3           | %K period and %D smoothing                    |

## Performance Considerations

- All indicators maintain minimal state and are optimized for real-time processing
- Serialization allows for state persistence across application restarts
- Reset functionality enables indicator reuse with different data sets
- Memory usage is proportional to the indicator's period length
