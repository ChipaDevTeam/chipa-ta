// Bollinger Bands indicator implementation for chipa-ta
// Based on ta-rs and TA-Lib

use core::fmt;

use serde::{Deserialize, Serialize};

use super::sd::StandardDeviation as Sd;
use crate::error::TaResult;
use crate::traits::Candle;
use crate::traits::{Next, Period, Reset};

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BollingerBands {
    period: usize,
    multiplier: f64,
    #[serde(skip)]
    sd: Sd,
}

/// Custom implementation of the Deserialize trait for BollingerBands
impl<'de> Deserialize<'de> for BollingerBands {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct BollingerBandsVisitor {
            period: usize,
            multiplier: f64,
        }
        // Deserialize the BollingerBands struct
        let bbv = BollingerBandsVisitor::deserialize(deserializer)?;

        // Initialize the StandardDeviation with the period
        let bb = BollingerBands {
            period: bbv.period,
            multiplier: bbv.multiplier,
            sd: Sd::new(bbv.period).map_err(serde::de::Error::custom)?,
        };

        Ok(bb)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BollingerBandsOutput {
    pub average: f64,
    pub upper: f64,
    pub lower: f64,
}

impl BollingerBands {
    pub fn new(period: usize, multiplier: f64) -> TaResult<Self> {
        Ok(Self {
            period,
            multiplier,
            sd: Sd::new(period)?,
        })
    }

    pub fn multiplier(&self) -> f64 {
        self.multiplier
    }
}

impl Period for BollingerBands {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<f64> for BollingerBands {
    type Output = BollingerBandsOutput;

    fn next(&mut self, input: f64) -> TaResult<Self::Output> {
        let sd = self.sd.next(input)?;
        let mean = self.sd.mean();

        Ok(Self::Output {
            average: mean,
            upper: mean + sd * self.multiplier,
            lower: mean - sd * self.multiplier,
        })
    }
}

impl<T: Candle> Next<&T> for BollingerBands {
    type Output = BollingerBandsOutput;

    fn next(&mut self, input: &T) -> TaResult<Self::Output> {
        self.next(input.close())
    }
}

impl Reset for BollingerBands {
    fn reset(&mut self) {
        self.sd.reset();
    }
}

impl Default for BollingerBands {
    fn default() -> Self {
        Self::new(9, 2_f64).unwrap()
    }
}

impl fmt::Display for BollingerBands {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BB({}, {})", self.period, self.multiplier)
    }
}

impl From<BollingerBandsOutput> for (f64, f64, f64) {
    fn from(mo: BollingerBandsOutput) -> Self {
        (mo.average, mo.upper, mo.lower)
    }
}

#[cfg(test)]
mod tests {
    use crate::helper::round;

    use super::*;

    #[test]
    fn test_new() {
        assert!(BollingerBands::new(0, 2_f64).is_err());
        assert!(BollingerBands::new(1, 2_f64).is_ok());
        assert!(BollingerBands::new(2, 2_f64).is_ok());
    }

    #[test]
    fn test_next() {
        let mut bb = BollingerBands::new(3, 2.0_f64).unwrap();

        let a = bb.next(2.0).unwrap();
        let b = bb.next(5.0).unwrap();
        let c = bb.next(1.0).unwrap();
        let d = bb.next(6.25).unwrap();

        assert_eq!(round(a.average), 2.0);
        assert_eq!(round(b.average), 3.5);
        assert_eq!(round(c.average), 2.667);
        assert_eq!(round(d.average), 4.083);

        assert_eq!(round(a.upper), 2.0);
        assert_eq!(round(b.upper), 6.5);
        assert_eq!(round(c.upper), 6.066);
        assert_eq!(round(d.upper), 8.562);

        assert_eq!(round(a.lower), 2.0);
        assert_eq!(round(b.lower), 0.5);
        assert_eq!(round(c.lower), -0.733);
        assert_eq!(round(d.lower), -0.395);
    }

    #[test]
    fn test_reset() {
        let mut bb = BollingerBands::new(5, 2.0_f64).unwrap();

        let out = bb.next(3.0).unwrap();

        assert_eq!(out.average, 3.0);
        assert_eq!(out.upper, 3.0);
        assert_eq!(out.lower, 3.0);

        bb.next(2.5).unwrap();
        bb.next(3.5).unwrap();
        bb.next(4.0).unwrap();

        let out = bb.next(2.0).unwrap();

        assert_eq!(out.average, 3.0);
        assert_eq!(round(out.upper), 4.414);
        assert_eq!(round(out.lower), 1.586);

        bb.reset();
        let out = bb.next(3.0).unwrap();
        assert_eq!(out.average, 3.0);
        assert_eq!(out.upper, 3.0);
        assert_eq!(out.lower, 3.0);
    }

    #[test]
    fn test_default() {
        BollingerBands::default();
    }

    #[test]
    fn test_display() {
        let bb = BollingerBands::new(10, 3.0_f64).unwrap();
        assert_eq!(format!("{}", bb), "BB(10, 3)");
    }
}
