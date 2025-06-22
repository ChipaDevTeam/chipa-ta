use core::fmt;
use std::ops::{Deref, DerefMut};
use serde::{Serialize, Deserialize};

use crate::{
    error::{TaError, TaResult}, indicators::indicator::Indicator as IndicatorEnum, traits::{Indicator, Next, Period, Reset}, types::{OutputShape, OutputType}
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct IndicatorState {
    pub indicator: IndicatorEnum,
    previous_output: Option<OutputType>,
}

impl Serialize for IndicatorState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.indicator.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for IndicatorState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let indicator = IndicatorEnum::deserialize(deserializer)?;
        Ok(Self {
            indicator,
            previous_output: None,
        })
    }
}

impl Indicator for IndicatorState {
    fn output_shape(&self) -> OutputShape {
        self.indicator.output_shape()
    }
}

impl Period for IndicatorState {
    fn period(&self) -> usize {
        self.indicator.period()
    }
}

impl fmt::Display for IndicatorState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Wrapper({})", self.indicator)
    }
}

impl IndicatorState {
    pub fn new(indicator: IndicatorEnum) -> Self {
        Self {
            indicator,
            previous_output: None,
        }
    }

    /// Calls next on the wrapped indicator, stores the output, and returns it.
    pub fn update<T>(&mut self, input: T) -> TaResult<()>
    where
        IndicatorEnum: Next<T, Output = OutputType>,
    {
        let output = self.indicator.next(input)?;
        self.previous_output = Some(output);
        Ok(())
    }

    /// Returns the previous output as a Result, or an error if not available.
    pub fn prev(&self) -> TaResult<OutputType> {
        self.previous_output.clone().ok_or_else(|| {
            TaError::NotInitialized("No previous output available".to_string())
        })
    }
}

// Deref and DerefMut to the inner IndicatorEnum
impl Deref for IndicatorState {
    type Target = IndicatorEnum;
    fn deref(&self) -> &Self::Target {
        &self.indicator
    }
}
impl DerefMut for IndicatorState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.indicator
    }
}

// From and Into for IndicatorEnum
impl From<IndicatorEnum> for IndicatorState {
    fn from(indicator: IndicatorEnum) -> Self {
        Self::new(indicator)
    }
}
impl From<IndicatorState> for IndicatorEnum {
    fn from(wrapper: IndicatorState) -> Self {
        wrapper.indicator
    }
}

// Implement Reset for the wrapper
impl Reset for IndicatorState {
    fn reset(&mut self) {
        self.indicator.reset();
        self.previous_output = None;
    }
}