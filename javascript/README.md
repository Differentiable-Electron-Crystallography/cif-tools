# CIF Parser - JavaScript/TypeScript (WebAssembly)

Fast CIF (Crystallographic Information File) parser for JavaScript and TypeScript, compiled from Rust to WebAssembly.

## Features

- ✅ **Full CIF 1.1 Support** - Complete syntax compliance including mmCIF/PDBx
- ✅ **High Performance** - Near-native speed with optimized WASM binary
- ✅ **TypeScript Ready** - Complete type definitions included
- ✅ **Multiple Targets** - Web browsers, Node.js, and bundlers (webpack, rollup, etc.)
- ✅ **Zero Dependencies** - Self-contained WASM module
- ✅ **Small Bundle** - Optimized binary size (~225KB gzipped)

## Installation

### From NPM (once published)

```bash
# For web browsers
npm install @cif-parser/web

# For Node.js
npm install @cif-parser/node

# For bundlers (webpack, rollup, etc.)
npm install @cif-parser/core
```

### Building from Source

```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build for web
wasm-pack build --target web --out-dir javascript/pkg

# Build for Node.js
wasm-pack build --target nodejs --out-dir javascript/pkg-node

# Build for bundlers
wasm-pack build --target bundler --out-dir javascript/pkg-bundler
```

## Quick Start

### Browser (ES Modules)

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>CIF Parser Demo</title>
</head>
<body>
    <script type="module">
        import init, { parse } from './pkg/cif_parser.js';

        async function run() {
            // Initialize WASM module
            await init();

            const cifContent = `
                data_example
                _cell_length_a 10.000
                _cell_length_b 20.000

                loop_
                _atom_site_label
                _atom_site_x
                _atom_site_y
                C1 0.123 0.456
                N1 0.234 0.567
            `;

            // Parse CIF content
            const doc = parse(cifContent);
            console.log(`Parsed ${doc.blockCount} blocks`);

            // Access first block
            const block = doc.first_block();
            if (block) {
                console.log(`Block name: ${block.name}`);

                // Get data item
                const cellA = block.get_item('_cell_length_a');
                if (cellA && cellA.is_numeric()) {
                    console.log(`Cell a: ${cellA.numeric_value}`);
                }

                // Access loop data
                const loop = block.get_loop(0);
                if (loop) {
                    console.log(`Loop has ${loop.numRows} rows, ${loop.numColumns} columns`);

                    // Iterate over rows
                    for (let i = 0; i < loop.numRows; i++) {
                        const label = loop.get_value_by_tag(i, '_atom_site_label');
                        const x = loop.get_value_by_tag(i, '_atom_site_x');
                        console.log(`Atom: ${label.text_value}, x: ${x.numeric_value}`);
                    }
                }
            }
        }

        run();
    </script>
</body>
</html>
```

### Node.js (CommonJS)

```javascript
const { parse } = require('./pkg-node/cif_parser.js');

const cifContent = `
    data_example
    _cell_length_a 10.000
    _author_name 'John Doe'
`;

try {
    const doc = parse(cifContent);
    const block = doc.first_block();

    console.log('Block name:', block.name);
    console.log('Items:', block.itemKeys);

    const author = block.get_item('_author_name');
    console.log('Author:', author.text_value);

} catch (error) {
    console.error('Parse error:', error);
}
```

### TypeScript (with bundler)

```typescript
import init, { parse, JsCifDocument, JsCifBlock } from '@cif-parser/core';

async function parseCif(content: string): Promise<JsCifDocument> {
    await init();
    return parse(content);
}

// Usage
const doc = await parseCif(cifContent);
const block: JsCifBlock | undefined = doc.first_block();
if (block) {
    const itemKeys: string[] = block.itemKeys;
    console.log('Data items:', itemKeys);
}
```

## API Reference

### Module Functions

```typescript
// Parse CIF content (convenience function)
function parse(content: string): JsCifDocument

// Get library version
function version(): string

// Get library author
function author(): string
```

### JsCifDocument

Root container for CIF data.

```typescript
class JsCifDocument {
    // Properties (getters)
    blockCount: number                    // Number of blocks
    blockNames: string[]                  // All block names

    // Methods
    parse(content: string): JsCifDocument // Static: parse CIF string
    get_block(index: number): JsCifBlock | undefined
    get_block_by_name(name: string): JsCifBlock | undefined
    first_block(): JsCifBlock | undefined

    // Legacy method aliases (for compatibility)
    get_block_count(): number
    get_first_block(): JsCifBlock | undefined
    get_block_names(): string[]
}
```

### JsCifBlock

Data block containing items, loops, and frames.

```typescript
class JsCifBlock {
    // Properties (getters)
    name: string                          // Block name
    itemKeys: string[]                    // All data item keys
    numLoops: number                      // Number of loops
    numFrames: number                     // Number of save frames

    // Methods
    get_item(key: string): JsCifValue | undefined
    get_loop(index: number): JsCifLoop | undefined
    find_loop(tag: string): JsCifLoop | undefined
    get_loop_tags(): string[]             // All loop tags
    get_frame(index: number): JsCifFrame | undefined

    // Legacy method aliases
    get_item_keys(): string[]
    get_loop_count(): number
    get_frame_count(): number
}
```

### JsCifLoop

Tabular data structure (rows and columns).

```typescript
class JsCifLoop {
    // Properties (getters)
    tags: string[]                        // Column headers
    numRows: number                       // Number of rows
    numColumns: number                    // Number of columns

    // Methods
    get_value(row: number, col: number): JsCifValue | undefined
    get_value_by_tag(row: number, tag: string): JsCifValue | undefined
    get_column(tag: string): JsCifValue[] | undefined
    get_row_dict(row: number): object     // Row as JS object
    is_empty(): boolean

    // Legacy method aliases
    get_tags(): string[]
    get_row_count(): number
    get_column_count(): number
}
```

### JsCifFrame

Save frame container (sub-block).

```typescript
class JsCifFrame {
    // Properties (getters)
    name: string                          // Frame name
    itemKeys: string[]                    // All data item keys
    numLoops: number                      // Number of loops

    // Methods
    get_item(key: string): JsCifValue | undefined
    get_loop(index: number): JsCifLoop | undefined

    // Legacy method aliases
    get_item_keys(): string[]
    get_loop_count(): number
}
```

### JsCifValue

Individual CIF value with type information.

```typescript
class JsCifValue {
    // Properties (getters)
    value_type: string                    // "Text", "Numeric", "Unknown", "NotApplicable"
    text_value: string | undefined        // Text content (if text type)
    numeric_value: number | undefined     // Numeric content (if numeric type)

    // Methods
    is_text(): boolean
    is_numeric(): boolean
    is_unknown(): boolean
    is_not_applicable(): boolean
}
```

## Usage Examples

### Accessing Data Items

```javascript
const block = doc.first_block();

// Using property getters (modern API)
const keys = block.itemKeys;
console.log('Data items:', keys);

// Get specific item
const cellA = block.get_item('_cell_length_a');
if (cellA && cellA.is_numeric()) {
    console.log('Cell a:', cellA.numeric_value);
}
```

### Working with Loops

```javascript
const loop = block.find_loop('_atom_site_label');

if (loop) {
    // Using property getters
    console.log(`${loop.numRows} rows × ${loop.numColumns} columns`);
    console.log('Columns:', loop.tags);

    // Get entire column
    const labels = loop.get_column('_atom_site_label');
    labels.forEach(val => console.log(val.text_value));

    // Get row as object
    const row = loop.get_row_dict(0);
    console.log('First row:', row);
}
```

### Error Handling

```javascript
try {
    const doc = parse(invalidCif);
} catch (error) {
    // Error includes line/column info when available
    console.error('Parse error:', error);
    // Example: "Invalid structure at line 5, col 3: Loop has no tags"
}
```

### React Integration

```jsx
import { useEffect, useState } from 'react';
import init, { parse } from '@cif-parser/core';

function CifViewer({ cifContent }) {
    const [isReady, setIsReady] = useState(false);
    const [doc, setDoc] = useState(null);

    useEffect(() => {
        init().then(() => setIsReady(true));
    }, []);

    useEffect(() => {
        if (isReady && cifContent) {
            try {
                setDoc(parse(cifContent));
            } catch (error) {
                console.error('Parse error:', error);
            }
        }
    }, [isReady, cifContent]);

    if (!isReady) return <div>Loading WASM...</div>;
    if (!doc) return <div>No document</div>;

    return (
        <div>
            <h2>{doc.blockCount} blocks</h2>
            {/* Render blocks... */}
        </div>
    );
}
```

## Performance Tips

- **Property Access**: Use property getters (e.g., `loop.tags`) instead of methods (e.g., `loop.get_tags()`) for cleaner code
- **Batch Operations**: Use `get_column()` to extract entire columns efficiently
- **Row Objects**: Use `get_row_dict()` for convenient row access as JavaScript objects
- **Binary Size**: The WASM binary is optimized (~225KB gzipped) for fast loading
- **Memory**: WASM memory is automatically managed; no manual cleanup needed

## Build Targets

### Web (ES Modules)
```bash
wasm-pack build --target web --out-dir javascript/pkg
```
Use in browsers with native ES module support.

### Node.js (CommonJS)
```bash
wasm-pack build --target nodejs --out-dir javascript/pkg-node
```
Use in Node.js applications.

### Bundler (Webpack, Rollup, etc.)
```bash
wasm-pack build --target bundler --out-dir javascript/pkg-bundler
```
Use with modern JavaScript bundlers that support WASM.

## Differences from Python API

The JavaScript API closely mirrors the Python API with these differences:

1. **Property Names**: JavaScript uses camelCase (`numRows`) while Python uses snake_case (`num_rows`)
2. **Type System**: JavaScript has runtime type checking via methods like `is_numeric()`
3. **Error Handling**: JavaScript uses try/catch instead of Python exceptions
4. **Initialization**: Web targets require `await init()` before use

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../LICENSE-MIT))

at your option.

## Links

- [Repository](https://github.com/Differentiable-Electron-Crystallography/cif-tools)
- [Documentation](https://docs.rs/cif-parser)
- [Issue Tracker](https://github.com/Differentiable-Electron-Crystallography/cif-tools/issues)
- [Python Package](https://pypi.org/project/cif-parser/)
