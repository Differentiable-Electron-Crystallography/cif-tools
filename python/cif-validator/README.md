# cif-validator

DDLm-based CIF (Crystallographic Information File) validation library written in Rust with Python bindings.

## Installation

```bash
pip install cif-validator
```

## Usage

```python
import cif_validator

# Validate CIF content against a DDLm dictionary
errors = cif_validator.validate(cif_content, dictionary_content)

if errors:
    for error in errors:
        print(f"Validation error: {error}")
else:
    print("Validation passed!")
```

## Development

This package is part of the [cif-tools](https://github.com/Differentiable-Electron-Crystallography/cif-tools) monorepo.

```bash
# Install in development mode
cd python && uv sync --extra dev
just python-develop cif-validator

# Run tests
just python-test cif-validator
```

## License

MIT OR Apache-2.0
