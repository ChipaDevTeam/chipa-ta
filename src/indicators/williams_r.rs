#[cfg(feature = "chipa_lang")]
use chipa_lang_utils::Lang;

use core::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    error::TaResult,
    traits::{Candle, IndicatorTrait, Next, Period, Reset},
    types::{OutputShape, Queue},
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "chipa_lang", derive(Lang))]
#[cfg_attr(
    feature = "chipa_lang",
    ct(grammar(WilliamsR(period)), wrapper(WilliamsRWrapper(usize)), may_fail)
)]
pub struct WilliamsR {
    highs: Queue<f64>,
    lows: Queue<f64>,
}

#[cfg(feature = "chipa_lang")]
struct WilliamsRWrapper {
    period: usize,
}

#[cfg(feature = "chipa_lang")]
impl From<&WilliamsR> for WilliamsRWrapper {
    fn from(williams: &WilliamsR) -> Self {
        Self {
            period: williams.period(),
        }
    }
}

impl Default for WilliamsR {
    fn default() -> Self {
        Self {
            highs: Queue::new(14).unwrap(),
            lows: Queue::new(14).unwrap(),
        }
    }
}

impl WilliamsR {
    pub fn new(period: usize) -> TaResult<Self> {
        Ok(Self {
            highs: Queue::new(period)?,
            lows: Queue::new(period)?,
        })
    }
}

/// Creating custom Serialize and deserialize implementations for WilliamsR
impl Serialize for WilliamsR {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct WilliamsRVisitor {
            period: usize,
        }
        let visitor = WilliamsRVisitor {
            period: self.period(),
        };
        visitor.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for WilliamsR {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct WilliamsRVisitor {
            period: usize,
        }

        let visitor = WilliamsRVisitor::deserialize(deserializer)?;
        WilliamsR::new(visitor.period).map_err(serde::de::Error::custom)
    }
}

impl IndicatorTrait for WilliamsR {
    fn output_shape(&self) -> OutputShape {
        OutputShape::Shape(1)
    }
}

impl fmt::Display for WilliamsR {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WilliamsR({})", self.period())
    }
}

impl Reset for WilliamsR {
    fn reset(&mut self) {
        self.highs.reset();
        self.lows.reset();
    }
}

impl Period for WilliamsR {
    fn period(&self) -> usize {
        self.highs.period()
    }
}

impl Next<f64> for WilliamsR {
    type Output = f64;

    fn next(&mut self, input: f64) -> TaResult<Self::Output> {
        let _ = self.highs.push(input);
        let _ = self.lows.push(input);

        if self.highs.len() < self.period() || self.lows.len() < self.period() {
            return Ok(0.0); // Not enough data to calculate Williams %R
        }

        let highest_high = self
            .highs
            .iter()
            .fold(f64::MIN, |arg0: f64, other: &f64| f64::max(arg0, *other));
        let lowest_low = self
            .lows
            .iter()
            .fold(f64::MAX, |arg0: f64, other: &f64| f64::min(arg0, *other));
        if highest_high == lowest_low {
            return Ok(0.0); // Avoid division by zero
        }
        let williams_r = (highest_high - input) / (highest_high - lowest_low) * -100.0;

        Ok(williams_r)
    }
}

impl<C: Candle> Next<&C> for WilliamsR {
    type Output = f64;

    fn next(&mut self, candle: &C) -> TaResult<Self::Output> {
        let _ = self.highs.push(candle.high());
        let _ = self.lows.push(candle.low());

        if self.highs.len() < self.period() || self.lows.len() < self.period() {
            return Ok(0.0); // Not enough data to calculate Williams %R
        }

        let highest_high = self
            .highs
            .iter()
            .fold(f64::MIN, |arg0: f64, other: &f64| f64::max(arg0, *other));
        let lowest_low = self
            .lows
            .iter()
            .fold(f64::MAX, |arg0: f64, other: &f64| f64::min(arg0, *other));
        let current_close = candle.close();
        if highest_high == lowest_low {
            return Ok(0.0); // Avoid division by zero
        }
        let williams_r = (highest_high - current_close) / (highest_high - lowest_low) * -100.0;

        Ok(williams_r)
    }
}
