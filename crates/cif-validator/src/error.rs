//! Error types for CIF validation with span preservation.
//!
//! All errors include source locations for IDE integration and rich error messages.

use cif_parser::Span;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Categories of validation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Unknown data name (not in dictionary)
    UnknownDataName,
    /// Type mismatch (e.g., text where Real expected)
    TypeError,
    /// Value outside allowed range
    RangeError,
    /// Value not in enumerated set
    EnumerationError,
    /// Missing mandatory item
    MissingMandatory,
    /// Invalid loop structure
    LoopStructure,
    /// Foreign key reference error
    LinkError,
    /// Dictionary loading/parsing error
    DictionaryError,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownDataName => write!(f, "unknown data name"),
            Self::TypeError => write!(f, "type error"),
            Self::RangeError => write!(f, "range error"),
            Self::EnumerationError => write!(f, "enumeration error"),
            Self::MissingMandatory => write!(f, "missing mandatory item"),
            Self::LoopStructure => write!(f, "loop structure error"),
            Self::LinkError => write!(f, "link error"),
            Self::DictionaryError => write!(f, "dictionary error"),
        }
    }
}

/// A validation error with full context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error category for programmatic handling
    pub category: ErrorCategory,
    /// Human-readable message
    pub message: String,
    /// Primary source location in input CIF
    pub span: Span,
    /// The data name involved (if applicable)
    pub data_name: Option<String>,
    /// Expected value/type (for type/enum errors)
    pub expected: Option<String>,
    /// Actual value found
    pub actual: Option<String>,
    /// Location in dictionary where this item is defined
    pub definition_span: Option<Span>,
    /// Suggestions for fixing the error
    pub suggestions: Vec<String>,
}

impl ValidationError {
    /// Create a new validation error
    pub fn new(category: ErrorCategory, message: impl Into<String>, span: Span) -> Self {
        Self {
            category,
            message: message.into(),
            span,
            data_name: None,
            expected: None,
            actual: None,
            definition_span: None,
            suggestions: Vec::new(),
        }
    }

    /// Create an unknown data name error
    pub fn unknown_data_name(name: impl Into<String>, span: Span) -> Self {
        let name = name.into();
        Self {
            category: ErrorCategory::UnknownDataName,
            message: format!("Unknown data name '{}'", name),
            span,
            data_name: Some(name),
            expected: None,
            actual: None,
            definition_span: None,
            suggestions: Vec::new(),
        }
    }

    /// Create a type error
    pub fn type_error(
        name: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
        span: Span,
    ) -> Self {
        let name = name.into();
        let expected = expected.into();
        let actual = actual.into();
        Self {
            category: ErrorCategory::TypeError,
            message: format!(
                "Type error for '{}': expected {}, got {}",
                name, expected, actual
            ),
            span,
            data_name: Some(name),
            expected: Some(expected),
            actual: Some(actual),
            definition_span: None,
            suggestions: Vec::new(),
        }
    }

    /// Create a range error
    pub fn range_error(
        name: impl Into<String>,
        value: f64,
        min: Option<f64>,
        max: Option<f64>,
        span: Span,
    ) -> Self {
        let name = name.into();
        let range_desc = match (min, max) {
            (Some(min), Some(max)) => format!("{} to {}", min, max),
            (Some(min), None) => format!(">= {}", min),
            (None, Some(max)) => format!("<= {}", max),
            (None, None) => "any value".to_string(),
        };

        Self {
            category: ErrorCategory::RangeError,
            message: format!(
                "Value {} for '{}' is outside allowed range {}",
                value, name, range_desc
            ),
            span,
            data_name: Some(name),
            expected: Some(range_desc),
            actual: Some(value.to_string()),
            definition_span: None,
            suggestions: Vec::new(),
        }
    }

    /// Create an enumeration error
    pub fn enumeration_error(
        name: impl Into<String>,
        actual: impl Into<String>,
        allowed: &[String],
        span: Span,
    ) -> Self {
        let name = name.into();
        let actual = actual.into();
        let allowed_str = allowed.join(", ");

        Self {
            category: ErrorCategory::EnumerationError,
            message: format!(
                "Value '{}' for '{}' is not in allowed values: [{}]",
                actual, name, allowed_str
            ),
            span,
            data_name: Some(name),
            expected: Some(format!("one of [{}]", allowed_str)),
            actual: Some(actual),
            definition_span: None,
            suggestions: Vec::new(),
        }
    }

    /// Create a missing mandatory item error
    pub fn missing_mandatory(name: impl Into<String>, block_span: Span) -> Self {
        let name = name.into();
        Self {
            category: ErrorCategory::MissingMandatory,
            message: format!("Missing mandatory item '{}'", name),
            span: block_span,
            data_name: Some(name),
            expected: None,
            actual: None,
            definition_span: None,
            suggestions: Vec::new(),
        }
    }

    /// Create a loop structure error
    pub fn loop_structure(message: impl Into<String>, span: Span) -> Self {
        Self {
            category: ErrorCategory::LoopStructure,
            message: message.into(),
            span,
            data_name: None,
            expected: None,
            actual: None,
            definition_span: None,
            suggestions: Vec::new(),
        }
    }

    /// Add a suggestion to this error
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }

    /// Add multiple suggestions to this error
    pub fn with_suggestions(mut self, suggestions: impl IntoIterator<Item = String>) -> Self {
        self.suggestions.extend(suggestions);
        self
    }

    /// Set the definition span
    pub fn with_definition_span(mut self, span: Span) -> Self {
        self.definition_span = Some(span);
        self
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at line {}, col {}",
            self.message, self.span.start_line, self.span.start_col
        )?;

        if !self.suggestions.is_empty() {
            write!(f, " (suggestions: {})", self.suggestions.join(", "))?;
        }

        Ok(())
    }
}

impl std::error::Error for ValidationError {}

/// Warning categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WarningCategory {
    /// Mixed categories in a loop
    MixedCategories,
    /// Deprecated item usage
    DeprecatedItem,
    /// Style recommendation
    Style,
    /// Unknown item in lenient mode
    UnknownItem,
}

/// A validation warning (non-fatal)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Warning category
    pub category: WarningCategory,
    /// Human-readable message
    pub message: String,
    /// Source location
    pub span: Span,
}

impl ValidationWarning {
    /// Create a new warning
    pub fn new(category: WarningCategory, message: impl Into<String>, span: Span) -> Self {
        Self {
            category,
            message: message.into(),
            span,
        }
    }

    /// Create a mixed categories warning
    pub fn mixed_categories(categories: &[String], span: Span) -> Self {
        Self {
            category: WarningCategory::MixedCategories,
            message: format!(
                "Loop contains items from multiple categories: [{}]",
                categories.join(", ")
            ),
            span,
        }
    }
}

impl fmt::Display for ValidationWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at line {}, col {}",
            self.message, self.span.start_line, self.span.start_col
        )
    }
}

/// Error type for dictionary loading/parsing
#[derive(Debug, Clone, Error)]
pub enum DictionaryError {
    /// CIF parsing error
    #[error("Failed to parse dictionary: {message}")]
    ParseError { message: String, span: Option<Span> },

    /// Missing required field in dictionary definition
    #[error("Missing required field '{field}' in definition for '{item}'")]
    MissingField {
        item: String,
        field: String,
        span: Span,
    },

    /// Invalid field value
    #[error("Invalid value for '{field}' in '{item}': {message}")]
    InvalidField {
        item: String,
        field: String,
        message: String,
        span: Span,
    },

    /// dREL parsing error
    #[error("Invalid dREL method in '{item}': {message}")]
    InvalidDrel {
        item: String,
        message: String,
        span: Span,
    },

    /// dREL references unknown item
    #[error("dREL method in '{item}' references unknown item '{referenced}'")]
    MissingDrelReference {
        item: String,
        referenced: String,
        span: Span,
    },

    /// IO error
    #[error("IO error: {0}")]
    IoError(String),
}

impl DictionaryError {
    /// Get the span associated with this error, if any
    pub fn span(&self) -> Option<Span> {
        match self {
            Self::ParseError { span, .. } => *span,
            Self::MissingField { span, .. } => Some(*span),
            Self::InvalidField { span, .. } => Some(*span),
            Self::InvalidDrel { span, .. } => Some(*span),
            Self::MissingDrelReference { span, .. } => Some(*span),
            Self::IoError(_) => None,
        }
    }
}

/// Result of validating a CIF document
#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    /// Whether the document is valid (no errors)
    pub is_valid: bool,
    /// Validation errors encountered
    pub errors: Vec<ValidationError>,
    /// Validation warnings (non-fatal issues)
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    /// Create a new empty result (valid)
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Add an error (marks result as invalid)
    pub fn add_error(&mut self, error: ValidationError) {
        self.is_valid = false;
        self.errors.push(error);
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    /// Get error count
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Get warning count
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }
}
