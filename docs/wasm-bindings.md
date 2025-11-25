# WebAssembly Bindings: Complete Reference

This document provides a comprehensive analysis of how the CIF parser Rust library is exposed to JavaScript/TypeScript via WebAssembly, including implementation details, design decisions, and API alignment with the Python bindings.

## Table of Contents

- [Overview](#overview)
- [Current Implementation](#current-implementation)
- [Wrapper Classes](#wrapper-classes)
- [Build Configuration](#build-configuration)
- [JavaScript API Design](#javascript-api-design)
- [Type Conversions](#type-conversions)
- [Error Handling](#error-handling)
- [API Alignment with Python](#api-alignment-with-python)
- [Memory Management](#memory-management)
- [Build Targets](#build-targets)
- [Performance Considerations](#performance-considerations)
- [Publishing to NPM](#publishing-to-npm)
- [Comparison with Python Bindings](#comparison-with-python-bindings)

## Overview

### Architecture

The CIF parser uses **wasm-bindgen** (version 0.2) with **wasm-pack** as the build system to create WebAssembly bindings. This enables the Rust-based parser to run in web browsers and Node.js with near-native performance.

```
┌─────────────────────────────────────┐
│   JavaScript/TypeScript Application │
├─────────────────────────────────────┤
│      JavaScript Bindings Layer      │
│      (pkg/cif_parser.js + .d.ts)   │
├─────────────────────────────────────┤
│    wasm-bindgen Generated Glue     │
│        (pkg/cif_parser_bg.js)      │
├─────────────────────────────────────┤
│         WASM Module (.wasm)         │
│    Compiled from src/wasm.rs       │
├─────────────────────────────────────┤
│      Core Rust Library              │
│    (src/lib.rs, src/ast/*, ...)    │
└─────────────────────────────────────┘
```

### Key Design Pattern: Complete Wrapper with Dual API

Each Rust AST type is wrapped in a corresponding JavaScript class that provides both modern property-based access and backward-compatible methods:

| Rust Type | WASM Wrapper | Modern API | Legacy Methods |
|-----------|--------------|------------|----------------|
| `CifValue` | `JsCifValue` | Properties + type checks | ✓ |
| `CifLoop` | `JsCifLoop` | `tags`, `numRows`, `numColumns` | `get_tags()`, etc. |
| `CifFrame` | `JsCifFrame` | `name`, `itemKeys`, `numLoops` | `get_item_keys()`, etc. |
| `CifBlock` | `JsCifBlock` | `name`, `itemKeys`, `numLoops` | `get_item_keys()`, etc. |
| `CifDocument` | `JsCifDocument` | `blockCount`, `blockNames` | `get_block_count()`, etc. |

### Key Files

- **`src/wasm.rs`** (458 lines) - All WASM bindings implementation
- **`javascript/package.json`** - NPM packaging configuration
- **`javascript/README.md`** - JavaScript/TypeScript documentation
- **`Cargo.toml`** - Rust dependencies with wasm-bindgen

## Current Implementation

### Technology Stack

- **wasm-bindgen**: 0.2 (latest stable)
- **wasm-pack**: Build and packaging tool
- **web-sys**: Browser API bindings
- **js-sys**: JavaScript standard library bindings
- **serde-wasm-bindgen**: Serialization support

### Compilation Flow

```
User runs: just wasm-build-web
           ↓
just → wasm-pack build --target web --out-dir javascript/pkg
           ↓
wasm-pack → cargo build --target wasm32-unknown-unknown
           ↓
Rust code in src/wasm.rs compiled with wasm-bindgen
           ↓
Generates .wasm binary + JavaScript glue code
           ↓
wasm-opt optimizes binary size
           ↓
TypeScript definitions generated
           ↓
NPM-ready package in javascript/pkg/
           ↓
User: import { parse } from './pkg/cif_parser.js'
```

## Wrapper Classes

### 1. JsCifValue (Lines 22-100)

Wraps the `CifValue` enum for type-safe value access.

**Structure:**
```rust
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsCifValue {
    value_type: String,
    text_value: Option<String>,
    numeric_value: Option<f64>,
}
```

**JavaScript API:**
```javascript
const value = block.get_item("_cell_length_a");

// Type checking
if (value.is_numeric()) {
    console.log(value.numeric_value);  // Option<f64> → number | undefined
}

// Properties
console.log(value.value_type);  // "Text", "Numeric", "Unknown", "NotApplicable"
```

**Features:**
- Type-checking methods: `is_text()`, `is_numeric()`, `is_unknown()`, `is_not_applicable()`
- Getters: `value_type`, `text_value`, `numeric_value`
- Serializable via serde for JSON conversion

**Implementation Pattern:**
```rust
#[wasm_bindgen]
impl JsCifValue {
    #[wasm_bindgen(getter)]
    pub fn value_type(&self) -> String {
        self.value_type.clone()
    }

    #[wasm_bindgen]
    pub fn is_numeric(&self) -> bool {
        self.value_type == "Numeric"
    }
}
```

### 2. JsCifLoop (Lines 103-195)

Wraps `CifLoop` for tabular data access.

**Structure:**
```rust
#[wasm_bindgen]
pub struct JsCifLoop {
    inner: CifLoop,
}
```

**JavaScript API:**
```javascript
const loop = block.get_loop(0);

// Modern property-based API
console.log(loop.tags);           // string[] - column headers
console.log(loop.numRows);        // number - row count
console.log(loop.numColumns);     // number - column count

// Access by position
const value = loop.get_value(row, col);

// Access by tag name
const atomType = loop.get_value_by_tag(row, "_atom_site_type_symbol");

// Get entire column as array
const xCoords = loop.get_column("_atom_site_fract_x");  // JsCifValue[]

// Get row as JavaScript object
const row = loop.get_row_dict(0);  // { "_col1": JsCifValue, ... }

// Check if empty
if (loop.is_empty()) { /* ... */ }
```

**Features:**
- Properties: `tags`, `numRows`, `numColumns` (modern API)
- Legacy methods: `get_tags()`, `get_row_count()`, `get_column_count()`
- Methods: `get_value()`, `get_value_by_tag()`, `get_column()`, `get_row_dict()`, `is_empty()`
- Returns wrapped `JsCifValue` objects

**Modern vs Legacy API:**
```javascript
// Modern (preferred)
const tags = loop.tags;
const rows = loop.numRows;

// Legacy (backward compatibility)
const tags = loop.get_tags();
const rows = loop.get_row_count();
```

**DuckDB/Pandas Integration:**
```javascript
// Convert loop to array of objects for data processing
const rows = [];
for (let i = 0; i < loop.numRows; i++) {
    rows.push(loop.get_row_dict(i));
}

// Use with DuckDB or similar
import * as duckdb from '@duckdb/duckdb-wasm';
const result = await db.query("SELECT * FROM rows WHERE ...");
```

### 3. JsCifFrame (Lines 203-250)

Wraps `CifFrame` for save frame structures.

**Structure:**
```rust
#[wasm_bindgen]
pub struct JsCifFrame {
    inner: CifFrame,
}
```

**JavaScript API:**
```javascript
const frame = block.get_frame(0);

// Properties (modern API)
console.log(frame.name);          // string
console.log(frame.itemKeys);      // string[]
console.log(frame.numLoops);      // number

// Access items
const value = frame.get_item("_restraint_type");

// Access loops
const loop = frame.get_loop(0);

// Legacy methods
const keys = frame.get_item_keys();
const count = frame.get_loop_count();
```

**Features:**
- Properties: `name`, `itemKeys`, `numLoops`
- Methods: `get_item()`, `get_loop()`
- Legacy aliases for backward compatibility

### 4. JsCifBlock (Lines 253-327)

Wraps `CifBlock` for data blocks.

**Structure:**
```rust
#[wasm_bindgen]
pub struct JsCifBlock {
    inner: CifBlock,
}
```

**JavaScript API:**
```javascript
const block = doc.get_block(0);

// Properties (modern API)
console.log(block.name);          // string
console.log(block.itemKeys);      // string[]
console.log(block.numLoops);      // number
console.log(block.numFrames);     // number

// Access items
const cellA = block.get_item("_cell_length_a");

// Access loops
const loop = block.get_loop(0);
const loop = block.find_loop("_atom_site_label");  // Find by tag
const allTags = block.get_loop_tags();             // All loop tags

// Access frames
const frame = block.get_frame(0);
```

**Features:**
- Full access to items, loops, and frames
- Properties for modern API
- Search methods: `find_loop()`, `get_loop_tags()`
- Legacy method aliases

### 5. JsCifDocument (Lines 330-420)

Wraps `CifDocument` (root container).

**Structure:**
```rust
#[wasm_bindgen]
pub struct JsCifDocument {
    inner: CifDocument,
}
```

**JavaScript API:**
```javascript
// Parsing (static method)
const doc = JsCifDocument.parse(cifString);

// Module-level convenience (preferred)
import { parse } from './pkg/cif_parser.js';
const doc = parse(cifString);

// Properties (modern API)
console.log(doc.blockCount);      // number
console.log(doc.blockNames);      // string[]

// Access blocks
const block = doc.get_block(0);                    // By index
const block = doc.get_block_by_name("protein");    // By name
const block = doc.first_block();                   // First block

// Legacy methods
const count = doc.get_block_count();
const names = doc.get_block_names();
const first = doc.get_first_block();
```

**Features:**
- Static method: `parse()`
- Properties: `blockCount`, `blockNames`
- Multiple access patterns for flexibility
- Modern property API with legacy method fallbacks

**Error Handling with Location Info:**
```rust
pub fn parse(input: &str) -> Result<JsCifDocument, String> {
    match CifDocument::parse(input) {
        Ok(doc) => Ok(JsCifDocument { inner: doc }),
        Err(e) => {
            let error_msg = match e {
                CifError::ParseError(msg) => format!("Parse error: {}", msg),
                CifError::InvalidStructure { message, location } => {
                    if let Some((line, col)) = location {
                        format!("Invalid structure at line {}, col {}: {}",
                                line, col, message)
                    } else {
                        format!("Invalid structure: {}", message)
                    }
                }
                CifError::IoError(err) => format!("IO error: {}", err),
            };
            Err(error_msg)
        }
    }
}
```

## Build Configuration

### Cargo.toml

```toml
[dependencies]
wasm-bindgen = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde-wasm-bindgen = "0.6"
serde_json = "1.0"
js-sys = "0.3"

[dependencies.web-sys]
version = "0.3"
features = ["console"]

[lib]
crate-type = ["cdylib", "rlib"]
```

**Key Configuration:**
- **wasm-bindgen**: Core WASM bindings
- **serde-wasm-bindgen**: JavaScript object conversion
- **web-sys/js-sys**: Browser and JavaScript APIs
- **Crate types**:
  - `cdylib` - Dynamic library for WASM
  - `rlib` - Static library for Rust-only usage

### NPM Package (javascript/package.json)

**Note:** Use `just` commands for building. The NPM scripts are shown for reference but `just` is the recommended build system.

**Recommended commands:**
```bash
just wasm-build-web       # Build for web
just wasm-build           # Build for Node.js
just wasm-build-bundler   # Build for bundlers
just wasm-build-all       # Build all targets
```

**NPM scripts (for reference):**
```json
{
  "name": "@cif-parser/core",
  "version": "0.1.0",
  "scripts": {
    "build": "wasm-pack build --target web --out-dir javascript/pkg",
    "build:node": "wasm-pack build --target nodejs --out-dir javascript/pkg-node",
    "build:bundler": "wasm-pack build --target bundler --out-dir javascript/pkg-bundler",
    "build:all": "npm run build && npm run build:node && npm run build:bundler"
  }
}
```

## JavaScript API Design

### Module-Level Functions

**Available in global scope:**
```javascript
import { parse, version, author } from './pkg/cif_parser.js';

const doc = parse(cifContent);        // Convenience function
console.log(version());                // "0.1.0"
console.log(author());                 // "Iain Maitland"
```

**Implementation:**
```rust
#[wasm_bindgen]
pub fn parse(content: &str) -> Result<JsCifDocument, String> {
    JsCifDocument::parse(content)
}

#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
```

### Modern API Design Principles

**1. Property Getters (Preferred)**
```javascript
// Modern - uses properties
const count = loop.numRows;
const tags = loop.tags;

// Legacy - uses methods
const count = loop.get_row_count();
const tags = loop.get_tags();
```

**2. JavaScript Object Returns**
```javascript
// get_row_dict returns a real JavaScript object
const row = loop.get_row_dict(0);
console.log(row._atom_site_label);    // Direct property access
console.log(Object.keys(row));        // All tags
```

**Implementation:**
```rust
#[wasm_bindgen]
pub fn get_row_dict(&self, row: usize) -> Result<JsValue, JsValue> {
    use js_sys::Object;

    if row >= self.inner.len() {
        return Err(JsValue::from_str("Row index out of bounds"));
    }

    let obj = Object::new();
    for (col, tag) in self.inner.tags.iter().enumerate() {
        if let Some(value) = self.inner.get(row, col) {
            let js_value: JsCifValue = value.into();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str(tag),
                &serde_wasm_bindgen::to_value(&js_value).unwrap_or(JsValue::NULL),
            )?;
        }
    }
    Ok(obj.into())
}
```

**3. Array Returns Instead of JSON**
```javascript
// Modern - returns array of JsCifValue
const column = loop.get_column("_atom_site_x");  // JsCifValue[]
column.forEach(val => {
    if (val.is_numeric()) {
        console.log(val.numeric_value);
    }
});

// Old approach would have returned JSON string requiring parse
```

## Type Conversions

### Rust to JavaScript Automatic Conversions

wasm-bindgen provides automatic type conversions:

```rust
// Basic types
String → string
f64 → number
bool → boolean
usize → number

// Collections
Vec<T> → Array (if T is compatible)
Option<T> → T | undefined

// Custom types
struct → class instance
Result<T, E> → T or throws error
```

### JavaScript to Rust

```rust
// Function parameters
&str ← string
f64 ← number
bool ← boolean
usize ← number
```

### Complex Type Mapping

| Rust Type | JavaScript Type | Notes |
|-----------|----------------|-------|
| `Option<String>` | `string \| undefined` | None becomes undefined |
| `Vec<JsCifValue>` | `JsCifValue[]` | Array of wrapper objects |
| `Result<T, String>` | `T` or throws | Errors become exceptions |
| Custom struct | Object/Class | Via serde or wasm-bindgen |

### Serialization with Serde

```rust
#[derive(Serialize, Deserialize)]
pub struct JsCifValue {
    value_type: String,
    text_value: Option<String>,
    numeric_value: Option<f64>,
}

// Can be converted to/from JsValue
let js_val = serde_wasm_bindgen::to_value(&cif_value)?;
let cif_val: JsCifValue = serde_wasm_bindgen::from_value(js_val)?;
```

## Error Handling

### Error Type Mapping

Rust errors are converted to JavaScript exceptions with detailed messages:

```rust
fn parse(input: &str) -> Result<JsCifDocument, String> {
    match CifDocument::parse(input) {
        Ok(doc) => Ok(JsCifDocument { inner: doc }),
        Err(e) => {
            let error_msg = match e {
                CifError::ParseError(msg) => {
                    format!("Parse error: {}", msg)
                }
                CifError::IoError(err) => {
                    format!("IO error: {}", err)
                }
                CifError::InvalidStructure { message, location } => {
                    if let Some((line, col)) = location {
                        format!("Invalid structure at line {}, col {}: {}",
                                line, col, message)
                    } else {
                        format!("Invalid structure: {}", message)
                    }
                }
            };
            Err(error_msg)
        }
    }
}
```

**Mapping:**
- `CifError::ParseError` → String exception
- `CifError::IoError` → String exception
- `CifError::InvalidStructure` → String with location info

### Usage in JavaScript

```javascript
try {
    const doc = parse(invalidCif);
} catch (error) {
    console.error(error);
    // "Invalid structure at line 5, col 3: Loop has no tags"
}
```

### Console Logging

Debug logging to browser console:

```rust
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
pub fn parse(input: &str) -> Result<JsCifDocument, String> {
    console_log!("Parsing CIF content of length: {}", input.len());
    // ...
}
```

## API Alignment with Python

### Design Goals

The WASM API is designed to closely mirror the Python API while being idiomatic to JavaScript:

| Aspect | Python | JavaScript | Alignment |
|--------|--------|------------|-----------|
| Naming | snake_case | camelCase | ✓ Converted |
| Properties | `loop.tags` | `loop.tags` | ✓ Identical |
| Methods | `loop.get_row_dict(0)` | `loop.get_row_dict(0)` | ✓ Identical |
| Iteration | `for row in loop` | Not yet | ⚠️ Future |
| Type checking | `value.is_numeric` | `value.is_numeric()` | ✓ Method call |

### Feature Parity

**Python API:**
```python
# Property access
tags = loop.tags
num_rows = len(loop)
num_cols = loop.num_columns

# Get row as dict
row = loop.get_row_dict(0)
for tag, value in row.items():
    print(f"{tag}: {value}")

# Get column
column = loop.get_column("_atom_site_x")
```

**JavaScript API (aligned):**
```javascript
// Property access
const tags = loop.tags;
const numRows = loop.numRows;      // Property instead of len()
const numCols = loop.numColumns;

// Get row as object
const row = loop.get_row_dict(0);
for (const [tag, value] of Object.entries(row)) {
    console.log(`${tag}: ${value}`);
}

// Get column
const column = loop.get_column("_atom_site_x");
```

### Key Differences

**1. Property vs Length:**
```python
# Python - len() protocol
num_rows = len(loop)
```
```javascript
// JavaScript - property
const numRows = loop.numRows;
```

**2. Type Checking:**
```python
# Python - property
if value.is_numeric:
    x = value.numeric
```
```javascript
// JavaScript - method (wasm-bindgen limitation)
if (value.is_numeric()) {
    const x = value.numeric_value;
}
```

**3. Iteration:**
```python
# Python - iterator protocol
for row in loop:
    process(row)
```
```javascript
// JavaScript - manual (no iterator yet)
for (let i = 0; i < loop.numRows; i++) {
    const row = loop.get_row_dict(i);
    process(row);
}
```

## Memory Management

### WASM Memory Model

- **Linear memory**: WASM has a single contiguous memory space
- **Garbage collection**: JavaScript GC manages WASM object lifetimes
- **No manual cleanup**: Unlike some WASM bindings, no `.free()` needed

### Ownership and Cloning

**Most operations clone data:**
```rust
#[wasm_bindgen(getter)]
pub fn tags(&self) -> Vec<String> {
    self.inner.tags.clone()  // Clone for safety
}
```

**Trade-offs:**
- ✅ Safe: JavaScript owns its data
- ✅ Simple: No lifetime management
- ⚠️ Slightly less efficient than borrowing
- ✓ Acceptable: Crystallographic data is typically small

### Memory Efficiency Tips

```javascript
// Efficient - reuse document
const doc = parse(cifContent);
for (let i = 0; i < doc.blockCount; i++) {
    processBlock(doc.get_block(i));
}

// Less efficient - parse repeatedly
for (const content of cifFiles) {
    const doc = parse(content);  // Creates new memory each time
}
```

## Build Targets

### 1. Web (ES Modules)

```bash
just wasm-build-web
```

**Usage:**
```html
<script type="module">
    import init, { parse } from './javascript/pkg/cif_parser.js';

    await init();  // Initialize WASM
    const doc = parse(cifContent);
</script>
```

**Files generated:**
- `cif_parser.js` - ES module wrapper
- `cif_parser_bg.wasm` - WASM binary
- `cif_parser.d.ts` - TypeScript definitions

### 2. Node.js (CommonJS)

```bash
just wasm-build
```

**Usage:**
```javascript
const { parse } = require('./javascript/pkg-node/cif_parser.js');

const doc = parse(cifContent);  // No init() needed
```

### 3. Bundler (Webpack, Rollup, etc.)

```bash
just wasm-build-bundler
```

**Usage:**
```javascript
import init, { parse } from '@cif-parser/core';

await init();
const doc = parse(cifContent);
```

## Performance Considerations

### Binary Size

**Optimized build:**
```bash
just wasm-build-web  # Builds in release mode by default
```

**Current size:** ~225KB (gzipped)

**Optimization in Cargo.toml:**
```toml
[profile.release]
opt-level = "z"          # Optimize for size
lto = true              # Link-time optimization
codegen-units = 1       # Single codegen unit
strip = true            # Strip symbols
```

### Parse Performance

- **Near-native speed**: ~90-95% of native Rust performance
- **Typical CIF**: <10ms parse time for average protein structure
- **Large mmCIF**: <100ms for large macromolecular structures

### Memory Usage

- **Overhead**: WASM adds ~10-20% memory overhead vs native
- **Cloning**: Data is cloned when crossing WASM boundary
- **Typical usage**: 1-5 MB for average crystallographic data

### Best Practices

```javascript
// Good - minimize boundary crossings
const tags = loop.tags;
const column = loop.get_column(tags[0]);

// Less efficient - many boundary crossings
for (let i = 0; i < loop.numRows; i++) {
    for (let j = 0; j < loop.numColumns; j++) {
        const val = loop.get_value(i, j);  // Crosses boundary each time
    }
}

// Better - batch operations
for (let i = 0; i < loop.numRows; i++) {
    const row = loop.get_row_dict(i);  // Single boundary crossing per row
    processRow(row);
}
```

## Publishing to NPM

### Package Configuration

**Generated package.json includes:**
```json
{
  "name": "@cif-parser/web",
  "version": "0.1.0",
  "files": [
    "cif_parser_bg.wasm",
    "cif_parser.js",
    "cif_parser.d.ts",
    "cif_parser_bg.js"
  ],
  "main": "cif_parser.js",
  "types": "cif_parser.d.ts"
}
```

### Build and Publish

```bash
# Build all targets
just wasm-build-all

# Publish to NPM
cd javascript/pkg && npm publish --access public
cd javascript/pkg-node && npm publish --access public --tag nodejs
cd javascript/pkg-bundler && npm publish --access public --tag bundler
```

## Comparison with Python Bindings

### Similarities

| Feature | Python | JavaScript | Status |
|---------|--------|------------|--------|
| Wrapper pattern | ✓ | ✓ | Identical |
| Property getters | ✓ | ✓ | Aligned |
| Type checking | ✓ | ✓ | Similar |
| Error messages | ✓ | ✓ | With location |
| Module functions | `parse()`, `parse_file()` | `parse()` | Partial |
| Row as dict/object | `get_row_dict()` | `get_row_dict()` | ✓ |
| Column access | `get_column()` → list | `get_column()` → array | ✓ |

### Differences

| Aspect | Python | JavaScript | Reason |
|--------|--------|------------|--------|
| Build system | Maturin + PyO3 | wasm-pack + wasm-bindgen | Different targets |
| Type checking | Properties | Methods | wasm-bindgen limitation |
| Length | `len()` protocol | `numRows` property | Language idioms |
| Iteration | `for x in loop` | Manual loop | Not yet implemented |
| File I/O | `parse_file()` | Not available | WASM security model |
| Naming | snake_case | camelCase | Language conventions |

### Implementation Comparison

**Python (PyO3):**
```rust
#[pymethods]
impl PyLoop {
    #[getter]
    fn tags(&self) -> Vec<String> {
        self.inner.tags.clone()
    }

    fn __len__(&self) -> usize {
        self.inner.len()
    }
}
```

**JavaScript (wasm-bindgen):**
```rust
#[wasm_bindgen]
impl JsCifLoop {
    #[wasm_bindgen(getter)]
    pub fn tags(&self) -> Vec<String> {
        self.inner.tags.clone()
    }

    #[wasm_bindgen(getter = numRows)]
    pub fn num_rows(&self) -> usize {
        self.inner.len()
    }
}
```

**Key differences:**
- Python uses `#[pymethods]`, WASM uses `#[wasm_bindgen]`
- Python has `__len__()` protocol, WASM uses explicit property
- Both support property getters with similar syntax

## Related Documentation

- [Python Bindings Reference](python-bindings.md) - Comparison and Python API
- [CIF Format Hierarchy](cif-format-hierarchy.md) - Understanding CIF structure
- [AST Construction](ast-construction.md) - How the parser builds the AST
- [wasm-bindgen Documentation](https://rustwasm.github.io/wasm-bindgen/) - Official docs
- [wasm-pack Documentation](https://rustwasm.github.io/wasm-pack/) - Build tool docs
