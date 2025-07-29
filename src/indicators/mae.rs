// Mean Absolute Error (MAE) indicator implementation for chipa-ta
// Based on ta-rs and TA-Lib

#[cfg(feature = "chipa_lang")]
use chipa_lang_utils::Lang;
use chipa_ta_utils::{TaUtilsError, TaUtilsResult};

use core::fmt;

use crate::error::TaResult;
use crate::traits::{Candle, IndicatorTrait, Next, Period, Reset};
use crate::types::OutputShape;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "chipa_lang", derive(Lang))]
#[cfg_attr(feature = "chipa_lang", ct(grammar(Mae(period)), may_fail))]
pub struct MeanAbsoluteError {
    pub period: usize,
    #[serde(skip)]
    pub values: Vec<f64>,
    #[serde(skip)]
    pub mean: f64,
}

impl Default for MeanAbsoluteError {
    fn default() -> Self {
        Self {
            period: 14,
            values: Vec::new(),
            mean: 0.0,
        }
    }
}

impl fmt::Display for MeanAbsoluteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MAE({})", self.period)
    }
}

impl MeanAbsoluteError {
    pub fn new(period: usize) -> TaResult<Self> {
        if period == 0 {
            return Err(TaUtilsError::InvalidParameter(
                "Period must be greater than 0".to_string(),
            ).into());
        }
        Ok(Self {
            period,
            values: Vec::new(),
            mean: 0.0,
        })
    }
}

impl IndicatorTrait for MeanAbsoluteError {
    fn output_shape(&self) -> OutputShape {
        OutputShape::Shape(1)
    }
}

impl Next<f64> for MeanAbsoluteError {
    type Output = f64;
    fn next(&mut self, input: f64) -> TaUtilsResult<Self::Output> {
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

    fn next(&mut self, input: &T) -> TaUtilsResult<Self::Output> {
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
