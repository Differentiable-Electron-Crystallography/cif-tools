//! Error types for CIF parsing.
//!
//! This module defines the error types that can occur during CIF file parsing
//! and provides conversions from underlying error types.

use crate::Rule;
use std::error::Error;
use std::fmt;

/// Custom error type for CIF parsing with enhanced error information.
///
/// # Error Categories
///
/// - **ParseError**: Grammar-level parsing failures (from PEST)
/// - **IoError**: File I/O failures
/// - **InvalidStructure**: Semantic validation failures with optional location info
///
/// # Location Tracking
///
/// Errors now include line and column information when available, making it easy
/// to pinpoint issues in CIF files.
#[derive(Debug)]
pub enum CifError {
    /// Grammar parsing error from PEST (already includes location info)
    ParseError(String),
    /// File I/O error
    IoError(std::io::Error),
    /// Semantic structure validation error with optional source location
    InvalidStructure {
        message: String,
        location: Option<(usize, usize)>, // (line, column)
    },
}

impl fmt::Display for CifError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CifError::ParseError(msg) => write!(f, "Parse error: {msg}"),
            CifError::IoError(err) => write!(f, "IO error: {err}"),
            CifError::InvalidStructure { message, location } => {
                if let Some((line, col)) = location {
                    write!(
                        f,
                        "Error at line {}, column {}: Invalid CIF structure: {}",
                        line, col, message
                    )
                } else {
                    write!(f, "Invalid CIF structure: {}", message)
                }
            }
        }
    }
}

impl Error for CifError {}

impl From<std::io::Error> for CifError {
    fn from(err: std::io::Error) -> Self {
        CifError::IoError(err)
    }
}

impl From<pest::error::Error<Rule>> for CifError {
    fn from(err: pest::error::Error<Rule>) -> Self {
        CifError::ParseError(format!("{err}"))
    }
}

impl CifError {
    /// Create an InvalidStructure error with the given message (no location)
    pub(crate) fn invalid_structure(msg: impl Into<String>) -> Self {
        CifError::InvalidStructure {
            message: msg.into(),
            location: None,
        }
    }

    /// Add location information to this error
    pub(crate) fn at_location(self, line: usize, col: usize) -> Self {
        match self {
            CifError::InvalidStructure { message, .. } => CifError::InvalidStructure {
                message,
                location: Some((line, col)),
            },
            other => other, // Can't add location to ParseError or IoError
        }
    }
}
