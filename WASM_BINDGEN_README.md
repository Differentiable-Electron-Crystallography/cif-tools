# WebAssembly Bindings with wasm-pack and wasm-bindgen

This document provides comprehensive documentation on how the WebAssembly (WASM) bindings for the CIF Parser are created using wasm-pack and wasm-bindgen.

## Table of Contents
- [Overview](#overview)
- [Architecture](#architecture)
- [Technology Stack](#technology-stack)
- [Directory Structure](#directory-structure)
- [How wasm-pack Works](#how-wasm-pack-works)
- [wasm-bindgen Integration](#wasm-bindgen-integration)
- [Type Mappings](#type-mappings)
- [Build Targets](#build-targets)
- [Building and Development](#building-and-development)
- [Testing](#testing)
- [Debugging](#debugging)
- [Performance Optimization](#performance-optimization)
- [Publishing to NPM](#publishing-to-npm)
- [Browser Integration](#browser-integration)
- [Node.js Integration](#nodejs-integration)
- [Troubleshooting](#troubleshooting)

## Overview

WebAssembly bindings allow the Rust-based CIF Parser to run in web browsers and Node.js environments with near-native performance. This is achieved through:

- **wasm-bindgen**: Facilitates high-level interactions between WASM modules and JavaScript
- **wasm-pack**: Build tool that orchestrates building, testing, and publishing WASM packages
- **web-sys/js-sys**: Bindings to Web APIs and JavaScript standard objects

## Architecture

```
┌────────────────────────────────────────┐
│     JavaScript/TypeScript Application   │
├────────────────────────────────────────┤
│         JavaScript API Layer            │
│      (pkg/cif_parser.js + .d.ts)       │
├────────────────────────────────────────┤
│    wasm-bindgen Generated Bindings     │
│         (pkg/cif_parser_bg.js)         │
├────────────────────────────────────────┤
│         WASM Module (.wasm file)       │
│    Compiled from src/wasm.rs + lib.rs  │
└────────────────────────────────────────┘
```

## Technology Stack

### wasm-bindgen (v0.2)
Provides:
- High-level bindings between WASM and JavaScript
- Automatic type conversions
- Memory management
- JavaScript class generation
- TypeScript definitions generation

### wasm-pack
Provides:
- Build orchestration
- Testing framework
- NPM package generation
- Documentation generation
- Publishing utilities

### web-sys & js-sys
Provide:
- Bindings to Web APIs (console, DOM, etc.)
- JavaScript standard library bindings
- Type-safe browser API access

## Directory Structure

```
cif-parser/
├── Cargo.toml                    # Rust package manifest
├── src/
│   ├── lib.rs                    # Core library
│   └── wasm.rs                   # WASM bindings
├── pkg/                          # Generated web package
│   ├── package.json
│   ├── cif_parser.js             # ES modules entry
│   ├── cif_parser_bg.wasm        # WASM binary
│   ├── cif_parser_bg.js          # Generated JS bindings
│   └── cif_parser.d.ts           # TypeScript definitions
├── pkg-node/                     # Generated Node.js package
│   ├── package.json
│   ├── cif_parser.js             # CommonJS entry
│   └── ...
└── pkg-bundler/                  # Generated bundler package
    ├── package.json
    ├── cif_parser.js             # Bundler-friendly entry
    └── ...
```

## How wasm-pack Works

### 1. Build Pipeline

```bash
# Basic build command
wasm-pack build --target web --out-dir pkg

# What happens:
1. Compile Rust to WASM (cargo build --target wasm32-unknown-unknown)
2. Run wasm-bindgen to generate JS bindings
3. Optimize WASM with wasm-opt
4. Generate package.json and TypeScript definitions
5. Create NPM-ready package
```

### 2. Build Stages

#### Stage 1: Rust Compilation
```rust
// Cargo.toml configuration
[dependencies]
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["console"] }
js-sys = "0.3"
serde-wasm-bindgen = "0.6"

[lib]
crate-type = ["cdylib"]  # Dynamic library for WASM
```

#### Stage 2: Binding Generation
```rust
// src/wasm.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct JsCifDocument {
    // Rust struct exposed to JavaScript
}

#[wasm_bindgen]
impl JsCifDocument {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Constructor callable from JS
    }
}
```

#### Stage 3: Optimization
- **wasm-opt**: Reduces binary size (often 10-20% reduction)
- **Tree shaking**: Removes unused code
- **Name mangling**: Shortens internal names

## wasm-bindgen Integration

### 1. Basic Types

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

// JavaScript usage:
// import { add } from './pkg/cif_parser.js';
// console.log(add(2, 3)); // 5
```

### 2. Complex Types

```rust
#[wasm_bindgen]
pub struct JsCifValue {
    value_type: String,
    text_value: Option<String>,
    numeric_value: Option<f64>,
}

#[wasm_bindgen]
impl JsCifValue {
    // Getter for JavaScript property access
    #[wasm_bindgen(getter)]
    pub fn value_type(&self) -> String {
        self.value_type.clone()
    }
    
    // Method callable from JavaScript
    #[wasm_bindgen]
    pub fn is_numeric(&self) -> bool {
        self.numeric_value.is_some()
    }
}
```

### 3. JavaScript Interop

```rust
use wasm_bindgen::prelude::*;
use web_sys::console;

// Call JavaScript console.log
#[wasm_bindgen]
pub fn debug_log(message: &str) {
    console::log_1(&message.into());
}

// Accept JavaScript objects
#[wasm_bindgen]
pub fn process_object(val: &JsValue) -> Result<(), JsValue> {
    // Convert JsValue to Rust types
    let obj: serde_json::Value = serde_wasm_bindgen::from_value(val.clone())?;
    Ok(())
}
```

### 4. Memory Management

```rust
#[wasm_bindgen]
pub struct LargeData {
    #[wasm_bindgen(skip)]  // Not exposed to JS
    internal_data: Vec<u8>,
}

#[wasm_bindgen]
impl LargeData {
    // Manual memory management
    pub fn free(self) {
        // Explicitly drop large data
        drop(self.internal_data);
    }
}
```

## Type Mappings

### Rust to JavaScript

| Rust Type | JavaScript Type | Notes |
|-----------|----------------|-------|
| `i32`, `u32` | `number` | 32-bit integers |
| `i64`, `u64` | `BigInt` | 64-bit integers |
| `f32`, `f64` | `number` | Floating point |
| `bool` | `boolean` | Direct mapping |
| `String` | `string` | UTF-8 conversion |
| `&str` | `string` | Borrowed string |
| `Vec<T>` | `Array` | If T is compatible |
| `Option<T>` | `T | undefined` | None becomes undefined |
| `Result<T, E>` | `T` or throws | Error throws JS exception |
| `struct` | `class` | With #[wasm_bindgen] |

### JavaScript to Rust

| JavaScript Type | Rust Type | Notes |
|----------------|-----------|-------|
| `number` | `f64`, `i32`, etc. | Based on signature |
| `BigInt` | `i64`, `u64` | 64-bit integers |
| `boolean` | `bool` | Direct mapping |
| `string` | `String` or `&str` | UTF-8 conversion |
| `Array` | `Vec<T>` or `js_sys::Array` | Type checking required |
| `undefined` | `Option::None` | When expecting Option |
| `null` | `Option::None` | When expecting Option |
| `object` | `JsValue` | Generic JS value |

## Build Targets

### 1. Web (ES Modules)
```bash
wasm-pack build --target web --out-dir pkg
```

**Usage in HTML:**
```html
<script type="module">
    import init, { JsCifDocument } from './pkg/cif_parser.js';
    
    async function run() {
        await init();  // Initialize WASM module
        const doc = JsCifDocument.parse(cifContent);
    }
    run();
</script>
```

### 2. Node.js (CommonJS)
```bash
wasm-pack build --target nodejs --out-dir pkg-node
```

**Usage in Node.js:**
```javascript
const { JsCifDocument } = require('./pkg-node/cif_parser.js');

const doc = JsCifDocument.parse(cifContent);
```

### 3. Bundler (Webpack/Rollup)
```bash
wasm-pack build --target bundler --out-dir pkg-bundler
```

**Usage with bundlers:**
```javascript
import init, { JsCifDocument } from 'cif-parser';

await init();
const doc = JsCifDocument.parse(cifContent);
```

### 4. No-modules (Legacy)
```bash
wasm-pack build --target no-modules --out-dir pkg-nomodules
```

**Usage with script tags:**
```html
<script src="./pkg-nomodules/cif_parser.js"></script>
<script>
    const { JsCifDocument } = wasm_bindgen;
    
    async function run() {
        await wasm_bindgen('./pkg-nomodules/cif_parser_bg.wasm');
        const doc = JsCifDocument.parse(cifContent);
    }
</script>
```

## Building and Development

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Or via cargo
cargo install wasm-pack
```

### Development Workflow

1. **Make changes** to `src/wasm.rs`

2. **Build for development**:
```bash
# Debug build (faster, larger)
wasm-pack build --dev --target web --out-dir pkg

# Release build (optimized)
wasm-pack build --target web --out-dir pkg
```

3. **Test in browser**:
```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>WASM Test</title>
</head>
<body>
    <script type="module">
        import init, { JsCifDocument } from './pkg/cif_parser.js';
        
        async function test() {
            await init();
            
            const content = `
                data_test
                _cell_length_a 10.0
            `;
            
            const doc = JsCifDocument.parse(content);
            console.log('Parsed blocks:', doc.get_block_count());
        }
        
        test();
    </script>
</body>
</html>
```

4. **Serve locally**:
```bash
# Python
python -m http.server 8000

# Node.js
npx http-server

# Or use any static file server
```

## Testing

### Unit Tests (Rust)

```rust
// src/wasm.rs
#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    
    #[wasm_bindgen_test]
    fn test_parse() {
        let doc = JsCifDocument::parse("data_test\n_item value");
        assert_eq!(doc.get_block_count(), 1);
    }
}
```

Run tests:
```bash
wasm-pack test --headless --firefox
wasm-pack test --headless --chrome
wasm-pack test --node
```

### Integration Tests (JavaScript)

```javascript
// tests/integration.test.js
import { expect } from 'chai';
import init, { JsCifDocument } from '../pkg/cif_parser.js';

describe('CIF Parser WASM', () => {
    before(async () => {
        await init();
    });
    
    it('should parse simple CIF', () => {
        const doc = JsCifDocument.parse('data_test\n_item value');
        expect(doc.get_block_count()).to.equal(1);
    });
});
```

## Debugging

### 1. Console Logging

```rust
use web_sys::console;

#[wasm_bindgen]
pub fn parse_with_debug(content: &str) -> JsCifDocument {
    console::log_1(&"Starting parse...".into());
    console::log_2(&"Content length:".into(), &content.len().into());
    
    // Parse and log result
    let result = JsCifDocument::parse(content);
    console::log_1(&"Parse complete!".into());
    
    result
}
```

### 2. Browser DevTools

Enable source maps for debugging:
```bash
wasm-pack build --dev --target web
```

In Chrome DevTools:
- Enable "WebAssembly Debugging" in Experiments
- Set breakpoints in Rust source (when source maps work)
- Use Memory inspector for WASM memory

### 3. Panic Handling

```rust
use wasm_bindgen::prelude::*;

// Better panic messages in browser console
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
```

Add to Cargo.toml:
```toml
[dependencies]
console_error_panic_hook = "0.1"
```

## Performance Optimization

### 1. Binary Size Optimization

```toml
# Cargo.toml
[profile.release]
opt-level = "z"          # Optimize for size
lto = true              # Link-time optimization
codegen-units = 1       # Single codegen unit
strip = true            # Strip symbols

[package.metadata.wasm-pack]
"wasm-opt" = ["-Oz"]    # Aggressive size optimization
```

### 2. Memory Management

```rust
// Avoid unnecessary allocations
#[wasm_bindgen]
impl JsCifDocument {
    // Return reference when possible
    #[wasm_bindgen(getter)]
    pub fn block_count(&self) -> usize {
        self.blocks.len()  // No allocation
    }
    
    // Use iterators instead of collecting
    pub fn find_block(&self, name: &str) -> Option<JsCifBlock> {
        self.blocks.iter()
            .find(|b| b.name == name)
            .map(|b| JsCifBlock::from(b))
    }
}
```

### 3. Minimize JS/WASM Boundary Crossings

```rust
// Bad: Multiple crossings
#[wasm_bindgen]
impl JsCifLoop {
    pub fn get_value(&self, row: usize, col: usize) -> JsCifValue {
        // Crosses boundary for each value
    }
}

// Good: Batch operations
#[wasm_bindgen]
impl JsCifLoop {
    pub fn get_row_json(&self, row: usize) -> String {
        // Single crossing, return JSON string
        serde_json::to_string(&self.get_row(row)).unwrap()
    }
}
```

### 4. Lazy Initialization

```rust
use once_cell::sync::Lazy;

static PARSER: Lazy<Parser> = Lazy::new(|| {
    // Heavy initialization only once
    Parser::new()
});

#[wasm_bindgen]
pub fn parse(content: &str) -> JsCifDocument {
    PARSER.parse(content)
}
```

## Publishing to NPM

### 1. Package Configuration

Update generated `package.json`:
```json
{
  "name": "@cif-parser/web",
  "version": "0.1.0",
  "description": "WebAssembly CIF parser for browsers",
  "main": "cif_parser.js",
  "types": "cif_parser.d.ts",
  "files": [
    "cif_parser_bg.wasm",
    "cif_parser.js",
    "cif_parser.d.ts",
    "cif_parser_bg.js"
  ],
  "keywords": ["cif", "crystallography", "wasm", "parser"],
  "author": "Your Name",
  "license": "MIT OR Apache-2.0",
  "repository": {
    "type": "git",
    "url": "https://github.com/your/repo"
  }
}
```

### 2. Build All Targets

```bash
# Build for different targets
wasm-pack build --target web --out-dir pkg --scope cif-parser
wasm-pack build --target nodejs --out-dir pkg-node --scope cif-parser
wasm-pack build --target bundler --out-dir pkg-bundler --scope cif-parser
```

### 3. Publish

```bash
# Login to NPM
npm login

# Publish packages
cd pkg && npm publish --access public
cd ../pkg-node && npm publish --access public
cd ../pkg-bundler && npm publish --access public
```

## Browser Integration

### Modern Browsers (ES Modules)

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>CIF Parser Demo</title>
</head>
<body>
    <input type="file" id="file-input" accept=".cif">
    <pre id="output"></pre>
    
    <script type="module">
        import init, { JsCifDocument } from './pkg/cif_parser.js';
        
        async function initialize() {
            await init();
            
            document.getElementById('file-input').addEventListener('change', async (e) => {
                const file = e.target.files[0];
                const content = await file.text();
                
                try {
                    const doc = JsCifDocument.parse(content);
                    const output = document.getElementById('output');
                    output.textContent = `Parsed ${doc.get_block_count()} blocks`;
                    
                    // Process first block
                    const block = doc.get_first_block();
                    if (block) {
                        output.textContent += `\nFirst block: ${block.name}`;
                        output.textContent += `\nItems: ${block.get_item_keys().join(', ')}`;
                    }
                } catch (error) {
                    console.error('Parse error:', error);
                }
            });
        }
        
        initialize();
    </script>
</body>
</html>
```

### React Integration

```jsx
// CifParser.jsx
import React, { useEffect, useState } from 'react';
import init, { JsCifDocument } from '@cif-parser/bundler';

function CifParser() {
    const [isReady, setIsReady] = useState(false);
    const [result, setResult] = useState(null);
    
    useEffect(() => {
        init().then(() => setIsReady(true));
    }, []);
    
    const handleParse = (content) => {
        if (!isReady) return;
        
        try {
            const doc = JsCifDocument.parse(content);
            setResult({
                blockCount: doc.get_block_count(),
                blockNames: doc.get_block_names()
            });
        } catch (error) {
            console.error('Parse error:', error);
        }
    };
    
    return (
        <div>
            {!isReady && <p>Loading WASM...</p>}
            {isReady && (
                <textarea 
                    onChange={(e) => handleParse(e.target.value)}
                    placeholder="Paste CIF content here"
                />
            )}
            {result && (
                <div>
                    <p>Blocks: {result.blockCount}</p>
                    <p>Names: {result.blockNames.join(', ')}</p>
                </div>
            )}
        </div>
    );
}
```

## Node.js Integration

### Basic Usage

```javascript
// parse-cif.js
const fs = require('fs');
const { JsCifDocument } = require('@cif-parser/node');

function parseCifFile(filepath) {
    const content = fs.readFileSync(filepath, 'utf8');
    const doc = JsCifDocument.parse(content);
    
    console.log(`Parsed ${doc.get_block_count()} blocks`);
    
    for (let i = 0; i < doc.get_block_count(); i++) {
        const block = doc.get_block(i);
        console.log(`Block ${i}: ${block.name}`);
        console.log(`  Items: ${block.get_item_keys().length}`);
        console.log(`  Loops: ${block.get_loop_count()}`);
    }
}

parseCifFile('structure.cif');
```

### Express.js API

```javascript
const express = require('express');
const { JsCifDocument } = require('@cif-parser/node');

const app = express();
app.use(express.text());

app.post('/parse', (req, res) => {
    try {
        const doc = JsCifDocument.parse(req.body);
        
        res.json({
            success: true,
            blocks: doc.get_block_count(),
            data: doc.get_block_names()
        });
    } catch (error) {
        res.status(400).json({
            success: false,
            error: error.message
        });
    }
});

app.listen(3000);
```

## Troubleshooting

### Build Issues

**Problem**: `error: failed to download binaryen`
```bash
# Solution: Install binaryen manually
# macOS
brew install binaryen

# Ubuntu/Debian
apt-get install binaryen

# Or skip optimization
wasm-pack build --no-opt
```

**Problem**: `Error: Cannot find module 'env'`
```javascript
// Solution: Ensure proper initialization
import init from './pkg/cif_parser.js';
await init();  // Must await before using other exports
```

### Runtime Issues

**Problem**: `RuntimeError: unreachable executed`
```rust
// Add better error handling
#[wasm_bindgen]
pub fn parse(content: &str) -> Result<JsCifDocument, JsValue> {
    CifDocument::parse(content)
        .map(|doc| JsCifDocument { inner: doc })
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
```

**Problem**: CORS errors when loading WASM
```javascript
// Solution 1: Serve from same origin
// Solution 2: Configure CORS headers
app.use((req, res, next) => {
    res.header('Access-Control-Allow-Origin', '*');
    if (req.url.endsWith('.wasm')) {
        res.header('Content-Type', 'application/wasm');
    }
    next();
});
```

### Performance Issues

**Problem**: Slow initial load
```javascript
// Solution: Preload WASM module
const wasmModule = await WebAssembly.compileStreaming(
    fetch('./pkg/cif_parser_bg.wasm')
);

// Later: Initialize with precompiled module
await init(wasmModule);
```

**Problem**: High memory usage
```rust
// Monitor memory in Rust
use web_sys::console;

#[wasm_bindgen]
pub fn memory_usage() -> usize {
    wasm_bindgen::memory().buffer().byte_length()
}

// Clean up large objects
#[wasm_bindgen]
impl JsCifDocument {
    pub fn free(self) {
        // Explicit cleanup
        drop(self);
    }
}
```

## Advanced Topics

### Streaming Compilation

```javascript
// Streaming compilation for faster startup
async function initWasm() {
    const response = await fetch('./pkg/cif_parser_bg.wasm');
    const module = await WebAssembly.compileStreaming(response);
    await init(module);
}
```

### Web Workers

```javascript
// worker.js
importScripts('./pkg/cif_parser.js');

let initialized = false;

self.addEventListener('message', async (e) => {
    if (!initialized) {
        await wasm_bindgen('./pkg/cif_parser_bg.wasm');
        initialized = true;
    }
    
    const { JsCifDocument } = wasm_bindgen;
    const doc = JsCifDocument.parse(e.data);
    
    self.postMessage({
        blockCount: doc.get_block_count(),
        blockNames: doc.get_block_names()
    });
});
```

### Custom Memory Management

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn allocate(size: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

#[wasm_bindgen]
pub fn deallocate(ptr: *mut u8, size: usize) {
    unsafe {
        Vec::from_raw_parts(ptr, size, size);
    }
}
```

### SharedArrayBuffer Support

```rust
use wasm_bindgen::prelude::*;
use js_sys::SharedArrayBuffer;

#[wasm_bindgen]
pub fn process_shared_buffer(buffer: &SharedArrayBuffer) {
    // Process shared memory
    let length = buffer.byte_length();
    // ...
}
```

## Resources

- [wasm-bindgen Book](https://rustwasm.github.io/wasm-bindgen/)
- [wasm-pack Documentation](https://rustwasm.github.io/wasm-pack/)
- [WebAssembly MDN](https://developer.mozilla.org/en-US/docs/WebAssembly)
- [Rust and WebAssembly Book](https://rustwasm.github.io/docs/book/)
- [web-sys Documentation](https://rustwasm.github.io/wasm-bindgen/api/web_sys/)
- [WebAssembly Studio](https://webassembly.studio/) - Online IDE