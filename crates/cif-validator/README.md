# cif-validator

DDLm-based validation for CIF (Crystallographic Information File) format.

## Status

🚧 **Under Development** - This crate is in early development stages.

## Overview

`cif-validator` provides comprehensive validation of CIF files against DDLm (Dictionary Definition Language methods) dictionaries. It is designed as a separate crate from `cif-parser` to maintain separation of concerns and allow users to parse CIF files without the overhead of validation when not needed.

## Planned Features

- **Dictionary Loading**: Parse and load DDLm dictionary files (using `cif-parser`)
- **Multi-Dictionary Composition**: Combine multiple dictionaries (core + powder + restraints, etc.)
- **Type System Validation**: Enforce DDLm types (Integer, Real, DateTime, Uri, etc.)
- **Constraint Checking**: Validate enumerations, ranges, and mandatory items
- **dREL Evaluation**: Execute embedded dictionary methods (future)

## Planned Usage

```rust
use cif_parser::Document;
use cif_validator::{Validator, ValidationMode};

// Parse CIF file
let doc = Document::parse(cif_content)?;

// Create validator with core dictionary
let validator = Validator::new()
    .with_core()?                      // Load cif_core.dic
    .with_dictionary("cif_pow.dic")?   // Add powder dictionary
    .with_mode(ValidationMode::Strict);

// Validate
let result = validator.validate(&doc)?;

if result.is_valid {
    println!("CIF is valid!");
} else {
    for error in result.errors {
        println!("Error at {}: {}", error.location, error.message);
    }
}
```

## Architecture

This validator follows the architecture patterns established by reference CIF implementations:

- **PyCIFRW**: Separates `ReadCif()` (parsing) from `ValidCifFile()` (validation)
- **COMCIFS/cif_api**: States validation is "above the level of the CIF API"

Benefits of separation:
- **Performance**: Skip validation for performance-critical use cases
- **Binary size**: Keep parser lightweight for WASM/Python
- **Flexibility**: Users choose when validation is needed
- **Clarity**: Explicit validation step in API

## Development

This crate is part of the `cif-tools` workspace. To build:

```bash
# Build validator
cargo build -p cif-validator

# Run tests
cargo test -p cif-validator

# Build with Python bindings (future)
cargo build -p cif-validator --features python
```

## Documentation

See the workspace-level [docs/](../../docs/) directory for:
- [dictionaries.md](../../docs/reference/dictionaries.md) - Comprehensive guide to CIF dictionaries and DDL
- Architecture documentation
- Implementation plans

## License

MIT OR Apache-2.0
