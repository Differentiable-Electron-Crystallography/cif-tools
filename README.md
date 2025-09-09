# CIF Parser

A general-purpose library for parsing CIF (Crystallographic Information File) files.

This repo contains the source code for each of the core rust library's bindings.
- Rust (cargo)
- Python (pypi)
- Javascript & Typescript (npm)

## Features

- ✅ Full CIF 1.1 syntax support
- ✅ mmCIF/PDBx compatible
- ✅ Type-safe value access with automatic numeric parsing
- ✅ Comprehensive input validation and error handling

- 🐍 **Python bindings** - Native Python package with full type hints
- 🌐 **WebAssembly support** - Use in browsers and Node.js with full TypeScript definitions
- 🚀 **High performance** - Near-native speed with optimized binaries
- 📦 **Multiple targets** - Rust crate, Python package, Web, Node.js, and bundler builds

## Quick Start

### Rust

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

### Python

```python
import cif_parser

# Parse a CIF string
cif_content = """
data_example
_cell_length_a 10.000
_cell_length_b 10.000

loop_
_atom_site_label
_atom_site_x
_atom_site_y
_atom_site_z
C1  0.123  0.456  0.789
N1  0.234  0.567  0.890
"""

doc = cif_parser.parse(cif_content)
block = doc.first_block()

# Access single values
cell_a = block.get_item('_cell_length_a')
if cell_a and cell_a.is_numeric:
    print(f"Cell a: {cell_a.numeric}")

# Access loop data
atom_loop = block.find_loop('_atom_site_label')
if atom_loop:
    for i in range(len(atom_loop)):
        label = atom_loop.get_by_tag(i, '_atom_site_label')
        x = atom_loop.get_by_tag(i, '_atom_site_x')
        print(f"Atom {i}: {label.text} at x={x.numeric}")

# Pythonic iteration and dict access
for block in doc:
    print(f"Block: {block.name}")
    for key in block.item_keys:
        value = block.get_item(key)
        print(f"  {key}: {value}")
```

## Parsing Files

### Rust

```rust
use cif_parser;

// Parse from file
let doc = cif_parser::parse_file("structure.cif")?;

// Or using the Document type
let doc = cif_parser::Document::from_file("structure.cif")?;
```

### Python

```python
import cif_parser

# Parse from file
doc = cif_parser.parse_file('structure.cif')

# Or using the Document class
doc = cif_parser.Document.from_file('structure.cif')
```

## Working with Values

The library automatically identifies value types:

### Rust

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

### Python

```python
# Type checking and access
if value.is_text:
    print(f"String: {value.text}")
elif value.is_numeric:
    print(f"Number: {value.numeric}")
elif value.is_unknown:
    print("Unknown value '?'")
elif value.is_not_applicable:
    print("Not applicable '.'")

# Convert to native Python types
native_value = value.to_python()  # str, float, or None

# String representation
print(value)  # Shows formatted value
print(repr(value))  # Shows Value(...) representation
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

## Python API Reference

The Python bindings provide a Pythonic interface to the CIF parser with full type hints and modern Python conventions.

### Classes

#### `Document`
Root container for CIF data with support for multiple blocks.

```python
# Static methods
doc = cif_parser.Document.parse(content: str) -> Document
doc = cif_parser.Document.from_file(path: str) -> Document

# Properties and methods
len(doc)                    # Number of blocks
doc.blocks                  # List of all blocks
doc.block_names            # List of block names
doc.first_block()          # First block (or None)
doc.get_block(index: int)   # Block by index
doc.get_block_by_name(name: str)  # Block by name

# Python protocols
doc[0]                     # Access by index
doc['block_name']          # Access by name
for block in doc: ...      # Iterator support
```

#### `Block`
Data block containing items, loops, and save frames.

```python
# Properties
block.name                 # Block name
block.item_keys           # List of item keys
block.num_loops           # Number of loops
block.num_frames          # Number of save frames
block.loops               # List of all loops
block.frames              # List of all frames

# Methods
block.get_item(key: str)           # Get item by key
block.items()                      # Dict of all items
block.get_loop(index: int)         # Get loop by index
block.find_loop(tag: str)          # Find loop containing tag
block.get_loop_tags()              # All loop tags
block.get_frame(index: int)        # Get frame by index
```

#### `Loop`
Tabular data structure with rows and columns.

```python
# Properties
loop.tags                  # Column headers
loop.num_columns          # Number of columns
len(loop)                 # Number of rows

# Methods
loop.get(row: int, col: int)              # Value by position
loop.get_by_tag(row: int, tag: str)       # Value by tag
loop.get_column(tag: str)                 # Entire column
loop.rows()                               # All rows as lists
loop.get_row_dict(row: int)               # Row as dict

# Python protocols
for row in loop: ...       # Iterator support (planned)
```

#### `Value`
Individual CIF value with type information.

```python
# Type checking
value.is_text              # True if text value
value.is_numeric           # True if numeric value
value.is_unknown           # True if unknown (?)
value.is_not_applicable    # True if not applicable (.)

# Value access
value.text                 # Text content (or None)
value.numeric              # Numeric content (or None)  
value.value_type           # Type as string
value.to_python()          # Convert to native Python type
```

### Error Handling

The Python bindings convert Rust errors to appropriate Python exceptions:

```python
try:
    doc = cif_parser.parse(invalid_content)
except ValueError as e:
    print(f"Parse error: {e}")

try:
    doc = cif_parser.parse_file('nonexistent.cif')
except IOError as e:
    print(f"File error: {e}")
```

### Type Hints and IDE Support

The package includes complete type stubs (`.pyi` files) for full IDE support:

```python
from cif_parser import Document, Block, Loop, Value
from typing import Optional, List, Dict

doc: Document = cif_parser.parse(content)
block: Optional[Block] = doc.first_block()
if block:
    loops: List[Loop] = block.loops
    items: Dict[str, Value] = block.items()
```

### Performance Tips

- Use `get_column()` to extract entire data columns efficiently
- Access items by key rather than iterating when possible
- The underlying Rust implementation provides near-native performance
- Memory usage is optimized for typical crystallographic data sizes

### Development and Building

```bash
# Install development dependencies
uv tool install maturin

# Build in development mode (faster, includes debug info)
# Note: Use forward compatibility flag for Python 3.13+
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 maturin develop --features python

# Build optimized wheel for distribution
maturin build --features python --release

# Test the installation
source .venv/bin/activate && python python_example.py

# Run tests
python -m pytest tests/

# Type checking
mypy python/cif_parser/

# Linting and formatting
ruff check python/
black python/
```

## WebAssembly (WASM) Usage

This library can be compiled to WebAssembly for use in web browsers and Node.js applications.

### Prerequisites

First, install `wasm-pack`:

```bash
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

### Building for the Web

```bash
# Build for web browsers
wasm-pack build --target web --out-dir pkg

# Build for Node.js
wasm-pack build --target nodejs --out-dir pkg-node

# Build for bundlers (webpack, etc.)
wasm-pack build --target bundler --out-dir pkg-bundler
```

### JavaScript/TypeScript Usage

After building, you can use the CIF parser in your JavaScript/TypeScript projects:

#### In the Browser (ES Modules)

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>CIF Parser Demo</title>
</head>
<body>
    <script type="module">
        import init, { JsCifDocument } from './pkg/cif_parser.js';
        
        async function run() {
            // Initialize the WASM module
            await init();
            
            // Parse CIF content
            const cifContent = `
                data_example
                _cell_length_a 10.000
                _cell_length_b 20.000
                
                loop_
                _atom_site_label
                _atom_site_x
                C1 0.123
                N1 0.456
            `;
            
            try {
                const doc = JsCifDocument.parse(cifContent);
                console.log(`Parsed ${doc.get_block_count()} blocks`);
                
                const block = doc.get_first_block();
                if (block) {
                    console.log(`Block name: ${block.name}`);
                    
                    // Get a data item
                    const cellA = block.get_item('_cell_length_a');
                    if (cellA && cellA.is_numeric()) {
                        console.log(`Cell a: ${cellA.numeric_value}`);
                    }
                    
                    // Access loop data
                    const loop = block.get_loop(0);
                    if (loop) {
                        console.log(`Loop has ${loop.get_row_count()} rows`);
                        for (let i = 0; i < loop.get_row_count(); i++) {
                            const label = loop.get_value_by_tag(i, '_atom_site_label');
                            console.log(`Atom ${i}: ${label.text_value}`);
                        }
                    }
                }
            } catch (error) {
                console.error('Parsing failed:', error);
            }
        }
        
        run();
    </script>
</body>
</html>
```

#### With Node.js

```javascript
const { JsCifDocument } = require('./pkg-node/cif_parser.js');

const cifContent = `
    data_example
    _cell_length_a 10.000
    _author_name 'John Doe'
`;

try {
    const doc = JsCifDocument.parse(cifContent);
    const block = doc.get_first_block();
    
    console.log('Block name:', block.name);
    console.log('Items:', block.get_item_keys());
    
    const author = block.get_item('_author_name');
    console.log('Author:', author.text_value);
    
} catch (error) {
    console.error('Error:', error);
}
```

#### With Webpack/Bundlers

```typescript
import init, { JsCifDocument } from 'cif-parser';

async function parseCif(content: string) {
    await init();
    
    const doc = JsCifDocument.parse(content);
    return doc;
}
```

### WASM API Reference

The WebAssembly API provides JavaScript-friendly wrappers:

#### `JsCifDocument`
- `static parse(content: string): JsCifDocument` - Parse CIF content
- `get_block_count(): number` - Number of data blocks
- `get_block(index: number): JsCifBlock | undefined` - Get block by index
- `get_block_by_name(name: string): JsCifBlock | undefined` - Get block by name
- `get_first_block(): JsCifBlock | undefined` - Get first block
- `get_block_names(): string[]` - Get all block names

#### `JsCifBlock`
- `name: string` - Block name
- `get_item_keys(): string[]` - All data item keys
- `get_item(key: string): JsCifValue | undefined` - Get data item
- `get_loop_count(): number` - Number of loops
- `get_loop(index: number): JsCifLoop | undefined` - Get loop by index
- `find_loop(tag: string): JsCifLoop | undefined` - Find loop containing tag
- `get_frame_count(): number` - Number of save frames
- `get_frame(index: number): JsCifFrame | undefined` - Get save frame

#### `JsCifLoop`
- `get_tags(): string[]` - Column headers
- `get_row_count(): number` - Number of rows
- `get_column_count(): number` - Number of columns
- `get_value(row: number, col: number): JsCifValue | undefined` - Get value by position
- `get_value_by_tag(row: number, tag: string): JsCifValue | undefined` - Get value by tag
- `get_column(tag: string): string | undefined` - Get entire column as JSON

#### `JsCifValue`
- `value_type: string` - "Text", "Numeric", "Unknown", or "NotApplicable"
- `text_value: string | undefined` - Text content (if text value)
- `numeric_value: number | undefined` - Numeric content (if numeric value)
- `is_text(): boolean`, `is_numeric(): boolean`, etc. - Type checks

### Key WASM Features

- ✅ **Full API Coverage** - All Rust functionality available in JavaScript
- ✅ **TypeScript Support** - Complete type definitions included
- ✅ **Multiple Targets** - Web browsers, Node.js, and bundlers
- ✅ **Error Handling** - JavaScript-friendly error messages
- ✅ **Performance** - Near-native parsing speed (~225KB WASM binary)
- ✅ **Memory Efficient** - Optimized for typical crystallographic data
- ✅ **Debug Support** - Built-in console logging
- ✅ **Zero Dependencies** - Self-contained WASM module

### Generated Files

After building with `wasm-pack`, you'll get:

```
pkg/                          # Web browser package
├── cif_parser.js            # JavaScript bindings
├── cif_parser.d.ts          # TypeScript definitions
├── cif_parser_bg.wasm       # Compiled WebAssembly binary
├── cif_parser_bg.wasm.d.ts  # WASM type definitions
└── package.json             # NPM package metadata

pkg-node/                    # Node.js package (CommonJS)
pkg-bundler/                 # Bundler package (webpack, etc.)
```

### Live Examples

This repository includes working examples:

- **`wasm-demo.html`** - Interactive web demo with full parsing visualization
- **`node-example.js`** - Complete Node.js usage example with detailed output
- **Examples in README** - Copy-paste ready code snippets

### Performance Considerations

- **Binary Size**: Optimized WASM binary (~225KB gzipped)
- **Parse Speed**: Near-native performance for typical CIF files
- **Memory Usage**: Efficient memory management optimized for crystallographic data
- **Bulk Access**: Use `get_column()` for extracting entire data columns
- **Debug Mode**: Console logging can be disabled for production builds

### Distribution & Deployment

#### Publishing to NPM

The generated WASM packages can be published to NPM:

```bash
# Publish web version
cd pkg && npm publish

# Publish Node.js version  
cd pkg-node && npm publish --tag nodejs

# Or publish both as scoped packages
cd pkg && npm publish --access public @your-scope/cif-parser-web
cd pkg-node && npm publish --access public @your-scope/cif-parser-node
```

#### CDN Usage

For web applications, you can serve the WASM files from a CDN:

```html
<script type="module">
    // Load from your CDN
    import init, { JsCifDocument } from 'https://cdn.yoursite.com/cif-parser/cif_parser.js';
    await init('https://cdn.yoursite.com/cif-parser/cif_parser_bg.wasm');
    
    const doc = JsCifDocument.parse(cifContent);
</script>
```

#### Integration with Frameworks

**React/Next.js:**
```javascript
import { useEffect, useState } from 'react';

function CifParser() {
    const [parser, setParser] = useState(null);
    
    useEffect(() => {
        import('./pkg/cif_parser.js').then(async (module) => {
            await module.default();
            setParser(module);
        });
    }, []);
    
    const parseCif = (content) => {
        if (parser) {
            return parser.JsCifDocument.parse(content);
        }
    };
    
    // ... rest of component
}
```

**Vue.js:**
```javascript
import { ref, onMounted } from 'vue';

export default {
    setup() {
        const parser = ref(null);
        
        onMounted(async () => {
            const module = await import('./pkg/cif_parser.js');
            await module.default();
            parser.value = module;
        });
        
        return { parser };
    }
}
```

**Angular:**
```typescript
import { Injectable } from '@angular/core';

@Injectable({
    providedIn: 'root'
})
export class CifParserService {
    private parser: any = null;
    
    async initialize() {
        if (!this.parser) {
            const module = await import('./assets/pkg/cif_parser.js');
            await module.default();
            this.parser = module;
        }
        return this.parser;
    }
    
    async parse(content: string) {
        const parser = await this.initialize();
        return parser.JsCifDocument.parse(content);
    }
}
```

### Troubleshooting Python

#### Python 3.13+ Compatibility

If you encounter build errors with Python 3.13+, use the forward compatibility flag:

```bash
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 maturin develop --features python
```

#### Missing Dependencies

**maturin not installed:**
```bash
uv tool install maturin
# or with pip: pip install maturin
```

**PyO3 version issues:**
The current version uses PyO3 0.21, which supports Python 3.8-3.12 officially, and 3.13+ with the forward compatibility flag.

### Troubleshooting WASM

#### Common Build Issues

**Missing `wasm-pack`:**
```bash
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

**Rust target not installed:**
```bash
rustup target add wasm32-unknown-unknown
```

**Build fails with dependency errors:**
```bash
# Clear Cargo cache and rebuild
cargo clean
rm Cargo.lock
wasm-pack build --target web
```

#### Runtime Issues

**WASM module fails to load:**
- Ensure WASM file is served with correct MIME type (`application/wasm`)
- Check browser console for detailed error messages
- Verify the WASM file path is correct

**Module initialization fails:**
- Always call `await init()` before using any WASM functions
- Handle initialization errors with try-catch blocks

**CORS issues when loading WASM:**
```javascript
// Serve WASM files with proper headers
app.use('/pkg', express.static('pkg', {
    setHeaders: (res, path) => {
        if (path.endsWith('.wasm')) {
            res.setHeader('Content-Type', 'application/wasm');
        }
    }
}));
```

#### Performance Optimization

**Reduce bundle size:**
- Use `--target bundler` for webpack/rollup to enable tree shaking
- Enable wasm-opt optimization (automatically enabled in release builds)

**Improve load times:**
- Serve WASM files with gzip compression
- Use HTTP/2 for better multiplexing
- Consider lazy loading for non-critical parsing

**Debug performance:**
```javascript
const start = performance.now();
const doc = JsCifDocument.parse(cifContent);
console.log(`Parsing took ${performance.now() - start}ms`);
```

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