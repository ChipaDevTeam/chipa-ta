use core::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    error::TaResult,
    indicators::ExponentialMovingAverage as Ema,
    traits::{Candle, Next, Period, Reset},
};

#[derive(Serialize, Deserialize)]
struct RsiSerializer {
    period: usize,
    #[serde(skip_serializing_if="Option::is_none")]
    up_ema: Option<Ema>,
    #[serde(skip_serializing_if="Option::is_none")]
    down_ema: Option<Ema>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelativeStrengthIndex {
    period: usize,
    up_ema: Ema,
    down_ema: Ema,
    prev_val: f64,
    is_new: bool,
}

impl Serialize for RelativeStrengthIndex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Only serialize the period
        match self.period == self.up_ema.period() && self.period == self.down_ema.period() {
            true => RsiSerializer {
                period: self.period,
                down_ema: None,
                up_ema: None
            }
            .serialize(serializer),
            false => RsiSerializer {
                period: self.period,
                down_ema: Some(self.down_ema.clone()),
                up_ema: Some(self.up_ema.clone())
            }
            .serialize(serializer)
        }
        
        
    }
}

impl<'de> Deserialize<'de> for RelativeStrengthIndex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Deserialize into the temporary struct
        let serializer = RsiSerializer::deserialize(deserializer)?;
        
        // Create a new RelativeStrengthIndex with the period
        Ok(RelativeStrengthIndex {
            period: serializer.period,
            up_ema: Ema::new(serializer.period).map_err(serde::de::Error::custom)?,
            down_ema: Ema::new(serializer.period).map_err(serde::de::Error::custom)?,
            prev_val: 0.0,
            is_new: true,
        })
    }
}


impl RelativeStrengthIndex {
    pub fn new(period: usize) -> TaResult<Self> {
        Ok(Self {
            period,
            up_ema: Ema::new(period)?,
            down_ema: Ema::new(period)?,
            prev_val: 0.0,
            is_new: true,
        })
    }
}

impl Period for RelativeStrengthIndex {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<f64> for RelativeStrengthIndex {
    type Output = f64;

    fn next(&mut self, input: f64) -> TaResult<Self::Output> {
        let mut up = 0.0;
        let mut down = 0.0;

        if self.is_new {
            self.is_new = false;
            // Initialize with some small seed numbers to avoid division by zero
            up = 0.1;
            down = 0.1;
        } else if input > self.prev_val {
            up = input - self.prev_val;
        } else {
            down = self.prev_val - input;
        }

        self.prev_val = input;
        let up_ema = self.up_ema.next(up)?;
        let down_ema = self.down_ema.next(down)?;
        Ok(100.0 * up_ema / (up_ema + down_ema))
    }
}

impl<T: Candle> Next<&T> for RelativeStrengthIndex {
    type Output = f64;

    fn next(&mut self, input: &T) -> TaResult<Self::Output> {
        self.next(input.close())
    }
}

impl Reset for RelativeStrengthIndex {
    fn reset(&mut self) {
        self.is_new = true;
        self.prev_val = 0.0;
        self.up_ema.reset();
        self.down_ema.reset();
    }
}

impl Default for RelativeStrengthIndex {
    fn default() -> Self {
        Self::new(14).unwrap()
    }
}

impl fmt::Display for RelativeStrengthIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RSI({})", self.period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        assert!(RelativeStrengthIndex::new(0).is_err());
        assert!(RelativeStrengthIndex::new(1).is_ok());
    }

    #[test]
    fn test_next() {
        let mut rsi = RelativeStrengthIndex::new(3).unwrap();
        assert_eq!(rsi.next(10.0).unwrap(), 50.0);
        assert_eq!(rsi.next(10.5).unwrap().round(), 86.0);
        assert_eq!(rsi.next(10.0).unwrap().round(), 35.0);
        assert_eq!(rsi.next(9.5).unwrap().round(), 16.0);
    }

    #[test]
    fn test_reset() {
        let mut rsi = RelativeStrengthIndex::new(3).unwrap();
        assert_eq!(rsi.next(10.0).unwrap(), 50.0);
        assert_eq!(rsi.next(10.5).unwrap().round(), 86.0);

        rsi.reset();
        assert_eq!(rsi.next(10.0).unwrap().round(), 50.0);
        assert_eq!(rsi.next(10.5).unwrap().round(), 86.0);
    }

    #[test]
    fn test_default() {
        RelativeStrengthIndex::default();
    }

    #[test]
    fn test_display() {
        let rsi = RelativeStrengthIndex::new(16).unwrap();
        assert_eq!(format!("{}", rsi), "RSI(16)");
    }

    #[test]
    fn test_serialize() {
        let sma = RelativeStrengthIndex::new(3).unwrap();
        let sma_string = serde_json::to_string(&sma).unwrap();
        assert_eq!(
            sma_string,
            r#"{"period":3}"#
        )
    }

    #[test]
    fn test_deserialize() {
        let sma_string =
            r#"{"period":3}"#;
        let sma_128 = RelativeStrengthIndex::new(3).unwrap();
        let sma_deserialized: RelativeStrengthIndex = serde_json::from_str(sma_string).unwrap();
        assert_eq!(sma_deserialized, sma_128)
    }
}
