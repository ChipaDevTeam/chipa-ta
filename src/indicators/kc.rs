use core::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    error::{TaError, TaResult},
    traits::{Candle, Indicator, Next, Period, Reset},
    types::OutputShape,
};

use super::{AverageTrueRange as Atr, ExponentialMovingAverage as Ema};

#[derive(Debug, PartialEq, Clone)]
pub struct KeltnerChannel {
    multiplier: f64,
    atr: Atr,
    ema: Ema,
}

/// Custom implementation of the Deserialize trait for KeltnerChannel
impl<'de> Deserialize<'de> for KeltnerChannel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct KeltnerChannelVisitor {
            period: usize,
            multiplier: f64,
        }
        // Deserialize the KeltnerChannel struct
        let kcv = KeltnerChannelVisitor::deserialize(deserializer)?;

        // Initialize the AverageTrueRange and ExponentialMovingAverage with the period
        let kc = KeltnerChannel {
            multiplier: kcv.multiplier,
            atr: Atr::new(kcv.period).map_err(serde::de::Error::custom)?,
            ema: Ema::new(kcv.period).map_err(serde::de::Error::custom)?,
        };

        Ok(kc)
    }
}

impl Serialize for KeltnerChannel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct KeltnerChannelVisitor {
            period: usize,
            multiplier: f64,
        }

        // Serialize the KeltnerChannel struct
        KeltnerChannelVisitor {
            period: self.ema.period(),
            multiplier: self.multiplier,
        }
        .serialize(serializer)
    }
}

pub struct KeltnerChannelOutput {
    pub upper_band: f64,
    pub middle_band: f64,
    pub lower_band: f64,
}

impl Default for KeltnerChannel {
    fn default() -> Self {
        Self::new(10, 2.0).unwrap()
    }
}

impl KeltnerChannel {
    pub fn new(period: usize, multiplier: f64) -> TaResult<Self> {
        if period == 0 {
            return Err(TaError::InvalidParameter(
                "Period must be greater than 0".to_string(),
            ));
        }
        Ok(Self {
            multiplier,
            atr: Atr::new(period)?,
            ema: Ema::new(period)?,
        })
    }

    pub fn multiplier(&self) -> f64 {
        self.multiplier
    }
}

impl Indicator for KeltnerChannel {
    fn output_shape(&self) -> OutputShape {
        OutputShape::Shape(3) // Upper, Middle, Lower
    }
}

impl fmt::Display for KeltnerChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "KC({}, {})", self.ema.period(), self.multiplier)
    }
}

impl Period for KeltnerChannel {
    fn period(&self) -> usize {
        self.ema.period()
    }
}

impl Reset for KeltnerChannel {
    fn reset(&mut self) {
        self.atr.reset();
        self.ema.reset();
    }
}

impl Next<f64> for KeltnerChannel {
    type Output = KeltnerChannelOutput;

    fn next(&mut self, input: f64) -> TaResult<Self::Output> {
        let atr_value = self.atr.next(input)?;
        let ema_value = self.ema.next(input)?;

        let upper_band = ema_value + (self.multiplier * atr_value);
        let lower_band = ema_value - (self.multiplier * atr_value);

        Ok(KeltnerChannelOutput {
            upper_band,
            middle_band: ema_value,
            lower_band,
        })
    }
}

impl<T: Candle> Next<&T> for KeltnerChannel {
    type Output = KeltnerChannelOutput;

    fn next(&mut self, input: &T) -> TaResult<Self::Output> {
        let tp = (input.high() + input.low() + input.close()) / 3.0;
        self.next(tp)
    }
}

impl From<KeltnerChannelOutput> for Vec<f64> {
    fn from(output: KeltnerChannelOutput) -> Self {
        vec![output.upper_band, output.middle_band, output.lower_band]
    }
}
