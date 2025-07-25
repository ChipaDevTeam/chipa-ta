use thiserror::Error;

use crate::types::OutputShape;

/// Errors that can occur during strategy validation or parsing.
#[derive(Error, Debug, PartialEq)]
pub enum StrategyError {
    /// An `If` node is missing an `else_branch`.
    #[error("If node is missing an else_branch")]
    MissingElseBranch,

    /// A `Sequence` node has no child nodes.
    #[error("Sequence node must contain at least one child")]
    EmptySequence,
    // Potential future errors: InvalidIndicator, ParseError, etc.
    #[error("Incompatible shapes: {indicator} vs {value} for '{name}'")]
    IncompatibleShapes {
        name: String,
        indicator: OutputShape,
        value: OutputShape,
    },
    #[error("Invalid indicator period: {period}")]
    InvalidIndicatorPeriod { period: usize },

    #[error("Poison Error: {0}")]
    Poison(String),

    #[error("Empty iterator: {0}")]
    EmptyIterator(String),

    #[error("Serialization Error: {0}")]
    Serialization(String),

    #[error("IO Error: {0}")]
    IO(String),

    #[error("Configuration error, {0}")]
    Configuration(String),
}
