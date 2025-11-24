# cif-parser

A general-purpose CIF (Crystallographic Information File) parser library written in Rust.

## Features

- ✅ Full CIF 1.1 and CIF 2.0 syntax support
- ✅ mmCIF/PDBx compatible
- ✅ Type-safe value access with automatic numeric parsing
- ✅ Comprehensive input validation and error handling
- ✅ Zero-copy parsing where possible
- ✅ Python bindings via PyO3 (optional)
- ✅ WebAssembly support via wasm-bindgen (optional)

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
cif-parser = "0.1"
```

## Basic Usage

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
            println!("Atom {}: {:?}", i, label);
        }
    }

    Ok(())
}
```

## Parsing Files

```rust
use cif_parser::Document;

// Parse from file
let doc = Document::from_file("structure.cif")?;
```

## Working with Values

```rust
use cif_parser::Value;

match value {
    Value::Text(s) => println!("String: {}", s),
    Value::Numeric(n) => println!("Number: {}", n),
    Value::Unknown => println!("Unknown value '?'"),
    Value::NotApplicable => println!("Not applicable '.'"),
}

// Convenience methods
if let Some(num) = value.as_numeric() {
    println!("Numeric value: {}", num);
}
```

## Data Structure

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

See the `examples/` directory:

- `basic_usage.rs` - Simple CIF parsing
- `mmcif_parser.rs` - Parsing PDBx/mmCIF files
- `advanced_features.rs` - Save frames and multiple blocks
- `file_io.rs` - Reading from files

Run examples with:
```bash
cargo run --example basic_usage
```

## Python Bindings

```bash
# Build and install
just python-develop
```

```python
import cif_parser

doc = cif_parser.parse_file('structure.cif')
block = doc.first_block()

# Access items
cell_a = block.get_item('_cell_length_a')
if cell_a and cell_a.is_numeric:
    print(f"Cell a: {cell_a.numeric}")

# Access loops
atom_loop = block.find_loop('_atom_site_label')
for i in range(len(atom_loop)):
    label = atom_loop.get_by_tag(i, '_atom_site_label')
    print(f"Atom: {label.text}")
```

Key classes: `Document`, `Block`, `Loop`, `Value`

## WebAssembly

```bash
# Build for web
just wasm-build-web

# Build for Node.js
just wasm-build
```

```javascript
import init, { parse } from './pkg/cif_parser.js';

await init();
const doc = parse(cifContent);
const block = doc.get_first_block();
console.log(block.name);
```

See `javascript/README.md` for full API reference.

## Performance

The parser uses the Pest parsing library with a PEG grammar for efficient parsing. It performs zero-copy parsing where possible and only allocates when necessary.

## Error Handling

The library provides detailed error messages for:
- Syntax errors with line/column information
- Invalid loop structures
- I/O errors when reading files
- Structural validation

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

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
