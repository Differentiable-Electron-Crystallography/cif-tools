"""Type stubs for cif_validator package.

DDLm-based CIF validation library with Python bindings.
Provides precise span information for errors and warnings.

Example usage:
    from cif_validator import Validator, ValidationMode

    validator = Validator()
    validator.add_dictionary(dictionary_content)
    validator.set_mode(ValidationMode.Strict)

    result = validator.validate(cif_content)
    for error in result.errors:
        print(f"Line {error.span.start_line}: {error.message}")
"""

from enum import IntEnum

__version__: str
__author__: str

class ErrorCategory(IntEnum):
    """Categories of validation errors."""

    UnknownDataName = 0
    TypeError = 1
    RangeError = 2
    EnumerationError = 3
    MissingMandatory = 4
    LoopStructure = 5
    LinkError = 6
    DictionaryError = 7

class WarningCategory(IntEnum):
    """Categories of validation warnings."""

    MixedCategories = 0
    DeprecatedItem = 1
    Style = 2
    UnknownItem = 3

class ValidationMode(IntEnum):
    """Validation strictness modes."""

    Strict = 0
    Lenient = 1
    Pedantic = 2

class Span:
    """Source location information (line/column positions, 1-indexed)."""

    @property
    def start_line(self) -> int: ...
    @property
    def start_col(self) -> int: ...
    @property
    def end_line(self) -> int: ...
    @property
    def end_col(self) -> int: ...
    def contains(self, line: int, col: int) -> bool: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...

class ValidationError:
    """A validation error with span information."""

    @property
    def category(self) -> ErrorCategory: ...
    @property
    def message(self) -> str: ...
    @property
    def span(self) -> Span: ...
    @property
    def data_name(self) -> str | None: ...
    @property
    def expected(self) -> str | None: ...
    @property
    def actual(self) -> str | None: ...
    @property
    def suggestions(self) -> list[str]: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...

class ValidationWarning:
    """A validation warning with span information."""

    @property
    def category(self) -> WarningCategory: ...
    @property
    def message(self) -> str: ...
    @property
    def span(self) -> Span: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...

class ValidationResult:
    """Result of validating a CIF document."""

    @property
    def is_valid(self) -> bool: ...
    @property
    def errors(self) -> list[ValidationError]: ...
    @property
    def warnings(self) -> list[ValidationWarning]: ...
    @property
    def error_count(self) -> int: ...
    @property
    def warning_count(self) -> int: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __bool__(self) -> bool: ...

class Validator:
    """CIF Validator for validating documents against DDLm dictionaries."""

    def __init__(self) -> None: ...
    def add_dictionary(self, dictionary_content: str) -> None: ...
    def add_dictionary_file(self, path: str) -> None: ...
    def set_mode(self, mode: ValidationMode) -> None: ...
    @property
    def mode(self) -> ValidationMode: ...
    def validate(self, cif_content: str) -> ValidationResult: ...
    def validate_file(self, path: str) -> ValidationResult: ...

def validate(cif_content: str, dictionary_content: str) -> ValidationResult:
    """Validate a CIF string against a dictionary string.

    Args:
        cif_content: CIF file content as string
        dictionary_content: DDLm dictionary content as string

    Returns:
        ValidationResult with errors and warnings

    Raises:
        ValueError: If CIF content or dictionary cannot be parsed
    """
    ...
