# CIF Tools

A Rust workspace providing tools for working with CIF (Crystallographic Information File) format.

## Crates

| Crate | Description | Status |
|-------|-------------|--------|
| [cif-parser](crates/cif-parser/) | General-purpose CIF parser with Python & WASM bindings | Stable |
| [cif-validator](crates/cif-validator/) | DDLm-based CIF validation | In Development |

## Features

- Full CIF 1.1 and CIF 2.0 syntax support
- Type-safe value access with automatic numeric parsing
- Python bindings via PyO3
- WebAssembly support for browsers and Node.js
- DDLm dictionary-based validation (coming soon)

## Quick Start

### Using cif-parser (Rust)

```toml
[dependencies]
cif-parser = "0.1"
```

```rust
use cif_parser::Document;

let doc = Document::parse(r#"
    data_example
    _cell.length_a 10.000
"#)?;

let block = doc.first_block().unwrap();
if let Some(cell_a) = block.get_item("_cell.length_a") {
    println!("Cell a: {:?}", cell_a.as_numeric());
}
```

### Using cif-parser (Python)

```python
import cif_parser

doc = cif_parser.parse_file('structure.cif')
block = doc.first_block()
print(f"Cell a: {block.get_item('_cell_length_a').numeric}")
```

See [crates/cif-parser/README.md](crates/cif-parser/README.md) for detailed usage.

## Building from Source

### Prerequisites

- [Rust](https://rustup.rs/) 1.70.0 or later
- [just](https://github.com/casey/just) - Command runner
- For Python bindings: Python 3.8+ and [uv](https://docs.astral.sh/uv/)
- For WASM bindings: [wasm-pack](https://rustwasm.github.io/wasm-pack/)

### Build Commands

```bash
# Build all Rust crates
cargo build

# Build specific crate
cargo build -p cif-parser
cargo build -p cif-validator

# Run tests
cargo test

# Build Python bindings (cif-parser)
just python-develop

# Build WASM bindings (cif-parser)
just wasm-build-web
```

## Project Structure

```
cif-tools/
├── crates/
│   ├── cif-parser/           # CIF parsing library
│   │   ├── src/              # Rust source
│   │   ├── python/           # Python package
│   │   └── javascript/       # WASM/JS package
│   └── cif-validator/        # CIF validation library
├── docs/                     # Documentation
└── justfile                  # Build commands
```

## Development

### Installing Git Hooks

```bash
just install-hooks
```

### Code Quality

```bash
# Run all checks
just ci

# Format all code
just fmt

# Individual language checks
just check-rust
just check-python
just check-js
```

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## References

- [CIF 1.1 Specification](https://www.iucr.org/resources/cif/spec/version1.1/cifsyntax)
- [CIF 2.0 Specification](https://www.iucr.org/resources/cif/spec/CIF2)
- [PDBx/mmCIF Dictionary](https://mmcif.wwpdb.org/)
