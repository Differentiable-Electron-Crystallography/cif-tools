# CIF Validator

Semantic validation for CIF documents against DDLm dictionaries.

## Overview

The `cif-validator` crate validates CIF files against DDLm dictionaries, checking types, constraints, ranges, and relationships. It builds on:

- **cif-parser** - Syntax parsing (structure)
- **cif-validator** - Semantic validation (meaning)
- **drel-parser** - Dictionary method analysis

**Key capabilities:**
- DDLm dictionary loading and composition
- Type validation (Real, Integer, Text, DateTime, etc.)
- Constraint checking (enumerations, ranges, mandatory items)
- Span preservation for IDE integration (hover, go-to-definition)
- ValidatedCIF type for definition lookup at source positions

---

## Key Concepts

### Dictionaries Are CIF Files

DDLm dictionaries are themselves CIF 2.0 files. Each save frame defines a data item or category:

```cif
save_cell.length_a
    _definition.id       '_cell.length_a'
    _alias.definition_id '_cell_length_a'
    _type.purpose        Measurand
    _type.contents       Real
    _enumeration.range   0.0:
    _units.code          angstroms
save_
```

This meta-circular design means `cif-parser` can load dictionaries—no separate parser needed. The validator then interprets dictionary content to build validation rules.

### DDL Versions

| Version | Status | Use | Notes |
|---------|--------|-----|-------|
| **DDL1** | Deprecated (2014) | Legacy small-molecule | Simple types |
| **DDL2** | Maintained | mmCIF/PDBx | Relational model for macromolecules |
| **DDLm** | Current (v4.2.0) | All modern dictionaries | Rich types, dREL methods, Unicode |

This validator supports **DDLm only**—the modern standard recommended by IUCr since 2014.

### dREL: Documentation, Not Code

dREL (dictionary Relational Expression Language) methods appear in dictionaries:

```drel
_crystal.density = 1.6605 * _cell.atomic_mass / _cell.volume
```

These are **parsed but not executed** at runtime. They serve:
1. **Documentation** - Describe how values relate
2. **Dictionary validation** - Ensure referenced items exist
3. **Dependency analysis** - Build graphs to detect cycles

---

## Architecture

```
CIF Document
    │
    ▼
cif-parser (syntax)
    │
    ▼
CifDocument AST
    │
    ├──────────────────┐
    ▼                  ▼
cif-validator      drel-parser
(semantic rules)   (method analysis)
    │                  │
    ▼                  ▼
ValidationResult   Dependency graphs
```

---

## Usage

### Rust

```rust
use cif_parser::CifDocument;
use cif_validator::{Validator, ValidationMode};

// Parse CIF file
let doc = CifDocument::from_file("structure.cif")?;

// Create validator with dictionary
let validator = Validator::new()
    .with_dictionary_file("cif_core.dic")?
    .with_mode(ValidationMode::Strict);

// Validate
let result = validator.validate(&doc)?;

if !result.is_valid {
    for error in &result.errors {
        println!("{} at line {}, col {}",
            error.message, error.span.start_line, error.span.start_col);
    }
}

// Or get ValidatedCIF for typed access + definition lookup
let validated = validator.validate_typed(doc)?;
let block = validated.first_block().unwrap();
let (value, definition) = block.get_with_def("_cell.length_a").unwrap();
```

### Python

```python
from cif_validator import Validator, ValidationMode

validator = Validator()
validator.add_dictionary_file("cif_core.dic")
validator.set_mode(ValidationMode.Strict)

result = validator.validate(cif_content)

if not result.is_valid:
    for error in result.errors:
        print(f"{error.message} at line {error.span.start_line}")
        if error.suggestions:
            print(f"  Did you mean: {error.suggestions}")
```

### JavaScript/WASM

```javascript
import { JsValidator, validate } from 'cif-validator';

const validator = new JsValidator();
validator.addDictionary(dictionaryContent);
validator.setMode(0); // 0=Strict, 1=Lenient, 2=Pedantic

const result = validator.validate(cifContent);

if (!result.isValid) {
    for (let i = 0; i < result.errorCount; i++) {
        const error = result.get_error(i);
        console.log(`${error.message} at ${error.span.startLine}:${error.span.startCol}`);
    }
}
```

---

## Key Types

### Dictionary

A complete DDLm dictionary (potentially composed from multiple files):

- **metadata** - Title, version, date, namespace
- **categories** - Category definitions indexed by name
- **items** - Data item definitions indexed by canonical name
- **aliases** - Map from legacy names to canonical names

### DataItem

A single data item definition:

- **name** - Canonical name (e.g., `_atom_site.label`)
- **category** - Category this item belongs to
- **aliases** - Legacy names (e.g., `_atom_site_label`)
- **type_info** - Type, container, purpose, source, units
- **constraints** - Enumeration, range, mandatory
- **description** - Human-readable description
- **drel_method** - dREL expression (for dictionary validation)

### TypeInfo

DDLm type information:

- **contents** - Content type (Real, Integer, Text, Code, DateTime, etc.)
- **container** - Container type (Single, List, Matrix, Table)
- **purpose** - Purpose (Measurand, Describe, Link, Key)
- **source** - Source (Recorded, Assigned, Derived)

### ValidatedCIF

A CIF document paired with dictionary metadata, enabling:
- Definition lookup at any source position (IDE hover)
- Typed accessors based on dictionary type information
- Block and loop wrappers with definition access

---

## Validation Modes

| Mode | Behavior |
|------|----------|
| **Strict** | All errors are fatal, unknown data names are errors |
| **Lenient** | Unknown data names are warnings, some type coercions allowed |
| **Pedantic** | Include stylistic warnings (e.g., deprecated items) |

---

## Error Categories

| Category | Description |
|----------|-------------|
| UnknownDataName | Data name not found in dictionary |
| TypeError | Value doesn't match expected type |
| RangeError | Numeric value outside allowed range |
| EnumerationError | Value not in allowed set |
| MissingMandatory | Required item missing from block |
| LoopStructure | Invalid loop structure |
| LinkError | Foreign key reference error |

All errors include:
- **message** - Human-readable description
- **span** - Source location (line, column)
- **suggestions** - "Did you mean...?" hints

---

## dREL Parser

The `drel-parser` crate parses dREL methods for dictionary validation (not runtime execution).

### Grammar

Based on the [COMCIFS annotated grammar](https://github.com/COMCIFS/dREL/blob/master/annotated-grammar.rst):

- **Literals**: Integer, Float, Imaginary, String, Null, Missing
- **Data names**: `_category.object` pattern
- **Operators**: Arithmetic (`+`, `-`, `*`, `/`, `**`), comparison, logical
- **Statements**: If/ElseIf/Else, For, Loop, Do, Repeat, With

### Usage

```rust
use drel_parser::{parse, extract_references, build_dependency_graph};

// Parse dREL method
let stmts = parse("_crystal.density = _cell.atomic_mass / _cell.volume")?;

// Extract item references
let refs = extract_references(&stmts);
for r in &refs {
    println!("{} at {}", r.full_name(), r.span);
}

// Build dependency graph
let graph = build_dependency_graph("_crystal.density", &stmts);
if let Some(cycle) = graph.find_cycle() {
    println!("Circular dependency: {:?}", cycle);
}
```

### CIF-Specific Loop Statement

```drel
Loop t as atom_type {
    _cell.atomic_mass += t.number_in_cell * t.atomic_mass
}
```

Iterates over category packets—unique to dREL.

---

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| DDL support | DDLm only | Modern standard, recommended by IUCr since 2014 |
| Dictionary bundling | Runtime only | User provides path; smaller binary, more flexible |
| dREL handling | Parse for validation only | dREL documents relationships; not executed at runtime |
| Multi-dictionary | Merge with override | Later dictionaries override earlier (for extensions) |

---

## Alias Support

Legacy CIF 1.0 names must map to DDLm names:

```
_diffrn_ambient_pressure  →  _diffrn.ambient_pressure
_atom_site_fract_x        →  _atom_site.fract_x
```

Dictionaries provide these mappings via `_alias.definition_id`. The validator resolves aliases automatically.

---

## References

- [DDLm Specification](https://www.iucr.org/resources/cif/ddl/ddlm) - IUCr DDLm documentation
- [dREL Paper](https://pubs.acs.org/doi/10.1021/ci300076w) - J. Chem. Inf. Model. 2012
- [CIF Core Dictionary](https://github.com/COMCIFS/cif_core) - Official cif_core.dic
- [COMCIFS GitHub](https://github.com/COMCIFS) - All CIF-related standards
- [DDLm Paper](https://doi.org/10.1021/ci300075z) - DDLm specification paper
