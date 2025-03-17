use serde::{Deserialize, Serialize};

use crate::error::TaResult;
use crate::indicators::AverageTrueRange as Atr;
use crate::traits::{Candle, Indicator, Next, Period, Reset};

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SuperTrend {
    multiplier: f64,
    period: usize,
    #[serde(skip)]
    atr: Atr,
}

pub struct SuperTrendOutput {
    upper_band: f64,
    lower_band: f64,
}

impl From<(f64, f64)> for SuperTrendOutput {
    fn from(value: (f64, f64)) -> Self {
        Self {
            upper_band: value.0,
            lower_band: value.1,
        }
    }
}

impl From<SuperTrendOutput> for Vec<f64> {
    fn from(value: SuperTrendOutput) -> Self {
        vec![value.lower_band, value.upper_band]
    }
}

impl Indicator for SuperTrend {}

impl Period for SuperTrend {
    fn period(&self) -> usize {
        self.period
    }
}

impl Default for SuperTrend {
    fn default() -> Self {
        Self {
            multiplier: 3.0,
            period: 10,
            atr: Atr::new(10),
        }
    }
}

impl SuperTrend {
    pub fn new(multiplier: f64, period: usize) -> TaResult<Self> {
        Ok(Self {
            multiplier,
            period,
            atr: Atr::new(period),
        })
    }
}

impl Reset for SuperTrend {
    fn reset(&mut self) {
        self.atr.reset();
    }
}

impl Next<f64> for SuperTrend {
    type Output = SuperTrendOutput;

    fn next(&mut self, input: f64) -> TaResult<Self::Output> {
        let atr = self.atr.next(input)?;
        let low = input + self.multiplier * atr;
        let high = input - self.multiplier * atr;
        Ok(SuperTrendOutput::from((low, high)))
    }
}

impl<T: Candle> Next<&T> for SuperTrend {
    type Output = SuperTrendOutput;

    fn next(&mut self, input: &T) -> TaResult<Self::Output> {
        let atr = self.atr.next(input)?;
        let val = (input.high() + input.low()) / 2.0;
        let low = val + self.multiplier * atr;
        let high = val - self.multiplier * atr;
        Ok(SuperTrendOutput::from((low, high)))
    }
}

impl<'de> Deserialize<'de> for SuperTrend {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Define a temporary struct to deserialize into
        #[derive(Deserialize)]
        struct SuperTrendDef {
            multiplier: f64,
            period: usize,
        }

        // Deserialize into the temporary struct
        let def = SuperTrendDef::deserialize(deserializer)?;

        // Use provided values or defaults
        let multiplier = def.multiplier;
        let period = def.period;

        // Create and return the SuperTrend with ATRs initialized based on period
        Ok(SuperTrend {
            multiplier,
            period,
            atr: Atr::new(period),
        })
    }
}
