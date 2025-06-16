// Mean Absolute Error (MAE) indicator implementation for chipa-ta
// Based on ta-rs and TA-Lib

use crate::error::{TaError, TaResult};
use crate::traits::{Candle, Next, Period, Reset};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeanAbsoluteError {
    pub period: usize,
    #[serde(skip)]
    pub values: Vec<f64>,
    #[serde(skip)]
    pub mean: f64,
}

impl MeanAbsoluteError {
    pub fn new(period: usize) -> TaResult<Self> {
        if period == 0 {
            return Err(TaError::InvalidParameter(
                "Period must be greater than 0".to_string(),
            ));
        }
        Ok(Self {
            period,
            values: Vec::new(),
            mean: 0.0,
        })
    }
}

impl Next<f64> for MeanAbsoluteError {
    type Output = f64;
    fn next(&mut self, input: f64) -> TaResult<Self::Output> {
        self.values.push(input);
        if self.values.len() > self.period {
            self.values.remove(0);
        }
        self.mean = self.values.iter().sum::<f64>() / self.values.len() as f64;
        let mae = self
            .values
            .iter()
            .map(|v| (v - self.mean).abs())
            .sum::<f64>()
            / self.values.len() as f64;
        Ok(mae)
    }
}

impl<T: Candle> Next<&T> for MeanAbsoluteError {
    type Output = f64;

    fn next(&mut self, input: &T) -> TaResult<Self::Output> {
        self.next(input.close())
    }
}


impl Period for MeanAbsoluteError {
    fn period(&self) -> usize {
        self.period
    }
}

impl Reset for MeanAbsoluteError {
    fn reset(&mut self) {
        self.values.clear();
        self.mean = 0.0;
    }
}
