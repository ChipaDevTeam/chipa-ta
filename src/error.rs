
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum TaError {
    #[error("InvalidParameter '{0}' found")]
    InvalidParameter(String),
    #[error("Empty iterator recieved on function '{0}'")]
    EmptyIterator(String),
    #[error("Incorrect output type, expected {expected}, got {actual}")]
    IncorrectOutputType { expected: String, actual: String },
    #[error("Unexpected error, {0}")]
    Unexpected(String),
}

pub type TaResult<T> = Result<T, TaError>;

#[cfg(feature = "js")]
impl From<TaError> for napi::Error {
    fn from(value: TaError) -> Self {
        Self::from_reason(value.to_string())
    }
}

#[cfg(feature="py")]
impl From<TaError> for pyo3::PyErr {
    fn from(value: TaError) -> Self {
        pyo3::exceptions::PyException::new_err(value.to_string())
    }
}