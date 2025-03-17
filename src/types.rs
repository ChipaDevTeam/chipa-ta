use core::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::error::TaError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Status<T, U, V> {
    Initial(T),
    Progress(U),
    Completed(V),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OutputType {
    Single(f64),
    Array(Vec<f64>),
}

impl<T: Default, U, V> Default for Status<T, U, V> {
    fn default() -> Self {
        Self::Initial(T::default())
    }
}

impl From<f64> for OutputType {
    fn from(value: f64) -> Self {
        Self::Single(value)
    }
}

impl From<Vec<f64>> for OutputType {
    fn from(value: Vec<f64>) -> Self {
        Self::Array(value)
    }
}

impl TryFrom<OutputType> for f64 {
    type Error = TaError;

    fn try_from(value: OutputType) -> Result<Self, Self::Error> {
        match value {
            OutputType::Single(output) => Ok(output),
            OutputType::Array(_) => Err(TaError::IncorrectOutputType {
                expected: "f64".to_string(),
                actual: "Vec<f64>".to_string(),
            }),
        }
    }
}

impl TryFrom<OutputType> for Vec<f64> {
    type Error = TaError;

    fn try_from(value: OutputType) -> Result<Self, Self::Error> {
        match value {
            OutputType::Array(output) => Ok(output),
            OutputType::Single(_) => Err(TaError::IncorrectOutputType {
                expected: "Vec<f64>".to_string(),
                actual: "f64".to_string(),
            }),
        }
    }
}
