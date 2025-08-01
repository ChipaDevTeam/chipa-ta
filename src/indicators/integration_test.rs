//! Integration test for CustomIndicator in the Indicator enum

#[cfg(test)]
mod tests {
    use crate::indicators::{
        custom::{CustomIndicator, wrap_indicator},
        indicator::Indicator,
        sma::SimpleMovingAverage,
    };
    use chipa_ta_utils::IndicatorTrait;
    use crate::traits::{Next, Period, Reset};
    use crate::types::OutputType;
    use chipa_ta_utils::{Bar, Candle, TaUtilsResult};
    use std::fmt;

    #[derive(Debug, Clone, PartialEq, Default)]
    struct TestIndicator {
        sum: f64,
        count: usize,
        period: usize,
    }

    impl fmt::Display for TestIndicator {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "TestIndicator(period: {})", self.period)
        }
    }

    impl TestIndicator {
        fn new(period: usize) -> Self {
            Self {
                sum: 0.0,
                count: 0,
                period,
            }
        }
    }

    impl IndicatorTrait for TestIndicator {
        fn output_shape(&self) -> crate::types::OutputShape {
            crate::types::OutputShape::Shape(1)
        }
    }

    impl Reset for TestIndicator {
        fn reset(&mut self) {
            self.sum = 0.0;
            self.count = 0;
        }
    }

    impl<C: Candle> Next<&C> for TestIndicator {
        type Output = OutputType;

        fn next(&mut self, input: &C) -> TaUtilsResult<Self::Output> {
            self.sum += input.price();
            self.count += 1;
            if self.count > self.period {
                // Simple average of last period values (not exact but good for testing)
                Ok(OutputType::Single(self.sum / self.count as f64))
            } else {
                Ok(OutputType::Single(self.sum / self.count as f64))
            }
        }
    }

    // Add the required implementation for dyn Candle
    impl Next<&dyn Candle> for TestIndicator {
        type Output = OutputType;

        fn next(&mut self, input: &dyn Candle) -> TaUtilsResult<Self::Output> {
            self.sum += input.price();
            self.count += 1;
            if self.count > self.period {
                // Simple average of last period values (not exact but good for testing)
                Ok(OutputType::Single(self.sum / self.count as f64))
            } else {
                Ok(OutputType::Single(self.sum / self.count as f64))
            }
        }
    }

    impl Period for TestIndicator {
        fn period(&self) -> usize {
            self.period
        }
    }

    #[test]
    fn test_custom_indicator_in_enum() {
        // Create a custom indicator
        let test_indicator = TestIndicator::new(5);
        let custom = CustomIndicator::new(test_indicator);

        // Wrap it in the Indicator enum
        let mut indicator = Indicator::Custom(custom);

        // Test that all enum methods work correctly
        assert_eq!(indicator.name(), "TestIndicator");
        assert_eq!(Period::period(&indicator), 5);

        // Test processing data
        let result1 = indicator.next(&Bar::new().set_price(10.0)).unwrap();
        let result2 = indicator.next(&Bar::new().set_price(20.0)).unwrap();

        // Verify results are reasonable (OutputType should be Single variant)
        match result1 {
            OutputType::Single(val) => assert!(val > 0.0),
            _ => panic!("Expected Single output type"),
        }
        match result2 {
            OutputType::Single(val) => assert!(val > 0.0),
            _ => panic!("Expected Single output type"),
        }

        // Test reset
        Reset::reset(&mut indicator);

        // Should work again after reset
        let result3 = indicator.next(&Bar::new().set_price(30.0)).unwrap();
        match result3 {
            OutputType::Single(val) => assert!(val > 0.0),
            _ => panic!("Expected Single output type"),
        }
    }

    #[test]
    fn test_helper_function_with_enum() {
        // Use the helper function to create the custom indicator
        let test_indicator = TestIndicator::new(3);
        let custom = wrap_indicator(test_indicator);
        let mut indicator = Indicator::Custom(custom);

        // Test that it works correctly
        assert_eq!(indicator.name(), "TestIndicator");
        assert_eq!(Period::period(&indicator), 3);

        // Process some data
        let _ = indicator.next(&Bar::new().set_price(5.0)).unwrap();
        let _ = indicator.next(&Bar::new().set_price(15.0)).unwrap();
        let result = indicator.next(&Bar::new().set_price(25.0)).unwrap();

        match result {
            OutputType::Single(val) => assert!(val > 0.0),
            _ => panic!("Expected Single output type"),
        }
    }

    #[test]
    fn test_custom_vs_native_indicator() {
        // Create a native SMA indicator
        let mut native_sma = Indicator::Sma(SimpleMovingAverage::new(3).unwrap());

        // Create a custom indicator that mimics SMA behavior
        let test_indicator = TestIndicator::new(3);
        let mut custom_indicator = Indicator::Custom(CustomIndicator::new(test_indicator));

        // Both should be able to process the same data
        let test_data = vec![
            Bar::new().set_close(10.0).set_price(10.0),
            Bar::new().set_close(20.0).set_price(20.0),
            Bar::new().set_close(30.0).set_price(30.0),
            Bar::new().set_close(40.0).set_price(40.0),
        ];

        let mut native_results = Vec::new();
        let mut custom_results = Vec::new();

        for value in test_data {
            if let Ok(result) = native_sma.next(&value) {
                native_results.push(result);
            }
            if let Ok(result) = custom_indicator.next(&value) {
                custom_results.push(result);
            }
        }

        // Both should have processed all values successfully
        assert_eq!(native_results.len(), 4);
        assert_eq!(custom_results.len(), 4);

        // Results don't need to be identical since our TestIndicator is simplified,
        // but both should have reasonable values
        for result in &native_results {
            match result {
                OutputType::Single(val) => assert!(val.is_finite()),
                OutputType::Array(vals) => {
                    for val in vals {
                        assert!(val.is_finite());
                    }
                }
                _ => {} // Other variants are acceptable for testing
            }
        }
        for result in &custom_results {
            match result {
                OutputType::Single(val) => assert!(val.is_finite()),
                OutputType::Array(vals) => {
                    for val in vals {
                        assert!(val.is_finite());
                    }
                }
                _ => {} // Other variants are acceptable for testing
            }
        }
    }

    #[test]
    fn test_custom_indicator_serialization() {
        // Test that the custom indicator can be cloned and compared
        let test_indicator = TestIndicator::new(7);
        let custom1 = CustomIndicator::new(test_indicator.clone());
        let custom2 = CustomIndicator::new(test_indicator);

        let indicator1 = Indicator::Custom(custom1);
        let indicator2 = Indicator::Custom(custom2);

        // Should be able to clone
        let cloned = indicator1.clone();

        // Names should match
        assert_eq!(indicator1.name(), indicator2.name());
        assert_eq!(indicator1.name(), cloned.name());

        // Periods should match
        assert_eq!(Period::period(&indicator1), Period::period(&indicator2));
        assert_eq!(Period::period(&indicator1), Period::period(&cloned));
    }
}
