# CIF Dictionaries and DDL (Dictionary Definition Language)

## Overview

CIF dictionaries are formal specifications that define the valid data names, types, relationships, and constraints for CIF files. They are themselves written in CIF syntax, creating a meta-circular, self-defining system where the language describes itself.

### Key Concepts

- **CIF Data Files**: Contain crystallographic data (e.g., atom coordinates, cell parameters)
- **CIF Dictionary Files (.dic)**: Define what makes a valid CIF data file
- **DDL (Dictionary Definition Language)**: The vocabulary used within dictionary files to describe data items
- **Meta-circularity**: Dictionary files are valid CIF documents that use CIF syntax to define CIF semantics

### Separation of Concerns

This parser is **dictionary-agnostic**:
- ✅ **Syntax Parsing** (this codebase): Understands CIF structure (blocks, frames, tags, values)
- ❌ **Semantic Validation** (not implemented): Validates data against dictionary rules

This design makes the parser universal - it works with any CIF dictionary (core, mmCIF, powder, etc.) without needing to understand domain-specific semantics.

---

## The Meta-Circular Self-Defining System

### How It Works

CIF dictionaries demonstrate a beautiful example of self-reference in formal systems:

```
CIF Syntax (STAR files)
    ↓ Used to write
DDLm Dictionary (written in CIF 2.0)
    ↓ Defines semantics of
Valid CIF Data Items
    ↓ Which are expressed in
CIF Syntax
```

### The Bootstrap Process

**Stage 0: Primitive Parser** (this codebase)
- Understands CIF syntax: data blocks, save frames, tags, values
- No semantic understanding required
- Can parse any CIF file, including dictionaries

**Stage 1: Parse the Dictionary**
- Use syntax parser to read `cif_core.dic` or other `.dic` files
- Extract save frames containing definitions
- Each save frame describes a data item or category

**Stage 2: Build Validation Rules** (not in this codebase)
- Process parsed dictionary to create validation logic
- Tags like `_type.contents`, `_enumeration.range`, `_units.code` become rules
- Enable semantic validation of user CIF files

**Key Insight**: You only need syntactic understanding to parse dictionaries, then build semantic understanding from their contents.

### Self-Referential Example

From `cif_core.dic`:

```cif
save_definition.id
    _definition.id         '_definition.id'      # ← Defines itself!
    _name.category_id      definition
    _type.contents         Text
    _description.text
;
    Unique identifier for this definition in the dictionary.
;
save_
```

The tag `_definition.id` is used within save frames to define what tags are valid, including `_definition.id` itself. This creates an elegant circularity where CIF syntax defines dictionary structure, and dictionaries define valid CIF semantics.

### Comparison to Other Meta-Circular Systems

**Similar to:**
- **Lisp interpreters written in Lisp** - `eval` defined using `eval`
- **BNF grammars defined in BNF** - grammar notation defined using itself
- **C compilers written in C** - compiled with earlier C compiler

**Resolution strategy:**
The apparent paradox is resolved through **staged bootstrap**: syntax parsing (understanding structure) enables dictionary loading, which enables semantic validation (understanding meaning).

---

## DDL Versions

### Overview

| Version | Status | CIF Version | First Published | Current Use |
|---------|--------|-------------|-----------------|-------------|
| **DDL1** | Deprecated (2014) | CIF 1.1 | 1991 | Legacy small-molecule dictionaries |
| **DDL2** | Maintained | CIF 1.1 | 1997 | mmCIF/PDBx (macromolecular) |
| **DDLm** | Current (v4.2.0) | CIF 2.0 | 2012 | All modern IUCr dictionaries |

### DDL1 (Original, Deprecated)

**Designed for:** Small-molecule crystallography

**Features:**
- Simple data typing (character strings or numeric values)
- Basic constraint definitions
- Flat structure without complex relationships

**Status:** Formally deprecated August 2014, but still in use due to widespread adoption

**Used by:**
- Original core CIF dictionary
- Powder diffraction dictionary
- Modulated structures dictionary
- Precision density dictionary

**Legacy versions available at:** https://github.com/COMCIFS/DDL1-legacy-dictionaries

### DDL2 (Relational Model)

**Designed for:** Macromolecular crystallography (proteins, nucleic acids)

**Features:**
- Relational database model with parent-child relationships
- Category hierarchies
- Can express complex data structures needed for biological macromolecules

**Status:** Still maintained, actively used for mmCIF

**Used by:**
- mmCIF (Macromolecular CIF) / PDBx dictionary
- Image dictionary
- Symmetry dictionary

**Key difference from DDL1:** DDL2's relational model allows expressing that "atom_site is a category that contains multiple atoms, each with coordinates, labels, and properties."

### DDLm (Methods, Current Standard)

**Current version:** 4.2.0 (as of July 2025)

**Designed for:** Universal DDL that subsumes both DDL1 and DDL2 capabilities

**Major features:**
- **Rich type system**: Nearly 20 data types
  - Integer, Real, Complex, Text, Code
  - Uri, DateTime, Version, SymOp
  - Imag (imaginary number component)
- **Complex data structures**: Lists, Matrices, Tables
- **Embedded dREL methods**: Dictionary-level validation and calculation functions
- **Mandatory uncertainty values**: Standard uncertainties for measurands
- **Unicode support**: Full UTF-8 character set
- **Enhanced constraints**: Sophisticated enumeration and range specifications

**Status:** Active development, current standard for all new IUCr dictionaries

**Used by:**
- Current `cif_core.dic` (v3.3.0)
- Restraints dictionary (`cif_rstr.dic`)
- Most modern IUCr dictionaries

**Key advancement:** DDLm includes dREL (dictionary RELational language), which allows embedding computational methods directly in the dictionary. For example, a dictionary can define that `_cell.volume` is calculated as `a * b * c * sqrt(1 - cos²α - cos²β - cos²γ + 2cosα·cosβ·cosγ)`.

---

## DDL and CIF Version Compatibility

### Version Mapping

```
CIF 1.1 (2006)  ←→  DDL1, DDL2
CIF 2.0 (2016)  ←→  DDLm
```

### Cross-Compatibility

The relationship is **not strictly one-to-one**. Validators can handle:
- CIF 1.1 files validated against DDL1 dictionaries ✅
- CIF 1.1 files validated against DDLm dictionaries ✅
- CIF 2.0 files validated against DDLm dictionaries ✅
- CIF 2.0 files validated against DDL1 dictionaries ⚠️ (limited, CIF 2.0 features won't validate)

### Why DDLm Requires CIF 2.0 Syntax

DDLm dictionaries themselves must be written in CIF 2.0 because they use:
- Complex data types (Lists, Tables)
- Unicode characters
- Embedded dREL methods
- Enhanced text field handling

However, DDLm dictionaries can validate both CIF 1.1 and CIF 2.0 data files.

---

## The Meta-Dictionary: ddl.dic

### What Is It?

`ddl.dic` is the "dictionary of dictionaries" - a DDLm dictionary that defines the DDLm language itself.

From the official documentation:
> "This dictionary contains the definitions of attributes that make up the DDLm dictionary definition language. It provides the **meta meta data** for all CIF dictionaries."

### The Complete Meta-Circular Loop

```
ddl.dic (written in CIF 2.0)
    ↓ defines attributes like
_definition.id, _type.contents, _enumeration.range
    ↓ which are used in
cif_core.dic (written using DDLm attributes)
    ↓ which defines tags like
_atom_site.fract_x, _cell.length_a
    ↓ which are used in
user_data.cif (crystallographic data)
```

**Key observation:** `ddl.dic` is itself a DDLm dictionary, so it describes itself using its own attributes. This is the ultimate meta-circularity.

### Example from ddl.dic

```cif
save_type.contents
    _definition.id         '_type.contents'
    _type.contents         Text                  # ← Uses itself to define itself!
    _enumeration.default   Text
    _description.text
;
    The type of this data item (Real, Integer, Text, etc.)
;
save_
```

---

## Example Dictionary Comparison

### cif_core.dic (Core Dictionary)

**Metadata:**
- DDL version: 4.2.0 (DDLm)
- Dictionary version: 3.3.0
- Last updated: 2025-11-05
- Format: CIF 2.0
- Namespace: CifCore

**Purpose:** Core data items common to all crystallographic domains

**Structure example:**
```cif
data_CIF_CORE
    _dictionary.title             CIF_CORE
    _dictionary.class             Instance
    _dictionary.version           3.3.0
    _dictionary.date              2025-11-05

save_atom_site.fract_x
    _definition.id                '_atom_site.fract_x'
    _alias.definition_id          '_atom_site_fract_x'
    _name.category_id             atom_site
    _name.object_id               fract_x
    _type.purpose                 Measurand
    _type.source                  Recorded
    _type.container               Single
    _type.contents                Real
    _enumeration.range            0.0:1.0
    _description.text
;
    Fractional coordinate of atom along the a-axis.
;
save_
```

### cif_rstr.dic (Restraints Dictionary)

**Metadata:**
- DDL version: 3.11.09 (DDLm - older version)
- Dictionary version: 3.1.1
- Last updated: 2024-05-15
- Format: CIF 2.0
- Namespace: CifRstr

**Purpose:** Documents restraints and constraints applied during structure refinement

**Repository:** https://github.com/COMCIFS/Restraints_Dictionary

**Key observation:** Both dictionaries use DDLm (showing it's the universal standard), but different versions, demonstrating the evolution of the DDL specification.

---

## Parser Support in This Codebase

### What This Parser CAN Do

✅ **Parse CIF dictionary files** (.dic files)
```rust
// This works!
let dict = Document::from_file("cif_core.dic")?;
let block = dict.first_block().unwrap();

// Access save frames (definitions)
for frame in &block.frames {
    println!("Definition: {}", frame.name);
    if let Some(def_id) = frame.get_item("_definition.id") {
        println!("  ID: {}", def_id.as_string().unwrap());
    }
}
```

✅ **Full CIF 1.1 support** including:
- Save frames (critical for dictionaries) - `src/cif.pest:232-236`
- All value types (text, numeric, unknown, inapplicable)
- Loops for tabular data
- Multiple data blocks
- Comments and whitespace handling

✅ **Extract dictionary metadata:**
```rust
// Get dictionary version
if let Some(version) = block.get_item("_dictionary.version") {
    println!("Dictionary version: {}", version.as_string().unwrap());
}

// Iterate through all definitions
for frame in &block.frames {
    // Each save frame is a data item or category definition
    // You can extract _type.contents, _enumeration.range, etc.
}
```

### What Would Be Needed for Full DDL Support

❌ **Not implemented** (would be separate projects):

1. **DDLm Semantic Interpreter**
   - Read save frames and build validation rules
   - Parse DDL attributes into structured validation logic

2. **dREL Evaluator**
   - Execute embedded computational methods
   - Calculate derived values
   - Validate complex relationships

3. **Type System Validator**
   - Enforce DDLm data types (Integer, Real, Uri, DateTime, etc.)
   - Check complex types (Lists, Matrices, Tables)
   - Validate standard uncertainties

4. **Constraint Checker**
   - Enforce enumeration ranges
   - Check mandatory vs optional items
   - Validate category relationships

5. **Full Semantic Validation**
   - Validate user CIF files against dictionary rules
   - Generate meaningful error messages
   - Check cross-item dependencies

### Implementation Path

If you wanted to build a full DDL validator:

```rust
// Stage 1: Parse dictionary (CURRENT CAPABILITY)
let dict = Document::from_file("cif_core.dic")?;

// Stage 2: Build validator (NOT IMPLEMENTED)
let validator = DictionaryValidator::from_document(dict)?;

// Stage 3: Validate user files (NOT IMPLEMENTED)
let user_cif = Document::from_file("my_structure.cif")?;
let result = validator.validate(&user_cif)?;

if result.is_valid() {
    println!("CIF file is valid!");
} else {
    for error in result.errors {
        println!("Error: {}", error);
    }
}
```

### Related Code Locations

**Save frame parsing:**
- Grammar: `src/cif.pest:232-237`
- Parser: `src/parser/block.rs:71-108`
- AST: `src/ast/frame.rs`

**Example of parsing dictionaries:**
- See `examples/mmcif_parser.rs` for parsing structured CIF data
- Same techniques apply to dictionary files

---

## Key Resources

### Academic Papers (Essential Reading)

#### The 2012 J. Chem. Inf. Model. Trilogy

These three papers form the complete specification for modern CIF/DDL:

1. **"Extensions to the STAR File Syntax"**
   - Authors: Nick Spadaccini and Sydney R. Hall
   - Citation: *J. Chem. Inf. Model.* 2012, 52(8), 1901-1906
   - DOI: [10.1021/ci300074v](https://doi.org/10.1021/ci300074v)
   - Describes CIF 2.0 enhancements to STAR syntax

2. **"DDLm: A New Dictionary Definition Language"** ⭐ PRIMARY REFERENCE
   - Authors: Nick Spadaccini and Sydney R. Hall
   - Citation: *J. Chem. Inf. Model.* 2012, 52(8), 1907-1916
   - DOI: [10.1021/ci300075z](https://doi.org/10.1021/ci300075z)
   - **The definitive DDLm specification**

3. **"dREL: A Relational Expression Language for Dictionary Methods"**
   - Authors: Nick Spadaccini, Ian R. Castleden, Doug du Boulay, and Sydney R. Hall
   - Citation: *J. Chem. Inf. Model.* 2012, 52(8), 1917-1925
   - DOI: [10.1021/ci300076w](https://doi.org/10.1021/ci300076w)
   - Describes the embedded methods language in DDLm

### Reference Book

**International Tables for Crystallography, Volume G**
- Title: "Definition and Exchange of Crystallographic Data"
- Editors: S. R. Hall and B. McMahon
- Publisher: International Union of Crystallography
- ISBN: 9780470689103 (2nd edition)
- Comprehensive reference for CIF 1.1, DDL1, and DDL2

### Official IUCr Resources

**Main pages:**
- CIF homepage: https://www.iucr.org/resources/cif
- DDL overview: https://www.iucr.org/resources/cif/spec/ddl
- DDL1 spec: https://www.iucr.org/resources/cif/ddl/ddl1
- DDL2 spec: https://www.iucr.org/resources/cif/ddl/ddl2
- DDLm spec: https://www.iucr.org/resources/cif/ddl/ddlm

### COMCIFS GitHub (Living Standards)

**Committee for the Maintenance of the CIF Standard**
- Organization: https://github.com/COMCIFS

**Key repositories:**
- **cif_core** - Core dictionary including `ddl.dic` and `cif_core.dic`
  - https://github.com/COMCIFS/cif_core
  - Also contains `CIF2-EBNF.txt` (CIF 2.0 syntax specification)

- **dREL** - dREL specification and implementations
  - https://github.com/COMCIFS/dREL
  - Includes `annotated_grammar.rst`
  - Reference implementations in JavaScript, Python, Julia

- **Restraints_Dictionary** - Crystallographic restraints
  - https://github.com/COMCIFS/Restraints_Dictionary

- **cif_api** - C API and reference implementation for CIF 2.0
  - https://github.com/COMCIFS/cif_api

**Other domain-specific dictionaries:**
- Powder_Dictionary - Powder diffraction
- imgCIF - Crystallographic images
- TopoCif - Topology
- Modulated_Structures - Modulated and composite structures
- Structure_Prediction_Dictionary - Predicted structures
- MultiBlock_Dictionary - Multi-container data
- cif_ed - Electron diffraction

**Legacy:**
- DDL1-legacy-dictionaries - Archive of deprecated DDL1 dictionaries
  - https://github.com/COMCIFS/DDL1-legacy-dictionaries

### dREL Implementations

- **JavaScript**: JsCifBrowser
- **Python**: PyCIFRW (includes dREL support)
- **Julia**: Implementation in COMCIFS/dREL

---

## Historical Timeline

### Evolution of CIF and DDL

**1991**
- CIF 1.0 introduced
- DDL1 published
- Initial adoption in small-molecule crystallography

**1997**
- mmCIF specification published
- DDL2 introduced for macromolecular structures
- Enables Protein Data Bank (PDB) to adopt mmCIF

**2006**
- CIF 1.1 formalized
- International Tables Volume G published
- DDL1 and DDL2 documented comprehensively

**2011-2012**
- DDLm development
- Three foundational papers published in *J. Chem. Inf. Model.*
- dREL specification completed

**2014**
- DDL1 formally deprecated (August 2014)
- Migration to DDLm begins for all IUCr dictionaries

**2016**
- CIF 2.0 specification officially adopted
- Enhanced syntax for Unicode, complex data types

**2014-2025**
- Ongoing migration of dictionaries from DDL1 to DDLm
- DDL2 maintained for mmCIF but not expanded
- All new dictionaries use DDLm

**Current State (2025)**
- DDLm version: 4.2.0
- cif_core.dic version: 3.3.0
- Universal adoption of DDLm for new dictionary development

---

## Philosophical Notes

### Self-Reference in Formal Systems

CIF dictionaries demonstrate principles explored in:
- **Gödel's Incompleteness Theorems**: Self-reference in formal systems
- **Hofstadter's "Gödel, Escher, Bach"**: Strange loops and tangled hierarchies
- **Metacircular evaluation**: Programs that interpret themselves

### Why Self-Definition Works

The key insight: **Separate syntax from semantics**

1. **Syntax** (structure) is simple and fixed
   - Data blocks, save frames, tags, values
   - Can be parsed without understanding meaning

2. **Semantics** (meaning) is complex and extensible
   - Defined by dictionaries
   - Can evolve without changing syntax

This separation enables:
- **Bootstrapping**: Simple parser → read dictionary → build validator
- **Extensibility**: New domains create new dictionaries using same syntax
- **Versioning**: Dictionaries can evolve independently of parser
- **Self-documentation**: The specification is machine-readable

### The Beauty of Meta-Circularity

> "The dictionary is not a paradox - it's a bootstrap ladder you can climb once you have the first rung (syntax parsing)."

By writing dictionaries in CIF syntax:
- Single format for data and metadata
- Tools that parse data can parse dictionaries
- Dictionaries are versionable, distributable data files
- The entire system is self-describing and machine-processable

This is a profound example of using a simple system (syntax parser) to enable a complex one (semantic validator), where the complex system eventually defines itself through meta-circular reference.

---

## Recommended Reading Order

For someone new to CIF dictionaries and DDL:

1. **This document** - Overview of concepts and architecture
2. **"DDLm: A New Dictionary Definition Language"** (2012 paper, DOI: 10.1021/ci300075z)
   - Best introduction to DDLm philosophy and design
3. **Browse `ddl.dic`** at https://github.com/COMCIFS/cif_core
   - See meta-circularity in action
4. **Browse `cif_core.dic`** at https://github.com/COMCIFS/cif_core
   - See how DDLm is used to define crystallographic data
5. **International Tables Volume G** - Comprehensive reference
6. **"dREL" paper** (2012, DOI: 10.1021/ci300076w) - For embedded methods
7. **Domain-specific dictionaries** - See DDLm applied to different fields

---

## Summary

### Key Takeaways

1. **CIF dictionaries are meta-circular**: They use CIF syntax to define CIF semantics

2. **DDL has three versions**:
   - DDL1 (deprecated) - simple types for small molecules
   - DDL2 (maintained) - relational model for macromolecules
   - DDLm (current) - universal DDL with methods and rich types

3. **This parser provides Stage 1**: Syntax parsing of dictionary files
   - Can read `.dic` files
   - Extract save frames and definitions
   - Foundation for building semantic validators

4. **Semantic validation is Stage 2**: Not implemented in this codebase
   - Would require DDLm interpreter
   - dREL evaluator for embedded methods
   - Type system and constraint validation

5. **DDL and CIF are inseparable**: DDL is not a separate language; it's CIF describing itself

6. **The system is beautifully self-documenting**: Machine-readable specifications enable automatic validation and tooling

### Practical Implications

- This parser can read any CIF dictionary (core, mmCIF, powder, etc.)
- Dictionary files are just CIF files with save frames containing definitions
- Building a semantic validator would be a separate project on top of this parser
- The meta-circular design means one parser works for all dictionaries

### The Elegant Design

> "CIF dictionaries demonstrate that self-definition is possible when you separate syntax from semantics. The parser doesn't need to understand what `_type.contents` means to parse a file that defines `_type.contents`. It only needs to recognize it as a valid tag-value pair."

This separation enables the entire crystallographic data ecosystem to be self-describing, extensible, and machine-processable - a testament to the foresight of the original CIF designers.
