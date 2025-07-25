/* auto-generated by NAPI-RS */
/* eslint-disable */
/**
 * Represents a financial candlestick with OHLCV (Open, High, Low, Close, Volume) data
 *
 * # Properties
 * * `price` - Current price or typical price
 * * `high` - Highest price during the period
 * * `low` - Lowest price during the period
 * * `open` - Opening price of the period
 * * `close` - Closing price of the period
 * * `volume` - Trading volume during the period
 */
export declare class Candle {
  price: number
  high: number
  low: number
  open: number
  close: number
  volume: number
  /**
   * Creates a new Candle instance with a single price value
   * All OHLC values will be set to the given price, and volume will be set to 0
   *
   * # Arguments
   * * `price` - The price value to use for all OHLC fields
   *
   * # Example
   * ```javascript
   * const candle = Candle.price(100);
   * // Creates a candle with:
   * // price: 100, high: 100, low: 100, open: 100, close: 100, volume: 0
   * ```
   */
  static price(price: number): Candle
  /**
   * Creates a new Candle instance with full OHLCV data
   *
   * # Arguments
   * * `price` - Current or typical price
   * * `high` - Highest price during the period
   * * `low` - Lowest price during the period
   * * `open` - Opening price of the period
   * * `close` - Closing price of the period
   * * `volume` - Trading volume during the period
   *
   * # Example
   * ```javascript
   * const candle = new Candle(100, 105, 95, 98, 102, 1000);
   * // Creates a candle with:
   * // price: 100 (typical price)
   * // high: 105 (highest price)
   * // low: 95 (lowest price)
   * // open: 98 (opening price)
   * // close: 102 (closing price)
   * // volume: 1000 (trading volume)
   * ```
   */
  constructor(price: number, high: number, low: number, open: number, close: number, volume: number)
  static fromString(json: unknown): Candle
  toJson(): unknown
}

/**
 * JavaScript bindings for various financial indicators.
 *
 * This implementation exposes a set of constructors and methods to create and use technical indicators
 * from JavaScript via NAPI. Supported indicators include:
 *
 * **Trend Following:**
 * - EMA (Exponential Moving Average)
 * - SMA (Simple Moving Average)
 * - SMMA (Smoothed Moving Average)
 * - Alligator (Three-line trend indicator)
 * - SuperTrend (Trend-following overlay)
 *
 * **Momentum & Oscillators:**
 * - RSI (Relative Strength Index)
 * - AO (Awesome Oscillator)
 * - STOCH (Stochastic Oscillator)
 * - Williams %R
 *
 * **Volatility:**
 * - ATR (Average True Range)
 * - TR (True Range)
 * - BB (Bollinger Bands)
 * - KC (Keltner Channel)
 * - SD (Standard Deviation)
 *
 * **Volume:**
 * - OBV (On-Balance Volume)
 *
 * **Other:**
 * - MACD (Moving Average Convergence Divergence)
 * - MAE (Mean Absolute Error)
 *
 * # Examples
 *
 * Creating an indicator:
 * ```javascript
 * const ema = Indicators.ema(14);
 * ```
 *
 * Serializing and restoring an indicator:
 * ```javascript
 * const json = indicator.toJson();
 * const restored = Indicators.fromString(json);
 * ```
 *
 * Calculating the next value:
 * ```javascript
 * const value = ema.next(100);
 * ```
 *
 * Calculating the next value using a candle:
 * ```javascript
 * const candle = new Candle(100, 105, 95, 98, 102, 1000);
 * const value = tr.nextCandle(candle);
 * ```
 *
 * Batched calculations:
 * ```javascript
 * const values = rsi.nextBatched([100, 101, 102]);
 * const candleValues = tr.nextCandles([candle1, candle2]);
 * ```
 *
 * # Methods
 * **Constructors:**
 * - `new()` - Creates a new empty indicator.
 * - `fromString(json)` - Restores an indicator from a JSON string.
 *
 * **Trend Indicators:**
 * - `ema(period)` - Creates an Exponential Moving Average indicator.
 * - `sma(period)` - Creates a Simple Moving Average indicator.
 * - `smma(period)` - Creates a Smoothed Moving Average indicator.
 * - `alligator(jaw_period, jaw_shift, teeth_period, teeth_shift, lips_period, lips_shift)` - Creates an Alligator indicator.
 * - `superTrend(multiplier, period)` - Creates a SuperTrend indicator.
 *
 * **Momentum Indicators:**
 * - `rsi(period)` - Creates a Relative Strength Index indicator.
 * - `ao(short_period, long_period)` - Creates an Awesome Oscillator indicator.
 * - `stoch(period, smoothing_period)` - Creates a Stochastic Oscillator indicator.
 * - `williamsR(period)` - Creates a Williams %R indicator.
 * - `macd(fast, slow, signal)` - Creates a MACD indicator.
 *
 * **Volatility Indicators:**
 * - `tr()` - Creates a True Range indicator.
 * - `atr(period)` - Creates an Average True Range indicator.
 * - `bb(period, k)` - Creates a Bollinger Bands indicator.
 * - `kc(period, multiplier)` - Creates a Keltner Channel indicator.
 * - `sd(period)` - Creates a Standard Deviation indicator.
 *
 * **Volume Indicators:**
 * - `obv()` - Creates an On-Balance Volume indicator.
 *
 * **Other Indicators:**
 * - `mae(period)` - Creates a Mean Absolute Error indicator.
 *
 * **Methods:**
 * - `toJson()` - Serializes the indicator to JSON.
 * - `next(input)` - Calculates the next value for a single input.
 * - `nextBatched(inputs)` - Calculates next values for an array of inputs.
 * - `nextCandle(candle)` - Calculates the next value using a candle.
 * - `nextCandles(candles)` - Calculates next values for an array of candles.
 *
 * All methods are available from JavaScript via the `Indicators` class.
 */
export declare class Indicator {
  /**
   * Creates a new empty Indicator instance
   *
   * # Example
   * ```javascript
   * const indicator = new Indicators();
   * ```
   */
  constructor()
  /**
   * Creates an Indicator instance from a JSON string
   *
   * # Arguments
   * * `json` - A JSON representation of an indicator
   *
   * # Example
   * ```javascript
   * const json = indicator.toJson();
   * const restored = Indicators.fromString(json);
   * ```
   */
  static fromString(json: unknown): Indicator
  /**
   * Creates an Exponential Moving Average (EMA) indicator
   *
   * # Arguments
   * * `period` - The period for the EMA calculation
   *
   * # Example
   * ```javascript
   * const ema = Indicators.ema(14);
   * ```
   */
  static ema(period: number): Indicator
  /**
   * Creates a Simple Moving Average (SMA) indicator
   *
   * # Arguments
   * * `period` - The period for the SMA calculation
   *
   * # Example
   * ```javascript
   * const sma = Indicators.sma(14);
   * ```
   */
  static sma(period: number): Indicator
  /**
   * Creates a Relative Strength Index (RSI) indicator
   *
   * # Arguments
   * * `period` - The period for the RSI calculation
   *
   * # Example
   * ```javascript
   * const rsi = Indicators.rsi(14);
   * ```
   */
  static rsi(period: number): Indicator
  /**
   * Creates a Moving Average Convergence Divergence (MACD) indicator
   *
   * # Arguments
   * * `fast_period` - The period for the fast EMA
   * * `slow_period` - The period for the slow EMA
   * * `signal_period` - The period for the signal line
   *
   * # Example
   * ```javascript
   * const macd = Indicators.macd(12, 26, 9);
   * ```
   */
  static macd(fastPeriod: number, slowPeriod: number, signalPeriod: number): Indicator
  /**
   * Creates a True Range (TR) indicator
   *
   * # Example
   * ```javascript
   * const tr = Indicators.tr();
   * ```
   */
  static tr(): Indicator
  /**
   * Creates an Average True Range (ATR) indicator
   *
   * # Arguments
   * * `period` - The period for the ATR calculation
   *
   * # Example
   * ```javascript
   * const atr = Indicators.atr(14);
   * ```
   */
  static atr(period: number): Indicator
  /**
   * Creates a SuperTrend indicator
   *
   * # Arguments
   * * `multiplier` - The multiplier for the ATR calculation
   * * `period` - The period for the ATR calculation
   *
   * # Example
   * ```javascript
   * const superTrend = Indicators.superTrend(3, 10);
   * ```
   */
  static superTrend(multiplier: number, period: number): Indicator
  /**
   * Creates an Alligator indicator with custom parameters
   *
   * # Arguments
   * * `jaw_period` - The period for the jaw line (commonly 13)
   * * `jaw_shift` - Forward shift for the jaw line (commonly 8)
   * * `teeth_period` - The period for the teeth line (commonly 8)
   * * `teeth_shift` - Forward shift for the teeth line (commonly 5)
   * * `lips_period` - The period for the lips line (commonly 5)
   * * `lips_shift` - Forward shift for the lips line (commonly 3)
   *
   * # Example
   * ```javascript
   * const alligator = Indicators.alligator(13, 8, 8, 5, 5, 3);
   * ```
   */
  static alligator(jawPeriod: number, jawShift: number, teethPeriod: number, teethShift: number, lipsPeriod: number, lipsShift: number): Indicator
  /**
   * Creates an Awesome Oscillator (AO) indicator
   *
   * # Arguments
   * * `short_period` - The period for the short SMA (commonly 5)
   * * `long_period` - The period for the long SMA (commonly 34)
   *
   * # Example
   * ```javascript
   * const ao = Indicators.ao(5, 34);
   * ```
   */
  static ao(shortPeriod: number, longPeriod: number): Indicator
  /**
   * Creates a Bollinger Bands (BB) indicator
   *
   * # Arguments
   * * `period` - The period for the moving average (commonly 20)
   * * `k` - The multiplier for the standard deviation (commonly 2.0)
   *
   * # Example
   * ```javascript
   * const bb = Indicators.bb(20, 2.0);
   * ```
   */
  static bb(period: number, k: number): Indicator
  /**
   * Creates a Keltner Channel (KC) indicator
   *
   * # Arguments
   * * `period` - The period for the EMA and ATR calculation (commonly 20)
   * * `multiplier` - The multiplier for the ATR (commonly 2.0)
   *
   * # Example
   * ```javascript
   * const kc = Indicators.kc(20, 2.0);
   * ```
   */
  static kc(period: number, multiplier: number): Indicator
  /**
   * Creates a Mean Absolute Error (MAE) indicator
   *
   * # Arguments
   * * `period` - The period for error calculation
   *
   * # Example
   * ```javascript
   * const mae = Indicators.mae(14);
   * ```
   */
  static mae(period: number): Indicator
  /**
   * Creates an On-Balance Volume (OBV) indicator
   *
   * # Example
   * ```javascript
   * const obv = Indicators.obv();
   * ```
   */
  static obv(): Indicator
  /**
   * Creates a Standard Deviation (SD) indicator
   *
   * # Arguments
   * * `period` - The period for standard deviation calculation (commonly 20)
   *
   * # Example
   * ```javascript
   * const sd = Indicators.sd(20);
   * ```
   */
  static sd(period: number): Indicator
  /**
   * Creates a Smoothed Moving Average (SMMA) indicator
   *
   * # Arguments
   * * `period` - The period for the SMMA calculation (commonly 14)
   *
   * # Example
   * ```javascript
   * const smma = Indicators.smma(14);
   * ```
   */
  static smma(period: number): Indicator
  /**
   * Creates a Stochastic Oscillator (STOCH) indicator
   *
   * # Arguments
   * * `period` - The period for %K calculation (commonly 14)
   * * `smoothing_period` - The period for %D smoothing (commonly 3)
   *
   * # Example
   * ```javascript
   * const stoch = Indicators.stoch(14, 3);
   * ```
   */
  static stoch(period: number, smoothingPeriod: number): Indicator
  /**
   * Creates a Williams %R (WILLR) indicator
   *
   * # Arguments
   * * `period` - The period for Williams %R calculation (commonly 14)
   *
   * # Example
   * ```javascript
   * const williamsR = Indicators.williamsR(14);
   * ```
   */
  static williamsR(period: number): Indicator
  /**
   * Converts the indicator to a JSON representation
   *
   * # Example
   * ```javascript
   * const indicator = Indicators.rsi(14);
   * const json = indicator.toJson();
   * ```
   */
  toJson(): unknown
  /**
   * Calculates the next value for a single input
   *
   * # Arguments
   * * `input` - The input value to process
   *
   * # Returns
   * A number or array of numbers depending on the indicator type
   *
   * # Example
   * ```javascript
   * const rsi = Indicators.rsi(14);
   * const value = rsi.next(100);
   * ```
   */
  next(input: number): unknown
  /**
   * Calculates the next values for an array of inputs
   *
   * # Arguments
   * * `input` - Array of input values to process
   *
   * # Returns
   * An array of results, one for each input value
   *
   * # Example
   * ```javascript
   * const rsi = Indicators.rsi(14);
   * const values = rsi.nextBatched([100, 101, 102]);
   * ```
   */
  nextBatched(input: Array<number>): Array<unknown>
  /**
   * Calculates the next value using a candle as input
   *
   * # Arguments
   * * `candle` - A candle object containing OHLCV data
   *
   * # Returns
   * A number or array of numbers depending on the indicator type
   *
   * # Example
   * ```javascript
   * const tr = Indicators.tr();
   * const candle = new Candle(100, 105, 95, 98, 102, 1000);
   * const value = tr.nextCandle(candle);
   * ```
   */
  nextCandle(candle: Candle): unknown
  /**
   * Calculates the next values using an array of candles as input
   *
   * # Arguments
   * * `candles` - Array of candle objects containing OHLCV data
   *
   * # Returns
   * An array of results, one for each candle
   *
   * # Example
   * ```javascript
   * const tr = Indicators.tr();
   * const candles = [
   *   new Candle(100, 105, 95, 98, 102, 1000),
   *   new Candle(102, 107, 97, 102, 105, 1200)
   * ];
   * const values = tr.nextCandles(candles);
   * ```
   */
  nextCandles(candles: Array<Candle>): Array<unknown>
}
