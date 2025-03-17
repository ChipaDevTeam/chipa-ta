use core::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    error::{TaError, TaResult},
    helper_types::Queue,
    traits::{Candle, Indicator, Next, Period, Reset},
    types::Status,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimpleMovingAverage {
    // index: Cycle,
    period: usize,
    #[serde(skip)]
    status: Status<(), Queue<f64>, Queue<f64>>,
}

impl Indicator for SimpleMovingAverage {}

impl SimpleMovingAverage {
    pub fn new(period: usize) -> TaResult<Self> {
        match period {
            0 => Err(TaError::InvalidParameter("0".to_string())),
            _ => Ok(Self {
                period,
                status: Status::Initial(()),
            }),
        }
    }
}

impl Period for SimpleMovingAverage {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<f64> for SimpleMovingAverage {
    type Output = f64;

    fn next(&mut self, input: f64) -> TaResult<Self::Output> {
        let (status, res) = match self.status.clone() {
            Status::Initial(_) => {
                let mut queue = Queue::new(self.period)?;
                queue.next_with(input);
                (Status::Progress(queue), input)
            }
            Status::Progress(mut queue) | Status::Completed(mut queue) => {
                if queue.next_with(input).is_some() {
                    let res = queue.iter().sum::<f64>() / self.period as f64;
                    (Status::Completed(queue), res)
                } else {
                    let res = queue.iter().sum::<f64>() / queue.len() as f64;
                    (Status::Progress(queue), res)
                }
            }
        };
        self.status = status;
        Ok(res)
    }
}

impl<T: Candle> Next<&T> for SimpleMovingAverage {
    type Output = f64;

    fn next(&mut self, input: &T) -> TaResult<Self::Output> {
        self.next(input.close())
    }
}

impl Reset for SimpleMovingAverage {
    fn reset(&mut self) {
        self.status = Status::Initial(());
    }
}

impl Default for SimpleMovingAverage {
    fn default() -> Self {
        Self::new(9).unwrap()
    }
}

impl fmt::Display for SimpleMovingAverage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SMA({})", self.period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new() {
        assert!(SimpleMovingAverage::new(0).is_err());
        assert!(SimpleMovingAverage::new(1).is_ok());
    }

    #[test]
    fn test_next() {
        let mut sma = SimpleMovingAverage::new(4).unwrap();
        assert_eq!(sma.next(4.0).unwrap(), 4.0);
        assert_eq!(sma.next(5.0).unwrap(), 4.5);
        assert_eq!(sma.next(6.0).unwrap(), 5.0);
        assert_eq!(sma.next(6.0).unwrap(), 5.25);
        assert_eq!(sma.next(6.0).unwrap(), 5.75);
        assert_eq!(sma.next(6.0).unwrap(), 6.0);
        assert_eq!(sma.next(2.0).unwrap(), 5.0);
    }

    #[test]
    fn test_reset() {
        let mut sma = SimpleMovingAverage::new(4).unwrap();
        assert_eq!(sma.next(4.0).unwrap(), 4.0);
        assert_eq!(sma.next(5.0).unwrap(), 4.5);
        assert_eq!(sma.next(6.0).unwrap(), 5.0);

        sma.reset();
        assert_eq!(sma.next(99.0).unwrap(), 99.0);
    }

    #[test]
    fn test_default() {
        SimpleMovingAverage::default();
    }

    #[test]
    fn test_display() {
        let sma = SimpleMovingAverage::new(5).unwrap();
        assert_eq!(format!("{}", sma), "SMA(5)");
    }

    #[test]
    fn test_serialize() {
        let sma = SimpleMovingAverage::new(128).unwrap();
        let sma_string = serde_json::to_string(&sma).unwrap();
        assert_eq!(sma_string, r#"{"period":128}"#)
    }

    #[test]
    fn test_deserialize() {
        let sma_string = r#"{"period":128}"#;
        let sma_128 = SimpleMovingAverage::new(128).unwrap();
        let sma_deserialized: SimpleMovingAverage = serde_json::from_str(sma_string).unwrap();
        assert_eq!(sma_deserialized, sma_128)
    }
}
