//! # CIF Validator
//!
//! DDLm-based validation for CIF (Crystallographic Information File) format.
//!
//! This crate provides comprehensive validation of CIF files against DDLm dictionaries,
//! including:
//! - Dictionary loading and parsing (using cif-parser)
//! - Multi-dictionary composition
//! - Type system validation (Integer, Real, DateTime, etc.)
//! - Constraint checking (enumerations, ranges, mandatory items)
//! - Span preservation for IDE integration
//! - ValidatedCIF type for definition lookup at source positions
//!
//! ## Usage
//!
//! ```rust,ignore
//! use cif_parser::CifDocument;
//! use cif_validator::{Validator, ValidationMode};
//!
//! // Parse CIF file
//! let doc = CifDocument::from_file("structure.cif")?;
//!
//! // Create validator with dictionary
//! let validator = Validator::new()
//!     .with_dictionary_file("cif_core.dic")?
//!     .with_mode(ValidationMode::Strict);
//!
//! // Validate
//! let result = validator.validate(&doc)?;
//!
//! if !result.is_valid {
//!     for error in &result.errors {
//!         println!("{}", error);
//!     }
//! }
//! ```
//!
//! ## Architecture
//!
//! This validator is built as a separate crate from `cif-parser` to maintain:
//! - **Separation of concerns**: Syntax parsing vs semantic validation
//! - **Optional complexity**: Users can parse without validating
//! - **Performance**: Skip validation for performance-critical use cases
//! - **Binary size**: Keep parser lightweight for WASM/Python

pub mod dictionary;
pub mod error;
pub mod validated;
mod validator;

#[cfg(feature = "python")]
pub mod python;

// Re-exports
pub use dictionary::{
    Category, CategoryClass, ContainerType, ContentType, DataItem, Dictionary, DictionaryMetadata,
    Purpose, RangeConstraint, Source, TypeInfo, ValueConstraints,
};
pub use error::{
    DictionaryError, ErrorCategory, ValidationError, ValidationResult, ValidationWarning,
    WarningCategory,
};
pub use validated::{
    FromCifValue, Measurand, TypedValue, ValidatedBlock, ValidatedCif, ValidatedLoop, ValidatedRow,
};
pub use validator::{ValidationEngine, ValidationMode};

use cif_parser::CifDocument;
use std::sync::Arc;

/// Main validator builder for CIF documents.
///
/// # Example
///
/// ```rust,ignore
/// use cif_validator::{Validator, ValidationMode};
///
/// let result = Validator::new()
///     .with_dictionary_file("cif_core.dic")?
///     .with_mode(ValidationMode::Strict)
///     .validate(&doc)?;
/// ```
#[derive(Debug, Default)]
pub struct Validator {
    dictionaries: Vec<Arc<Dictionary>>,
    mode: ValidationMode,
}

impl Validator {
    /// Create a new validator with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load a dictionary from a file path.
    pub fn with_dictionary_file(
        mut self,
        path: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let doc = CifDocument::from_file(path)?;
        let dict = dictionary::load_dictionary(&doc).map_err(|errors| {
            let msg = errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            std::io::Error::new(std::io::ErrorKind::InvalidData, msg)
        })?;
        self.dictionaries.push(Arc::new(dict));
        Ok(self)
    }

    /// Load a dictionary from a CIF string.
    pub fn with_dictionary_str(
        mut self,
        content: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let doc = CifDocument::parse(content)?;
        let dict = dictionary::load_dictionary(&doc).map_err(|errors| {
            let msg = errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            std::io::Error::new(std::io::ErrorKind::InvalidData, msg)
        })?;
        self.dictionaries.push(Arc::new(dict));
        Ok(self)
    }

    /// Add a pre-loaded dictionary.
    pub fn with_dictionary(mut self, dict: Dictionary) -> Self {
        self.dictionaries.push(Arc::new(dict));
        self
    }

    /// Set the validation mode.
    pub fn with_mode(mut self, mode: ValidationMode) -> Self {
        self.mode = mode;
        self
    }

    /// Validate a CIF document.
    ///
    /// Returns a `ValidationResult` containing any errors and warnings.
    pub fn validate(
        &self,
        doc: &CifDocument,
    ) -> Result<ValidationResult, Box<dyn std::error::Error + Send + Sync>> {
        let combined = self.combine_dictionaries()?;
        let engine = ValidationEngine::new(&combined, self.mode);
        Ok(engine.validate(doc))
    }

    /// Validate and return a ValidatedCif with typed access.
    ///
    /// This allows looking up dictionary definitions at any source position.
    pub fn validate_typed(
        &self,
        doc: CifDocument,
    ) -> Result<ValidatedCif, Box<dyn std::error::Error + Send + Sync>> {
        let combined = Arc::new(self.combine_dictionaries()?);
        Ok(ValidatedCif::new(doc, combined))
    }

    /// Get the combined dictionary (for advanced use cases).
    pub fn combined_dictionary(
        &self,
    ) -> Result<Dictionary, Box<dyn std::error::Error + Send + Sync>> {
        self.combine_dictionaries()
    }

    fn combine_dictionaries(&self) -> Result<Dictionary, Box<dyn std::error::Error + Send + Sync>> {
        if self.dictionaries.is_empty() {
            return Err("No dictionaries loaded".into());
        }

        let mut combined = (*self.dictionaries[0]).clone();
        for dict in &self.dictionaries[1..] {
            combined.merge((**dict).clone());
        }
        Ok(combined)
    }
}

/// Convenience function to validate a CIF string against a dictionary file.
///
/// # Example
///
/// ```rust,ignore
/// use cif_validator::validate;
///
/// let result = validate(cif_content, "cif_core.dic")?;
/// println!("Valid: {}", result.is_valid);
/// ```
pub fn validate(
    cif_content: &str,
    dict_path: &str,
) -> Result<ValidationResult, Box<dyn std::error::Error + Send + Sync>> {
    let doc = CifDocument::parse(cif_content)?;
    Validator::new()
        .with_dictionary_file(dict_path)?
        .validate(&doc)
}

/// Convenience function to load a dictionary from a file.
pub fn load_dictionary_file(
    path: &str,
) -> Result<Dictionary, Box<dyn std::error::Error + Send + Sync>> {
    let doc = CifDocument::from_file(path)?;
    dictionary::load_dictionary(&doc).map_err(|errors| {
        let msg = errors
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, msg))
            as Box<dyn std::error::Error + Send + Sync>
    })
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
    fn test_full_validation_flow() {
        // Create a simple dictionary
        let dict_content = r#"
#\#CIF_2.0
data_TEST_DICT
    _dictionary.title             TEST_DICT
    _dictionary.version           1.0.0

save_cell
    _definition.id                CELL
    _definition.scope             Category
    _definition.class             Set
save_

save_cell.length_a
    _definition.id                '_cell.length_a'
    _name.category_id             cell
    _name.object_id               length_a
    _type.purpose                 Measurand
    _type.container               Single
    _type.contents                Real
    _enumeration.range            0.0:
    _description.text             'Unit cell length a in angstroms'
save_
"#;

        let validator = Validator::new()
            .with_dictionary_str(dict_content)
            .expect("Failed to load dictionary");

        // Valid CIF
        let valid_cif = r#"
data_test
_cell.length_a 10.5
"#;
        let doc = CifDocument::parse(valid_cif).unwrap();
        let result = validator.validate(&doc).unwrap();
        assert!(result.is_valid, "Expected valid, got: {:?}", result.errors);

        // Invalid CIF (negative value)
        let invalid_cif = r#"
data_test
_cell.length_a -5.0
"#;
        let doc = CifDocument::parse(invalid_cif).unwrap();
        let result = validator.validate(&doc).unwrap();
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_validated_cif_definition_lookup() {
        let dict_content = r#"
#\#CIF_2.0
data_TEST_DICT

save_cell.length_a
    _definition.id                '_cell.length_a'
    _type.contents                Real
    _description.text             'Unit cell length a'
save_
"#;

        let cif_content = r#"
data_test
_cell.length_a 10.5
"#;

        let validator = Validator::new()
            .with_dictionary_str(dict_content)
            .expect("Failed to load dictionary");

        let doc = CifDocument::parse(cif_content).unwrap();
        let validated = validator.validate_typed(doc).unwrap();

        // The value "10.5" should be on line 3 (1-indexed)
        // Note: exact position depends on parsing, this is a conceptual test
        let block = validated.first_block().unwrap();
        let (value, def) = block.get_with_def("_cell.length_a").unwrap();

        assert!(value.is_numeric());
        assert!(def.is_some());
        assert_eq!(
            def.unwrap().description,
            Some("Unit cell length a".to_string())
        );
    }
}
