//! Foundation error types.

use std::fmt;

/// Top-level error type for the foundation crate.
#[derive(Debug, Clone, PartialEq)]
pub enum FoundationError {
    /// An overflow occurred during an arithmetic operation.
    Overflow(String),
    /// A value was outside the acceptable range.
    OutOfRange(String),
    /// A validation check failed.
    ValidationFailed(String),
    /// A serialization or deserialization error occurred.
    SerializationError(String),
    /// A hashing error occurred.
    HashError(String),
    /// A context-related error occurred.
    ContextError(String),
    /// A transport-related error occurred.
    TransportError(String),
    /// An ID-related error occurred.
    IdError(String),
    /// A generic internal error.
    InternalError(String),
}

impl fmt::Display for FoundationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FoundationError::Overflow(msg) => write!(f, "Overflow: {}", msg),
            FoundationError::OutOfRange(msg) => write!(f, "OutOfRange: {}", msg),
            FoundationError::ValidationFailed(msg) => write!(f, "ValidationFailed: {}", msg),
            FoundationError::SerializationError(msg) => {
                write!(f, "SerializationError: {}", msg)
            }
            FoundationError::HashError(msg) => write!(f, "HashError: {}", msg),
            FoundationError::ContextError(msg) => write!(f, "ContextError: {}", msg),
            FoundationError::TransportError(msg) => write!(f, "TransportError: {}", msg),
            FoundationError::IdError(msg) => write!(f, "IdError: {}", msg),
            FoundationError::InternalError(msg) => write!(f, "InternalError: {}", msg),
        }
    }
}

impl std::error::Error for FoundationError {}

/// Alias for Results using FoundationError.
pub type FoundationResult<T> = Result<T, FoundationError>;
