use core::fmt::Debug;
use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{error::{TaError, TaResult}, traits::{Period, Reset}};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum OutputTypeCmpError {
    #[error("Type mismatch")]
    TypeMismatch,
    #[error("Length mismatch between two arrays, array1: {0}, array2: {1}")]
    LengthMismatch(usize, usize),
}
// Can you help me emprove the Queue struct? the goal is to make it like a Vec but with a fixed capacity that removes the oldest element when a new one is added beyond its capacity.
// it also implements the Period and Reset traits, allowing it to be used in a similar way to Cycle.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Queue<T> {
    queue: Vec<T>,
    period: usize
}

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

impl OutputType {
    /// Compare OutputType to another OutputType with a comparison function.
    pub fn cmp_output<F>(&self, other: &OutputType, cmp: F) -> TaResult<bool>
    where
        F: Fn(f64, f64) -> bool,
    {
        match (self, other) {
            (OutputType::Single(a), OutputType::Single(b)) => Ok(cmp(*a, *b)),
            (OutputType::Array(a), OutputType::Array(b)) => {
                if a.len() != b.len() {
                    Err(OutputTypeCmpError::LengthMismatch(a.len(), b.len()).into())
                } else {
                    Ok(a.iter().zip(b).all(|(v1, v2)| cmp(*v1, *v2)))
                }
            }
            _ => Err(OutputTypeCmpError::TypeMismatch.into()),
        }
    }
}

impl<T: Default + Clone> Queue<T> {
    pub fn new(period: usize) -> TaResult<Self> {
        if period == 0 {
            return Err(TaError::InvalidParameter("Period must be greater than 0".to_string()));
        }
        Ok(Self {
            queue: Vec::with_capacity(period),
            period,
        })
    }

    pub fn push(&mut self, value: T) -> Option<T> {
        self.queue.push(value);
        if self.queue.len() > self.period {
            let removed = self.queue.remove(0);
            Some(removed)
        } else {
            None
        }
    }
}

impl<T> Deref for Queue<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.queue
    }
}

impl<T> DerefMut for Queue<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.queue
    }
}

impl<T> Period for Queue<T> {
    fn period(&self) -> usize {
        self.period
    }
}
impl<T> Reset for Queue<T> {
    fn reset(&mut self) {
        self.queue = Vec::with_capacity(self.period);
    }
}

