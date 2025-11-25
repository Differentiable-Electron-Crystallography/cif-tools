//! Error types for dREL parsing

use thiserror::Error;

/// Errors that can occur during dREL parsing
#[derive(Error, Debug)]
pub enum DrelError {
    /// Grammar/syntax error from PEST parser
    #[error("Parse error: {0}")]
    ParseError(Box<pest::error::Error<crate::Rule>>),

    /// Invalid AST construction
    #[error("Invalid structure at {location}: {message}")]
    InvalidStructure {
        /// Error message describing the issue
        message: String,
        /// Location in source (line:column)
        location: String,
    },

    /// Unexpected token or construct
    #[error("Unexpected {found} at {location}, expected {expected}")]
    Unexpected {
        /// What was found
        found: String,
        /// What was expected
        expected: String,
        /// Location in source
        location: String,
    },
}

impl From<pest::error::Error<crate::Rule>> for DrelError {
    fn from(err: pest::error::Error<crate::Rule>) -> Self {
        Self::ParseError(Box::new(err))
    }
}

impl DrelError {
    /// Create an invalid structure error
    pub fn invalid_structure(message: impl Into<String>, line: usize, col: usize) -> Self {
        Self::InvalidStructure {
            message: message.into(),
            location: format!("{}:{}", line, col),
        }
    }

    /// Create an unexpected token error
    pub fn unexpected(
        found: impl Into<String>,
        expected: impl Into<String>,
        line: usize,
        col: usize,
    ) -> Self {
        Self::Unexpected {
            found: found.into(),
            expected: expected.into(),
            location: format!("{}:{}", line, col),
        }
    }
}
