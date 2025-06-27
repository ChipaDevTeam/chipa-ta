pub mod alligator;
pub mod ao;
pub mod atr;
pub mod bb;
pub mod ema;
pub mod indicator;
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
pub use indicator::Indicator;
pub use macd::MovingAverageConvergenceDivergence;
pub use mae::MeanAbsoluteError;
pub use rsi::RelativeStrengthIndex;
pub use sd::StandardDeviation;
pub use sma::SimpleMovingAverage;
pub use stoch::StochasticOscillator;
pub use super_trend::SuperTrend;
pub use tr::TrueRange;

use crate::traits::Candle;

pub use serde::{Deserialize, Serialize};
#[cfg(feature = "js")]
pub mod js {
    use crate::{
        indicators::indicator::NoneIndicator, traits::Candle, traits::Next, types::OutputType,
    };

    use super::*;
    use napi::{Env, JsUnknown};
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
    #[napi(js_name = "Candle")]
    #[derive(Clone, Serialize, Deserialize, Debug)]
    pub struct CandleJs {
        pub price: f64,
        pub high: f64,
        pub low: f64,
        pub open: f64,
        pub close: f64,
        pub volume: f64,
    }

    impl Candle for CandleJs {
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
    impl CandleJs {
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
        pub fn from_string(json: JsUnknown, env: Env) -> napi::Result<Self> {
            let candle = env.from_js_value(json)?;
            Ok(candle)
        }

        #[napi]
        pub fn to_json(&self, env: Env) -> napi::Result<JsUnknown> {
            env.to_js_value(&self)
        }
    }

    /// JavaScript bindings for various financial indicators.
    ///
    /// This implementation exposes a set of constructors and methods to create and use technical indicators
    /// from JavaScript via NAPI. Supported indicators include EMA, SMA, RSI, MACD, TR, ATR, and SuperTrend.
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
    /// - `new()` - Creates a new empty indicator.
    /// - `fromString(json)` - Restores an indicator from a JSON string.
    /// - `ema(period)` - Creates an Exponential Moving Average indicator.
    /// - `sma(period)` - Creates a Simple Moving Average indicator.
    /// - `rsi(period)` - Creates a Relative Strength Index indicator.
    /// - `macd(fast, slow, signal)` - Creates a MACD indicator.
    /// - `tr()` - Creates a True Range indicator.
    /// - `atr(period)` - Creates an Average True Range indicator.
    /// - `superTrend(multiplier, period)` - Creates a SuperTrend indicator.
    /// - `toJson()` - Serializes the indicator to JSON.
    /// - `next(input)` - Calculates the next value for a single input.
    /// - `nextBatched(inputs)` - Calculates next values for an array of inputs.
    /// - `nextCandle(candle)` - Calculates the next value using a candle.
    /// - `nextCandles(candles)` - Calculates next values for an array of candles.
    ///
    /// All methods are available from JavaScript via the `Indicators` class.
    #[napi(js_name = "Indicators")]
    #[derive(Clone)]
    pub struct IndicatorJs {
        inner: Indicator,
    }

    impl Serialize for IndicatorJs {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.inner.serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for IndicatorJs {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let inner = Indicator::deserialize(deserializer)?;
            Ok(Self { inner })
        }
    }

    impl Default for IndicatorJs {
        fn default() -> Self {
            Self {
                inner: Indicator::None(NoneIndicator),
            }
        }
    }

    #[napi]
    impl IndicatorJs {
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
        pub fn from_string(json: JsUnknown, env: Env) -> napi::Result<Self> {
            let inner: Indicator = env.from_js_value(json)?;
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
            let inner = Indicator::ema(period as usize)?;
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
            let inner = Indicator::sma(period as usize)?;
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
            let inner = Indicator::rsi(period as usize)?;
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
            let inner = Indicator::macd(
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
            let inner = Indicator::tr();
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
            let inner = Indicator::atr(period as usize)?;
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
            let inner = Indicator::super_trend(multiplier, period as usize)?;
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
        pub fn to_json(&self, env: Env) -> napi::Result<JsUnknown> {
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
        pub fn next(&mut self, env: Env, input: f64) -> napi::Result<JsUnknown> {
            let output = self
                .inner
                .next(input)
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;
            match output {
                OutputType::Array(arr) => {
                    let mut js_arr = env.create_array_with_length(arr.len())?;
                    for (i, val) in arr.iter().enumerate() {
                        js_arr.set_element(i as u32, env.create_double(*val)?)?;
                    }
                    Ok(js_arr.into_unknown())
                }
                OutputType::Single(val) => Ok(env.create_double(val)?.into_unknown()),
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
        pub fn next_batched(&mut self, env: Env, input: Vec<f64>) -> napi::Result<Vec<JsUnknown>> {
            input.iter().map(|e| self.next(env, *e)).collect()
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
        pub fn next_candle(&mut self, env: Env, candle: &CandleJs) -> napi::Result<JsUnknown> {
            let output = self
                .inner
                .next(candle)
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;
            match output {
                OutputType::Array(arr) => {
                    let mut js_arr = env.create_array_with_length(arr.len())?;
                    for (i, val) in arr.iter().enumerate() {
                        js_arr.set_element(i as u32, env.create_double(*val)?)?;
                    }
                    Ok(js_arr.into_unknown())
                }
                OutputType::Single(val) => Ok(env.create_double(val)?.into_unknown()),
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
            candles: Vec<&CandleJs>,
        ) -> napi::Result<Vec<JsUnknown>> {
            candles
                .into_iter()
                .map(|c| self.next_candle(env, c))
                .collect()
        }
    }
}

#[cfg(feature = "py")]
pub mod py {
    use crate::traits::Candle as CandleTrait;
    use crate::{traits::Next, types::OutputType};
    use pyo3::{
        exceptions::PyValueError, pyclass, pymethods, Bound, IntoPyObject, IntoPyObjectExt, PyAny,
        PyResult, Python,
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

    use super::*;

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
                "Serialized output mismatch for {:?}",
                indicator
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
                "Deserialized indicator mismatch for json: {}",
                json
            );
        }
    }
}
