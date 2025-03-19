pub mod atr;
pub mod ema;
pub mod macd;
pub mod rsi;
pub mod sma;
pub mod super_trend;
pub mod tr;
// #[cfg(feature="js")]

pub use atr::AverageTrueRange;
pub use ema::ExponentialMovingAverage;
pub use macd::MovingAverageConvergenceDivergence;
pub use rsi::RelativeStrengthIndex;
pub use serde::{Deserialize, Serialize};
pub use sma::SimpleMovingAverage;
pub use super_trend::SuperTrend;
pub use tr::TrueRange;

use crate::{
    error::TaResult,
    traits::{Candle, Indicator as IndicatorTrait, Next, Period, Reset},
    types::OutputType,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Indicator {
    None(NoneIndicator),
    Sma(SimpleMovingAverage),
    Ema(ExponentialMovingAverage),
    Rsi(RelativeStrengthIndex),
    Macd(MovingAverageConvergenceDivergence),
    Tr(TrueRange),
    Atr(AverageTrueRange),
    SuperTrend(SuperTrend),
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NoneIndicator;

impl Period for NoneIndicator {
    fn period(&self) -> usize {
        0
    }
}

impl IndicatorTrait for NoneIndicator {}

impl Default for Indicator {
    fn default() -> Self {
        Self::None(NoneIndicator)
    }
}
impl Next<f64> for Indicator {
    type Output = OutputType;

    fn next(&mut self, input: f64) -> TaResult<Self::Output> {
        match self {
            Self::None(indicator) => indicator.next(input).map(OutputType::from),
            Self::Ema(indicator) => indicator.next(input).map(OutputType::from),
            Self::Sma(indicator) => indicator.next(input).map(OutputType::from),
            Self::Rsi(indicator) => indicator.next(input).map(OutputType::from),
            Self::Macd(indicator) => indicator
                .next(input)
                .map(|o| o.to_vec())
                .map(OutputType::from),
            Self::Tr(indicator) => indicator.next(input).map(OutputType::from),
            Self::Atr(indicator) => indicator.next(input).map(OutputType::from),
            Self::SuperTrend(indicator) => indicator
                .next(input)
                .map(|o| OutputType::Array(Vec::from(o))),
        }
    }
}

impl<T: Candle> Next<&T> for Indicator {
    type Output = OutputType;

    fn next(&mut self, input: &T) -> TaResult<Self::Output> {
        match self {
            Self::None(indicator) => indicator.next(input).map(OutputType::from),
            Self::Ema(indicator) => indicator.next(input).map(OutputType::from),
            Self::Sma(indicator) => indicator.next(input).map(OutputType::from),
            Self::Rsi(indicator) => indicator.next(input).map(OutputType::from),
            Self::Macd(indicator) => indicator
                .next(input)
                .map(|o| o.to_vec())
                .map(OutputType::from),
            Self::Tr(indicator) => indicator.next(input).map(OutputType::from),
            Self::Atr(indicator) => indicator.next(input).map(OutputType::from),
            Self::SuperTrend(indicator) => indicator
                .next(input)
                .map(|o| OutputType::Array(Vec::from(o))),
        }
    }
}

impl Reset for Indicator {
    fn reset(&mut self) {
        match self {
            Self::None(indicator) => indicator.reset(),
            Self::Ema(indicator) => indicator.reset(),
            Self::Sma(indicator) => indicator.reset(),
            Self::Rsi(indicator) => indicator.reset(),
            Self::Macd(indicator) => indicator.reset(),
            Self::Tr(indicator) => indicator.reset(),
            Self::Atr(indicator) => indicator.reset(),
            Self::SuperTrend(indicator) => indicator.reset(),
        }
    }
}

impl Period for Indicator {
    fn period(&self) -> usize {
        match self {
            Self::None(indicator) => indicator.period(),
            Self::Ema(indicator) => indicator.period(),
            Self::Sma(indicator) => indicator.period(),
            Self::Rsi(indicator) => indicator.period(),
            Self::Macd(indicator) => indicator.period(),
            Self::Tr(indicator) => indicator.period(),
            Self::Atr(indicator) => indicator.period(),
            Self::SuperTrend(indicator) => indicator.period(),
        }
    }
}

impl Reset for NoneIndicator {
    fn reset(&mut self) {}
}

impl Next<f64> for NoneIndicator {
    type Output = f64;

    fn next(&mut self, input: f64) -> TaResult<Self::Output> {
        Ok(input)
    }
}

impl<T: Candle> Next<&T> for NoneIndicator {
    type Output = f64;

    fn next(&mut self, input: &T) -> TaResult<Self::Output> {
        self.next(input.close())
    }
}

impl Indicator {
    pub fn none() -> Self {
        Self::None(NoneIndicator)
    }

    pub fn ema(period: usize) -> TaResult<Self> {
        Ok(Self::Ema(ExponentialMovingAverage::new(period)?))
    }

    pub fn sma(period: usize) -> TaResult<Self> {
        Ok(Self::Sma(SimpleMovingAverage::new(period)?))
    }

    pub fn rsi(period: usize) -> TaResult<Self> {
        Ok(Self::Rsi(RelativeStrengthIndex::new(period)?))
    }

    pub fn macd(fast_period: usize, slow_period: usize, signal_period: usize) -> TaResult<Self> {
        Ok(Self::Macd(MovingAverageConvergenceDivergence::new(
            fast_period,
            slow_period,
            signal_period,
        )?))
    }

    pub fn tr() -> Self {
        Self::Tr(TrueRange::new())
    }

    pub fn atr(period: usize) -> Self {
        Self::Atr(AverageTrueRange::new(period))
    }

    pub fn super_trend(multiplier: f64, period: usize) -> TaResult<Self> {
        Ok(Self::SuperTrend(SuperTrend::new(multiplier, period)?))
    }
}

#[cfg(feature = "js")]
pub mod js {
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
    #[derive(Clone, Serialize, Deserialize)]
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

    #[napi(js_name = "Indicators")]
    #[derive(Clone)]
    pub struct IndicatorJs {
        inner: Indicator,
    }

    impl Serialize for IndicatorJs {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer {
            self.inner.serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for IndicatorJs {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de> {
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
        pub fn atr(period: u32) -> Self {
            let inner = Indicator::atr(period as usize);
            Self { inner }
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

#[cfg(feature="py")]
pub mod py {
    use pyo3::{exceptions::PyValueError, pyclass, pymethods, Bound, IntoPyObject, IntoPyObjectExt, PyAny, PyResult, Python};
    use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
    use serde::{Deserialize, Serialize};
    use crate::{traits::Next, types::OutputType};

    use super::{Indicator as IndicatorsRs, Candle as CandleTrait};

    #[gen_stub_pyclass]
    #[pyclass]
    #[derive(Clone)]
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
        inner: IndicatorsRs
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
                S: serde::Serializer {
            self.inner.serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for Indicator {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de> {
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
            Ok(serde_json::from_str(&json).map_err(|e| PyValueError::new_err(e.to_string()))?)
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
        pub fn macd(fast_period: usize, slow_period: usize, signal_period: usize) -> PyResult<Self> {
            let inner = IndicatorsRs::macd(
                fast_period,
                slow_period,
                signal_period,
            )?;
            Ok(Self { inner })
        }

        #[staticmethod]
        pub fn tr() -> Self {
            let inner = IndicatorsRs::tr();
            Self { inner }
        }

        #[staticmethod]
        pub fn atr(period: usize) -> Self {
            let inner = IndicatorsRs::atr(period);
            Self { inner }
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
            let output = self
                .inner
                .next(input)?;
            match output {
                OutputType::Array(arr) => arr.into_pyobject(py),
                OutputType::Single(val) => val.into_bound_py_any(py),
            }
        }

        pub fn next_batched<'py>(&mut self, input: Vec<f64>, py: Python<'py>) -> PyResult<Vec<Bound<'py, PyAny>>> {
            input.iter().map(|e| self.next(*e, py)).collect()
        }

        pub fn next_candle<'py>(&mut self, candle: Candle, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
            let output = self
                .inner
                .next(&candle)?;
            match output {
                OutputType::Array(arr) => arr.into_bound_py_any(py),
                OutputType::Single(val) => val.into_bound_py_any(py),
            }
        }

        pub fn next_candles<'py>(
            &mut self,
            candles: Vec<Candle>,
            py: Python<'py>
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
        let super_trend = Indicator::SuperTrend(SuperTrend::new(3.0, 10).unwrap());
        let atr = Indicator::Atr(AverageTrueRange::new(5));
        let tr = Indicator::Tr(TrueRange::new());
        let macd = Indicator::Macd(MovingAverageConvergenceDivergence::new(3, 4, 7).unwrap());
        let rsi = Indicator::Rsi(RelativeStrengthIndex::new(3).unwrap());
        let sma = Indicator::Sma(SimpleMovingAverage::new(9).unwrap());
        let ema = Indicator::Ema(ExponentialMovingAverage::new(9).unwrap());
        let none = Indicator::None(NoneIndicator);

        let super_trend_string = serde_json::to_string(&super_trend).unwrap();
        let atr_string = serde_json::to_string(&atr).unwrap();
        let tr_string = serde_json::to_string(&tr).unwrap();
        let macd_string = serde_json::to_string(&macd).unwrap();
        let rsi_string = serde_json::to_string(&rsi).unwrap();
        let sma_string = serde_json::to_string(&sma).unwrap();
        let ema_string = serde_json::to_string(&ema).unwrap();
        let none_string = serde_json::to_string(&none).unwrap();

        assert_eq!(
            super_trend_string,
            r#"{"type":"SuperTrend","multiplier":3.0,"period":10}"#
        );
        assert_eq!(atr_string, r#"{"type":"Atr","period":5}"#);
        assert_eq!(tr_string, r#"{"type":"Tr"}"#);
        assert_eq!(
            macd_string,
            r#"{"type":"Macd","fast_ema":{"period":3},"slow_ema":{"period":4},"signal_ema":{"period":7}}"#
        );
        assert_eq!(
            rsi_string,
            r#"{"type":"Rsi","period":3}"#
        );
        assert_eq!(sma_string, r#"{"type":"Sma","period":9}"#);
        assert_eq!(ema_string, r#"{"type":"Ema","period":9}"#);
        assert_eq!(none_string, r#"{"type":"None"}"#);
    }

    #[test]
    fn test_deserialize() {
        let super_trend_string = r#"{"type":"SuperTrend","multiplier":3.0,"period":10}"#;
        let atr_string = r#"{"type":"Atr","period":5}"#;
        let tr_string = r#"{"type":"Tr"}"#;
        let macd_string = r#"{"type":"Macd","fast_ema":{"period":3},"slow_ema":{"period":4},"signal_ema":{"period":7}}"#;
        let rsi_string = r#"{"type":"Rsi","period":3}"#;
        let sma_string = r#"{"type":"Sma","period":9}"#;
        let ema_string = r#"{"type":"Ema","period":9}"#;
        let none_string = r#"{"type":"None"}"#;

        let super_trend: Indicator = serde_json::from_str(super_trend_string).unwrap();
        let atr: Indicator = serde_json::from_str(atr_string).unwrap();
        let tr: Indicator = serde_json::from_str(tr_string).unwrap();
        let macd: Indicator = serde_json::from_str(macd_string).unwrap();
        let rsi: Indicator = serde_json::from_str(rsi_string).unwrap();
        let sma: Indicator = serde_json::from_str(sma_string).unwrap();
        let ema: Indicator = serde_json::from_str(ema_string).unwrap();
        let none: Indicator = serde_json::from_str(none_string).unwrap();

        let super_trend_check = Indicator::SuperTrend(SuperTrend::new(3.0, 10).unwrap());
        let atr_check = Indicator::Atr(AverageTrueRange::new(5));
        let tr_check = Indicator::Tr(TrueRange::new());
        let macd_check =
        Indicator::Macd(MovingAverageConvergenceDivergence::new(3, 4, 7).unwrap());
        let rsi_check = Indicator::Rsi(RelativeStrengthIndex::new(3).unwrap());
        let sma_check = Indicator::Sma(SimpleMovingAverage::new(9).unwrap());
        let ema_check = Indicator::Ema(ExponentialMovingAverage::new(9).unwrap());
        let none_check = Indicator::None(NoneIndicator);

        assert_eq!(super_trend, super_trend_check);
        assert_eq!(atr, atr_check);
        assert_eq!(tr, tr_check);
        assert_eq!(macd, macd_check);
        assert_eq!(rsi, rsi_check);
        assert_eq!(sma, sma_check);
        assert_eq!(ema, ema_check);
        assert_eq!(none, none_check);
    }
}
