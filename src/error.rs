use chipa_ta_utils::OutputShape;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TaError {
    /// When parsing an indicator from a string (representing the indicator in the CT format) fails.
    #[error("Invalid Chipa Trading Lang format: {0}")]
    InvalidChipaLangFormat(String),
    #[error("Utils error: {0}")]
    Utils(#[from] chipa_ta_utils::TaUtilsError),
    #[error("Empty iterator recieved on function '{0}'")]
    EmptyIterator(String),
    #[error("Incorrect output type, expected {expected}, got {actual}")]
    IncorrectOutputType { expected: String, actual: String },
    #[error("Unexpected error, {0}")]
    Unexpected(String),
    #[error("Cmp error, {0}")]
    Cmp(#[from] OutputError),
    /// Error originating from the strategy module.
    #[cfg(feature = "strategy")]
    #[error("Strategy error: {0}")]
    Strategy(#[from] crate::strategy::StrategyError),
    #[error("Serde processing error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Not initialized, tried to call a method that requires initialization, error: {0}")]
    NotInitialized(String),

    /// Unallowed operation, such as trying to modify an immutable reference.
    #[error("Unallowed operation: {0}")]
    Unallowed(String),
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum OutputError {
    #[error("Type mismatch")]
    TypeMismatch,
    #[error("Length mismatch between two arrays, array1: {0}, array2: {1}")]
    LengthMismatch(usize, usize),
    #[error("Invalid output shape {0}")]
    InvalidOutputShape(OutputShape),
}

impl PartialEq for TaError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TaError::Utils(a), TaError::Utils(b)) => a == b,
            (TaError::EmptyIterator(a), TaError::EmptyIterator(b)) => a == b,
            (
                TaError::IncorrectOutputType {
                    expected: a,
                    actual: b,
                },
                TaError::IncorrectOutputType {
                    expected: c,
                    actual: d,
                },
            ) => a == c && b == d,
            (TaError::Unexpected(a), TaError::Unexpected(b)) => a == b,
            (TaError::Cmp(a), TaError::Cmp(b)) => a == b,
            (TaError::Strategy(a), TaError::Strategy(b)) => a == b,
            (TaError::Serde(a), TaError::Serde(b)) => a.to_string() == b.to_string(),
            _ => false,
        }
    }
}

pub type TaResult<T> = Result<T, TaError>;

#[cfg(feature = "js")]
impl From<TaError> for napi::Error {
    fn from(value: TaError) -> Self {
        Self::from_reason(value.to_string())
    }
}

#[cfg(feature = "py")]
impl From<TaError> for pyo3::PyErr {
    fn from(value: TaError) -> Self {
        pyo3::exceptions::PyException::new_err(value.to_string())
    }
}
