"""CIF Validator - DDLm-based validation for CIF files.

This package provides Python bindings for a CIF validation library written in Rust.
It validates CIF documents against DDLm dictionaries with precise span information
for errors and warnings, enabling IDE integration and detailed error reporting.

Basic usage:
    >>> from cif_validator import Validator, ValidationMode
    >>>
    >>> # Create a validator and add a dictionary
    >>> validator = Validator()
    >>> validator.add_dictionary(dictionary_content)
    >>> validator.set_mode(ValidationMode.Strict)
    >>>
    >>> # Validate CIF content
    >>> result = validator.validate(cif_content)
    >>> if result.is_valid:
    ...     print("Validation passed!")
    ... else:
    ...     for error in result.errors:
    ...         print(f"Error at line {error.span.start_line}: {error.message}")

Simple one-shot validation:
    >>> from cif_validator import validate
    >>>
    >>> result = validate(cif_content, dictionary_content)
    >>> for error in result.errors:
    ...     print(f"{error.category}: {error.message}")

Classes:
    Validator: Reusable validator for validating multiple CIF documents
    ValidationResult: Result of validation containing errors and warnings
    ValidationError: A validation error with span information
    ValidationWarning: A validation warning with span information
    Span: Source location information (line/column)

Enums:
    ValidationMode: Validation strictness (Strict, Lenient, Pedantic)
    ErrorCategory: Type of validation error
    WarningCategory: Type of validation warning

Functions:
    validate(cif_content, dictionary): One-shot validation function
"""

from ._cif_validator import (
    ErrorCategory,
    # Span type
    Span,
    ValidationError,
    # Enums
    ValidationMode,
    # Result types
    ValidationResult,
    ValidationWarning,
    # Validator class
    Validator,
    WarningCategory,
    __version__,
    # Main validation function
    validate,
)

__all__ = [
    # Version
    "__version__",
    # Main function
    "validate",
    # Validator class
    "Validator",
    # Result types
    "ValidationResult",
    "ValidationError",
    "ValidationWarning",
    # Span
    "Span",
    # Enums
    "ValidationMode",
    "ErrorCategory",
    "WarningCategory",
]

# Package metadata
__author__ = "Iain Maitland"
__email__ = "iain@iainmaitland.com"
__license__ = "MIT OR Apache-2.0"
