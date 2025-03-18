use core::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    error::TaResult,
    helper_types::Queue,
    traits::{Candle, Indicator, Next, Period, Reset},
    types::Status,
};

use super::tr::TrueRange;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AverageTrueRange {
    period: usize,
    #[serde(skip)]
    true_range: TrueRange,
    #[serde(skip_serializing_if = "Option::is_none")]
    atr: Option<f64>,
    #[serde(skip)]
    status: Status<(), Queue<f64>, Queue<f64>>,
}

impl Indicator for AverageTrueRange {}

impl AverageTrueRange {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            true_range: TrueRange::new(),
            atr: None,
            status: Status::Initial(()),
        }
    }
}

impl Default for AverageTrueRange {
    fn default() -> Self {
        Self::new(14) // Default period is 14
    }
}

impl fmt::Display for AverageTrueRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ATR({})", self.period)
    }
}

impl Next<f64> for AverageTrueRange {
    type Output = f64;

    fn next(&mut self, input: f64) -> TaResult<Self::Output> {
        let (status, res) = match self.status.clone() {
            Status::Initial(()) => {
                let mut queue = Queue::new(self.period)?;
                queue.next_with(input);
                (Status::Progress(queue), None)
            }
            Status::Progress(mut queue) => {
                self.true_range.reset();
                if let Some(itm) = queue.next_with(input) {
                    self.true_range.next(itm)?;
                    let sum = self
                        .true_range
                        .next_batched(queue.iter().cloned())?
                        .iter()
                        .sum::<f64>();
                    let atr = sum / queue.len() as f64;
                    (Status::Completed(queue), Some(atr))
                } else {
                    let sum = self
                        .true_range
                        .next_batched(queue.iter().cloned())?
                        .iter()
                        .sum::<f64>();
                    let atr = sum / queue.len() as f64;
                    (Status::Progress(queue), Some(atr))
                }
            }
            Status::Completed(mut queue) => {
                queue.next_with(input);

                if let Some(prev_atr) = self.atr {
                    let tr = self.true_range.next(input)?;
                    let new_atr = (prev_atr * (self.period - 1) as f64 + tr) / self.period as f64;
                    (Status::Completed(queue), Some(new_atr))
                } else {
                    return Err(crate::error::TaError::Unexpected(
                        "This should never happend".to_string(),
                    ));
                }
            }
        };
        self.status = status;
        self.atr = res;
        Ok(self.atr.unwrap_or(0.0))
    }
}

impl<T: Candle> Next<&T> for AverageTrueRange {
    type Output = f64;
    fn next(&mut self, input: &T) -> TaResult<Self::Output> {
        self.next(input.close())
    }
}

impl Period for AverageTrueRange {
    fn period(&self) -> usize {
        self.period
    }
}

impl Reset for AverageTrueRange {
    fn reset(&mut self) {
        self.true_range.reset();
        self.atr = None;
    }
}

#[cfg(test)]
mod tests {
    use crate::helper_types::Bar;

    use super::*;

    #[test]
    fn test_atr_initial_calculation() -> TaResult<()> {
        let mut atr = AverageTrueRange::new(3);
        let candles = [Bar::new().set_close(0.0),
            Bar::new().set_close(1.0),
            Bar::new().set_close(3.0)];

        let atr1 = atr.next(&candles[0])?;
        let atr2 = atr.next(&candles[1])?;
        let atr3 = atr.next(&candles[2])?;
        assert_eq!(atr1, 0.0);
        assert_eq!(atr2, 0.5);
        assert_eq!(atr3, 1.0);

        Ok(())
    }
}
