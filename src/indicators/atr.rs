use core::fmt;

use serde::{Deserialize, Serialize};

use crate::{error::TaResult, indicators::{ExponentialMovingAverage, TrueRange}, traits::{Candle, Indicator, Next, Period, Reset}, types::OutputShape};

#[derive(Debug, Clone, PartialEq)]
pub struct AverageTrueRange {
    true_range: TrueRange,
    ema: ExponentialMovingAverage,
}

/// Custom implementation of the Serialize and Deserialize traits for AverageTrueRange
impl Serialize for AverageTrueRange {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct AtrVisitor {
            period: usize,
        }
        AtrVisitor {
            period: self.ema.period(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AverageTrueRange {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct AtrVisitor {
            period: usize,
        }
        let visitor = AtrVisitor::deserialize(deserializer)?;
        Self::new(visitor.period)
            .map_err(serde::de::Error::custom)
    }
}

impl AverageTrueRange {
    pub fn new(period: usize) -> TaResult<Self> {
        Ok(Self {
            true_range: TrueRange::new(),
            ema: ExponentialMovingAverage::new(period)?,
        })
    }
}

impl Indicator for AverageTrueRange {
    fn output_shape(&self) -> OutputShape {
        OutputShape::Shape(1)
    }
}

impl Period for AverageTrueRange {
    fn period(&self) -> usize {
        self.ema.period()
    }
}

impl Next<f64> for AverageTrueRange {
    type Output = f64;

    fn next(&mut self, input: f64) -> TaResult<Self::Output> {
        self.ema.next(self.true_range.next(input)?)
    }
}

impl<C: Candle> Next<&C> for AverageTrueRange {
    type Output = f64;

    fn next(&mut self, input: &C) -> TaResult<Self::Output> {
        self.ema.next(self.true_range.next(input)?)
    }
}

impl Reset for AverageTrueRange {
    fn reset(&mut self) {
        self.true_range.reset();
        self.ema.reset();
    }
}

impl Default for AverageTrueRange {
    fn default() -> Self {
        Self::new(14).unwrap()
    }
}

impl fmt::Display for AverageTrueRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ATR({})", self.ema.period())
    }
}

#[cfg(test)]
mod tests {
    use crate::helper_types::Bar;

    use super::*;

    #[test]
    fn test_new() {
        assert!(AverageTrueRange::new(0).is_err());
        assert!(AverageTrueRange::new(1).is_ok());
    }
    #[test]
    fn test_next() {
        let mut atr = AverageTrueRange::new(3).unwrap();

        let bar1 = Bar::new().set_high(10).set_low(7.5).set_close(9);
        let bar2 = Bar::new().set_high(11).set_low(9).set_close(9.5);
        let bar3 = Bar::new().set_high(9).set_low(5).set_close(8);

        assert_eq!(atr.next(&bar1).unwrap(), 2.5);
        assert_eq!(atr.next(&bar2).unwrap(), 2.25);
        assert_eq!(atr.next(&bar3).unwrap(), 3.375);
    }

    #[test]
    fn test_reset() {
        let mut atr = AverageTrueRange::new(9).unwrap();

        let bar1 = Bar::new().set_high(10).set_low(7.5).set_close(9);
        let bar2 = Bar::new().set_high(11).set_low(9).set_close(9.5);

        atr.next(&bar1).unwrap();
        atr.next(&bar2).unwrap();

        atr.reset();
        let bar3 = Bar::new().set_high(60).set_low(15).set_close(51);
        assert_eq!(atr.next(&bar3).unwrap(), 45.0);
    }

    #[test]
    fn test_default() {
        AverageTrueRange::default();
    }

    #[test]
    fn test_display() {
        let indicator = AverageTrueRange::new(8).unwrap();
        assert_eq!(format!("{}", indicator), "ATR(8)");
    }
}