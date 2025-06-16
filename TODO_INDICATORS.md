# TODO: Technical Indicators to Implement

## Pending Indicators

- [X] **Keltner Channels (KC)**

  - Volatility-based indicator using EMA and ATR
  - Upper/Lower bands around price action

- [X] **On-Balance Volume (OBV)**

  - Volume-based momentum indicator
  - Cumulative volume based on price direction

- [X] **Awesome Oscillator (AO)**

  - Momentum indicator using 5 and 34-period moving averages
  - Histogram showing momentum changes

- [X] **Williams %R**

  - Momentum oscillator (0 to -100 range)
  - Measures overbought/oversold conditions

- [ ] **Alligator**

  - Trend-following indicator with 3 smoothed moving averages
  - Jaw (13), Teeth (8), Lips (5) lines

- [ ] **Average Directional Index (ADX)**

  - Trend strength indicator
  - Includes +DI and -DI components

- [ ] **Parabolic SAR**

  - Stop and Reverse indicator
  - Trail stops that follow price trends

- [ ] **ZigZag**

  - Price action filter
  - Connects significant price swings

- [ ] **William's Fractals**

  - Support/resistance level identifier
  - 5-bar reversal pattern detector

- [ ] **Stochastic RSI**

  - RSI applied to Stochastic oscillator
  - Enhanced overbought/oversold signals

- [ ] **Normal Stochastic**

  - Classic %K and %D oscillator
  - Momentum indicator (0-100 range)

- [ ] **Standard Deviation**

  - Volatility measure
  - Statistical dispersion indicator

- [ ] **Spread**

  - Price difference indicator
  - Between different instruments/timeframes

- [ ] **Commodity Channel Index (CCI)**
  - Momentum oscillator
  - Identifies cyclical trends

## Price Action Patterns

- [ ] **Hammer**

  - Bullish reversal candlestick pattern
  - Small body with long lower shadow, little to no upper shadow
  - Forms at bottom of downtrend

- [ ] **Shooting Star**

  - Bearish reversal candlestick pattern
  - Small body with long upper shadow, little to no lower shadow
  - Forms at top of uptrend

- [ ] **Engulfing Candle (Bullish/Bearish)**

  - Bullish: Large green candle completely engulfs previous red candle
  - Bearish: Large red candle completely engulfs previous green candle
  - Strong reversal signal

- [ ] **Inside Bar**
  - Current bar's high/low is within previous bar's range
  - Indicates consolidation or indecision
  - Often precedes breakout moves

## Chart Patterns

- [ ] **Head and Shoulders**

  - Bearish reversal pattern with three peaks
  - Center peak (head) higher than two outer peaks (shoulders)
  - Neckline connects the two troughs

- [ ] **Inverse Head and Shoulders**

  - Bullish reversal pattern with three troughs
  - Center trough (head) lower than two outer troughs (shoulders)
  - Neckline connects the two peaks

- [ ] **Double Top**

  - Bearish reversal pattern
  - Two peaks at approximately same level
  - Valley between peaks confirms pattern

- [ ] **Double Bottom**

  - Bullish reversal pattern
  - Two troughs at approximately same level
  - Peak between troughs confirms pattern

- [ ] **Ascending Triangle**

  - Bullish continuation pattern
  - Horizontal resistance line with rising support line
  - Breakout typically upward

- [ ] **Descending Triangle**

  - Bearish continuation pattern
  - Horizontal support line with falling resistance line
  - Breakout typically downward

- [ ] **Symmetrical Triangle**
  - Continuation pattern (direction depends on prevailing trend)
  - Converging support and resistance lines
  - Breakout can be in either direction

## Implementation Priority

1. High Priority: OBV, Williams %R, Standard Deviation, Hammer, Shooting Star
2. Medium Priority: Keltner Channels, ADX, Stochastic indicators, Engulfing patterns
3. Low Priority: Alligator, ZigZag, William's Fractals, Chart patterns

## Notes

- Consider grouping similar indicators (momentum, volatility, trend)
- Pattern recognition may require multi-bar analysis
- Chart patterns need historical data validation
