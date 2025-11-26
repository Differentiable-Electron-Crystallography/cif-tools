//! Python bindings for cif-validator using PyO3
//!
//! This module provides Python bindings for CIF validation functionality.
//! It is only compiled when the `python` feature is enabled.

use pyo3::prelude::*;

use crate::{ErrorCategory, ValidationMode, ValidationWarning, Validator, WarningCategory};
use cif_parser::{CifDocument, Span};

/// Python wrapper for source location (Span)
///
/// Tracks where a value or error appears in the source CIF file.
/// Useful for LSP/IDE features, error reporting, and highlighting.
#[pyclass(name = "Span")]
#[derive(Clone, Copy)]
pub struct PySpan {
    inner: Span,
}

#[pymethods]
impl PySpan {
    /// Starting line number (1-indexed)
    #[getter]
    fn start_line(&self) -> usize {
        self.inner.start_line
    }

    /// Starting column number (1-indexed)
    #[getter]
    fn start_col(&self) -> usize {
        self.inner.start_col
    }

    /// Ending line number (1-indexed)
    #[getter]
    fn end_line(&self) -> usize {
        self.inner.end_line
    }

    /// Ending column number (1-indexed)
    #[getter]
    fn end_col(&self) -> usize {
        self.inner.end_col
    }

    /// Check if a line and column position is within this span
    fn contains(&self, line: usize, col: usize) -> bool {
        self.inner.contains(line, col)
    }

    /// String representation (e.g., "1:5-3:10")
    fn __str__(&self) -> String {
        format!("{}", self.inner)
    }

    /// Debug representation
    fn __repr__(&self) -> String {
        format!(
            "Span(start_line={}, start_col={}, end_line={}, end_col={})",
            self.inner.start_line, self.inner.start_col, self.inner.end_line, self.inner.end_col
        )
    }
}

impl From<Span> for PySpan {
    fn from(span: Span) -> Self {
        PySpan { inner: span }
    }
}

/// Python enum for error categories
#[pyclass(name = "ErrorCategory", eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PyErrorCategory {
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

#[pymethods]
impl PyErrorCategory {
    fn __str__(&self) -> &'static str {
        match self {
            PyErrorCategory::UnknownDataName => "unknown data name",
            PyErrorCategory::TypeError => "type error",
            PyErrorCategory::RangeError => "range error",
            PyErrorCategory::EnumerationError => "enumeration error",
            PyErrorCategory::MissingMandatory => "missing mandatory item",
            PyErrorCategory::LoopStructure => "loop structure error",
            PyErrorCategory::LinkError => "link error",
            PyErrorCategory::DictionaryError => "dictionary error",
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "ErrorCategory.{}",
            match self {
                PyErrorCategory::UnknownDataName => "UnknownDataName",
                PyErrorCategory::TypeError => "TypeError",
                PyErrorCategory::RangeError => "RangeError",
                PyErrorCategory::EnumerationError => "EnumerationError",
                PyErrorCategory::MissingMandatory => "MissingMandatory",
                PyErrorCategory::LoopStructure => "LoopStructure",
                PyErrorCategory::LinkError => "LinkError",
                PyErrorCategory::DictionaryError => "DictionaryError",
            }
        )
    }
}

impl From<ErrorCategory> for PyErrorCategory {
    fn from(category: ErrorCategory) -> Self {
        match category {
            ErrorCategory::UnknownDataName => PyErrorCategory::UnknownDataName,
            ErrorCategory::TypeError => PyErrorCategory::TypeError,
            ErrorCategory::RangeError => PyErrorCategory::RangeError,
            ErrorCategory::EnumerationError => PyErrorCategory::EnumerationError,
            ErrorCategory::MissingMandatory => PyErrorCategory::MissingMandatory,
            ErrorCategory::LoopStructure => PyErrorCategory::LoopStructure,
            ErrorCategory::LinkError => PyErrorCategory::LinkError,
            ErrorCategory::DictionaryError => PyErrorCategory::DictionaryError,
        }
    }
}

/// Python enum for warning categories
#[pyclass(name = "WarningCategory", eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PyWarningCategory {
    /// Mixed categories in a loop
    MixedCategories = 0,
    /// Deprecated item usage
    DeprecatedItem = 1,
    /// Style recommendation
    Style = 2,
    /// Unknown item in lenient mode
    UnknownItem = 3,
}

#[pymethods]
impl PyWarningCategory {
    fn __str__(&self) -> &'static str {
        match self {
            PyWarningCategory::MixedCategories => "mixed categories",
            PyWarningCategory::DeprecatedItem => "deprecated item",
            PyWarningCategory::Style => "style",
            PyWarningCategory::UnknownItem => "unknown item",
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "WarningCategory.{}",
            match self {
                PyWarningCategory::MixedCategories => "MixedCategories",
                PyWarningCategory::DeprecatedItem => "DeprecatedItem",
                PyWarningCategory::Style => "Style",
                PyWarningCategory::UnknownItem => "UnknownItem",
            }
        )
    }
}

impl From<WarningCategory> for PyWarningCategory {
    fn from(category: WarningCategory) -> Self {
        match category {
            WarningCategory::MixedCategories => PyWarningCategory::MixedCategories,
            WarningCategory::DeprecatedItem => PyWarningCategory::DeprecatedItem,
            WarningCategory::Style => PyWarningCategory::Style,
            WarningCategory::UnknownItem => PyWarningCategory::UnknownItem,
        }
    }
}

/// Python enum for validation modes
#[pyclass(name = "ValidationMode", eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PyValidationMode {
    /// Strict validation - all checks enabled
    Strict = 0,
    /// Lenient validation - unknown items are warnings
    Lenient = 1,
    /// Pedantic validation - extra style checks
    Pedantic = 2,
}

#[pymethods]
impl PyValidationMode {
    fn __str__(&self) -> &'static str {
        match self {
            PyValidationMode::Strict => "strict",
            PyValidationMode::Lenient => "lenient",
            PyValidationMode::Pedantic => "pedantic",
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "ValidationMode.{}",
            match self {
                PyValidationMode::Strict => "Strict",
                PyValidationMode::Lenient => "Lenient",
                PyValidationMode::Pedantic => "Pedantic",
            }
        )
    }
}

impl From<PyValidationMode> for ValidationMode {
    fn from(mode: PyValidationMode) -> Self {
        match mode {
            PyValidationMode::Strict => ValidationMode::Strict,
            PyValidationMode::Lenient => ValidationMode::Lenient,
            PyValidationMode::Pedantic => ValidationMode::Pedantic,
        }
    }
}

/// A validation error with full context and span information
#[pyclass(name = "ValidationError")]
#[derive(Clone)]
pub struct PyValidationError {
    /// Error category for programmatic handling
    #[pyo3(get)]
    pub category: PyErrorCategory,
    /// Human-readable message
    #[pyo3(get)]
    pub message: String,
    /// Primary source location in input CIF
    #[pyo3(get)]
    pub span: PySpan,
    /// The data name involved (if applicable)
    #[pyo3(get)]
    pub data_name: Option<String>,
    /// Expected value/type (for type/enum errors)
    #[pyo3(get)]
    pub expected: Option<String>,
    /// Actual value found
    #[pyo3(get)]
    pub actual: Option<String>,
    /// Suggestions for fixing the error
    #[pyo3(get)]
    pub suggestions: Vec<String>,
}

#[pymethods]
impl PyValidationError {
    fn __str__(&self) -> String {
        format!(
            "{} at line {}, col {}",
            self.message, self.span.inner.start_line, self.span.inner.start_col
        )
    }

    fn __repr__(&self) -> String {
        format!(
            "ValidationError(category={}, message='{}', span={})",
            self.category.__repr__(),
            self.message,
            self.span.__repr__()
        )
    }
}

impl From<&crate::ValidationError> for PyValidationError {
    fn from(error: &crate::ValidationError) -> Self {
        PyValidationError {
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

/// A validation warning (non-fatal)
#[pyclass(name = "ValidationWarning")]
#[derive(Clone)]
pub struct PyValidationWarning {
    /// Warning category
    #[pyo3(get)]
    pub category: PyWarningCategory,
    /// Human-readable message
    #[pyo3(get)]
    pub message: String,
    /// Source location
    #[pyo3(get)]
    pub span: PySpan,
}

#[pymethods]
impl PyValidationWarning {
    fn __str__(&self) -> String {
        format!(
            "{} at line {}, col {}",
            self.message, self.span.inner.start_line, self.span.inner.start_col
        )
    }

    fn __repr__(&self) -> String {
        format!(
            "ValidationWarning(category={}, message='{}', span={})",
            self.category.__repr__(),
            self.message,
            self.span.__repr__()
        )
    }
}

impl From<&ValidationWarning> for PyValidationWarning {
    fn from(warning: &ValidationWarning) -> Self {
        PyValidationWarning {
            category: warning.category.into(),
            message: warning.message.clone(),
            span: warning.span.into(),
        }
    }
}

/// Result of validating a CIF document
#[pyclass(name = "ValidationResult")]
#[derive(Clone)]
pub struct PyValidationResult {
    /// Whether the document is valid (no errors)
    #[pyo3(get)]
    pub is_valid: bool,
    /// Validation errors encountered
    #[pyo3(get)]
    pub errors: Vec<PyValidationError>,
    /// Validation warnings (non-fatal issues)
    #[pyo3(get)]
    pub warnings: Vec<PyValidationWarning>,
}

#[pymethods]
impl PyValidationResult {
    /// Get the number of errors
    #[getter]
    fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Get the number of warnings
    #[getter]
    fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    fn __str__(&self) -> String {
        if self.is_valid {
            format!("Valid ({} warnings)", self.warnings.len())
        } else {
            format!(
                "Invalid ({} errors, {} warnings)",
                self.errors.len(),
                self.warnings.len()
            )
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "ValidationResult(is_valid={}, errors={}, warnings={})",
            self.is_valid,
            self.errors.len(),
            self.warnings.len()
        )
    }

    fn __bool__(&self) -> bool {
        self.is_valid
    }
}

impl From<crate::ValidationResult> for PyValidationResult {
    fn from(result: crate::ValidationResult) -> Self {
        PyValidationResult {
            is_valid: result.is_valid,
            errors: result.errors.iter().map(|e| e.into()).collect(),
            warnings: result.warnings.iter().map(|w| w.into()).collect(),
        }
    }
}

/// CIF Validator class for validating CIF documents against DDLm dictionaries
#[pyclass(name = "Validator")]
pub struct PyValidator {
    dictionaries: Vec<String>,
    mode: ValidationMode,
}

#[pymethods]
impl PyValidator {
    /// Create a new validator
    #[new]
    fn new() -> Self {
        PyValidator {
            dictionaries: Vec::new(),
            mode: ValidationMode::Strict,
        }
    }

    /// Add a dictionary from a string
    fn add_dictionary(&mut self, dictionary_content: &str) -> PyResult<()> {
        // Validate that the dictionary can be parsed
        CifDocument::parse(dictionary_content).map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Failed to parse dictionary: {}", e))
        })?;
        self.dictionaries.push(dictionary_content.to_string());
        Ok(())
    }

    /// Add a dictionary from a file path
    fn add_dictionary_file(&mut self, path: &str) -> PyResult<()> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            pyo3::exceptions::PyIOError::new_err(format!("Failed to read dictionary file: {}", e))
        })?;
        self.add_dictionary(&content)
    }

    /// Set the validation mode
    fn set_mode(&mut self, mode: PyValidationMode) {
        self.mode = mode.into();
    }

    /// Get the current validation mode
    #[getter]
    fn mode(&self) -> PyValidationMode {
        match self.mode {
            ValidationMode::Strict => PyValidationMode::Strict,
            ValidationMode::Lenient => PyValidationMode::Lenient,
            ValidationMode::Pedantic => PyValidationMode::Pedantic,
        }
    }

    /// Validate a CIF document string
    fn validate(&self, cif_content: &str) -> PyResult<PyValidationResult> {
        // Parse the CIF content
        let doc = CifDocument::parse(cif_content).map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Failed to parse CIF content: {}", e))
        })?;

        // Build the validator
        let mut validator = Validator::new().with_mode(self.mode);

        for dict_content in &self.dictionaries {
            validator = validator.with_dictionary_str(dict_content).map_err(|e| {
                pyo3::exceptions::PyValueError::new_err(format!("Failed to load dictionary: {}", e))
            })?;
        }

        // If no dictionaries loaded, return error
        if self.dictionaries.is_empty() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "No dictionaries loaded. Call add_dictionary() first.",
            ));
        }

        // Validate
        let result = validator.validate(&doc).map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Validation failed: {}", e))
        })?;

        Ok(result.into())
    }

    /// Validate a CIF file
    fn validate_file(&self, path: &str) -> PyResult<PyValidationResult> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            pyo3::exceptions::PyIOError::new_err(format!("Failed to read CIF file: {}", e))
        })?;
        self.validate(&content)
    }
}

/// Validate a CIF string against a dictionary string (convenience function)
///
/// This is a simple one-shot validation function. For validating multiple
/// documents against the same dictionary, use the Validator class instead.
///
/// # Arguments
///
/// * `cif_content` - CIF file content as a string
/// * `dictionary_content` - DDLm dictionary content as a string
///
/// # Returns
///
/// A ValidationResult object with errors and warnings.
#[pyfunction]
fn validate(cif_content: &str, dictionary_content: &str) -> PyResult<PyValidationResult> {
    // Parse the CIF content
    let doc = CifDocument::parse(cif_content).map_err(|e| {
        pyo3::exceptions::PyValueError::new_err(format!("Failed to parse CIF content: {}", e))
    })?;

    // Create validator with dictionary
    let validator = Validator::new()
        .with_dictionary_str(dictionary_content)
        .map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Failed to load dictionary: {}", e))
        })?;

    // Validate
    let result = validator.validate(&doc).map_err(|e| {
        pyo3::exceptions::PyValueError::new_err(format!("Validation failed: {}", e))
    })?;

    Ok(result.into())
}

/// Python module for CIF validation.
#[pymodule]
fn _cif_validator(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Functions
    m.add_function(wrap_pyfunction!(validate, m)?)?;

    // Classes
    m.add_class::<PyValidator>()?;
    m.add_class::<PyValidationResult>()?;
    m.add_class::<PyValidationError>()?;
    m.add_class::<PyValidationWarning>()?;
    m.add_class::<PySpan>()?;

    // Enums
    m.add_class::<PyErrorCategory>()?;
    m.add_class::<PyWarningCategory>()?;
    m.add_class::<PyValidationMode>()?;

    // Module metadata
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__author__", "Iain Maitland")?;
    m.add(
        "__doc__",
        "DDLm-based CIF (Crystallographic Information File) validator",
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_category_conversion() {
        assert_eq!(
            PyErrorCategory::from(ErrorCategory::TypeError),
            PyErrorCategory::TypeError
        );
        assert_eq!(
            PyErrorCategory::from(ErrorCategory::RangeError),
            PyErrorCategory::RangeError
        );
    }

    #[test]
    fn test_validation_mode_conversion() {
        assert_eq!(
            ValidationMode::from(PyValidationMode::Strict),
            ValidationMode::Strict
        );
        assert_eq!(
            ValidationMode::from(PyValidationMode::Lenient),
            ValidationMode::Lenient
        );
    }
}
