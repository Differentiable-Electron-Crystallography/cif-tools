"""Type stubs for the native CIF validator module (_cif_validator).

This module provides the Rust-based validation implementation with
precise span information for errors and warnings.
"""

from enum import IntEnum

__version__: str
__author__: str

class ErrorCategory(IntEnum):
    """Categories of validation errors."""

    UnknownDataName = 0
    """Unknown data name (not in dictionary)"""
    TypeError = 1
    """Type mismatch (e.g., text where Real expected)"""
    RangeError = 2
    """Value outside allowed range"""
    EnumerationError = 3
    """Value not in enumerated set"""
    MissingMandatory = 4
    """Missing mandatory item"""
    LoopStructure = 5
    """Invalid loop structure"""
    LinkError = 6
    """Foreign key reference error"""
    DictionaryError = 7
    """Dictionary loading/parsing error"""

class WarningCategory(IntEnum):
    """Categories of validation warnings."""

    MixedCategories = 0
    """Mixed categories in a loop"""
    DeprecatedItem = 1
    """Deprecated item usage"""
    Style = 2
    """Style recommendation"""
    UnknownItem = 3
    """Unknown item in lenient mode"""

class ValidationMode(IntEnum):
    """Validation strictness modes."""

    Strict = 0
    """Strict validation - all checks enabled, unknown items are errors"""
    Lenient = 1
    """Lenient validation - unknown items are warnings"""
    Pedantic = 2
    """Pedantic validation - extra style checks enabled"""

class Span:
    """Source location information for a token or error.

    Tracks where a value or error appears in the source CIF file.
    All positions are 1-indexed.

    Attributes:
        start_line: Starting line number (1-indexed)
        start_col: Starting column number (1-indexed)
        end_line: Ending line number (1-indexed)
        end_col: Ending column number (1-indexed)
    """

    @property
    def start_line(self) -> int:
        """Starting line number (1-indexed)."""
        ...

    @property
    def start_col(self) -> int:
        """Starting column number (1-indexed)."""
        ...

    @property
    def end_line(self) -> int:
        """Ending line number (1-indexed)."""
        ...

    @property
    def end_col(self) -> int:
        """Ending column number (1-indexed)."""
        ...

    def contains(self, line: int, col: int) -> bool:
        """Check if a position is within this span.

        Args:
            line: Line number to check (1-indexed)
            col: Column number to check (1-indexed)

        Returns:
            True if the position is within this span
        """
        ...

    def __str__(self) -> str:
        """String representation (e.g., '1:5-3:10')."""
        ...

    def __repr__(self) -> str:
        """Debug representation."""
        ...

class ValidationError:
    """A validation error with full context and span information.

    Attributes:
        category: Error category for programmatic handling
        message: Human-readable error message
        span: Source location in the CIF file
        data_name: The data name involved (if applicable)
        expected: Expected value/type (for type/enum errors)
        actual: Actual value found
        suggestions: List of suggestions for fixing the error
    """

    @property
    def category(self) -> ErrorCategory:
        """Error category for programmatic handling."""
        ...

    @property
    def message(self) -> str:
        """Human-readable error message."""
        ...

    @property
    def span(self) -> Span:
        """Primary source location in input CIF."""
        ...

    @property
    def data_name(self) -> str | None:
        """The data name involved (if applicable)."""
        ...

    @property
    def expected(self) -> str | None:
        """Expected value/type (for type/enum errors)."""
        ...

    @property
    def actual(self) -> str | None:
        """Actual value found."""
        ...

    @property
    def suggestions(self) -> list[str]:
        """Suggestions for fixing the error."""
        ...

    def __str__(self) -> str:
        """Formatted error message with location."""
        ...

    def __repr__(self) -> str:
        """Debug representation."""
        ...

class ValidationWarning:
    """A validation warning (non-fatal issue).

    Attributes:
        category: Warning category
        message: Human-readable warning message
        span: Source location in the CIF file
    """

    @property
    def category(self) -> WarningCategory:
        """Warning category."""
        ...

    @property
    def message(self) -> str:
        """Human-readable warning message."""
        ...

    @property
    def span(self) -> Span:
        """Source location in the CIF file."""
        ...

    def __str__(self) -> str:
        """Formatted warning message with location."""
        ...

    def __repr__(self) -> str:
        """Debug representation."""
        ...

class ValidationResult:
    """Result of validating a CIF document.

    Attributes:
        is_valid: True if no errors were found
        errors: List of validation errors
        warnings: List of validation warnings
        error_count: Number of errors
        warning_count: Number of warnings

    The result can be used as a boolean (True if valid):
        >>> if result:
        ...     print("Valid!")
    """

    @property
    def is_valid(self) -> bool:
        """Whether the document is valid (no errors)."""
        ...

    @property
    def errors(self) -> list[ValidationError]:
        """List of validation errors."""
        ...

    @property
    def warnings(self) -> list[ValidationWarning]:
        """List of validation warnings."""
        ...

    @property
    def error_count(self) -> int:
        """Number of validation errors."""
        ...

    @property
    def warning_count(self) -> int:
        """Number of validation warnings."""
        ...

    def __str__(self) -> str:
        """Summary string (e.g., 'Invalid (3 errors, 1 warnings)')."""
        ...

    def __repr__(self) -> str:
        """Debug representation."""
        ...

    def __bool__(self) -> bool:
        """True if valid (no errors)."""
        ...

class Validator:
    """CIF Validator for validating documents against DDLm dictionaries.

    The Validator class allows you to load one or more dictionaries and
    validate multiple CIF documents against them.

    Example:
        >>> validator = Validator()
        >>> validator.add_dictionary(core_dict_content)
        >>> validator.set_mode(ValidationMode.Strict)
        >>>
        >>> result = validator.validate(cif_content)
        >>> if result.is_valid:
        ...     print("Valid!")
        ... else:
        ...     for error in result.errors:
        ...         print(f"Line {error.span.start_line}: {error.message}")
    """

    def __init__(self) -> None:
        """Create a new validator with default settings."""
        ...

    def add_dictionary(self, dictionary_content: str) -> None:
        """Add a dictionary from a string.

        Args:
            dictionary_content: DDLm dictionary content as string

        Raises:
            ValueError: If the dictionary cannot be parsed
        """
        ...

    def add_dictionary_file(self, path: str) -> None:
        """Add a dictionary from a file path.

        Args:
            path: Path to a DDLm dictionary file

        Raises:
            IOError: If the file cannot be read
            ValueError: If the dictionary cannot be parsed
        """
        ...

    def set_mode(self, mode: ValidationMode) -> None:
        """Set the validation mode.

        Args:
            mode: Validation strictness mode
        """
        ...

    @property
    def mode(self) -> ValidationMode:
        """Current validation mode."""
        ...

    def validate(self, cif_content: str) -> ValidationResult:
        """Validate a CIF document string.

        Args:
            cif_content: CIF file content as string

        Returns:
            ValidationResult with errors and warnings

        Raises:
            ValueError: If no dictionaries are loaded or CIF cannot be parsed
        """
        ...

    def validate_file(self, path: str) -> ValidationResult:
        """Validate a CIF file.

        Args:
            path: Path to a CIF file

        Returns:
            ValidationResult with errors and warnings

        Raises:
            IOError: If the file cannot be read
            ValueError: If no dictionaries are loaded or CIF cannot be parsed
        """
        ...

def validate(cif_content: str, dictionary_content: str) -> ValidationResult:
    """Validate a CIF string against a dictionary string.

    This is a convenience function for one-shot validation. For validating
    multiple documents against the same dictionary, use the Validator class
    instead for better performance.

    Args:
        cif_content: CIF file content as string
        dictionary_content: DDLm dictionary content as string

    Returns:
        ValidationResult with errors and warnings

    Raises:
        ValueError: If CIF content or dictionary cannot be parsed

    Example:
        >>> result = validate(cif_content, dictionary_content)
        >>> if result.is_valid:
        ...     print("Valid!")
        ... else:
        ...     for error in result.errors:
        ...         print(f"Line {error.span.start_line}: {error.message}")
    """
    ...
