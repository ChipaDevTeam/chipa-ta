pub mod alligator;
pub mod ao;
pub mod atr;
pub mod bb;
pub mod custom;
pub mod ema;
pub mod indicator;
#[cfg(test)]
pub mod integration_test;
pub mod kc;
pub mod macd;
pub mod mae;
pub mod obv;
pub mod rsi;
pub mod sd;
pub mod sma;
pub mod smma;
pub mod stoch;
pub mod super_trend;
pub mod tr;
pub mod williams_r;
// #[cfg(feature="js")]
pub use atr::AverageTrueRange;
pub use bb::BollingerBands;
pub use ema::ExponentialMovingAverage;
pub use macd::MovingAverageConvergenceDivergence;
pub use mae::MeanAbsoluteError;
pub use rsi::RelativeStrengthIndex;
pub use sd::StandardDeviation;
pub use sma::SimpleMovingAverage;
pub use stoch::StochasticOscillator;
pub use super_trend::SuperTrend;
pub use tr::TrueRange;

pub use custom::CustomIndicator;
pub use serde::{Deserialize, Serialize};
#[cfg(feature = "js")]
pub mod js {
    use crate::{
        indicators::indicator::NoneIndicator, traits::Candle as CandleRs, traits::Next,
        types::OutputType,
    };

    use super::{indicator::Indicator as IndicatorRs, *};
    use napi::bindgen_prelude::*;
    use napi_derive::napi;
    /// Represents a financial candlestick with OHLCV (Open, High, Low, Close, Volume) data
    ///
    /// # Properties
    /// * `price` - Current price or typical price
    /// * `high` - Highest price during the period
    /// * `low` - Lowest price during the period
    /// * `open` - Opening price of the period
    /// * `close` - Closing price of the period
    /// * `volume` - Trading volume during the period
    #[napi]
    #[derive(Clone, Serialize, Deserialize, Debug)]
    pub struct Candle {
        pub price: f64,
        pub high: f64,
        pub low: f64,
        pub open: f64,
        pub close: f64,
        pub volume: f64,
    }

    impl CandleRs for Candle {
        /// Returns the closing price of the candle
        fn close(&self) -> f64 {
            self.close
        }

        /// Returns the highest price of the candle
        fn high(&self) -> f64 {
            self.high
        }

        /// Returns the lowest price of the candle
        fn low(&self) -> f64 {
            self.low
        }

        /// Returns the opening price of the candle
        fn open(&self) -> f64 {
            self.open
        }

        /// Returns the current or typical price of the candle
        fn price(&self) -> f64 {
            self.price
        }

        /// Returns the trading volume of the candle
        fn volume(&self) -> f64 {
            self.volume
        }
    }

    #[napi]
    impl Candle {
        /// Creates a new Candle instance with a single price value
        /// All OHLC values will be set to the given price, and volume will be set to 0
        ///
        /// # Arguments
        /// * `price` - The price value to use for all OHLC fields
        ///
        /// # Example
        /// ```javascript
        /// const candle = Candle.price(100);
        /// // Creates a candle with:
        /// // price: 100, high: 100, low: 100, open: 100, close: 100, volume: 0
        /// ```
        #[napi(factory)]
        pub fn price(price: f64) -> Self {
            Self {
                volume: 0.0,
                open: price,
                close: price,
                high: price,
                low: price,
                price,
            }
        }

        /// Creates a new Candle instance with full OHLCV data
        ///
        /// # Arguments
        /// * `price` - Current or typical price
        /// * `high` - Highest price during the period
        /// * `low` - Lowest price during the period
        /// * `open` - Opening price of the period
        /// * `close` - Closing price of the period
        /// * `volume` - Trading volume during the period
        ///
        /// # Example
        /// ```javascript
        /// const candle = new Candle(100, 105, 95, 98, 102, 1000);
        /// // Creates a candle with:
        /// // price: 100 (typical price)
        /// // high: 105 (highest price)
        /// // low: 95 (lowest price)
        /// // open: 98 (opening price)
        /// // close: 102 (closing price)
        /// // volume: 1000 (trading volume)
        /// ```
        #[napi(constructor)]
        pub fn new(price: f64, high: f64, low: f64, open: f64, close: f64, volume: f64) -> Self {
            Self {
                price,
                high,
                low,
                open,
                close,
                volume,
            }
        }

        #[napi(factory)]
        pub fn from_string(json: Unknown, env: Env) -> napi::Result<Self> {
            let candle = env.from_js_value(json)?;
            Ok(candle)
        }

        #[napi]
        pub fn to_json(&self, env: Env) -> napi::Result<Unknown> {
            env.to_js_value(&self)
        }
    }

    /// JavaScript bindings for various financial indicators.
    ///
    /// This implementation exposes a set of constructors and methods to create and use technical indicators
    /// from JavaScript via NAPI. Supported indicators include:
    ///
    /// **Trend Following:**
    /// - EMA (Exponential Moving Average)
    /// - SMA (Simple Moving Average)
    /// - SMMA (Smoothed Moving Average)
    /// - Alligator (Three-line trend indicator)
    /// - SuperTrend (Trend-following overlay)
    ///
    /// **Momentum & Oscillators:**
    /// - RSI (Relative Strength Index)
    /// - AO (Awesome Oscillator)
    /// - STOCH (Stochastic Oscillator)
    /// - Williams %R
    ///
    /// **Volatility:**
    /// - ATR (Average True Range)
    /// - TR (True Range)
    /// - BB (Bollinger Bands)
    /// - KC (Keltner Channel)
    /// - SD (Standard Deviation)
    ///
    /// **Volume:**
    /// - OBV (On-Balance Volume)
    ///
    /// **Other:**
    /// - MACD (Moving Average Convergence Divergence)
    /// - MAE (Mean Absolute Error)
    ///
    /// # Examples
    ///
    /// Creating an indicator:
    /// ```javascript
    /// const ema = Indicators.ema(14);
    /// ```
    ///
    /// Serializing and restoring an indicator:
    /// ```javascript
    /// const json = indicator.toJson();
    /// const restored = Indicators.fromString(json);
    /// ```
    ///
    /// Calculating the next value:
    /// ```javascript
    /// const value = ema.next(100);
    /// ```
    ///
    /// Calculating the next value using a candle:
    /// ```javascript
    /// const candle = new Candle(100, 105, 95, 98, 102, 1000);
    /// const value = tr.nextCandle(candle);
    /// ```
    ///
    /// Batched calculations:
    /// ```javascript
    /// const values = rsi.nextBatched([100, 101, 102]);
    /// const candleValues = tr.nextCandles([candle1, candle2]);
    /// ```
    ///
    /// # Methods
    /// **Constructors:**
    /// - `new()` - Creates a new empty indicator.
    /// - `fromString(json)` - Restores an indicator from a JSON string.
    ///
    /// **Trend Indicators:**
    /// - `ema(period)` - Creates an Exponential Moving Average indicator.
    /// - `sma(period)` - Creates a Simple Moving Average indicator.
    /// - `smma(period)` - Creates a Smoothed Moving Average indicator.
    /// - `alligator(jaw_period, jaw_shift, teeth_period, teeth_shift, lips_period, lips_shift)` - Creates an Alligator indicator.
    /// - `superTrend(multiplier, period)` - Creates a SuperTrend indicator.
    ///
    /// **Momentum Indicators:**
    /// - `rsi(period)` - Creates a Relative Strength Index indicator.
    /// - `ao(short_period, long_period)` - Creates an Awesome Oscillator indicator.
    /// - `stoch(period, smoothing_period)` - Creates a Stochastic Oscillator indicator.
    /// - `williamsR(period)` - Creates a Williams %R indicator.
    /// - `macd(fast, slow, signal)` - Creates a MACD indicator.
    ///
    /// **Volatility Indicators:**
    /// - `tr()` - Creates a True Range indicator.
    /// - `atr(period)` - Creates an Average True Range indicator.
    /// - `bb(period, k)` - Creates a Bollinger Bands indicator.
    /// - `kc(period, multiplier)` - Creates a Keltner Channel indicator.
    /// - `sd(period)` - Creates a Standard Deviation indicator.
    ///
    /// **Volume Indicators:**
    /// - `obv()` - Creates an On-Balance Volume indicator.
    ///
    /// **Other Indicators:**
    /// - `mae(period)` - Creates a Mean Absolute Error indicator.
    ///
    /// **Methods:**
    /// - `toJson()` - Serializes the indicator to JSON.
    /// - `next(input)` - Calculates the next value for a single input.
    /// - `nextBatched(inputs)` - Calculates next values for an array of inputs.
    /// - `nextCandle(candle)` - Calculates the next value using a candle.
    /// - `nextCandles(candles)` - Calculates next values for an array of candles.
    ///
    /// All methods are available from JavaScript via the `Indicators` class.
    #[napi]
    #[derive(Clone)]
    pub struct Indicator {
        inner: IndicatorRs,
    }

    impl Serialize for Indicator {
        fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.inner.serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for Indicator {
        fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let inner = IndicatorRs::deserialize(deserializer)?;
            Ok(Self { inner })
        }
    }

    impl Default for Indicator {
        fn default() -> Self {
            Self {
                inner: IndicatorRs::None(NoneIndicator {}),
            }
        }
    }

    #[napi]
    impl Indicator {
        /// Creates a new empty Indicator instance
        ///
        /// # Example
        /// ```javascript
        /// const indicator = new Indicators();
        /// ```
        #[napi(constructor)]
        pub fn new() -> Self {
            Self::default()
        }

        /// Creates an Indicator instance from a JSON string
        ///
        /// # Arguments
        /// * `json` - A JSON representation of an indicator
        ///
        /// # Example
        /// ```javascript
        /// const json = indicator.toJson();
        /// const restored = Indicators.fromString(json);
        /// ```
        #[napi(factory)]
        pub fn from_string(json: Unknown, env: Env) -> napi::Result<Self> {
            let inner: IndicatorRs = env.from_js_value(json)?;
            Ok(Self { inner })
        }

        /// Creates an Exponential Moving Average (EMA) indicator
        ///
        /// # Arguments
        /// * `period` - The period for the EMA calculation
        ///
        /// # Example
        /// ```javascript
        /// const ema = Indicators.ema(14);
        /// ```
        #[napi(factory)]
        pub fn ema(period: u32) -> napi::Result<Self> {
            let inner = IndicatorRs::ema(period as usize)?;
            Ok(Self { inner })
        }

        /// Creates a Simple Moving Average (SMA) indicator
        ///
        /// # Arguments
        /// * `period` - The period for the SMA calculation
        ///
        /// # Example
        /// ```javascript
        /// const sma = Indicators.sma(14);
        /// ```
        #[napi(factory)]
        pub fn sma(period: u32) -> napi::Result<Self> {
            let inner = IndicatorRs::sma(period as usize)?;
            Ok(Self { inner })
        }

        /// Creates a Relative Strength Index (RSI) indicator
        ///
        /// # Arguments
        /// * `period` - The period for the RSI calculation
        ///
        /// # Example
        /// ```javascript
        /// const rsi = Indicators.rsi(14);
        /// ```
        #[napi(factory)]
        pub fn rsi(period: u32) -> napi::Result<Self> {
            let inner = IndicatorRs::rsi(period as usize)?;
            Ok(Self { inner })
        }

        /// Creates a Moving Average Convergence Divergence (MACD) indicator
        ///
        /// # Arguments
        /// * `fast_period` - The period for the fast EMA
        /// * `slow_period` - The period for the slow EMA
        /// * `signal_period` - The period for the signal line
        ///
        /// # Example
        /// ```javascript
        /// const macd = Indicators.macd(12, 26, 9);
        /// ```
        #[napi(factory)]
        pub fn macd(fast_period: u32, slow_period: u32, signal_period: u32) -> napi::Result<Self> {
            let inner = IndicatorRs::macd(
                fast_period as usize,
                slow_period as usize,
                signal_period as usize,
            )?;
            Ok(Self { inner })
        }

        /// Creates a True Range (TR) indicator
        ///
        /// # Example
        /// ```javascript
        /// const tr = Indicators.tr();
        /// ```
        #[napi(factory)]
        pub fn tr() -> Self {
            let inner = IndicatorRs::tr();
            Self { inner }
        }

        /// Creates an Average True Range (ATR) indicator
        ///
        /// # Arguments
        /// * `period` - The period for the ATR calculation
        ///
        /// # Example
        /// ```javascript
        /// const atr = Indicators.atr(14);
        /// ```
        #[napi(factory)]
        pub fn atr(period: u32) -> napi::Result<Self> {
            let inner = IndicatorRs::atr(period as usize)?;
            Ok(Self { inner })
        }

        /// Creates a SuperTrend indicator
        ///
        /// # Arguments
        /// * `multiplier` - The multiplier for the ATR calculation
        /// * `period` - The period for the ATR calculation
        ///
        /// # Example
        /// ```javascript
        /// const superTrend = Indicators.superTrend(3, 10);
        /// ```
        #[napi(factory)]
        pub fn super_trend(multiplier: f64, period: u32) -> napi::Result<Self> {
            let inner = IndicatorRs::super_trend(multiplier, period as usize)?;
            Ok(Self { inner })
        }

        /// Creates an Alligator indicator with custom parameters
        ///
        /// # Arguments
        /// * `jaw_period` - The period for the jaw line (commonly 13)
        /// * `jaw_shift` - Forward shift for the jaw line (commonly 8)
        /// * `teeth_period` - The period for the teeth line (commonly 8)
        /// * `teeth_shift` - Forward shift for the teeth line (commonly 5)
        /// * `lips_period` - The period for the lips line (commonly 5)
        /// * `lips_shift` - Forward shift for the lips line (commonly 3)
        ///
        /// # Example
        /// ```javascript
        /// const alligator = Indicators.alligator(13, 8, 8, 5, 5, 3);
        /// ```
        #[napi(factory)]
        pub fn alligator(
            jaw_period: u32,
            jaw_shift: u32,
            teeth_period: u32,
            teeth_shift: u32,
            lips_period: u32,
            lips_shift: u32,
        ) -> napi::Result<Self> {
            let inner = IndicatorRs::alligator(
                jaw_period as usize,
                jaw_shift as usize,
                teeth_period as usize,
                teeth_shift as usize,
                lips_period as usize,
                lips_shift as usize,
            )?;
            Ok(Self { inner })
        }

        /// Creates an Awesome Oscillator (AO) indicator
        ///
        /// # Arguments
        /// * `short_period` - The period for the short SMA (commonly 5)
        /// * `long_period` - The period for the long SMA (commonly 34)
        ///
        /// # Example
        /// ```javascript
        /// const ao = Indicators.ao(5, 34);
        /// ```
        #[napi(factory)]
        pub fn ao(short_period: u32, long_period: u32) -> napi::Result<Self> {
            let inner = IndicatorRs::ao(short_period as usize, long_period as usize)?;
            Ok(Self { inner })
        }

        /// Creates a Bollinger Bands (BB) indicator
        ///
        /// # Arguments
        /// * `period` - The period for the moving average (commonly 20)
        /// * `k` - The multiplier for the standard deviation (commonly 2.0)
        ///
        /// # Example
        /// ```javascript
        /// const bb = Indicators.bb(20, 2.0);
        /// ```
        #[napi(factory)]
        pub fn bb(period: u32, k: f64) -> napi::Result<Self> {
            let inner = IndicatorRs::bb(period as usize, k)?;
            Ok(Self { inner })
        }

        /// Creates a Keltner Channel (KC) indicator
        ///
        /// # Arguments
        /// * `period` - The period for the EMA and ATR calculation (commonly 20)
        /// * `multiplier` - The multiplier for the ATR (commonly 2.0)
        ///
        /// # Example
        /// ```javascript
        /// const kc = Indicators.kc(20, 2.0);
        /// ```
        #[napi(factory)]
        pub fn kc(period: u32, multiplier: f64) -> napi::Result<Self> {
            let inner = IndicatorRs::kc(period as usize, multiplier)?;
            Ok(Self { inner })
        }

        /// Creates a Mean Absolute Error (MAE) indicator
        ///
        /// # Arguments
        /// * `period` - The period for error calculation
        ///
        /// # Example
        /// ```javascript
        /// const mae = Indicators.mae(14);
        /// ```
        #[napi(factory)]
        pub fn mae(period: u32) -> napi::Result<Self> {
            let inner = IndicatorRs::mae(period as usize)?;
            Ok(Self { inner })
        }

        /// Creates an On-Balance Volume (OBV) indicator
        ///
        /// # Example
        /// ```javascript
        /// const obv = Indicators.obv();
        /// ```
        #[napi(factory)]
        pub fn obv() -> Self {
            let inner = IndicatorRs::obv();
            Self { inner }
        }

        /// Creates a Standard Deviation (SD) indicator
        ///
        /// # Arguments
        /// * `period` - The period for standard deviation calculation (commonly 20)
        ///
        /// # Example
        /// ```javascript
        /// const sd = Indicators.sd(20);
        /// ```
        #[napi(factory)]
        pub fn sd(period: u32) -> napi::Result<Self> {
            let inner = IndicatorRs::sd(period as usize)?;
            Ok(Self { inner })
        }

        /// Creates a Smoothed Moving Average (SMMA) indicator
        ///
        /// # Arguments
        /// * `period` - The period for the SMMA calculation (commonly 14)
        ///
        /// # Example
        /// ```javascript
        /// const smma = Indicators.smma(14);
        /// ```
        #[napi(factory)]
        pub fn smma(period: u32) -> napi::Result<Self> {
            let inner = IndicatorRs::smma(period as usize)?;
            Ok(Self { inner })
        }

        /// Creates a Stochastic Oscillator (STOCH) indicator
        ///
        /// # Arguments
        /// * `period` - The period for %K calculation (commonly 14)
        /// * `smoothing_period` - The period for %D smoothing (commonly 3)
        ///
        /// # Example
        /// ```javascript
        /// const stoch = Indicators.stoch(14, 3);
        /// ```
        #[napi(factory)]
        pub fn stoch(period: u32, smoothing_period: u32) -> napi::Result<Self> {
            let inner = IndicatorRs::stoch(period as usize, smoothing_period as usize)?;
            Ok(Self { inner })
        }

        /// Creates a Williams %R (WILLR) indicator
        ///
        /// # Arguments
        /// * `period` - The period for Williams %R calculation (commonly 14)
        ///
        /// # Example
        /// ```javascript
        /// const williamsR = Indicators.williamsR(14);
        /// ```
        #[napi(factory)]
        pub fn williams_r(period: u32) -> napi::Result<Self> {
            let inner = IndicatorRs::williams_r(period as usize)?;
            Ok(Self { inner })
        }

        /// Converts the indicator to a JSON representation
        ///
        /// # Example
        /// ```javascript
        /// const indicator = Indicators.rsi(14);
        /// const json = indicator.toJson();
        /// ```
        #[napi]
        pub fn to_json(&self, env: Env) -> napi::Result<Unknown> {
            env.to_js_value(&self)
        }

        /// Calculates the next value for a single input
        ///
        /// # Arguments
        /// * `input` - The input value to process
        ///
        /// # Returns
        /// A number or array of numbers depending on the indicator type
        ///
        /// # Example
        /// ```javascript
        /// const rsi = Indicators.rsi(14);
        /// const value = rsi.next(100);
        /// ```
        #[napi]
        pub fn next(&mut self, env: Env, input: f64) -> napi::Result<Unknown> {
            let output = self
                .inner
                .next(input)
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;
            match output {
                OutputType::Array(arr) => env.to_js_value(&arr),
                OutputType::Single(val) => env.to_js_value(&val),
                OutputType::Open => env.to_js_value(&"open"),
                OutputType::Close => env.to_js_value(&"close"),
                OutputType::High => env.to_js_value(&"high"),
                OutputType::Low => env.to_js_value(&"low"),
                OutputType::Volume => env.to_js_value(&"volume"),
                OutputType::Custom(vals) => env.to_js_value(&vals),
                OutputType::Static(static_val) => env.to_js_value(&static_val),
                OutputType::Statics(static_vals) => env.to_js_value(&static_vals),
            }
        }

        /// Calculates the next values for an array of inputs
        ///
        /// # Arguments
        /// * `input` - Array of input values to process
        ///
        /// # Returns
        /// An array of results, one for each input value
        ///
        /// # Example
        /// ```javascript
        /// const rsi = Indicators.rsi(14);
        /// const values = rsi.nextBatched([100, 101, 102]);
        /// ```
        #[napi]
        pub fn next_batched(&mut self, env: Env, input: Vec<f64>) -> napi::Result<Vec<Unknown>> {
            let raw = self.inner.next_batched(input.into_iter());
            raw.into_iter().map(|r| env.to_js_value(&r)).collect()
        }

        /// Calculates the next value using a candle as input
        ///
        /// # Arguments
        /// * `candle` - A candle object containing OHLCV data
        ///
        /// # Returns
        /// A number or array of numbers depending on the indicator type
        ///
        /// # Example
        /// ```javascript
        /// const tr = Indicators.tr();
        /// const candle = new Candle(100, 105, 95, 98, 102, 1000);
        /// const value = tr.nextCandle(candle);
        /// ```
        #[napi]
        pub fn next_candle(&mut self, env: Env, candle: &Candle) -> napi::Result<Unknown> {
            let output = self
                .inner
                .next(candle)
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;
            match output {
                OutputType::Array(arr) => env.to_js_value(&arr),
                OutputType::Single(val) => env.to_js_value(&val),
                OutputType::Open => env.to_js_value(&"open"),
                OutputType::Close => env.to_js_value(&"close"),
                OutputType::High => env.to_js_value(&"high"),
                OutputType::Low => env.to_js_value(&"low"),
                OutputType::Volume => env.to_js_value(&"volume"),
                OutputType::Custom(vals) => env.to_js_value(&vals),
                OutputType::Static(static_val) => env.to_js_value(&static_val),
                OutputType::Statics(static_vals) => env.to_js_value(&static_vals),
            }
        }

        /// Calculates the next values using an array of candles as input
        ///
        /// # Arguments
        /// * `candles` - Array of candle objects containing OHLCV data
        ///
        /// # Returns
        /// An array of results, one for each candle
        ///
        /// # Example
        /// ```javascript
        /// const tr = Indicators.tr();
        /// const candles = [
        ///   new Candle(100, 105, 95, 98, 102, 1000),
        ///   new Candle(102, 107, 97, 102, 105, 1200)
        /// ];
        /// const values = tr.nextCandles(candles);
        /// ```
        #[napi]
        pub fn next_candles(
            &mut self,
            env: Env,
            candles: Vec<&Candle>,
        ) -> napi::Result<Vec<Unknown>> {
            let results = self.inner.next_batched(candles.into_iter())?;
            results.into_iter().map(|e| env.to_js_value(&e)).collect()
        }
    }
}

#[cfg(feature = "py")]
pub mod py {
    use crate::traits::Candle as CandleTrait;
    use crate::{traits::Next, types::OutputType};
    use pyo3::{
        Bound, IntoPyObject, IntoPyObjectExt, PyAny, PyResult, Python, exceptions::PyValueError,
        pyclass, pymethods,
    };
    use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
    use serde::{Deserialize, Serialize};

    use super::Indicator as IndicatorsRs;

    #[gen_stub_pyclass]
    #[pyclass]
    #[derive(Clone, Debug)]
    pub struct Candle {
        pub price: f64,
        pub high: f64,
        pub low: f64,
        pub open: f64,
        pub close: f64,
        pub volume: f64,
    }

    #[gen_stub_pyclass]
    #[pyclass]
    #[derive(Clone, Default)]
    pub struct Indicator {
        inner: IndicatorsRs,
    }

    impl CandleTrait for Candle {
        /// Returns the closing price of the candle
        fn close(&self) -> f64 {
            self.close
        }

        /// Returns the highest price of the candle
        fn high(&self) -> f64 {
            self.high
        }

        /// Returns the lowest price of the candle
        fn low(&self) -> f64 {
            self.low
        }

        /// Returns the opening price of the candle
        fn open(&self) -> f64 {
            self.open
        }

        /// Returns the current or typical price of the candle
        fn price(&self) -> f64 {
            self.price
        }

        /// Returns the trading volume of the candle
        fn volume(&self) -> f64 {
            self.volume
        }
    }

    impl Serialize for Indicator {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.inner.serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for Indicator {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let inner = IndicatorsRs::deserialize(deserializer)?;
            Ok(Self { inner })
        }
    }
    #[gen_stub_pymethods]
    #[pymethods]
    impl Candle {
        #[new]
        pub fn new(price: f64, high: f64, low: f64, open: f64, close: f64, volume: f64) -> Self {
            Self {
                price,
                high,
                low,
                open,
                close,
                volume,
            }
        }

        #[staticmethod]
        pub fn price(price: f64) -> Self {
            Self {
                volume: 0.0,
                open: price,
                close: price,
                high: price,
                low: price,
                price,
            }
        }
    }

    #[gen_stub_pymethods]
    #[pymethods]
    impl Indicator {
        #[new]
        pub fn new() -> Self {
            Self::default()
        }

        #[staticmethod]
        pub fn from_string(json: String) -> PyResult<Self> {
            serde_json::from_str(&json).map_err(|e| PyValueError::new_err(e.to_string()))
        }

        #[staticmethod]
        pub fn ema(period: usize) -> PyResult<Self> {
            let inner = IndicatorsRs::ema(period)?;
            Ok(Self { inner })
        }

        #[staticmethod]
        pub fn sma(period: usize) -> PyResult<Self> {
            let inner = IndicatorsRs::sma(period)?;
            Ok(Self { inner })
        }

        #[staticmethod]
        pub fn rsi(period: usize) -> PyResult<Self> {
            let inner = IndicatorsRs::rsi(period)?;
            Ok(Self { inner })
        }

        #[staticmethod]
        pub fn macd(
            fast_period: usize,
            slow_period: usize,
            signal_period: usize,
        ) -> PyResult<Self> {
            let inner = IndicatorsRs::macd(fast_period, slow_period, signal_period)?;
            Ok(Self { inner })
        }

        #[staticmethod]
        pub fn tr() -> Self {
            let inner = IndicatorsRs::tr();
            Self { inner }
        }

        #[staticmethod]
        pub fn atr(period: usize) -> PyResult<Self> {
            let inner = IndicatorsRs::atr(period)?;
            Ok(Self { inner })
        }

        #[staticmethod]
        pub fn super_trend(multiplier: f64, period: usize) -> PyResult<Self> {
            let inner = IndicatorsRs::super_trend(multiplier, period)?;
            Ok(Self { inner })
        }

        pub fn to_json(&self) -> PyResult<String> {
            serde_json::to_string(&self).map_err(|e| PyValueError::new_err(e.to_string()))
        }

        pub fn next<'py>(&mut self, input: f64, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
            let output = self.inner.next(input)?;
            match output {
                OutputType::Array(arr) => arr.into_pyobject(py),
                OutputType::Single(val) => val.into_bound_py_any(py),
                _ => Err(PyValueError::new_err(
                    "Unexpected output type from indicator",
                )),
            }
        }

        pub fn next_batched<'py>(
            &mut self,
            input: Vec<f64>,
            py: Python<'py>,
        ) -> PyResult<Vec<Bound<'py, PyAny>>> {
            input.iter().map(|e| self.next(*e, py)).collect()
        }

        pub fn next_candle<'py>(
            &mut self,
            candle: Candle,
            py: Python<'py>,
        ) -> PyResult<Bound<'py, PyAny>> {
            let output = self.inner.next(&candle)?;
            match output {
                OutputType::Array(arr) => arr.into_bound_py_any(py),
                OutputType::Single(val) => val.into_bound_py_any(py),
                _ => Err(PyValueError::new_err(
                    "Unexpected output type from indicator",
                )),
            }
        }

        pub fn next_candles<'py>(
            &mut self,
            candles: Vec<Candle>,
            py: Python<'py>,
        ) -> PyResult<Vec<Bound<'py, PyAny>>> {
            candles
                .into_iter()
                .map(|c| self.next_candle(c, py))
                .collect()
        }
    }
}

#[cfg(test)]
mod indicators_test {

    use super::{indicator::Indicator, *};

    #[test]
    fn test_serialize() {
        let cases = vec![
            (Indicator::None(Default::default()), r#"{"type":"None"}"#),
            (
                Indicator::Sma(SimpleMovingAverage::new(5).unwrap()),
                r#"{"type":"Sma","period":5}"#,
            ),
            (
                Indicator::Ema(ExponentialMovingAverage::new(5).unwrap()),
                r#"{"type":"Ema","period":5}"#,
            ),
            (
                Indicator::Rsi(RelativeStrengthIndex::new(14).unwrap()),
                r#"{"type":"Rsi","period":14}"#,
            ),
            (
                Indicator::Macd(MovingAverageConvergenceDivergence::new(12, 26, 9).unwrap()),
                r#"{"type":"Macd","fast_ema":12,"slow_ema":26,"signal_ema":9}"#,
            ),
            (Indicator::Tr(TrueRange::new()), r#"{"type":"Tr"}"#),
            (
                Indicator::Atr(AverageTrueRange::new(14).unwrap()),
                r#"{"type":"Atr","period":14}"#,
            ),
            (
                Indicator::SuperTrend(SuperTrend::new(10.0, 3).unwrap()),
                r#"{"type":"SuperTrend","multiplier":10.0,"period":3}"#,
            ),
            (
                Indicator::Bb(BollingerBands::new(20, 2.0).unwrap()),
                r#"{"type":"Bb","period":20,"multiplier":2.0}"#,
            ),
            (
                Indicator::Stoch(StochasticOscillator::new(14, 3).unwrap()),
                r#"{"type":"Stoch","period":14,"smoothing_period":3}"#,
            ),
            (
                Indicator::Mae(MeanAbsoluteError::new(10).unwrap()),
                r#"{"type":"Mae","period":10}"#,
            ),
            (
                Indicator::Sd(StandardDeviation::new(10).unwrap()),
                r#"{"type":"Sd","period":10}"#,
            ),
        ];

        for (indicator, expected_json) in cases {
            let serialized = serde_json::to_string(&indicator).unwrap();
            assert_eq!(
                serialized, expected_json,
                "Serialized output mismatch for {indicator:?}"
            );
        }
    }

    #[test]
    fn test_deserialize() {
        let cases = vec![
            (r#"{"type":"None"}"#, Indicator::None(Default::default())),
            (
                r#"{"type":"Sma","period":5}"#,
                Indicator::Sma(SimpleMovingAverage::new(5).unwrap()),
            ),
            (
                r#"{"type":"Ema","period":5}"#,
                Indicator::Ema(ExponentialMovingAverage::new(5).unwrap()),
            ),
            (
                r#"{"type":"Rsi","period":14}"#,
                Indicator::Rsi(RelativeStrengthIndex::new(14).unwrap()),
            ),
            (
                r#"{"type":"Macd","fast_ema":12,"slow_ema":26,"signal_ema":9}"#,
                Indicator::Macd(MovingAverageConvergenceDivergence::new(12, 26, 9).unwrap()),
            ),
            (r#"{"type":"Tr"}"#, Indicator::Tr(TrueRange::new())),
            (
                r#"{"type":"Atr","period":14}"#,
                Indicator::Atr(AverageTrueRange::new(14).unwrap()),
            ),
            (
                r#"{"type":"SuperTrend","multiplier":10.0,"period":3}"#,
                Indicator::SuperTrend(SuperTrend::new(10.0, 3).unwrap()),
            ),
            (
                r#"{"type":"Bb","period":20,"multiplier":2.0}"#,
                Indicator::Bb(BollingerBands::new(20, 2.0).unwrap()),
            ),
            (
                r#"{"type":"Stoch","period":14,"smoothing_period":3}"#,
                Indicator::Stoch(StochasticOscillator::new(14, 3).unwrap()),
            ),
            (
                r#"{"type":"Mae","period":10}"#,
                Indicator::Mae(MeanAbsoluteError::new(10).unwrap()),
            ),
            (
                r#"{"type":"Sd","period":10}"#,
                Indicator::Sd(StandardDeviation::new(10).unwrap()),
            ),
        ];

        for (json, expected_indicator) in cases {
            let indicator: Indicator = serde_json::from_str(json).unwrap();
            assert_eq!(
                indicator, expected_indicator,
                "Deserialized indicator mismatch for json: {json}"
            );
        }
    }
}
