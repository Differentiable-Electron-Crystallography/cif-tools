"""CIF Validator - DDLm-based validation for CIF files.

This package provides Python bindings for a CIF validation library written in Rust.
It validates CIF documents against DDLm dictionaries.

Basic usage:
    >>> import cif_validator
    >>>
    >>> # Validate CIF content against a dictionary
    >>> errors = cif_validator.validate(cif_content, dictionary_content)
    >>> for error in errors:
    ...     print(error)

Functions:
    validate(cif_content, dictionary): Validate CIF content against a DDLm dictionary
"""

from ._cif_validator import (
    __version__,
    validate,
)

__all__ = [
    "validate",
    "__version__",
]

# Package metadata
__author__ = "Iain Maitland"
__email__ = "iain@iainmaitland.com"
__license__ = "MIT OR Apache-2.0"
