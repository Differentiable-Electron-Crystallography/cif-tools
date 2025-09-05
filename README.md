# CIF Parser

A robust, general-purpose Rust library for parsing CIF (Crystallographic Information File) and mmCIF/PDBx files.

## Features

- ✅ Full CIF 1.1 syntax support
- ✅ mmCIF/PDBx compatible
- ✅ Support for data blocks, save frames, and loops
- ✅ Handles all CIF value types (quoted, unquoted, text fields)
- ✅ Special value recognition (`?` for unknown, `.` for not applicable)
- ✅ Type-safe value access with automatic numeric parsing
- ✅ Comprehensive error handling
- ✅ Zero-copy parsing where possible

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
cif-parser = "0.1.0"
```

## Quick Start

```rust
use cif_parser::{Document, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse a CIF string
    let cif_content = r#"
        data_example
        _cell.length_a   10.000
        _cell.length_b   10.000
        
        loop_
        _atom_site.label
        _atom_site.x
        _atom_site.y
        _atom_site.z
        C1  0.123  0.456  0.789
        N1  0.234  0.567  0.890
    "#;
    
    let doc = Document::parse(cif_content)?;
    let block = doc.first_block().unwrap();
    
    // Access single values
    if let Some(cell_a) = block.get_item("_cell.length_a") {
        println!("Cell a: {:?}", cell_a.as_numeric());
    }
    
    // Access loop data
    if let Some(atom_loop) = block.find_loop("_atom_site.label") {
        for i in 0..atom_loop.len() {
            let label = atom_loop.get_by_tag(i, "_atom_site.label");
            let x = atom_loop.get_by_tag(i, "_atom_site.x");
            println!("Atom {}: {:?}", i, label);
        }
    }
    
    Ok(())
}
```

## Parsing Files

```rust
use cif_parser;

// Parse from file
let doc = cif_parser::parse_file("structure.cif")?;

// Or using the Document type
let doc = cif_parser::Document::from_file("structure.cif")?;
```

## Working with Values

The library automatically identifies value types:

```rust
use cif_parser::Value;

match value {
    Value::Text(s) => println!("String: {}", s),
    Value::Numeric(n) => println!("Number: {}", n),
    Value::Unknown => println!("Unknown value '?'"),
    Value::NotApplicable => println!("Not applicable '.'"),
}

// Convenience methods
if let Some(text) = value.as_string() {
    // Handle as string
}
if let Some(num) = value.as_numeric() {
    // Handle as number
}
```

## Data Structure

The parsed CIF document has the following hierarchy:

```
Document
├── Block (data_blockname)
│   ├── Items (key-value pairs)
│   ├── Loops
│   │   ├── Tags
│   │   └── Values (rows of data)
│   └── Frames (save frames)
│       ├── Items
│       └── Loops
└── Block (another block)
```

## Examples

Check out the `examples/` directory for more usage examples:

- `basic_usage.rs` - Simple CIF parsing
- `mmcif_parser.rs` - Parsing PDBx/mmCIF files
- `advanced_features.rs` - Save frames and multiple blocks
- `file_io.rs` - Reading from files

Run examples with:
```bash
cargo run --example basic_usage
```

## API Documentation

Generate and view the API documentation:

```bash
cargo doc --open
```

## Supported CIF Features

- **Data Blocks**: `data_blockname`
- **Global Blocks**: `global_`
- **Save Frames**: `save_framename ... save_`
- **Loops**: Multi-column tabular data
- **Value Types**:
  - Unquoted strings
  - Single-quoted strings
  - Double-quoted strings
  - Multi-line text fields (`;...;`)
  - Numeric values (auto-detected)
  - Special values (`?` and `.`)
- **Comments**: Lines starting with `#`

## Performance

The parser uses the Pest parsing library with a PEG grammar for efficient parsing. It performs zero-copy parsing where possible and only allocates when necessary.

## Error Handling

The library provides detailed error messages for:
- Syntax errors with line/column information
- Invalid loop structures
- I/O errors when reading files
- Structural validation

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## References

- [CIF 1.1 Specification](https://www.iucr.org/resources/cif/spec/version1.1/cifsyntax)
- [PDBx/mmCIF Dictionary](https://mmcif.wwpdb.org/)
- [Pest Parser](https://pest.rs/)