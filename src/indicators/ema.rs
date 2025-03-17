use core::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    defaults::default_true,
    error::{TaError, TaResult},
    traits::{Candle, Indicator, Next, Period, Reset},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExponentialMovingAverage {
    period: usize,
    k: f64,
    #[serde(skip)]
    current: f64,
    #[serde(skip)]
    #[serde(default = "default_true")]
    is_new: bool,
}

impl Indicator for ExponentialMovingAverage {}

impl ExponentialMovingAverage {
    pub fn new(period: usize) -> TaResult<Self> {
        match period {
            0 => Err(TaError::InvalidParameter("0".to_string())),
            _ => Ok(Self {
                period,
                k: 2.0 / (period + 1) as f64,
                current: 0.0,
                is_new: true,
            }),
        }
    }
}

impl Period for ExponentialMovingAverage {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<f64> for ExponentialMovingAverage {
    type Output = f64;

    fn next(&mut self, input: f64) -> TaResult<Self::Output> {
        if self.is_new {
            self.is_new = false;
            self.current = input;
        } else {
            self.current = self.k * input + (1.0 - self.k) * self.current;
        }
        Ok(self.current)
    }
}

impl<T: Candle> Next<&T> for ExponentialMovingAverage {
    type Output = f64;

    fn next(&mut self, input: &T) -> TaResult<Self::Output> {
        self.next(input.close())
    }
}

impl Reset for ExponentialMovingAverage {
    fn reset(&mut self) {
        self.current = 0.0;
        self.is_new = true;
    }
}

impl Default for ExponentialMovingAverage {
    fn default() -> Self {
        Self::new(9).unwrap()
    }
}

impl fmt::Display for ExponentialMovingAverage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EMA({})", self.period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        assert!(ExponentialMovingAverage::new(0).is_err());
        assert!(ExponentialMovingAverage::new(1).is_ok());
    }

    #[test]
    fn test_next() {
        let mut ema = ExponentialMovingAverage::new(3).unwrap();

        assert_eq!(ema.next(2.0).unwrap(), 2.0);
        assert_eq!(ema.next(5.0).unwrap(), 3.5);
        assert_eq!(ema.next(1.0).unwrap(), 2.25);
        assert_eq!(ema.next(6.25).unwrap(), 4.25);

        // let mut ema = ExponentialMovingAverage::new(3).unwrap();
        // let bar1 = Bar::new().close(2);
        // let bar2 = Bar::new().close(5);
        // assert_eq!(ema.next(&bar1), 2.0);
        // assert_eq!(ema.next(&bar2), 3.5);
    }

    #[test]
    fn test_reset() {
        let mut ema = ExponentialMovingAverage::new(5).unwrap();

        assert_eq!(ema.next(4.0).unwrap(), 4.0);
        ema.next(10.0).unwrap();
        ema.next(15.0).unwrap();
        ema.next(20.0).unwrap();
        assert_ne!(ema.next(4.0).unwrap(), 4.0);

        ema.reset();
        assert_eq!(ema.next(4.0).unwrap(), 4.0);
    }

    #[test]
    fn test_default() {
        ExponentialMovingAverage::default();
    }

    #[test]
    fn test_display() {
        let ema = ExponentialMovingAverage::new(7).unwrap();
        assert_eq!(format!("{}", ema), "EMA(7)");
    }

    #[test]
    fn test_serialize() {
        let sma = ExponentialMovingAverage::new(3).unwrap();
        let sma_string = serde_json::to_string(&sma).unwrap();
        assert_eq!(sma_string, r#"{"period":3,"k":0.5}"#)
    }

    #[test]
    fn test_deserialize() {
        let sma_string = r#"{"period":3,"k":0.5}"#;
        let sma_128 = ExponentialMovingAverage::new(3).unwrap();
        let sma_deserialized: ExponentialMovingAverage = serde_json::from_str(sma_string).unwrap();
        assert_eq!(sma_deserialized, sma_128)
    }
}
