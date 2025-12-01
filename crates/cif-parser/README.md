# cif-parser

A Rust library for parsing CIF (Crystallographic Information File) documents with full support for both CIF 1.1 and CIF 2.0 specifications.

## Features

- Full CIF 1.1 and CIF 2.0 syntax support
- Version detection and dialect-specific handling
- Type-safe value access with numeric parsing and uncertainty extraction
- Span tracking for precise error and dialect resolution failure reporting
- Python bindings via PyO3 (optional)
- WebAssembly support via wasm-bindgen (optional)

## Architecture

The parser uses a **two-pass architecture** that cleanly separates syntax parsing from version-specific semantics:

```
Input String
     │
     ▼
┌─────────────────────────────────────────┐
│  Pass 1: Syntax Parsing                 │
│  PEST grammar → RawDocument             │
│  (version-agnostic, lossless)           │
└─────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────┐
│  Pass 2: Dialect Resolution             │
│  RawDocument → CifDocument              │
│  (version-specific rules applied)       │
│                                         │
│  CIF 1.1: permissive, transforms        │
│  CIF 2.0: strict, validates             │
└─────────────────────────────────────────┘
     │
     ▼
  CifDocument (typed, ready to use)
```

### Why Two Passes?

CIF 1.1 and CIF 2.0 have significant semantic differences:

| Construct | CIF 1.1 | CIF 2.0 |
|-----------|---------|---------|
| `[a, b, c]` | Text literal | List value |
| `{'key': val}` | Text literal | Table value |
| `'''text'''` | Text literal | Triple-quoted string |
| `'O''Brien'` | Unescapes to `O'Brien` | Error (use triple quotes) |
| Empty block name | Allowed | Error |

The two-pass design handles this by:
1. **Pass 1** parses syntax without making version decisions, preserving all information
2. **Pass 2** applies version-specific rules (via `VersionRules` trait) to transform or reject constructs

### Extensibility

The `VersionRules` trait can be implemented to create custom dialects beyond CIF 1.1 and 2.0. This is useful for handling CIF files from software that interprets the grammar loosely or adds vendor-specific extensions. A custom dialect can:

- Accept non-standard constructs that would normally be rejected
- Transform legacy or malformed syntax into valid structures
- Apply domain-specific validation rules

This enables the parser to be tweaked for real-world CIF files that don't strictly conform to the published specifications.

### Version Detection

**CIF 2.0 rules are only applied if the magic header is present:**

```
#\#CIF_2.0
data_example
...
```

Files without this header are parsed as CIF 1.1 (permissive mode). This matches the CIF specification requirement that CIF 2.0 files MUST declare themselves.

**Parsing modes:**

| Input | Behavior |
|-------|----------|
| Has `#\#CIF_2.0` header | Strict CIF 2.0 rules (rejects invalid constructs) |
| No magic header | Permissive CIF 1.1 rules (transforms/allows legacy syntax) |
| CIF 1.1 + `upgrade_guidance` option | Parse as 1.1, also report what would fail CIF 2.0 |

The magic header requirement is enforced as a `VersionRule` (`cif2-missing-magic-header`), so upgrade guidance will report it as the first violation for CIF 1.1 files.

## Quick Start

```toml
[dependencies]
cif-parser = "0.1"
```

## Basic Usage

```rust
use cif_parser::{Document, CifError};

fn main() -> Result<(), CifError> {
    let cif = r#"
data_example
_cell.length_a   10.000(5)
_cell.length_b   20.000

loop_
_atom_site.label
_atom_site.type_symbol
_atom_site.fract_x
C1  C  0.1234
N1  N  0.5678
"#;

    let doc = Document::parse(cif)?;
    let block = doc.first_block().unwrap();

    // Access single values
    let cell_a = block.get_item("_cell.length_a").unwrap();
    println!("Cell a: {:?}", cell_a.as_numeric());           // Some(10.0)
    println!("Uncertainty: {:?}", cell_a.uncertainty());     // Some(0.0005)

    // Access loop data
    let atoms = block.find_loop("_atom_site.label").unwrap();
    for row in 0..atoms.len() {
        let label = atoms.get_by_tag(row, "_atom_site.label").unwrap();
        let x = atoms.get_by_tag(row, "_atom_site.fract_x").unwrap();
        println!("{}: x = {:?}", label.as_string().unwrap(), x.as_numeric());
    }

    Ok(())
}
```

## Upgrade Guidance

When parsing a CIF 1.1 file, you can request a report of what would need to change for CIF 2.0 compliance:

```rust
use cif_parser::{parse_string_with_options, ParseOptions};

// This file has no magic header → parsed as CIF 1.1
// But it uses doubled-quote escaping which CIF 2.0 forbids
let cif = "data_test\n_name 'O''Brien'\n";

let result = parse_string_with_options(
    cif,
    ParseOptions::new().upgrade_guidance(true)
)?;

// Parsing succeeds (CIF 1.1 rules applied)
assert_eq!(result.document.version, cif_parser::Version::V1_1);

// But we get a list of CIF 2.0 violations
for issue in &result.upgrade_issues {
    println!("{}", issue);
    // [cif2-no-doubled-quotes] Doubled quotes not allowed at line 2, column 7
    // (suggestion: use triple-quoted string instead)
}
```

This is useful for tools that want to help users migrate legacy CIF files to CIF 2.0.

## Data Structure

```
CifDocument
├── version: CifVersion (V1_1 or V2_0)
├── blocks: Vec<CifBlock>
│   ├── name: String
│   ├── items: HashMap<String, CifValue>
│   ├── loops: Vec<CifLoop>
│   │   ├── tags: Vec<String>
│   │   └── values: Vec<CifValue>
│   └── frames: Vec<CifFrame>  (save frames)
└── span: Span (source location)
```

## Value Types

```rust
use cif_parser::CifValue;

// Check value type
value.is_unknown()        // ?
value.is_not_applicable() // .
value.is_numeric()        // 123.45 or 1.23(4)
value.is_text()           // quoted or unquoted string
value.is_list()           // [a, b, c] (CIF 2.0 only)
value.is_table()          // {'k': v} (CIF 2.0 only)

// Extract values
value.as_string()                    // Option<&str>
value.as_numeric()                   // Option<f64>
value.as_numeric_with_uncertainty()  // Option<(f64, Option<f64>)>
value.as_list()                      // Option<&[CifValue]>
value.as_table()                     // Option<&HashMap<String, CifValue>>
```

## Module Organization

```
cif_parser/
├── lib.rs          # Public API, ParseOptions, ParseResult
├── error.rs        # CifError type
├── ast/            # Final typed AST (CifDocument, CifBlock, etc.)
├── raw/
│   ├── ast/        # Lossless intermediate types (RawDocument, etc.)
│   └── parser/     # Pass 1: PEST grammar → RawDocument
└── rules/          # Pass 2: Version-specific resolution
    ├── mod.rs      # VersionRules trait, VersionViolation
    ├── cif1.rs     # Cif1Rules (permissive)
    └── cif2.rs     # Cif2Rules (strict)
```

## Python Bindings

```bash
just python-develop
```

```python
import cif_parser

doc = cif_parser.parse_string("data_test\n_item value\n")
block = doc.first_block()

cell_a = block.get_item("_cell_length_a")
if cell_a and cell_a.is_numeric:
    print(f"Cell a: {cell_a.numeric}")
```

## WebAssembly

```bash
just wasm-build-web
```

```javascript
import init, { parse } from './pkg/cif_parser.js';

await init();
const doc = parse(cifContent);
const block = doc.get_first_block();
console.log(block.name);
```

## Performance

Benchmarks on a 1.1MB CIF file (`4hzh.cif` - 24,580 lines):

| Stage | Time | Description |
|-------|------|-------------|
| PEST parse | 26 ms | Grammar parsing (lazy) |
| + Tree traversal | 37 ms | Full PEST tree traversal |
| **Full 2-pass** | **43 ms** | Complete parsing to typed AST |

The two-pass architecture adds ~14% overhead compared to raw PEST traversal. This is a reasonable tradeoff for clean separation of syntax parsing from dialect-specific resolution.

Run benchmarks with:
```bash
cargo bench -p cif-parser
```

## Error Handling

Errors include source location information:

```rust
use cif_parser::CifError;

match result {
    Err(CifError::ParseError(msg)) => {
        // Grammar-level error (from PEST), includes line/column
    }
    Err(CifError::InvalidStructure { message, location }) => {
        // Semantic error with optional (line, column)
    }
    Err(CifError::IoError(e)) => {
        // File I/O error
    }
    Ok(doc) => { /* success */ }
}
```

## License

Licensed under either of Apache License 2.0 or MIT license at your option.
