#[cfg(feature = "chipa_lang")]
use chipa_lang_utils::Lang;

use core::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    error::TaResult,
    indicators::ema::ExponentialMovingAverage as Ema,
    traits::{Candle, IndicatorTrait, Next, Period, Reset},
    types::OutputShape,
};

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::duplicated_attributes)]
#[cfg_attr(feature = "chipa_lang", derive(Lang))]
#[cfg_attr(
    feature = "chipa_lang",
    ct(
        grammar(Macd(fast_period, slow_period, signal_period)),
        wrapper(MovingAverageConvergenceDivergenceWrapper(usize, usize, usize)),
        may_fail
    )
)]
pub struct MovingAverageConvergenceDivergence {
    fast_ema: Ema,
    slow_ema: Ema,
    signal_ema: Ema,
}

#[cfg(feature = "chipa_lang")]
struct MovingAverageConvergenceDivergenceWrapper {
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
}

#[cfg(feature = "chipa_lang")]
impl From<&MovingAverageConvergenceDivergence> for MovingAverageConvergenceDivergenceWrapper {
    fn from(macd: &MovingAverageConvergenceDivergence) -> Self {
        MovingAverageConvergenceDivergenceWrapper {
            fast_period: macd.fast_ema.period(),
            slow_period: macd.slow_ema.period(),
            signal_period: macd.signal_ema.period(),
        }
    }
}

/// Creating custom Serialize and deserialize implementations for MovingAverageConvergenceDivergence
impl Serialize for MovingAverageConvergenceDivergence {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct MovingAverageConvergenceDivergenceVisitor {
            fast_ema: usize,
            slow_ema: usize,
            signal_ema: usize,
        }
        let visitor = MovingAverageConvergenceDivergenceVisitor {
            fast_ema: self.fast_ema.period(),
            slow_ema: self.slow_ema.period(),
            signal_ema: self.signal_ema.period(),
        };
        visitor.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MovingAverageConvergenceDivergence {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct MovingAverageConvergenceDivergenceVisitor {
            fast_ema: usize,
            slow_ema: usize,
            signal_ema: usize,
        }
        let visitor = MovingAverageConvergenceDivergenceVisitor::deserialize(deserializer)?;
        MovingAverageConvergenceDivergence::new(
            visitor.fast_ema,
            visitor.slow_ema,
            visitor.signal_ema,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl MovingAverageConvergenceDivergence {
    pub fn new(fast_period: usize, slow_period: usize, signal_period: usize) -> TaResult<Self> {
        Ok(Self {
            fast_ema: Ema::new(fast_period)?,
            slow_ema: Ema::new(slow_period)?,
            signal_ema: Ema::new(signal_period)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MovingAverageConvergenceDivergenceOutput {
    pub macd: f64,
    pub signal: f64,
    pub histogram: f64,
}

impl IndicatorTrait for MovingAverageConvergenceDivergence {
    fn output_shape(&self) -> OutputShape {
        OutputShape::Shape(3) // MACD, Signal, Histogram
    }
}

impl MovingAverageConvergenceDivergenceOutput {
    pub fn to_vec(&self) -> Vec<f64> {
        vec![self.macd, self.signal, self.histogram]
    }
}

impl From<MovingAverageConvergenceDivergenceOutput> for (f64, f64, f64) {
    fn from(mo: MovingAverageConvergenceDivergenceOutput) -> Self {
        (mo.macd, mo.signal, mo.histogram)
    }
}

impl Next<f64> for MovingAverageConvergenceDivergence {
    type Output = MovingAverageConvergenceDivergenceOutput;

    fn next(&mut self, input: f64) -> TaResult<Self::Output> {
        let fast_val = self.fast_ema.next(input)?;
        let slow_val = self.slow_ema.next(input)?;

        let macd = fast_val - slow_val;
        let signal = self.signal_ema.next(macd)?;
        let histogram = macd - signal;

        Ok(MovingAverageConvergenceDivergenceOutput {
            macd,
            signal,
            histogram,
        })
    }
}

impl<T: Candle> Next<&T> for MovingAverageConvergenceDivergence {
    type Output = MovingAverageConvergenceDivergenceOutput;

    fn next(&mut self, input: &T) -> TaResult<Self::Output> {
        self.next(input.close())
    }
}

impl Period for MovingAverageConvergenceDivergence {
    /// Since the MACD indicator has multiple periods, we will only take the longes
    fn period(&self) -> usize {
        self.slow_ema.period()
    }
}

impl Reset for MovingAverageConvergenceDivergence {
    fn reset(&mut self) {
        self.fast_ema.reset();
        self.slow_ema.reset();
        self.signal_ema.reset();
    }
}

impl Default for MovingAverageConvergenceDivergence {
    fn default() -> Self {
        Self::new(12, 26, 9).unwrap()
    }
}

impl fmt::Display for MovingAverageConvergenceDivergence {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "MACD({}, {}, {})",
            self.fast_ema.period(),
            self.slow_ema.period(),
            self.signal_ema.period()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    type Macd = MovingAverageConvergenceDivergence;

    fn round(nums: (f64, f64, f64)) -> (f64, f64, f64) {
        let n0 = (nums.0 * 100.0).round() / 100.0;
        let n1 = (nums.1 * 100.0).round() / 100.0;
        let n2 = (nums.2 * 100.0).round() / 100.0;
        (n0, n1, n2)
    }

    #[test]
    fn test_new() {
        assert!(Macd::new(0, 1, 1).is_err());
        assert!(Macd::new(1, 0, 1).is_err());
        assert!(Macd::new(1, 1, 0).is_err());
        assert!(Macd::new(1, 1, 1).is_ok());
    }

    #[test]
    fn test_macd() {
        let mut macd = Macd::new(3, 6, 4).unwrap();

        assert_eq!(round(macd.next(2.0).unwrap().into()), (0.0, 0.0, 0.0));
        assert_eq!(round(macd.next(3.0).unwrap().into()), (0.21, 0.09, 0.13));
        assert_eq!(round(macd.next(4.2).unwrap().into()), (0.52, 0.26, 0.26));
        assert_eq!(round(macd.next(7.0).unwrap().into()), (1.15, 0.62, 0.54));
        assert_eq!(round(macd.next(6.7).unwrap().into()), (1.15, 0.83, 0.32));
        assert_eq!(round(macd.next(6.5).unwrap().into()), (0.94, 0.87, 0.07));
    }

    #[test]
    fn test_reset() {
        let mut macd = Macd::new(3, 6, 4).unwrap();

        assert_eq!(round(macd.next(2.0).unwrap().into()), (0.0, 0.0, 0.0));
        assert_eq!(round(macd.next(3.0).unwrap().into()), (0.21, 0.09, 0.13));

        macd.reset();

        assert_eq!(round(macd.next(2.0).unwrap().into()), (0.0, 0.0, 0.0));
        assert_eq!(round(macd.next(3.0).unwrap().into()), (0.21, 0.09, 0.13));
    }

    #[test]
    fn test_default() {
        Macd::default();
    }

    #[test]
    fn test_display() {
        let indicator = Macd::new(13, 30, 10).unwrap();
        assert_eq!(format!("{indicator}"), "MACD(13, 30, 10)");
    }

    #[test]
    fn test_serialize() {
        let macd = MovingAverageConvergenceDivergence::new(3, 4, 7).unwrap();
        let macd_string = serde_json::to_string(&macd).unwrap();
        assert_eq!(macd_string, r#"{"fast_ema":3,"slow_ema":4,"signal_ema":7}"#)
    }

    #[test]
    fn test_deserialize() {
        let macd_string = r#"{"fast_ema":3,"slow_ema":4,"signal_ema":7}"#;
        let macd_check = MovingAverageConvergenceDivergence::new(3, 4, 7).unwrap();
        let macd_deserialized: MovingAverageConvergenceDivergence =
            serde_json::from_str(macd_string).unwrap();
        assert_eq!(macd_deserialized, macd_check)
    }
}
