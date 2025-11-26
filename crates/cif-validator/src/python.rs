//! Python bindings for cif-validator using PyO3
//!
//! This module provides Python bindings for CIF validation functionality.
//! It is only compiled when the `python` feature is enabled.

use pyo3::prelude::*;

use crate::{ValidationMode, Validator};
use cif_parser::Document;

/// Validate a CIF document against a DDLm dictionary.
///
/// This is a convenience function that creates a validator and validates
/// the provided CIF content.
///
/// # Arguments
///
/// * `cif_content` - CIF file content as a string
/// * `_dictionary` - DDLm dictionary content (currently unused, reserved for future)
///
/// # Returns
///
/// A list of validation error messages. Empty list if validation passes.
///
/// # Errors
///
/// Returns a `ValueError` if the CIF content cannot be parsed.
#[pyfunction]
fn validate(cif_content: &str, _dictionary: &str) -> PyResult<Vec<String>> {
    // Parse the CIF content
    let doc = Document::parse(cif_content).map_err(|e| {
        pyo3::exceptions::PyValueError::new_err(format!("Failed to parse CIF content: {}", e))
    })?;

    // Create validator and validate
    // TODO: Load dictionary when dictionary parsing is implemented
    let validator = Validator::new().with_mode(ValidationMode::Strict);

    let result = validator.validate(&doc).map_err(|e| {
        pyo3::exceptions::PyValueError::new_err(format!("Validation failed: {}", e))
    })?;

    // Convert errors to strings
    let error_messages: Vec<String> = result.errors.iter().map(|e| e.to_string()).collect();

    Ok(error_messages)
}

/// Python module for CIF validation.
#[pymodule]
fn _cif_validator(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(validate, m)?)?;
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
    fn test_validate_simple_cif() {
        let cif = "data_test\n_cell_length_a 10.0\n";
        let dict = ""; // Empty dictionary for now

        let errors = validate(cif, dict).unwrap();
        // Currently stub implementation returns no errors
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_invalid_cif() {
        let cif = "this is not valid CIF";
        let dict = "";

        let result = validate(cif, dict);
        assert!(result.is_err());
    }
}
