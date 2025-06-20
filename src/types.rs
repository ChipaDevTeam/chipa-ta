use core::fmt::{self, Debug};
use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{
    error::{TaError, TaResult},
    traits::{Candle, Period, Reset},
};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum OutputError {
    #[error("Type mismatch")]
    TypeMismatch,
    #[error("Length mismatch between two arrays, array1: {0}, array2: {1}")]
    LengthMismatch(usize, usize),
    #[error("Invalid output shape {0}")]
    InvalidOutputShape(OutputShape),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutputShape {
    Shape(usize), // Normal shape, using enum in case in the future we want to add more shapes
    Tensor(Vec<Box<OutputShape>>),
}

impl fmt::Display for OutputShape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputShape::Shape(size) => write!(f, "Shape({})", size),
            OutputShape::Tensor(shapes) => {
                write!(f, "Tensor(")?;
                for (i, shape) in shapes.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", shape)?;
                }
                write!(f, ")")
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Statics {
    Greater,
    Equal,
    Less,
    True,
    False,
}

impl PartialEq<f64> for Statics {
    fn eq(&self, _other: &f64) -> bool {
        match self {
            Statics::Greater => false,
            Statics::Equal => true,
            Statics::Less => false,
            Statics::True => true,
            Statics::False => false,
        }
    }
}

impl PartialOrd<f64> for Statics {
    fn partial_cmp(&self, _other: &f64) -> Option<std::cmp::Ordering> {
        match self {
            Statics::Greater => Some(std::cmp::Ordering::Greater),
            Statics::Equal => Some(std::cmp::Ordering::Equal),
            Statics::Less => Some(std::cmp::Ordering::Less),
            Statics::True => Some(std::cmp::Ordering::Equal),
            Statics::False => Some(std::cmp::Ordering::Equal),
        }
    }
}

// Can you help me emprove the Queue struct? the goal is to make it like a Vec but with a fixed capacity that removes the oldest element when a new one is added beyond its capacity.
// it also implements the Period and Reset traits, allowing it to be used in a similar way to Cycle.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Queue<T> {
    queue: Vec<T>,
    period: usize,
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
    Open,
    Close,
    High,
    Low,
    Volume,
    Custom(Vec<OutputType>),
    Static(Statics),
    Statics(Vec<Statics>),
}

impl OutputType {
    pub fn output_shape(&self) -> TaResult<OutputShape> {
        match self {
            OutputType::Single(_) => Ok(OutputShape::Shape(1)),
            OutputType::Array(arr) => OutputShape::Shape(arr.len()).validate(),
            OutputType::Open
            | OutputType::Close
            | OutputType::High
            | OutputType::Low
            | OutputType::Volume => Ok(OutputShape::Shape(1)),
            OutputType::Custom(vec) => OutputShape::Tensor(
                vec.iter()
                    .map(|o| o.output_shape())
                    .collect::<TaResult<Vec<OutputShape>>>()?
                    .into_iter()
                    .map(Box::new)
                    .collect(),
            )
            .validate(),
            OutputType::Static(_) => Ok(OutputShape::Shape(1)),
            OutputType::Statics(vec) => OutputShape::Shape(vec.len()).validate(),
        }
    }

    /// Turn any OutputType into actual Single/Array by pulling from the candle.
    pub fn resolve<C: Candle>(&self, data: &C) -> TaResult<OutputType> {
        match self {
            OutputType::Single(_) | OutputType::Array(_) => Ok(self.clone()),
            OutputType::Open => Ok(OutputType::Single(data.open())),
            OutputType::Close => Ok(OutputType::Single(data.close())),
            OutputType::High => Ok(OutputType::Single(data.high())),
            OutputType::Low => Ok(OutputType::Single(data.low())),
            OutputType::Volume => Ok(OutputType::Single(data.volume())),
            OutputType::Custom(vec) => {
                let mut out = Vec::with_capacity(vec.len());
                for ot in vec {
                    match ot.resolve(data)? {
                        OutputType::Single(v) => out.push(v),
                        _ => {
                            return Err(TaError::IncorrectOutputType {
                                expected: "Single".into(),
                                actual: "Array".into(),
                            })
                        }
                    }
                }
                Ok(OutputType::Array(out))
            }
            OutputType::Static(_) => Ok(self.clone()),
            OutputType::Statics(_) => Ok(self.clone()),
        }
    }
}

impl OutputShape {
    pub fn validate(&self) -> TaResult<Self> {
        match self {
            OutputShape::Shape(size) if *size > 0 => Ok(self.clone()),
            OutputShape::Tensor(vec) if !vec.is_empty() => {
                // If the size of all the elements of the tensor are 1 then return a Shape(len(vec))
                if vec.iter().all(|s| **s == OutputShape::Shape(1)) {
                    return Ok(OutputShape::Shape(vec.len()));
                }
                for shape in vec {
                    shape.validate()?;
                }
                Ok(self.clone())
            }
            shape => Err(TaError::from(OutputError::InvalidOutputShape(
                shape.clone(),
            ))),
        }
    }
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
            _ => Err(TaError::IncorrectOutputType {
                expected: "f64".to_string(),
                actual: "Other".to_string(),
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
            _ => Err(TaError::IncorrectOutputType {
                expected: "Vec<f64>".to_string(),
                actual: "Other".to_string(),
            }),
        }
    }
}

// TODO: Implement PartialEq and PartialOrd for OutputType using std::f64::EPSILON
impl PartialOrd for OutputType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (OutputType::Single(a), OutputType::Single(b)) => a.partial_cmp(b),
            (OutputType::Array(a), OutputType::Array(b)) => {
                if a.len() != b.len() {
                    None
                } else {
                    // Compare each element in the arrays and return None if any comparison fails
                    // Also return none if there is a mismatch in types
                    let mut equals = Vec::new();
                    for (a, b) in a.iter().zip(b.iter()) {
                        if let Some(ordering) = a.partial_cmp(b) {
                            equals.push(ordering);
                        }
                    }
                    if equals.is_empty() {
                        return None;
                    }
                    match equals.iter().all(|&o| o == equals[0]) {
                        true => Some(equals[0]),
                        false => None, // If any ordering is different, return None
                    }
                }
            }
            (OutputType::Single(a), OutputType::Static(b))
            | (OutputType::Static(b), OutputType::Single(a)) => b.partial_cmp(a),
            (OutputType::Array(a), OutputType::Statics(b))
            | (OutputType::Statics(b), OutputType::Array(a)) => {
                if a.len() != b.len() {
                    None
                } else {
                    // Compare each element in the arrays and return None if any comparison fails
                    let mut equals = Vec::new();
                    for (a, b) in a.iter().zip(b.iter()) {
                        if let Some(ordering) = b.partial_cmp(a) {
                            equals.push(ordering);
                        }
                    }
                    if equals.is_empty() {
                        return None;
                    }
                    match equals.iter().all(|&o| o == equals[0]) {
                        true => Some(equals[0]),
                        false => None, // If any ordering is different, return None
                    }
                }
            }
            _ => None,
        }
    }
}

impl<T: Default + Clone> Queue<T> {
    pub fn new(period: usize) -> TaResult<Self> {
        if period == 0 {
            return Err(TaError::InvalidParameter(
                "Period must be greater than 0".to_string(),
            ));
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
