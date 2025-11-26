# CIF Validator

A Rust crate for validating CIF files against DDLm dictionaries, with Python and WASM bindings.

## Overview

The `cif-validator` crate provides semantic validation of CIF (Crystallographic Information File) documents against DDLm dictionaries. It builds on `cif-parser` for syntax parsing and `drel-parser` for dictionary consistency checking.

### Key Features

- **DDLm Dictionary Support**: Load and parse DDLm dictionaries (CIF 2.0 format)
- **Type Validation**: Check values match dictionary-defined types (Real, Integer, Text, etc.)
- **Constraint Checking**: Validate enumeration sets, numeric ranges, mandatory items
- **Span Preservation**: All errors include source locations for IDE integration
- **ValidatedCIF Type**: Map dictionary definitions to source positions for hover/go-to-definition
- **Multi-language Bindings**: Python (PyO3) and WASM (wasm-bindgen)

## Architecture

```
cif-validator/
├── src/
│   ├── lib.rs               # Public API, Validator builder
│   ├── dictionary/          # Dictionary representation and loading
│   │   ├── mod.rs
│   │   ├── types.rs         # Dictionary, Category, DataItem, TypeInfo
│   │   ├── loader.rs        # Parse .dic files using cif-parser
│   │   └── validator.rs     # Validate dictionary internal consistency
│   ├── validator/           # CIF validation engine
│   │   ├── mod.rs
│   │   ├── engine.rs        # Core validation logic
│   │   ├── types.rs         # Type checking (Real, Integer, Text, etc.)
│   │   └── constraints.rs   # Enumeration, range, mandatory checking
│   ├── validated.rs         # ValidatedCIF with span-to-definition mapping
│   ├── error.rs             # ValidationError/Warning with spans
│   ├── python.rs            # PyO3 bindings
│   └── wasm.rs              # wasm-bindgen bindings
└── dics/
    └── cif_core.dic         # Core dictionary for testing
```

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| DDL support | DDLm only | Modern standard, recommended by IUCr since 2014 |
| Dictionary bundling | Runtime only | User provides path; smaller binary, more flexible |
| dREL handling | Parse for validation only | dREL documents relationships; not executed at runtime |
| Typed CIF | Runtime + future schema gen | ValidatedCIF now, Pydantic/TS schemas later |

## Usage

### Rust

```rust
use cif_parser::Document;
use cif_validator::{Validator, ValidationMode, ValidatedCif};

// Parse CIF and dictionary
let cif_doc = Document::from_file("structure.cif")?;

// Create validator with dictionary
let validator = Validator::new()
    .with_dictionary_file("cif_core.dic")?
    .with_mode(ValidationMode::Strict);

// Validate
let result = validator.validate(&cif_doc)?;

if !result.is_valid {
    for error in &result.errors {
        println!("{} at line {}, col {}",
            error.message, error.span.start_line, error.span.start_col);
    }
}

// Or get ValidatedCIF for typed access
let validated = validator.validate_typed(cif_doc)?;

// IDE feature: get definition at cursor position
if let Some(defn) = validated.definition_at(5, 10) {
    println!("Item: {}", defn.name);
    println!("Type: {:?}", defn.type_info.contents);
    println!("Description: {}", defn.description.unwrap_or_default());
}
```

### Python

```python
from cif_validator import Dictionary, Validator

# Load dictionary
dict = Dictionary.from_file("cif_core.dic")

# Validate CIF content
validator = Validator()
validator.with_dictionary(dict)
validator.with_mode("strict")

result = validator.validate_string(cif_content)

if not result.is_valid:
    for error in result.errors:
        print(f"{error.message} at line {error.line}, col {error.column}")
        if error.suggestions:
            print(f"  Did you mean: {error.suggestions}")

# Get definition at source position (for IDE hover)
validated = validator.validate_to_typed(cif_content)
defn = validated.definition_at(line=5, col=10)
if defn:
    print(f"{defn.name}: {defn.description}")
```

### JavaScript/WASM

```javascript
import { Dictionary, validate, definitionAt } from '@cif-tools/validator';

const dict = Dictionary.parse(dictContent);
const result = validate(cifContent, dict);

if (!result.isValid) {
    for (const error of result.getErrors()) {
        console.log(`${error.message} at ${error.line}:${error.column}`);
    }
}

// IDE feature: get definition at cursor
const defn = definitionAt(cifContent, dict, line, col);
if (defn) {
    console.log(defn.name, defn.description);
}
```

## Key Types

### Dictionary Types

```rust
/// Complete DDLm dictionary
pub struct Dictionary {
    pub metadata: DictionaryMetadata,
    pub categories: HashMap<String, Category>,
    pub items: HashMap<String, DataItem>,      // canonical name → item
    pub aliases: HashMap<String, String>,       // alias → canonical name
}

/// A single data item definition
pub struct DataItem {
    pub name: String,                           // "_diffrn.ambient_pressure"
    pub category: String,                       // "diffrn"
    pub object: String,                         // "ambient_pressure"
    pub aliases: Vec<String>,                   // ["_diffrn_ambient_pressure"]
    pub type_info: TypeInfo,
    pub constraints: ValueConstraints,
    pub description: Option<String>,
    pub span: Span,                             // Location in dictionary file
}

/// DDLm type information
pub struct TypeInfo {
    pub contents: ContentType,                  // Real, Integer, Text, Word, etc.
    pub container: ContainerType,               // Single, List, Matrix, Table
    pub purpose: Purpose,                       // Measurand, Describe, Link, Key
    pub units: Option<String>,
}
```

### Validation Types

```rust
/// Validation result
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

/// A validation error with full context
pub struct ValidationError {
    pub category: ErrorCategory,
    pub message: String,
    pub span: Span,                             // Location in input CIF
    pub data_name: Option<String>,
    pub expected: Option<String>,
    pub actual: Option<String>,
    pub suggestions: Vec<String>,               // "Did you mean...?"
}

pub enum ErrorCategory {
    UnknownDataName,    // Not in dictionary
    TypeError,          // Wrong type (e.g., text where Real expected)
    RangeError,         // Value outside allowed range
    EnumerationError,   // Value not in allowed set
    MissingMandatory,   // Required item missing
    LoopStructure,      // Invalid loop structure
}
```

### ValidatedCIF

```rust
/// A CIF document validated against a dictionary
pub struct ValidatedCif {
    document: CifDocument,
    dictionary: Arc<Dictionary>,
    span_index: SpanIndex,
}

impl ValidatedCif {
    /// Look up definition at a source position (for IDE hover)
    pub fn definition_at(&self, line: usize, col: usize) -> Option<&DataItem>;

    /// Get typed value with definition attached
    pub fn get_typed<T: FromCifValue>(&self, block: &str, item: &str) -> Option<TypedValue<T>>;
}
```

## Key Insights from Design

### 1. Dictionaries Are CIF Files

DDLm dictionaries are themselves CIF 2.0 files. Each save frame defines either a category or a data item:

```cif
save_diffrn.ambient_pressure
    _definition.id                '_diffrn.ambient_pressure'
    _alias.definition_id          '_diffrn_ambient_pressure'
    _type.purpose                 Measurand
    _type.contents                Real
    _enumeration.range            0.0:
    _units.code                   kilopascals
save_
```

This means we use `cif-parser` to load dictionaries - no separate parser needed.

### 2. dREL Is Documentation, Not Code

dREL (dictionary Relational Expression Language) methods in dictionaries describe relationships but are **not executed at runtime**. They serve two purposes:

1. **Documentation**: Describe how values relate (e.g., density = mass/volume)
2. **Dictionary validation**: If a dREL method references an item not in the dictionary, the dictionary itself is invalid

Software implementing CIF validation is free to compute values however it wants - dREL is a reference implementation, not the actual code.

### 3. Alias Support Is Essential

Legacy CIF 1.0 names (e.g., `_diffrn_ambient_pressure`) must map to DDLm names (`_diffrn.ambient_pressure`). The dictionary provides these mappings via `_alias.definition_id`.

### 4. Spans Enable IDE Features

Every AST node in `cif-parser` carries a `Span` (start_line, start_col, end_line, end_col). This enables:

- **Error reporting**: "Type error at line 42, col 5"
- **Hover info**: Look up dictionary definition at cursor position
- **Go-to-definition**: Jump from CIF value to dictionary save frame

### 5. Case Insensitivity

Per CIF specification, data names are case-insensitive. All lookups normalize to lowercase.

## Validation Modes

- **Strict**: All errors are fatal, unknown data names are errors
- **Lenient**: Unknown data names are warnings, some type coercions allowed
- **Pedantic**: Include stylistic warnings (e.g., deprecated items)

## Future: Schema Generation

The API is designed to support future schema generation:

```bash
# Future tool (not in initial implementation)
cif-schema-gen --dictionary cif_core.dic --output schemas/
```

This would generate:
- **Pydantic models** for Python with full type hints
- **TypeScript interfaces** for JavaScript
- **Rust structs** for compile-time type safety

The `FromCifValue` trait and category-based organization enable this future work.

## Related Documentation

- [CIF Dictionaries and DDL](dictionaries.md) - Deep dive into dictionary structure
- [Python Bindings](python-bindings.md) - PyO3 patterns used
- [WASM Bindings](wasm-bindings.md) - wasm-bindgen patterns used
- [Spans](spans.md) - Source location tracking

## References

- [DDLm Specification](https://www.iucr.org/resources/cif/ddl/ddlm) - IUCr DDLm documentation
- [dREL Paper](https://pubs.acs.org/doi/10.1021/ci300076w) - J. Chem. Inf. Model. 2012
- [CIF Core Dictionary](https://github.com/COMCIFS/cif_core) - Official cif_core.dic
