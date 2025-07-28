#[cfg(feature = "chipa_lang")]
use chipa_lang_utils::Lang;
use chipa_ta_utils::{TaUtilsError, TaUtilsResult};

use core::fmt;
use serde::{Deserialize, Serialize};

use crate::{
    error::TaResult,
    traits::{Candle, IndicatorTrait, Next, Period, Reset},
    types::OutputShape,
    types::Queue,
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "chipa_lang", derive(Lang))]
#[cfg_attr(feature = "chipa_lang", ct(grammar(Smma(period)), may_fail))]
pub struct SmoothedMovingAverage {
    period: usize,
    queue: Queue<f64>,
    smma: Option<f64>,
}

/// Custom implementation of the Serialize and Deserialize traits for SmoothedMovingAverage
impl Serialize for SmoothedMovingAverage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct SmoothedMovingAverageVisitor {
            period: usize,
        }
        SmoothedMovingAverageVisitor {
            period: self.period,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SmoothedMovingAverage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct SmoothedMovingAverageVisitor {
            period: usize,
        }
        let visitor = SmoothedMovingAverageVisitor::deserialize(deserializer)?;
        Ok(Self {
            period: visitor.period,
            queue: Queue::new(visitor.period).map_err(serde::de::Error::custom)?,
            smma: None,
        })
    }
}

impl SmoothedMovingAverage {
    pub fn new(period: usize) -> TaResult<Self> {
        if period < 2 {
            return Err(TaUtilsError::InvalidParameter(period.to_string()).into());
        }
        // Initialize the queue with an element so once it returns the first value, it has a valid state (as the returned value will be the dummy one we pass at the start)
        let mut queue = Queue::new(period)?;

        queue.push(0.0);
        Ok(Self {
            period,
            queue,
            smma: None,
        })
    }
}

impl IndicatorTrait for SmoothedMovingAverage {
    fn output_shape(&self) -> OutputShape {
        OutputShape::Shape(1)
    }
}

impl Period for SmoothedMovingAverage {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<f64> for SmoothedMovingAverage {
    type Output = f64;

    fn next(&mut self, input: f64) -> TaUtilsResult<Self::Output> {
        // Fill the queue until we have enough values for the first average
        if self.smma.is_none() {
            match self.queue.push(input) {
                Some(_) => {
                    let sum: f64 = self.queue.iter().sum();
                    let avg = sum / self.period as f64;
                    self.smma = Some(avg);
                    return Ok(avg);
                }
                None => return Ok(input),
            }
        }

        // Calculate subsequent SMMAs
        if let Some(prev_smma) = self.smma {
            let smma = (prev_smma * (self.period as f64 - 1.0) + input) / self.period as f64;
            self.smma = Some(smma);
            Ok(smma)
        } else {
            // Should not happen, but fallback
            Ok(input)
        }
    }
}

impl<T: Candle> Next<&T> for SmoothedMovingAverage {
    type Output = f64;

    fn next(&mut self, input: &T) -> TaUtilsResult<Self::Output> {
        self.next(input.close())
    }
}

impl Reset for SmoothedMovingAverage {
    fn reset(&mut self) {
        self.queue = Queue::new(self.period).unwrap();
        self.queue.push(0.0); // Push a dummy value to maintain the initial state
        self.smma = None;
    }
}

impl Default for SmoothedMovingAverage {
    fn default() -> Self {
        Self::new(14).unwrap()
    }
}

impl fmt::Display for SmoothedMovingAverage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SMMA({})", self.period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smma_basic() {
        let mut smma = SmoothedMovingAverage::new(3).unwrap();
        assert_eq!(smma.next(1.0).unwrap(), 1.0);
        assert_eq!(smma.next(2.0).unwrap(), 2.0);
        // First average: (1+2+3)/3 = 2.0
        assert_eq!(smma.next(3.0).unwrap(), 2.0);
        // Next: (2.0*2 + 4)/3 = 2.67
        assert!((smma.next(4.0).unwrap() - 2.6666667).abs() < 1e-6);
        // Next: (2.67*2 + 5)/3 = 3.44
        assert!((smma.next(5.0).unwrap() - 3.4444444).abs() < 1e-6);
    }
}
