use serde::{Deserialize, Serialize};

use crate::{error::TaResult, traits::{Candle, Next, Period, Reset}};

use super::SimpleMovingAverage as Sma;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AwesomeOscillator {
    long_sma: Sma,
    short_sma: Sma,
}

/// Custom implementation of the Deserialize trait for AwesomeOscillator
impl<'de> Deserialize<'de> for AwesomeOscillator {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct AwesomeOscillatorVisitor {
            short_period: usize,
            long_period: usize,
        }
        let visitor = AwesomeOscillatorVisitor::deserialize(deserializer)?;
        AwesomeOscillator::new(visitor.short_period, visitor.long_period)
            .map_err(serde::de::Error::custom)
    }
}

impl Serialize for AwesomeOscillator {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct AwesomeOscillatorVisitor {
            short_period: usize,
            long_period: usize,
        }
        AwesomeOscillatorVisitor {
            short_period: self.short_sma.period(),
            long_period: self.long_sma.period(),
        }
        .serialize(serializer)
    }
}

impl Period for AwesomeOscillator {
    fn period(&self) -> usize {
        self.long_sma.period().max(self.short_sma.period())
    }
}

impl Reset for AwesomeOscillator {
    fn reset(&mut self) {
        self.long_sma.reset();
        self.short_sma.reset();
    }
}

impl AwesomeOscillator {
    pub fn new(short_period: usize, long_period: usize) -> TaResult<Self> {
        Ok(Self {
            long_sma: Sma::new(long_period)?,
            short_sma: Sma::new(short_period)?,
        })
    }
}

impl Next<f64> for AwesomeOscillator {
    type Output = f64;

    fn next(&mut self, input: f64) -> TaResult<Self::Output> {
        let short_value = self.short_sma.next(input)?;
        let long_value = self.long_sma.next(input)?;
        Ok(short_value - long_value)
    }
}

impl<C: Candle> Next<&C> for AwesomeOscillator {
    type Output = f64;

    fn next(&mut self, input: &C) -> TaResult<Self::Output> {
        let mp = (input.high() + input.low()) / 2.0; // Median Price
        let short_value = self.short_sma.next(mp)?;
        let long_value = self.long_sma.next(mp)?;
        Ok(short_value - long_value)
    }
}