use std::fmt;

use crate::error::{TaError, TaResult};
use crate::traits::{Candle, IndicatorTrait, Next, Period, Reset};
use crate::types::OutputShape;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct StandardDeviation {
    period: usize,
    #[serde(skip)]
    index: usize,
    #[serde(skip)]
    count: usize,
    #[serde(skip)]
    m: f64,
    #[serde(skip)]
    m2: f64,
    #[serde(skip)]
    deque: Box<[f64]>,
}

/// Custom implementation of the Deserialize trait for StandardDeviation
impl<'de> Deserialize<'de> for StandardDeviation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct StandardDeviationVisitor {
            period: usize,
        }
        let visitor = StandardDeviationVisitor::deserialize(deserializer)?;
        StandardDeviation::new(visitor.period).map_err(serde::de::Error::custom)
    }
}

impl StandardDeviation {
    pub fn new(period: usize) -> TaResult<Self> {
        match period {
            0 => Err(TaError::InvalidParameter(
                "Period must be greater than 0".to_string(),
            )),
            _ => Ok(Self {
                period,
                index: 0,
                count: 0,
                m: 0.0,
                m2: 0.0,
                deque: vec![0.0; period].into_boxed_slice(),
            }),
        }
    }

    pub(super) fn mean(&self) -> f64 {
        self.m
    }
}

impl IndicatorTrait for StandardDeviation {
    fn output_shape(&self) -> OutputShape {
        OutputShape::Shape(1)
    }
}

impl Period for StandardDeviation {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<f64> for StandardDeviation {
    type Output = f64;

    fn next(&mut self, input: f64) -> TaResult<Self::Output> {
        let old_val = self.deque[self.index];
        self.deque[self.index] = input;

        self.index = if self.index + 1 < self.period {
            self.index + 1
        } else {
            0
        };

        if self.count < self.period {
            self.count += 1;
            let delta = input - self.m;
            self.m += delta / self.count as f64;
            let delta2 = input - self.m;
            self.m2 += delta * delta2;
        } else {
            let delta = input - old_val;
            let old_m = self.m;
            self.m += delta / self.period as f64;
            let delta2 = input - self.m + old_val - old_m;
            self.m2 += delta * delta2;
        }
        if self.m2 < 0.0 {
            self.m2 = 0.0;
        }

        Ok((self.m2 / self.count as f64).sqrt())
    }
}

impl<T: Candle> Next<&T> for StandardDeviation {
    type Output = f64;

    fn next(&mut self, input: &T) -> TaResult<Self::Output> {
        self.next(input.close())
    }
}

impl Reset for StandardDeviation {
    fn reset(&mut self) {
        self.index = 0;
        self.count = 0;
        self.m = 0.0;
        self.m2 = 0.0;
        for i in 0..self.period {
            self.deque[i] = 0.0;
        }
    }
}

impl Default for StandardDeviation {
    fn default() -> Self {
        Self::new(9).unwrap()
    }
}

impl fmt::Display for StandardDeviation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SD({})", self.period)
    }
}

#[cfg(test)]
mod tests {
    use crate::helper::round;

    use super::*;

    #[test]
    fn test_new() {
        assert!(StandardDeviation::new(0).is_err());
        assert!(StandardDeviation::new(1).is_ok());
    }

    #[test]
    fn test_next() {
        let mut sd = StandardDeviation::new(4).unwrap();
        assert_eq!(sd.next(10.0).unwrap(), 0.0);
        assert_eq!(sd.next(20.0).unwrap(), 5.0);
        assert_eq!(round(sd.next(30.0).unwrap()), 8.165);
        assert_eq!(round(sd.next(20.0).unwrap()), 7.071);
        assert_eq!(round(sd.next(10.0).unwrap()), 7.071);
        assert_eq!(round(sd.next(100.0).unwrap()), 35.355);
    }

    #[test]
    fn test_next_floating_point_error() {
        let mut sd = StandardDeviation::new(6).unwrap();
        assert_eq!(sd.next(1.872).unwrap(), 0.0);
        assert_eq!(round(sd.next(1.0).unwrap()), 0.436);
        assert_eq!(round(sd.next(1.0).unwrap()), 0.411);
        assert_eq!(round(sd.next(1.0).unwrap()), 0.378);
        assert_eq!(round(sd.next(1.0).unwrap()), 0.349);
        assert_eq!(round(sd.next(1.0).unwrap()), 0.325);
        assert_eq!(round(sd.next(1.0).unwrap()), 0.0);
    }

    #[test]
    fn test_next_same_values() {
        let mut sd = StandardDeviation::new(3).unwrap();
        assert_eq!(sd.next(4.2).unwrap(), 0.0);
        assert_eq!(sd.next(4.2).unwrap(), 0.0);
        assert_eq!(sd.next(4.2).unwrap(), 0.0);
        assert_eq!(sd.next(4.2).unwrap(), 0.0);
    }

    #[test]
    fn test_reset() {
        let mut sd = StandardDeviation::new(4).unwrap();
        assert_eq!(sd.next(10.0).unwrap(), 0.0);
        assert_eq!(sd.next(20.0).unwrap(), 5.0);
        assert_eq!(round(sd.next(30.0).unwrap()), 8.165);

        sd.reset();
        assert_eq!(sd.next(20.0).unwrap(), 0.0);
    }

    #[test]
    fn test_default() {
        StandardDeviation::default();
    }

    #[test]
    fn test_display() {
        let sd = StandardDeviation::new(5).unwrap();
        assert_eq!(format!("{}", sd), "SD(5)");
    }
}
