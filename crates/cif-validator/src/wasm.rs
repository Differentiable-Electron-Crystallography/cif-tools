//! WebAssembly bindings for the CIF validator.
//!
//! This module provides JavaScript-compatible wrappers around the core CIF validation
//! functionality, using wasm-bindgen for seamless interop with JavaScript.

use crate::{
    ErrorCategory, ValidationError, ValidationMode, ValidationResult, ValidationWarning, Validator,
    WarningCategory,
};
use cif_parser::CifDocument;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// Console logging for debugging
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

/// JavaScript-compatible representation of error categories
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JsErrorCategory {
    /// Unknown data name (not in dictionary)
    UnknownDataName = 0,
    /// Type mismatch (e.g., text where Real expected)
    TypeError = 1,
    /// Value outside allowed range
    RangeError = 2,
    /// Value not in enumerated set
    EnumerationError = 3,
    /// Missing mandatory item
    MissingMandatory = 4,
    /// Invalid loop structure
    LoopStructure = 5,
    /// Foreign key reference error
    LinkError = 6,
    /// Dictionary loading/parsing error
    DictionaryError = 7,
}

impl From<ErrorCategory> for JsErrorCategory {
    fn from(category: ErrorCategory) -> Self {
        match category {
            ErrorCategory::UnknownDataName => JsErrorCategory::UnknownDataName,
            ErrorCategory::TypeError => JsErrorCategory::TypeError,
            ErrorCategory::RangeError => JsErrorCategory::RangeError,
            ErrorCategory::EnumerationError => JsErrorCategory::EnumerationError,
            ErrorCategory::MissingMandatory => JsErrorCategory::MissingMandatory,
            ErrorCategory::LoopStructure => JsErrorCategory::LoopStructure,
            ErrorCategory::LinkError => JsErrorCategory::LinkError,
            ErrorCategory::DictionaryError => JsErrorCategory::DictionaryError,
        }
    }
}

/// JavaScript-compatible representation of warning categories
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JsWarningCategory {
    /// Mixed categories in a loop
    MixedCategories = 0,
    /// Deprecated item usage
    DeprecatedItem = 1,
    /// Style recommendation
    Style = 2,
    /// Unknown item in lenient mode
    UnknownItem = 3,
}

impl From<WarningCategory> for JsWarningCategory {
    fn from(category: WarningCategory) -> Self {
        match category {
            WarningCategory::MixedCategories => JsWarningCategory::MixedCategories,
            WarningCategory::DeprecatedItem => JsWarningCategory::DeprecatedItem,
            WarningCategory::Style => JsWarningCategory::Style,
            WarningCategory::UnknownItem => JsWarningCategory::UnknownItem,
        }
    }
}

/// JavaScript-compatible representation of validation modes
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JsValidationMode {
    /// Strict validation - all checks enabled
    Strict = 0,
    /// Lenient validation - unknown items are warnings
    Lenient = 1,
    /// Pedantic validation - extra style checks
    Pedantic = 2,
}

impl From<JsValidationMode> for ValidationMode {
    fn from(mode: JsValidationMode) -> Self {
        match mode {
            JsValidationMode::Strict => ValidationMode::Strict,
            JsValidationMode::Lenient => ValidationMode::Lenient,
            JsValidationMode::Pedantic => ValidationMode::Pedantic,
        }
    }
}

/// JavaScript-compatible representation of a source span for validation
#[wasm_bindgen(js_name = "ValidatorSpan")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct JsValidatorSpan {
    start_line: usize,
    start_col: usize,
    end_line: usize,
    end_col: usize,
}

#[wasm_bindgen(js_class = "ValidatorSpan")]
impl JsValidatorSpan {
    /// Starting line number (1-indexed)
    #[wasm_bindgen(getter = startLine)]
    pub fn start_line(&self) -> usize {
        self.start_line
    }

    /// Starting column number (1-indexed)
    #[wasm_bindgen(getter = startCol)]
    pub fn start_col(&self) -> usize {
        self.start_col
    }

    /// Ending line number (1-indexed)
    #[wasm_bindgen(getter = endLine)]
    pub fn end_line(&self) -> usize {
        self.end_line
    }

    /// Ending column number (1-indexed)
    #[wasm_bindgen(getter = endCol)]
    pub fn end_col(&self) -> usize {
        self.end_col
    }
}

impl From<cif_parser::Span> for JsValidatorSpan {
    fn from(span: cif_parser::Span) -> Self {
        JsValidatorSpan {
            start_line: span.start_line,
            start_col: span.start_col,
            end_line: span.end_line,
            end_col: span.end_col,
        }
    }
}

/// JavaScript-compatible representation of a validation error
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsValidationError {
    category: JsErrorCategory,
    message: String,
    span: JsValidatorSpan,
    data_name: Option<String>,
    expected: Option<String>,
    actual: Option<String>,
    suggestions: Vec<String>,
}

#[wasm_bindgen]
impl JsValidationError {
    /// Get the error category
    #[wasm_bindgen(getter)]
    pub fn category(&self) -> JsErrorCategory {
        self.category
    }

    /// Get the error message
    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }

    /// Get the source span
    #[wasm_bindgen(getter)]
    pub fn span(&self) -> JsValidatorSpan {
        self.span
    }

    /// Get the data name involved (if applicable)
    #[wasm_bindgen(getter = dataName)]
    pub fn data_name(&self) -> Option<String> {
        self.data_name.clone()
    }

    /// Get the expected value/type
    #[wasm_bindgen(getter)]
    pub fn expected(&self) -> Option<String> {
        self.expected.clone()
    }

    /// Get the actual value found
    #[wasm_bindgen(getter)]
    pub fn actual(&self) -> Option<String> {
        self.actual.clone()
    }

    /// Get suggestions for fixing the error
    #[wasm_bindgen(getter)]
    pub fn suggestions(&self) -> Vec<String> {
        self.suggestions.clone()
    }

    /// Get a formatted string representation
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string_js(&self) -> String {
        format!(
            "{} at line {}, col {}",
            self.message, self.span.start_line, self.span.start_col
        )
    }
}

impl From<&ValidationError> for JsValidationError {
    fn from(error: &ValidationError) -> Self {
        JsValidationError {
            category: error.category.into(),
            message: error.message.clone(),
            span: error.span.into(),
            data_name: error.data_name.clone(),
            expected: error.expected.clone(),
            actual: error.actual.clone(),
            suggestions: error.suggestions.clone(),
        }
    }
}

/// JavaScript-compatible representation of a validation warning
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsValidationWarning {
    category: JsWarningCategory,
    message: String,
    span: JsValidatorSpan,
}

#[wasm_bindgen]
impl JsValidationWarning {
    /// Get the warning category
    #[wasm_bindgen(getter)]
    pub fn category(&self) -> JsWarningCategory {
        self.category
    }

    /// Get the warning message
    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }

    /// Get the source span
    #[wasm_bindgen(getter)]
    pub fn span(&self) -> JsValidatorSpan {
        self.span
    }

    /// Get a formatted string representation
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string_js(&self) -> String {
        format!(
            "{} at line {}, col {}",
            self.message, self.span.start_line, self.span.start_col
        )
    }
}

impl From<&ValidationWarning> for JsValidationWarning {
    fn from(warning: &ValidationWarning) -> Self {
        JsValidationWarning {
            category: warning.category.into(),
            message: warning.message.clone(),
            span: warning.span.into(),
        }
    }
}

/// JavaScript-compatible representation of a validation result
#[wasm_bindgen]
pub struct JsValidationResult {
    is_valid: bool,
    errors: Vec<JsValidationError>,
    warnings: Vec<JsValidationWarning>,
}

#[wasm_bindgen]
impl JsValidationResult {
    /// Check if the document is valid (no errors)
    #[wasm_bindgen(getter = isValid)]
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }

    /// Get the number of errors
    #[wasm_bindgen(getter = errorCount)]
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Get the number of warnings
    #[wasm_bindgen(getter = warningCount)]
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    /// Get an error by index
    #[wasm_bindgen]
    pub fn get_error(&self, index: usize) -> Option<JsValidationError> {
        self.errors.get(index).cloned()
    }

    /// Get a warning by index
    #[wasm_bindgen]
    pub fn get_warning(&self, index: usize) -> Option<JsValidationWarning> {
        self.warnings.get(index).cloned()
    }

    /// Get all errors as a JavaScript array
    #[wasm_bindgen(getter)]
    pub fn errors(&self) -> JsValue {
        match serde_wasm_bindgen::to_value(&self.errors) {
            Ok(value) => value,
            Err(e) => {
                console_log!("Error serializing errors: {:?}", e);
                JsValue::UNDEFINED
            }
        }
    }

    /// Get all warnings as a JavaScript array
    #[wasm_bindgen(getter)]
    pub fn warnings(&self) -> JsValue {
        match serde_wasm_bindgen::to_value(&self.warnings) {
            Ok(value) => value,
            Err(e) => {
                console_log!("Error serializing warnings: {:?}", e);
                JsValue::UNDEFINED
            }
        }
    }

    /// Get all error messages as strings
    #[wasm_bindgen(getter = errorMessages)]
    pub fn error_messages(&self) -> Vec<String> {
        self.errors.iter().map(|e| e.to_string_js()).collect()
    }

    /// Get all warning messages as strings
    #[wasm_bindgen(getter = warningMessages)]
    pub fn warning_messages(&self) -> Vec<String> {
        self.warnings.iter().map(|w| w.to_string_js()).collect()
    }
}

impl From<ValidationResult> for JsValidationResult {
    fn from(result: ValidationResult) -> Self {
        JsValidationResult {
            is_valid: result.is_valid,
            errors: result.errors.iter().map(|e| e.into()).collect(),
            warnings: result.warnings.iter().map(|w| w.into()).collect(),
        }
    }
}

/// JavaScript-compatible CIF validator
#[wasm_bindgen]
pub struct JsValidator {
    dictionaries: Vec<String>,
    mode: ValidationMode,
}

#[wasm_bindgen]
impl JsValidator {
    /// Create a new validator
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_log!("Creating new JsValidator");
        JsValidator {
            dictionaries: Vec::new(),
            mode: ValidationMode::Strict,
        }
    }

    /// Add a dictionary from a string
    #[wasm_bindgen(js_name = addDictionary)]
    pub fn add_dictionary(&mut self, dictionary_content: &str) -> Result<(), JsValue> {
        // Validate that the dictionary can be parsed
        CifDocument::parse(dictionary_content).map_err(|e| {
            JsValue::from(js_sys::Error::new(&format!(
                "Failed to parse dictionary: {}",
                e
            )))
        })?;
        self.dictionaries.push(dictionary_content.to_string());
        console_log!("Added dictionary ({} total)", self.dictionaries.len());
        Ok(())
    }

    /// Set the validation mode
    #[wasm_bindgen(js_name = setMode)]
    pub fn set_mode(&mut self, mode: JsValidationMode) {
        self.mode = mode.into();
        console_log!("Set validation mode to {:?}", self.mode);
    }

    /// Validate a CIF document
    #[wasm_bindgen]
    pub fn validate(&self, cif_content: &str) -> Result<JsValidationResult, JsValue> {
        console_log!("Validating CIF content ({} bytes)", cif_content.len());

        // Parse the CIF content
        let doc = CifDocument::parse(cif_content).map_err(|e| {
            JsValue::from(js_sys::Error::new(&format!(
                "Failed to parse CIF content: {}",
                e
            )))
        })?;

        // Build the validator
        let mut validator = Validator::new().with_mode(self.mode);

        for dict_content in &self.dictionaries {
            validator = validator.with_dictionary_str(dict_content).map_err(|e| {
                JsValue::from(js_sys::Error::new(&format!(
                    "Failed to load dictionary: {}",
                    e
                )))
            })?;
        }

        // If no dictionaries loaded, return error
        if self.dictionaries.is_empty() {
            return Err(JsValue::from(js_sys::Error::new(
                "No dictionaries loaded. Call addDictionary() first.",
            )));
        }

        // Validate
        let result = validator
            .validate(&doc)
            .map_err(|e| JsValue::from(js_sys::Error::new(&format!("Validation failed: {}", e))))?;

        console_log!(
            "Validation complete: {} errors, {} warnings",
            result.error_count(),
            result.warning_count()
        );

        Ok(result.into())
    }
}

impl Default for JsValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate a CIF string against a dictionary string (convenience function)
///
/// This is a simple one-shot validation function. For validating multiple
/// documents against the same dictionary, use `JsValidator` instead.
#[wasm_bindgen]
pub fn validate(
    cif_content: &str,
    dictionary_content: &str,
) -> Result<JsValidationResult, JsValue> {
    console_log!(
        "Validating CIF ({} bytes) against dictionary ({} bytes)",
        cif_content.len(),
        dictionary_content.len()
    );

    // Parse the CIF content
    let doc = CifDocument::parse(cif_content).map_err(|e| {
        JsValue::from(js_sys::Error::new(&format!(
            "Failed to parse CIF content: {}",
            e
        )))
    })?;

    // Create validator with dictionary
    let validator = Validator::new()
        .with_dictionary_str(dictionary_content)
        .map_err(|e| {
            JsValue::from(js_sys::Error::new(&format!(
                "Failed to load dictionary: {}",
                e
            )))
        })?;

    // Validate
    let result = validator
        .validate(&doc)
        .map_err(|e| JsValue::from(js_sys::Error::new(&format!("Validation failed: {}", e))))?;

    console_log!(
        "Validation complete: {} errors, {} warnings",
        result.error_count(),
        result.warning_count()
    );

    Ok(result.into())
}

/// Get the version of the CIF validator
#[wasm_bindgen(js_name = "validatorVersion")]
pub fn validator_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Get the author of the CIF validator
#[wasm_bindgen(js_name = "validatorAuthor")]
pub fn validator_author() -> String {
    "Iain Maitland".to_string()
}

/// Initialize the WASM module (call this explicitly if needed)
#[wasm_bindgen(js_name = "initValidator")]
pub fn validator_init() {
    console_log!("CIF Validator WASM module initialized");
}

/// Simple test function to verify WASM is working
#[wasm_bindgen(js_name = "testValidatorWasm")]
pub fn test_validator_wasm() -> String {
    console_log!("WASM test function called");
    "CIF Validator WASM module is working!".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_validator_creation() {
        let validator = JsValidator::new();
        assert!(validator.dictionaries.is_empty());
        assert_eq!(validator.mode, ValidationMode::Strict);
    }

    #[test]
    fn test_error_category_conversion() {
        assert_eq!(
            JsErrorCategory::from(ErrorCategory::TypeError),
            JsErrorCategory::TypeError
        );
        assert_eq!(
            JsErrorCategory::from(ErrorCategory::RangeError),
            JsErrorCategory::RangeError
        );
    }

    #[test]
    fn test_validation_mode_conversion() {
        assert_eq!(
            ValidationMode::from(JsValidationMode::Strict),
            ValidationMode::Strict
        );
        assert_eq!(
            ValidationMode::from(JsValidationMode::Lenient),
            ValidationMode::Lenient
        );
    }
}
