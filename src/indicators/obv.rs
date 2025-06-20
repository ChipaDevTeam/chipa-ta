use core::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    error::TaResult,
    traits::{Candle, Indicator, Next, Period, Reset}, types::OutputShape,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OnBalanceVolume {
    #[serde(skip)]
    obv: f64,
    #[serde(skip)]
    prev_close: Option<f64>,
}

impl Default for OnBalanceVolume {
    fn default() -> Self {
        Self::new()
    }
}

impl OnBalanceVolume {
    pub fn new() -> Self {
        Self {
            obv: 0.0,
            prev_close: None,
        }
    }
}

impl Indicator for OnBalanceVolume {
    fn output_shape(&self) -> OutputShape {
        OutputShape::Shape(1)
    }
}

impl fmt::Display for OnBalanceVolume {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OBV")
    }
}

impl Period for OnBalanceVolume {
    fn period(&self) -> usize {
        1 // OBV is a single value indicator, so period is always 1
    }
}

impl Reset for OnBalanceVolume {
    fn reset(&mut self) {
        self.obv = 0.0;
        self.prev_close = None;
    }
}

impl<C: Candle> Next<&C> for OnBalanceVolume {
    type Output = f64;

    fn next(&mut self, candle: &C) -> TaResult<Self::Output> {
        match self.prev_close {
            Some(prev) => {
                if candle.close() > prev {
                    self.obv += candle.volume();
                } else if candle.close() < prev {
                    self.obv -= candle.volume();
                }
            }
            None => {
                self.obv = candle.volume(); // Initialize OBV with the first volume
            }
        }
        self.prev_close = Some(candle.close());

        Ok(self.obv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Candle;

    #[derive(Debug, Clone)]
    struct TestCandle {
        close: f64,
        volume: f64,
    }

    impl Candle for TestCandle {
        fn close(&self) -> f64 {
            self.close
        }

        fn volume(&self) -> f64 {
            self.volume
        }

        fn price(&self) -> f64 {
            self.close()
        }
    }

    #[test]
    fn test_on_balance_volume() {
        let mut obv = OnBalanceVolume::new();
        let candles = vec![
            TestCandle {
                close: 100.0,
                volume: 10.0,
            },
            TestCandle {
                close: 105.0,
                volume: 20.0,
            },
            TestCandle {
                close: 102.0,
                volume: 15.0,
            },
            TestCandle {
                close: 108.0,
                volume: 25.0,
            },
            TestCandle {
                close: 104.0,
                volume: 30.0,
            },
        ];

        for candle in candles {
            let result = obv.next(&candle).unwrap();
            println!(
                "OBV after candle with close {} and volume {}: {}",
                candle.close(),
                candle.volume(),
                result
            );
        }
    }

    #[test]
    fn test_serialize() {
        let obv: OnBalanceVolume = OnBalanceVolume::new();
        let obv_string = serde_json::to_string(&obv).unwrap();
        assert_eq!(obv_string, r#"{}"#)
    }

    #[test]
    fn test_deserialize() {
        let obv_string = r#"{}"#;
        let obv_check = OnBalanceVolume::new();
        let obv_deserialized: OnBalanceVolume = serde_json::from_str(obv_string).unwrap();
        assert_eq!(obv_deserialized, obv_check)
    }
}
