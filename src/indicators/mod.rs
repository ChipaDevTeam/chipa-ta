pub mod atr;
pub mod ema;
pub mod macd;
pub mod rsi;
pub mod sma;
pub mod super_trend;
pub mod tr;
// #[cfg(feature="js")]

use atr::AverageTrueRange;
use ema::ExponentialMovingAverage;
use macd::MovingAverageConvergenceDivergence;
use rsi::RelativeStrengthIndex;
use serde::{Deserialize, Serialize};
use sma::SimpleMovingAverage;
use super_trend::SuperTrend;
use tr::TrueRange;

use crate::{
    error::TaResult,
    traits::{Candle, Indicator, Next, Period, Reset},
    types::OutputType,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Indicators {
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

impl Indicator for NoneIndicator {}

impl Default for Indicators {
    fn default() -> Self {
        Self::None(NoneIndicator)
    }
}
impl Next<f64> for Indicators {
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

impl<T: Candle> Next<&T> for Indicators {
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

impl Reset for Indicators {
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

impl Period for Indicators {
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

impl Indicators {
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
    #[napi(constructor, js_name = "Candle")]
    #[derive(Clone)]
    pub struct CandleJs {
        pub price: f64,
        pub high: f64,
        pub low: f64,
        pub open: f64,
        pub close: f64,
        pub volume: f64,
    }

    impl Candle for CandleJs {
        fn close(&self) -> f64 {
            self.close
        }

        fn high(&self) -> f64 {
            self.high
        }

        fn low(&self) -> f64 {
            self.low
        }

        fn open(&self) -> f64 {
            self.open
        }

        fn price(&self) -> f64 {
            self.price
        }

        fn volume(&self) -> f64 {
            self.volume
        }
    }

    #[napi]
    impl CandleJs {
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
    }

    #[napi(js_name = "Indicators")]
    pub struct IndicatorJs {
        inner: Indicators,
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
            let inner = Indicators::deserialize(deserializer)?;
            Ok(Self { inner })
        }
    }

    impl Default for IndicatorJs {
        fn default() -> Self {
            Self {
                inner: Indicators::None(NoneIndicator),
            }
        }
    }

    #[napi]
    impl IndicatorJs {
        #[napi(constructor)]
        pub fn new() -> Self {
            Self::default()
        }

        #[napi(factory)]
        pub fn from_string(json: JsUnknown, env: Env) -> napi::Result<Self> {
            let inner: Indicators = env.from_js_value(json)?;
            Ok(Self { inner })
        }

        #[napi(factory)]
        pub fn ema(period: u32) -> napi::Result<Self> {
            let inner = Indicators::ema(period as usize)?;
            Ok(Self { inner })
        }

        #[napi(factory)]
        pub fn sma(period: u32) -> napi::Result<Self> {
            let inner = Indicators::sma(period as usize)?;
            Ok(Self { inner })
        }

        #[napi(factory)]
        pub fn rsi(period: u32) -> napi::Result<Self> {
            let inner = Indicators::rsi(period as usize)?;
            Ok(Self { inner })
        }

        #[napi(factory)]
        pub fn macd(fast_period: u32, slow_period: u32, signal_period: u32) -> napi::Result<Self> {
            let inner = Indicators::macd(
                fast_period as usize,
                slow_period as usize,
                signal_period as usize,
            )?;
            Ok(Self { inner })
        }

        #[napi(factory)]
        pub fn tr() -> Self {
            let inner = Indicators::tr();
            Self { inner }
        }

        #[napi(factory)]
        pub fn atr(period: u32) -> Self {
            let inner = Indicators::atr(period as usize);
            Self { inner }
        }

        #[napi(factory)]
        pub fn super_trend(multiplier: f64, period: u32) -> napi::Result<Self> {
            let inner = Indicators::super_trend(multiplier, period as usize)?;
            Ok(Self { inner })
        }

        #[napi]
        pub fn to_json(&self, env: Env) -> napi::Result<JsUnknown> {
            env.to_js_value(&self)
        }

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

        #[napi]
        pub fn next_batched(&mut self, env: Env, input: Vec<f64>) -> napi::Result<Vec<JsUnknown>> {
            input.iter().map(|e| self.next(env, *e)).collect()
        }

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

#[cfg(test)]
mod indicators_test {
    use super::*;

    #[test]
    fn test_serialize() {
        let super_trend = Indicators::SuperTrend(SuperTrend::new(3.0, 10).unwrap());
        let atr = Indicators::Atr(AverageTrueRange::new(5));
        let tr = Indicators::Tr(TrueRange::new());
        let macd = Indicators::Macd(MovingAverageConvergenceDivergence::new(3, 4, 7).unwrap());
        let rsi = Indicators::Rsi(RelativeStrengthIndex::new(3).unwrap());
        let sma = Indicators::Sma(SimpleMovingAverage::new(9).unwrap());
        let ema = Indicators::Ema(ExponentialMovingAverage::new(9).unwrap());
        let none = Indicators::None(NoneIndicator);

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

        let super_trend: Indicators = serde_json::from_str(super_trend_string).unwrap();
        let atr: Indicators = serde_json::from_str(atr_string).unwrap();
        let tr: Indicators = serde_json::from_str(tr_string).unwrap();
        let macd: Indicators = serde_json::from_str(macd_string).unwrap();
        let rsi: Indicators = serde_json::from_str(rsi_string).unwrap();
        let sma: Indicators = serde_json::from_str(sma_string).unwrap();
        let ema: Indicators = serde_json::from_str(ema_string).unwrap();
        let none: Indicators = serde_json::from_str(none_string).unwrap();

        let super_trend_check = Indicators::SuperTrend(SuperTrend::new(3.0, 10).unwrap());
        let atr_check = Indicators::Atr(AverageTrueRange::new(5));
        let tr_check = Indicators::Tr(TrueRange::new());
        let macd_check =
            Indicators::Macd(MovingAverageConvergenceDivergence::new(3, 4, 7).unwrap());
        let rsi_check = Indicators::Rsi(RelativeStrengthIndex::new(3).unwrap());
        let sma_check = Indicators::Sma(SimpleMovingAverage::new(9).unwrap());
        let ema_check = Indicators::Ema(ExponentialMovingAverage::new(9).unwrap());
        let none_check = Indicators::None(NoneIndicator);

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
