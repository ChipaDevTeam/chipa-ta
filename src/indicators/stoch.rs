// Stochastic Oscillator implementation for chipa-ta
// Based on ta-rs and TA-Lib

#[cfg(feature = "chipa_lang")]
use chipa_lang_utils::Lang;

use core::fmt;

use serde::{Deserialize, Serialize};

use crate::error::TaResult;
use crate::indicators::SimpleMovingAverage as Sma;
use crate::traits::{Candle, IndicatorTrait};
use crate::traits::{Next, Period, Reset};
use crate::types::OutputShape;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "chipa_lang", derive(Lang))]
#[cfg_attr(
    feature = "chipa_lang",
    ct(grammar(Stoch(period, smoothing_period)), may_fail)
)]
pub struct StochasticOscillator {
    pub period: usize,
    pub smoothing_period: usize, // Smoothing period for %D
    #[serde(skip)]
    pub values: Vec<(f64, f64, f64)>, // (high, low, close)
    #[serde(skip)]
    pub d: Sma,
}

/// Custom implementation of the Deserialize trait for StochasticOscillator
/// to handle the `Sma` and 'values' fields correctly.
impl<'de> Deserialize<'de> for StochasticOscillator {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct StochasticOscillatorVisitor {
            period: usize,
            smoothing_period: usize,
        }
        let visitor = StochasticOscillatorVisitor::deserialize(deserializer)?;
        Ok(Self {
            period: visitor.period,
            smoothing_period: visitor.smoothing_period,
            values: Vec::with_capacity(visitor.period),
            d: Sma::new(visitor.smoothing_period).map_err(serde::de::Error::custom)?,
        })
    }
}

impl Default for StochasticOscillator {
    fn default() -> Self {
        Self {
            period: 14,
            smoothing_period: 3,
            values: Vec::with_capacity(14),
            d: Sma::new(3).unwrap(),
        }
    }
}

impl StochasticOscillator {
    pub fn new(period: usize, smoothing_period: usize) -> TaResult<Self> {
        if period == 0 {
            return Err(crate::error::TaError::InvalidParameter(
                "Period must be greater than 0".to_string(),
            ));
        }
        Ok(Self {
            period,
            smoothing_period,
            values: Vec::with_capacity(period),
            d: Sma::new(smoothing_period)?,
        })
    }
}
impl IndicatorTrait for StochasticOscillator {
    fn output_shape(&self) -> OutputShape {
        OutputShape::Shape(2)
    }
}

impl fmt::Display for StochasticOscillator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "STOCH({}, {})", self.period, self.smoothing_period)
    }
}

impl<T: Candle> Next<&T> for StochasticOscillator {
    type Output = (f64, f64);

    /// Calculates the Stochastic Oscillator value for the given candle.
    /// Returns data in a range from 0.0 to 100.0
    fn next(&mut self, input: &T) -> TaResult<Self::Output> {
        self.values.push((input.high(), input.low(), input.close()));
        if self.values.len() > self.period {
            self.values.remove(0);
        }
        let highest_high = self.values.iter().map(|v| v.0).fold(f64::MIN, f64::max);
        let lowest_low = self.values.iter().map(|v| v.1).fold(f64::MAX, f64::min);
        let close = input.close();
        let k = if highest_high - lowest_low == 0.0 {
            0.0
        } else {
            (close - lowest_low) / (highest_high - lowest_low)
        };
        let d = self.d.next(k)?;
        Ok((k * 100.0, d * 100.0)) // Return %K and %D values
    }
}

impl Period for StochasticOscillator {
    fn period(&self) -> usize {
        self.period
    }
}

impl Reset for StochasticOscillator {
    fn reset(&mut self) {
        self.values.clear();
    }
}

#[cfg(test)]
mod tests {
    use crate::helper_types::Bar;

    use super::*;

    // Helper for approximate float comparison
    fn assert_approx_eq(a: f64, b: f64, epsilon: f64) {
        assert!(
            (a - b).abs() <= epsilon,
            "{a} is not approximately equal to {b}"
        );
    }

    #[test]
    fn test_stochastic_oscillator_basic_calculation() -> TaResult<()> {
        let mut stoch = StochasticOscillator::new(3, 2)?; // 3-period %K, 2-period SMA for %D

        let candles = vec![
            Bar::new().set_high(20.0).set_low(10.0).set_close(15.0), // C1
            Bar::new().set_high(22.0).set_low(12.0).set_close(20.0), // C2
            Bar::new().set_high(28.0).set_low(18.0).set_close(25.0), // C3
            Bar::new().set_high(30.0).set_low(20.0).set_close(28.0), // C4
            Bar::new().set_high(27.0).set_low(19.0).set_close(22.0), // C5
            Bar::new().set_high(25.0).set_low(15.0).set_close(16.0), // C6
        ];

        // C1: (H=20, L=10, C=15)
        // K: (15-10)/(20-10) = 0.5 * 100 = 50.0
        // D: NaN (SMA not warmed up yet)
        let (k1, _d1) = stoch.next(&candles[0])?;
        assert_approx_eq(k1, 50.0, 0.001);

        // C2: (H=22, L=12, C=20)
        // Window: [C1, C2] => HH=22, LL=10
        // K: (20-10)/(22-10) = 10/12 = 0.8333... * 100 = 83.333
        // D: SMA(50.0, 83.333) = (50.0 + 83.333) / 2 = 66.666
        let (k2, d2) = stoch.next(&candles[1])?;
        assert_approx_eq(k2, 83.333, 0.001);
        assert_approx_eq(d2, 66.666, 0.001);

        // C3: (H=28, L=18, C=25)
        // Window: [C1, C2, C3] => HH=28, LL=10
        // K: (25-10)/(28-10) = 15/18 = 0.8333... * 100 = 83.333
        // D: SMA(83.333, 83.333) = 83.333 (using the *last two* K values: k2 and k3)
        let (k3, d3) = stoch.next(&candles[2])?;
        assert_approx_eq(k3, 83.333, 0.001);
        assert_approx_eq(d3, 83.333, 0.001);

        // C4: (H=30, L=20, C=28)
        // Window: [C2, C3, C4] (C1 removed) => HH=30, LL=12
        // K: (28-12)/(30-12) = 16/18 = 0.8888... * 100 = 88.888
        // D: SMA(83.333, 88.888) = (83.333 + 88.888) / 2 = 86.111
        let (k4, d4) = stoch.next(&candles[3])?;
        assert_approx_eq(k4, 88.888, 0.001);
        assert_approx_eq(d4, 86.111, 0.001);

        // C5: (H=27, L=19, C=22)
        // Window: [C3, C4, C5] => HH=30, LL=18
        // K: (22-18)/(30-18) = 4/12 = 0.3333... * 100 = 33.333
        // D: SMA(88.888, 33.333) = (88.888 + 33.333) / 2 = 61.111
        let (k5, d5) = stoch.next(&candles[4])?;
        assert_approx_eq(k5, 33.333, 0.001);
        assert_approx_eq(d5, 61.111, 0.001);

        // C6: (H=25, L=15, C=16)
        // Window: [C4, C5, C6] => HH=30, LL=15
        // K: (16-15)/(30-15) = 1/15 = 0.0666... * 100 = 6.666
        // D: SMA(33.333, 6.666) = (33.333 + 6.666) / 2 = 19.999
        let (k6, d6) = stoch.next(&candles[5])?;
        assert_approx_eq(k6, 6.666, 0.001);
        assert_approx_eq(d6, 19.999, 0.0001);

        Ok(())
    }

    #[test]
    fn test_stochastic_oscillator_serialization_deserialization() -> TaResult<()> {
        let mut stoch = StochasticOscillator::new(14, 3)?; // Default periods
        let candles = vec![
            Bar::new().set_high(100.0).set_low(90.0).set_close(95.0),
            Bar::new().set_high(105.0).set_low(92.0).set_close(103.0),
            Bar::new().set_high(110.0).set_low(98.0).set_close(100.0),
            Bar::new().set_high(112.0).set_low(105.0).set_close(110.0),
            Bar::new().set_high(108.0).set_low(95.0).set_close(97.0),
        ];

        // Process some data to build up internal state
        let mut results_before_serde = Vec::new();
        for candle in &candles {
            results_before_serde.push(stoch.next(candle)?);
        }

        // Serialize the indicator
        let serialized = serde_json::to_string(&stoch).expect("Failed to serialize");
        println!("Serialized StochasticOscillator: {serialized}");

        // Assert that 'values' and 'd' are not in the serialized output
        assert!(!serialized.contains("values"));
        // assert!(!serialized.contains("d"));
        assert!(serialized.contains(r#""period":14"#));
        assert!(serialized.contains(r#""smoothing_period":3"#));

        // Deserialize the indicator
        let mut deserialized_stoch: StochasticOscillator =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        // Assert that the parameters are correct after deserialization
        assert_eq!(deserialized_stoch.period, 14);
        assert_eq!(deserialized_stoch.smoothing_period, 3);
        // Internal state (values and d) should be reset/empty and correctly initialized
        assert!(deserialized_stoch.values.is_empty());
        // SMA's sum and values should be reset
        // Note: Direct comparison of `d` requires `SimpleMovingAverage` to implement `PartialEq`
        // and its internal state to be comparable. For safety, we'll re-process data.
        assert_eq!(deserialized_stoch.d.period(), 3);
        // assert_eq!(deserialized_stoch.d.sum, 0.0);
        // assert!(deserialized_stoch.d.values.is_empty());

        // Process the same data again with the deserialized indicator
        let mut results_after_serde = Vec::new();
        for candle in &candles {
            results_after_serde.push(deserialized_stoch.next(candle)?);
        }

        // The results should be identical, as the indicator was reset to its initial state
        // upon deserialization and then re-processed the same data.
        // This is a crucial test for correct deserialization of indicators with internal state.
        for i in 0..results_before_serde.len() {
            let (k_before, d_before) = results_before_serde[i];
            let (k_after, d_after) = results_after_serde[i];
            // Compare results allowing for floating point inaccuracies
            assert_approx_eq(k_before, k_after, 0.001);
            if d_before.is_nan() {
                assert!(d_after.is_nan());
            } else {
                assert_approx_eq(d_before, d_after, 0.001);
            }
        }
        Ok(())
    }

    #[test]
    fn test_stochastic_oscillator_default() -> TaResult<()> {
        let stoch = StochasticOscillator::default();
        assert_eq!(stoch.period, 14);
        assert_eq!(stoch.smoothing_period, 3);
        assert!(stoch.values.is_empty());
        assert_eq!(stoch.d.period(), 3); // Check internal SMA period
        Ok(())
    }

    #[test]
    fn test_stochastic_oscillator_reset() -> TaResult<()> {
        let mut stoch = StochasticOscillator::new(5, 3)?;
        let candle = Bar::new().set_high(100.0).set_low(90.0).set_close(95.0);

        // Push some data
        for _ in 0..5 {
            stoch.next(&candle)?;
        }
        assert_eq!(stoch.values.len(), 5);

        stoch.reset();
        assert!(stoch.values.is_empty());

        Ok(())
    }
}
