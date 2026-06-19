//! Error types for the Relix lexer and parser.
//!
//! All recoverable failures in the lexer and parser are reported as
//! [`RelixError`] values rather than panics, so callers can handle them
//! programmatically.

use std::fmt;

/// The error type for Relix lexing and parsing operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelixError {
    /// A human-readable description of the error.
    message: String,
}

impl RelixError {
    /// Creates a new error with the given message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Returns the error message.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for RelixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for RelixError {}

impl From<String> for RelixError {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

impl From<&str> for RelixError {
    fn from(message: &str) -> Self {
        Self::new(message)
    }
}
