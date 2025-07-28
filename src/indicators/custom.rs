use chipa_ta_utils::{OutputType, TaUtilsResult};
use core::fmt;
use std::sync::{Arc, Mutex};

#[cfg(feature = "chipa_lang")]
use chipa_lang_utils::errors::LangResult;
#[cfg(feature = "chipa_lang")]
use chipa_lang_utils::traits::Indexable;
#[cfg(feature = "chipa_lang")]
use chipa_lang_utils::{Lang, Pair, Rule};
use serde::{Deserialize, Serialize};

use crate::traits::{Candle, IndicatorTrait, Next, Period, Reset};
use crate::types::OutputShape;

/// A simplified trait for dynamic indicators that avoids object safety issues
pub trait DynIndicator: fmt::Debug + Send + Sync {
    /// Get the indicator's name
    fn name(&self) -> String;

    /// Get the indicator's period
    fn period(&self) -> usize;

    /// Get the output shape
    fn output_shape(&self) -> OutputShape;

    /// Reset the indicator
    fn reset(&mut self);

    /// Process a candle
    fn next_candle(&mut self, input: &dyn Candle) -> TaUtilsResult<OutputType>;

    /// Clone the indicator
    fn clone_dyn(&self) -> Box<dyn DynIndicator>;

    /// Check if supports price input
    fn supports_price_input(&self) -> bool {
        true
    }

    /// Convert to CT string (if chipa_lang feature is enabled)
    #[cfg(feature = "chipa_lang")]
    fn to_ct(&self) -> String {
        format!("Custom({})", self.name())
    }
}

// Implement DynIndicator for any compatible type
impl<T> DynIndicator for T
where
    T: IndicatorTrait
        + Reset
        + Period
        + fmt::Debug
        + Clone
        + Send
        + Sync
        + for<'a> Next<&'a dyn Candle, Output = OutputType>
        + 'static,
{
    fn name(&self) -> String {
        // Extract just the type name without the Display format
        std::any::type_name::<T>()
            .split("::")
            .last()
            .unwrap_or("Unknown")
            .to_string()
    }

    fn period(&self) -> usize {
        Period::period(self)
    }

    fn output_shape(&self) -> OutputShape {
        IndicatorTrait::output_shape(self)
    }

    fn reset(&mut self) {
        Reset::reset(self);
    }

    fn next_candle(&mut self, input: &dyn Candle) -> TaUtilsResult<OutputType> {
        // Convert candle to price for processing
        self.next(input)
    }

    fn clone_dyn(&self) -> Box<dyn DynIndicator> {
        Box::new(self.clone())
    }

    #[cfg(feature = "chipa_lang")]
    fn to_ct(&self) -> String {
        // Default implementation - specific types can override in their Lang impl
        format!("Custom({})", self.name())
    }
}

/// A non-generic CustomIndicator that can hold any indicator implementing DynIndicator.
/// This version can be used in enums and provides dynamic dispatch.
///
/// # Features
///
/// - **Dynamic**: Can hold any indicator at runtime
/// - **Enum Compatible**: No generics, can be used in enums
/// - **Thread Safe**: Uses Arc<Mutex<>> for safe sharing
/// - **Flexible**: Supports both price and candle inputs
///
/// # Examples
///
/// ```rust
/// use chipa_ta::indicators::{RelativeStrengthIndex, CustomIndicator};
///
/// // Wrap any indicator
/// let rsi = RelativeStrengthIndex::new(14).unwrap();
/// let custom = CustomIndicator::new(rsi);
///
/// // Use it like any other indicator
/// let mut custom = custom;
/// let result = custom.next(100.0)?;
/// ```
#[derive(Debug, Clone)]
pub struct CustomIndicator {
    inner: Arc<Mutex<Box<dyn DynIndicator>>>,
    cached_name: String, // Cache for performance
}

/// Helper function to wrap any compatible indicator
pub fn wrap_indicator<T>(indicator: T) -> CustomIndicator
where
    T: DynIndicator + 'static,
{
    CustomIndicator::new(indicator)
}

impl CustomIndicator {
    /// Create a new CustomIndicator wrapping the given indicator
    pub fn new<T>(indicator: T) -> Self
    where
        T: DynIndicator + 'static,
    {
        let name = indicator.name();
        Self {
            inner: Arc::new(Mutex::new(Box::new(indicator))),
            cached_name: name,
        }
    }

    /// Get the name of the wrapped indicator
    pub fn name(&self) -> String {
        self.cached_name.clone()
    }

    /// Try to get a reference to the inner indicator (for advanced usage)
    pub fn with_inner<R>(&self, f: impl FnOnce(&dyn DynIndicator) -> R) -> R {
        let inner = self.inner.lock().expect("Failed to lock CustomIndicator");
        f(&**inner)
    }

    /// Try to get a mutable reference to the inner indicator (for advanced usage)
    pub fn with_inner_mut<R>(&mut self, f: impl FnOnce(&mut dyn DynIndicator) -> R) -> R {
        let mut inner = self.inner.lock().expect("Failed to lock CustomIndicator");
        f(&mut **inner)
    }
}

impl Default for CustomIndicator {
    fn default() -> Self {
        // Create a default NoneIndicator-like implementation
        #[derive(Debug, Clone)]
        struct DefaultIndicator;

        impl DynIndicator for DefaultIndicator {
            fn name(&self) -> String {
                "Default".to_string()
            }
            fn period(&self) -> usize {
                0
            }
            fn output_shape(&self) -> OutputShape {
                OutputShape::Shape(1)
            }
            fn reset(&mut self) {}

            fn next_candle(&mut self, input: &dyn Candle) -> TaUtilsResult<OutputType> {
                Ok(OutputType::Single(input.price()))
            }
            fn clone_dyn(&self) -> Box<dyn DynIndicator> {
                Box::new(self.clone())
            }
        }

        Self::new(DefaultIndicator)
    }
}

impl fmt::Display for CustomIndicator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CustomIndicator({})", self.cached_name)
    }
}

impl PartialEq for CustomIndicator {
    fn eq(&self, other: &Self) -> bool {
        // Compare based on name and pointer equality as a reasonable approximation
        self.cached_name == other.cached_name && Arc::ptr_eq(&self.inner, &other.inner)
    }
}

// Note: Serialization is not trivial for trait objects, so we provide basic implementations
impl Serialize for CustomIndicator {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize just the name - reconstructing the full indicator would need more context
        self.cached_name.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CustomIndicator {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Cannot deserialize trait objects easily, return a default
        Err(serde::de::Error::custom(
            "CustomIndicator deserialization not supported - use specific indicator types",
        ))
    }
}

// Trait implementations that delegate to the inner indicator
impl IndicatorTrait for CustomIndicator {
    fn output_shape(&self) -> OutputShape {
        self.with_inner(|inner| inner.output_shape())
    }
}

impl Reset for CustomIndicator {
    fn reset(&mut self) {
        self.with_inner_mut(|inner| inner.reset());
    }
}

impl<C: Candle> Next<&C> for CustomIndicator {
    type Output = OutputType;

    fn next(&mut self, input: &C) -> TaUtilsResult<Self::Output> {
        self.with_inner_mut(|inner| inner.next_candle(input))
    }
}

impl Period for CustomIndicator {
    fn period(&self) -> usize {
        self.with_inner(|inner| inner.period())
    }
}

// Conditional Lang implementation for chipa_lang feature
#[cfg(feature = "chipa_lang")]
impl Lang for CustomIndicator {
    fn to_ct(&self) -> String {
        self.with_inner(|inner| inner.to_ct())
    }

    fn from_ct(_input: &str) -> LangResult<Self> {
        // Cannot construct without knowing the specific type
        Err(chipa_lang_utils::errors::LangErrorKind::Unallowed(
            "Cannot construct CustomIndicator from CT string without type information".to_string(),
        )
        .lang())
    }

    fn from_pair(_pair: Pair<Rule>) -> LangResult<Self> {
        // Cannot construct without knowing the specific type
        Err(chipa_lang_utils::errors::LangErrorKind::Unallowed(
            "Cannot construct CustomIndicator from pair without type information".to_string(),
        )
        .lang())
    }
}

#[cfg(feature = "chipa_lang")]
impl Indexable for CustomIndicator {
    fn available_fields(&self) -> Vec<String> {
        // Basic fields that make sense for any indicator
        vec![
            "value".to_string(),
            "period".to_string(),
            "name".to_string(),
        ]
    }

    fn supports_indexing(&self) -> bool {
        false // Simplified - don't support complex indexing for trait objects
    }

    fn supports_field_access(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Example of how to use the new non-generic CustomIndicator
    #[derive(Debug, Clone, PartialEq, Default)]
    struct MockIndicator {
        value: f64,
    }

    impl fmt::Display for MockIndicator {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "MockIndicator(value: {})", self.value)
        }
    }

    impl MockIndicator {
        fn new(value: f64) -> Self {
            Self { value }
        }
    }

    impl IndicatorTrait for MockIndicator {
        fn output_shape(&self) -> OutputShape {
            OutputShape::Shape(1)
        }
    }

    impl Reset for MockIndicator {
        fn reset(&mut self) {
            self.value = 0.0;
        }
    }

    impl Next<f64> for MockIndicator {
        type Output = f64;

        fn next(&mut self, input: f64) -> TaUtilsResult<Self::Output> {
            self.value = input * 2.0; // Simple transformation
            Ok(self.value)
        }
    }

    impl<C: Candle> Next<&C> for MockIndicator {
        type Output = f64;

        fn next(&mut self, input: &C) -> TaUtilsResult<Self::Output> {
            self.value = input.price() * 2.0;
            Ok(self.value)
        }
    }

    impl Period for MockIndicator {
        fn period(&self) -> usize {
            1
        }
    }

    #[test]
    fn test_custom_indicator_basic_usage() {
        let mock = MockIndicator::new(0.0);
        let mut custom = CustomIndicator::new(mock);

        // Test that it works like the original indicator
        let result = custom.next(5.0).unwrap();
        assert_eq!(result, 10.0);

        // Test reset
        Reset::reset(&mut custom);

        // Test period
        assert_eq!(Period::period(&custom), 1);

        // Test name
        assert_eq!(custom.name(), "MockIndicator");
    }

    #[test]
    fn test_helper_function() {
        let mock = MockIndicator::new(5.0);
        let custom = wrap_indicator(mock);

        assert_eq!(custom.name(), "MockIndicator");
        assert_eq!(Period::period(&custom), 1);
    }

    #[test]
    fn test_default_implementation() {
        let custom = CustomIndicator::default();
        assert_eq!(custom.name(), "Default");
        assert_eq!(Period::period(&custom), 0);
    }
}
