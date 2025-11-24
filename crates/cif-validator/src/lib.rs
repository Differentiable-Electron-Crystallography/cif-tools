//! # CIF Validator
//!
//! DDLm-based validation for CIF (Crystallographic Information File) format.
//!
//! This crate provides comprehensive validation of CIF files against DDLm dictionaries,
//! including:
//! - Dictionary loading and parsing (using cif-parser)
//! - Multi-dictionary composition
//! - Type system validation (Integer, Real, DateTime, etc.)
//! - Constraint checking (enumerations, ranges)
//! - dREL expression evaluation (future)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use cif_parser::Document;
//! use cif_validator::{Validator, ValidationMode};
//!
//! // Parse CIF file
//! let doc = Document::parse(cif_content)?;
//!
//! // Create validator with core dictionary
//! let validator = Validator::new()
//!     .with_core()?
//!     .with_mode(ValidationMode::Strict);
//!
//! // Validate
//! let result = validator.validate(&doc)?;
//! ```
//!
//! ## Architecture
//!
//! This validator is built as a separate crate from `cif-parser` to maintain:
//! - **Separation of concerns**: Syntax parsing vs semantic validation
//! - **Optional complexity**: Users can parse without validating
//! - **Performance**: Skip validation for performance-critical use cases
//! - **Binary size**: Keep parser lightweight for WASM/Python
//!
//! ## Status
//!
//! 🚧 **Under Development** - This crate is in early development.
//!
//! Current capabilities:
//! - ✅ Workspace structure established
//! - ⚠️ Dictionary loading (planned)
//! - ⚠️ Type validation (planned)
//! - ⚠️ Constraint checking (planned)
//! - ⚠️ dREL evaluation (planned)

use cif_parser::Document;

/// Validation mode controlling strictness of validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationMode {
    /// Strict mode: All errors are fatal
    Strict,
    /// Lenient mode: Some errors become warnings
    Lenient,
    /// Pedantic mode: Include stylistic warnings
    Pedantic,
}

/// Result of validating a CIF document
#[derive(Debug)]
pub struct ValidationResult {
    /// Whether the document is valid
    pub is_valid: bool,
    /// Validation errors encountered
    pub errors: Vec<ValidationError>,
    /// Validation warnings (non-fatal issues)
    pub warnings: Vec<ValidationWarning>,
}

/// A validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Location in the document where error occurred
    pub location: String,
    /// Error message
    pub message: String,
    /// Error category
    pub category: ErrorCategory,
}

/// Categories of validation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Unknown data name (not in dictionary)
    UnknownDataName,
    /// Invalid data type for this name
    TypeError,
    /// Value outside allowed range
    RangeError,
    /// Value not in enumerated list
    EnumerationError,
    /// Missing mandatory data item
    MissingMandatory,
    /// Invalid loop structure
    LoopStructure,
}

/// A validation warning (non-fatal)
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    /// Location in the document
    pub location: String,
    /// Warning message
    pub message: String,
}

/// Main validator for CIF documents
///
/// # Example
///
/// ```rust,ignore
/// let validator = Validator::new()
///     .with_core()?
///     .with_dictionary("cif_pow.dic")?;
/// ```
#[derive(Debug)]
pub struct Validator {
    mode: ValidationMode,
    // TODO: Add dictionary storage
}

impl Validator {
    /// Create a new validator with default settings
    pub fn new() -> Self {
        Self {
            mode: ValidationMode::Strict,
        }
    }

    /// Set the validation mode
    pub fn with_mode(mut self, mode: ValidationMode) -> Self {
        self.mode = mode;
        self
    }

    /// Load the CIF core dictionary
    ///
    /// TODO: Implement dictionary loading
    pub fn with_core(self) -> Result<Self, String> {
        // TODO: Load cif_core.dic
        Ok(self)
    }

    /// Load an additional dictionary file
    ///
    /// TODO: Implement dictionary loading and composition
    pub fn with_dictionary(self, _path: &str) -> Result<Self, String> {
        // TODO: Load dictionary at path
        Ok(self)
    }

    /// Validate a CIF document
    ///
    /// TODO: Implement actual validation logic
    pub fn validate(&self, _doc: &Document) -> Result<ValidationResult, String> {
        // Stub implementation - always returns valid
        Ok(ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = Validator::new();
        assert_eq!(validator.mode, ValidationMode::Strict);
    }

    #[test]
    fn test_validation_modes() {
        let strict = Validator::new().with_mode(ValidationMode::Strict);
        let lenient = Validator::new().with_mode(ValidationMode::Lenient);
        let pedantic = Validator::new().with_mode(ValidationMode::Pedantic);

        assert_eq!(strict.mode, ValidationMode::Strict);
        assert_eq!(lenient.mode, ValidationMode::Lenient);
        assert_eq!(pedantic.mode, ValidationMode::Pedantic);
    }

    #[test]
    fn test_stub_validation() {
        // This test will be replaced with real validation tests
        let validator = Validator::new();
        let doc_str = "data_test\n_test.value 42\n";
        let doc = Document::parse(doc_str).unwrap();

        let result = validator.validate(&doc).unwrap();
        assert!(result.is_valid);
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.warnings.len(), 0);
    }
}
