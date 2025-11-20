# CIF Version Detection and Support

This document describes how CIF version detection works and how to use version-specific features in the parser.

## Overview

The parser supports both **CIF 1.1** and **CIF 2.0** specifications with automatic version detection based on the magic header.

### Version Detection

- **CIF 2.0**: Files starting with `#\#CIF_2.0`
- **CIF 1.1**: Files without the magic header (default)

## CIF 2.0 Features

CIF 2.0 adds several new features over CIF 1.1:

1. **Lists**: Ordered collections `[value1 value2 value3]`
2. **Tables**: Key-value pairs `{key1:value1 key2:value2}`
3. **Triple-quoted strings**: `"""..."""` and `'''...'''`
4. **Full Unicode support** (UTF-8)
5. **Newlines in quoted strings**

## API Usage

### Python Bindings

```python
from cif_parser import Document, Version

# Parse a CIF file (auto-detects version)
doc = Document.parse(cif_content)

# Check the version
print(doc.version)  # Version.V1_1 or Version.V2_0
print(doc.is_cif1())  # True/False
print(doc.is_cif2())  # True/False

# Version properties
if doc.version.is_cif2:
    print("This is a CIF 2.0 document")

# Working with CIF 2.0 values
block = doc.first_block()
value = block.get_item("_coordinates")

if value.is_list:
    print(f"List value: {value}")
    # Convert to Python list
    py_list = value.to_python()

if value.is_table:
    print(f"Table value: {value}")
    # Convert to Python dict
    py_dict = value.to_python()
```

### WASM/JavaScript Bindings

```javascript
const cif = require('cif-parser');

// Parse a CIF file (auto-detects version)
const doc = cif.parse(cifContent);

// Check the version
console.log(doc.version.toString());  // "CIF 1.1" or "CIF 2.0"
console.log(doc.isCif1());  // true/false
console.log(doc.isCif2());  // true/false

// Version properties
if (doc.version.isCif2()) {
    console.log("This is a CIF 2.0 document");
}

// Working with CIF 2.0 values
const block = doc.first_block();
const value = block.get_item("_coordinates");

if (value.is_list()) {
    console.log("List value:", value.list_value);
}

if (value.is_table()) {
    console.log("Table value:", value.table_value);
}
```

### Rust API

```rust
use cif_parser::{Document, CifVersion};

// Parse a CIF file (auto-detects version)
let doc = Document::parse(cif_content)?;

// Check the version
match doc.version {
    CifVersion::V1_1 => println!("CIF 1.1 document"),
    CifVersion::V2_0 => println!("CIF 2.0 document"),
}

// Working with values
let block = doc.first_block().unwrap();
if let Some(value) = block.items.get("_coordinates") {
    if let Some(list) = value.as_list() {
        println!("List with {} items", list.len());
    }
    if let Some(table) = value.as_table() {
        println!("Table with {} entries", table.len());
    }
}
```

## Value Type Methods

All bindings expose these methods for checking value types:

### Common to Both Versions

- `is_text()` / `is_text` - Check if value is text
- `is_numeric()` / `is_numeric` - Check if value is numeric
- `is_unknown()` / `is_unknown` - Check if value is `?` (unknown)
- `is_not_applicable()` / `is_not_applicable` - Check if value is `.` (not applicable)

### CIF 2.0 Only

- `is_list()` / `is_list` - Check if value is a list
- `is_table()` / `is_table` - Check if value is a table

## Type Safety

The parser ensures type safety:

- **CIF 1.1 documents** will never contain `List` or `Table` values
- **CIF 2.0 documents** can contain all value types
- Attempting to use CIF 2.0 features in a CIF 1.1 file will result in parse errors
- The `is_list()` and `is_table()` methods will always return `false` for CIF 1.1 values

## Examples

See the following example files:

- Python: `examples/version_detection.py`
- JavaScript: `examples/version_detection.js`

## Forcing a Specific Version (Rust Only)

```rust
use cif_parser::{Document, CifVersion};

// Force CIF 2.0 parsing even without magic header
let doc = Document::parse_with_version(content, CifVersion::V2_0)?;

// Force CIF 1.1 for strict compatibility
let doc = Document::parse_with_version(content, CifVersion::V1_1)?;
```

Note: Version forcing is currently only available in the Rust API.

## Migration Guide

### From CIF 1.1 to CIF 2.0

When migrating from CIF 1.1 to CIF 2.0:

1. Add the `#\#CIF_2.0` magic header to your files
2. Optionally use new features like lists and tables
3. Update your code to check `doc.version` if needed
4. CIF 2.0 is backward compatible - all CIF 1.1 files are valid CIF 2.0

### Handling Both Versions

```python
# Python example
doc = Document.parse(content)

if doc.is_cif2():
    # Handle CIF 2.0 specific features
    block = doc.first_block()
    for key in block.item_keys:
        value = block.get_item(key)
        if value.is_list:
            # Process list value
            pass
        elif value.is_table:
            # Process table value
            pass
else:
    # CIF 1.1 - only basic types available
    pass
```

## References

- [CIF 1.1 Specification](https://www.iucr.org/resources/cif/spec/version1.1/cifsyntax)
- [CIF 2.0 EBNF Grammar](https://www.iucr.org/__data/assets/text_file/0009/112131/CIF2-ENBF.txt)
- [CIF 2.0 vs CIF 1.1 Comparison](./cif-versions.md)
