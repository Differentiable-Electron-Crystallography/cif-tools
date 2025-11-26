# cif-validator

DDLm-based CIF (Crystallographic Information File) validation library written in Rust with Python bindings.

## Features

- **Fast validation** - Rust-powered validation with Python bindings
- **Precise error locations** - Every error includes exact line/column span information
- **Multiple validation modes** - Strict, Lenient, and Pedantic modes
- **Multi-dictionary support** - Combine core + powder + restraints dictionaries
- **Type checking** - Validates DDLm types (Integer, Real, DateTime, etc.)
- **Constraint checking** - Validates enumerations, ranges, and mandatory items

## Installation

```bash
pip install cif-validator
```

## Quick Start

### Simple One-Shot Validation

```python
from cif_validator import validate

result = validate(cif_content, dictionary_content)

if result.is_valid:
    print("Validation passed!")
else:
    for error in result.errors:
        print(f"Line {error.span.start_line}: {error.message}")
```

### Using the Validator Class

For validating multiple files against the same dictionary:

```python
from cif_validator import Validator, ValidationMode

# Create validator and load dictionary
validator = Validator()
validator.add_dictionary(core_dict_content)
validator.add_dictionary(powder_dict_content)  # Optional: add more dictionaries
validator.set_mode(ValidationMode.Strict)

# Validate multiple files
for cif_file in cif_files:
    result = validator.validate_file(cif_file)
    if not result.is_valid:
        print(f"{cif_file}: {result.error_count} errors")
```

## API Reference

### Functions

#### `validate(cif_content: str, dictionary_content: str) -> ValidationResult`

One-shot validation of CIF content against a dictionary.

### Classes

#### `Validator`

Reusable validator for validating multiple CIF documents.

```python
validator = Validator()
validator.add_dictionary(dict_content)      # Add dictionary from string
validator.add_dictionary_file("core.dic")   # Add dictionary from file
validator.set_mode(ValidationMode.Strict)   # Set validation mode
result = validator.validate(cif_content)    # Validate string
result = validator.validate_file("data.cif") # Validate file
```

#### `ValidationResult`

Result of validation containing errors and warnings.

```python
result.is_valid      # bool: True if no errors
result.errors        # list[ValidationError]: All errors
result.warnings      # list[ValidationWarning]: All warnings
result.error_count   # int: Number of errors
result.warning_count # int: Number of warnings

# Can be used as boolean
if result:
    print("Valid!")
```

#### `ValidationError`

A validation error with precise location information.

```python
error.category     # ErrorCategory: Type of error
error.message      # str: Human-readable message
error.span         # Span: Location in source file
error.data_name    # str | None: The data name involved
error.expected     # str | None: Expected value/type
error.actual       # str | None: Actual value found
error.suggestions  # list[str]: Fix suggestions
```

#### `ValidationWarning`

A validation warning (non-fatal).

```python
warning.category  # WarningCategory: Type of warning
warning.message   # str: Human-readable message
warning.span      # Span: Location in source file
```

#### `Span`

Source location information (1-indexed).

```python
span.start_line  # int: Starting line (1-indexed)
span.start_col   # int: Starting column (1-indexed)
span.end_line    # int: Ending line (1-indexed)
span.end_col     # int: Ending column (1-indexed)
span.contains(line, col)  # Check if position is within span
```

### Enums

#### `ValidationMode`

- `ValidationMode.Strict` - All checks enabled, unknown items are errors
- `ValidationMode.Lenient` - Unknown items are warnings instead of errors
- `ValidationMode.Pedantic` - Extra style checks enabled

#### `ErrorCategory`

- `UnknownDataName` - Data name not in dictionary
- `TypeError` - Type mismatch (e.g., text where Real expected)
- `RangeError` - Value outside allowed range
- `EnumerationError` - Value not in allowed set
- `MissingMandatory` - Required item missing
- `LoopStructure` - Invalid loop structure
- `LinkError` - Foreign key reference error
- `DictionaryError` - Dictionary loading error

#### `WarningCategory`

- `MixedCategories` - Loop contains items from multiple categories
- `DeprecatedItem` - Using a deprecated item
- `Style` - Style recommendation
- `UnknownItem` - Unknown item (in lenient mode)

## Example: IDE Integration

The precise span information enables IDE features like error highlighting:

```python
from cif_validator import Validator, ValidationMode

validator = Validator()
validator.add_dictionary_file("cif_core.dic")

result = validator.validate(document_text)

# Convert to LSP diagnostics
diagnostics = []
for error in result.errors:
    diagnostics.append({
        "range": {
            "start": {"line": error.span.start_line - 1, "character": error.span.start_col - 1},
            "end": {"line": error.span.end_line - 1, "character": error.span.end_col - 1}
        },
        "severity": 1,  # Error
        "message": error.message,
        "source": "cif-validator"
    })
```

## Development

This package is part of the [cif-tools](https://github.com/Differentiable-Electron-Crystallography/cif-tools) monorepo.

```bash
# Install in development mode
cd python && uv sync --extra dev
just python-develop cif-validator

# Run tests
just python-test cif-validator

# Type checking
just python-typecheck cif-validator
```

## License

MIT OR Apache-2.0
