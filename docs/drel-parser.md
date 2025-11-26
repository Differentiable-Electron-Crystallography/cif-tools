# drel-parser

A parser for dREL (dictionary Relational Expression Language), the methods language used in DDLm dictionaries for expressing data relationships in CIF.

## Overview

dREL is designed to express complex data relationships in a simple, canonical form that is readable by non-programmers and can be machine-parsed for validation and analysis. This crate provides **parse-only** functionality - it does not evaluate dREL expressions.

### Design Philosophy

From the [original dREL paper](https://pubs.acs.org/doi/10.1021/ci300076w) (Spadaccini et al., 2012):

- dREL provides **reference implementations** for validation, not optimized computation
- Expressions are canonical and reverse-translatable to typographical formulas
- Each script is a **pure function** with no side effects
- Missing values trigger recursive method execution

### Why Parse-Only?

For validation purposes, we need to:
1. Extract references to understand what data items a method depends on
2. Build dependency graphs to detect cycles and determine evaluation order
3. Validate that referenced items exist in the dictionary

We do **not** need to evaluate dREL because:
- Dictionary methods are reference implementations, not production code
- Actual CIF software typically implements optimized versions
- Validation focuses on structural correctness, not computed values

## Architecture

```
drel-parser/
├── src/
│   ├── lib.rs              # Public API
│   ├── drel.pest           # PEG grammar
│   ├── error.rs            # Error types
│   ├── ast/                # Abstract Syntax Tree
│   │   ├── mod.rs          # Re-exports
│   │   ├── span.rs         # Source location tracking
│   │   ├── expr.rs         # Expression types (Expr, ExprKind)
│   │   ├── stmt.rs         # Statement types (Stmt, StmtKind)
│   │   └── operator.rs     # Operators
│   ├── parser/             # PEST → AST conversion
│   │   ├── expr.rs         # Expression parsing
│   │   ├── stmt.rs         # Statement parsing
│   │   └── helpers.rs      # Utilities (span extraction)
│   └── analysis/           # Static analysis
│       ├── references.rs   # Reference extraction
│       └── dependencies.rs # Dependency graphs
```

## Grammar

The grammar is based on the [COMCIFS annotated grammar](https://github.com/COMCIFS/dREL/blob/master/annotated-grammar.rst) and implemented using [PEST](https://pest.rs/).

### Literals

```pest
integer     = @{ hex_integer | octal_integer | binary_integer | decimal_integer }
float       = @{ digits "." digits? exponent? | ... }
imaginary   = @{ (float | integer) ~ ("j" | "J") }
string      = @{ triple_quoted | single_quoted }
null_literal    = { ^"Null" }
missing_literal = { ^"Missing" }
```

### Data Names

CIF data names follow the pattern `_category.object`:

```pest
data_name = @{ "_" ~ identifier ~ "." ~ identifier }
```

Examples: `_cell.length_a`, `_atom_site.fract_x`, `_diffrn_radiation.probe`

### Operators

**Arithmetic** (by precedence):
- Power: `**`
- Multiply/Divide/Cross: `*`, `/`, `^`
- Add/Subtract: `+`, `-`

**Comparison**: `==`, `!=`, `<`, `>`, `<=`, `>=`, `in`, `not in`

**Logical**: `and`/`&&`, `or`/`||`, `not`/`!`

**Assignment**: `=`, `+=`, `-=`, `*=`, `++=` (append), `--=` (prepend)

### Expressions

Expressions follow standard precedence rules:

```
expression = or_expr
or_expr    = and_expr (or_op and_expr)*
and_expr   = not_expr (and_op not_expr)*
not_expr   = not_op? comparison
comparison = add_expr (comp_op add_expr)?
add_expr   = mul_expr (add_op mul_expr)*
mul_expr   = power_expr (mul_op power_expr)*
power_expr = unary_expr (power_op unary_expr)*
unary_expr = unary_op? postfix_expr
postfix_expr = primary (subscription | attribute_ref | call)*
```

### Statements

**Control Flow**:
```drel
If (condition) { ... }
ElseIf (condition) { ... }
Else { ... }
```

**Loops**:
```drel
For x in list { ... }
Do i = 1, 10, 2 { ... }
Repeat { ... Break }
```

**CIF-Specific Loop** (iterates over category packets):
```drel
Loop a as atom_site {
    total += a.occupancy
}

Loop t as atom_type : idx Where (t.symbol == "C") {
    count += 1
}
```

**With Statement** (local binding or category alias):
```drel
With x = expensive_calculation() { ... }
With c as cell    # Alias persists for rest of method
```

## Usage

### Basic Parsing

```rust
use drel_parser::{parse, parse_expr};

// Parse a full program (multiple statements)
let stmts = parse(r#"
    Loop t as atom_type {
        _cell.atomic_mass += t.number_in_cell * t.atomic_mass
    }
"#)?;

// Parse a single expression
let expr = parse_expr("_cell.length_a * _cell.length_b")?;
```

### Reference Extraction

Extract all data item references from parsed code:

```rust
use drel_parser::{parse, extract_references};

let stmts = parse("_crystal.density = _cell.atomic_mass / _cell.volume")?;
let refs = extract_references(&stmts);

for r in &refs {
    println!("{} at {}: {:?}", r.full_name(), r.span, r.kind);
}
// Output:
// _crystal.density at 1:1: DataName
// _cell.atomic_mass at 1:22: DataName
// _cell.volume at 1:43: DataName
```

Reference kinds:
- `DataName` - Full data name reference (`_category.object`)
- `Category` - Category reference in Loop/With statements
- `Identifier` - Unresolved identifier (could be builtin function or category)

### Dependency Graphs

Build and analyze dependency graphs:

```rust
use drel_parser::{parse, build_dependency_graph};

let stmts = parse("_crystal.density = _cell.atomic_mass / _cell.volume")?;
let graph = build_dependency_graph("_crystal.density", &stmts);

// What does _crystal.density depend on?
if let Some(deps) = graph.get_dependencies("_crystal.density") {
    for dep in deps {
        println!("  depends on: {}", dep);
    }
}

// Check for cycles with source locations
if let Some((cycle, spans)) = graph.find_cycle_with_spans() {
    println!("Circular dependency detected:");
    for (i, item) in cycle.iter().enumerate() {
        if i < spans.len() {
            println!("  {} (at {})", item, spans[i]);
        }
    }
}

// Get evaluation order
let order = graph.topological_sort()?;
```

## Source Location Tracking

All AST nodes carry source location information via the `Span` type. This enables:
- Precise error messages with line/column numbers
- IDE features like go-to-definition
- Dependency cycle reporting with locations

### Span Type

```rust
pub struct Span {
    pub start_line: usize,  // 1-indexed
    pub start_col: usize,   // 1-indexed
    pub end_line: usize,
    pub end_col: usize,
}

impl Span {
    pub fn new(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Self;
    pub fn point(line: usize, col: usize) -> Self;  // Single point
    pub fn merge(self, other: Span) -> Self;        // Combine spans
    pub fn contains(&self, line: usize, col: usize) -> bool;
}

// Display formats
// Single point: "1:5"
// Same line: "1:5-10"
// Multi-line: "1:5-3:10"
```

### Accessing Spans

```rust
use drel_parser::{parse_expr, ExprKind};

let expr = parse_expr("_cell.volume")?;

// Every expression has a span
println!("Expression spans: {}", expr.span);  // e.g., "1:1-1:12"

// Access the expression kind
match &expr.kind {
    ExprKind::DataName { category, object } => {
        println!("Data name: _{}.{}", category, object);
    }
    _ => {}
}
```

## AST Types

All AST nodes use a struct wrapper pattern to separate the node kind from metadata (span).

### Expressions

```rust
/// Expression with source location
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

/// Expression variants
pub enum ExprKind {
    // Literals
    Integer(i64),
    Float(f64),
    Imaginary { value: f64 },
    String(String),
    Null,
    Missing,

    // References
    Identifier(String),
    DataName { category: String, object: String },

    // Operations
    BinaryOp { left: Box<Expr>, op: BinaryOperator, right: Box<Expr> },
    UnaryOp { op: UnaryOperator, operand: Box<Expr> },

    // Postfix
    Subscription { target: Box<Expr>, subscripts: Vec<Subscript> },
    AttributeRef { target: Box<Expr>, attribute: String },
    FunctionCall { function: Box<Expr>, args: Vec<Expr> },

    // Containers
    List(Vec<Expr>),
    Table(Vec<(String, Expr)>),
}
```

### Statements

```rust
/// Statement with source location
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

/// Statement variants
pub enum StmtKind {
    Assignment { target: Expr, op: AssignOp, value: Expr },
    If { condition: Expr, then_block: Vec<Stmt>, elseif_blocks: Vec<(Expr, Vec<Stmt>)>, else_block: Option<Vec<Stmt>> },
    For { var: String, iterable: Expr, body: Vec<Stmt> },
    Loop { var: String, category: String, index_var: Option<String>, condition: Option<Expr>, body: Vec<Stmt> },
    Do { var: String, start: Expr, end: Expr, step: Option<Expr>, body: Vec<Stmt> },
    Repeat { body: Vec<Stmt> },
    With { var: String, value: Expr, body: Vec<Stmt> },
    FunctionDef { name: String, params: Vec<String>, body: Vec<Stmt> },
    Break,
    Next,
    Expr(Expr),
}
```

### Item References

```rust
/// A reference to a CIF item found in dREL code
pub struct ItemReference {
    pub kind: ReferenceKind,
    pub category: String,
    pub object: Option<String>,
    pub span: Span,  // Source location
}

pub enum ReferenceKind {
    DataName,   // _category.object
    Category,   // Category in Loop/With
    Identifier, // Unresolved name
}
```

## Real-World Examples

### Cell Atomic Mass (from cif_core.dic)

```drel
mass = 0.
Loop t as atom_type {
    mass += t.number_in_cell * t.atomic_mass
}
_cell.atomic_mass = mass
```

### Matrix Construction

```drel
With c as cell
_cell.convert_Uij_to_betaij = 1.4142 * Pi * Matrix([
    [ c.reciprocal_length_a, 0, 0 ],
    [ 0, c.reciprocal_length_b, 0 ],
    [ 0, 0, c.reciprocal_length_c ]
])
```

### Conditional Logic

```drel
If (_diffrn_radiation.probe == "neutron") {
    _units.code = "neutrons_per_millimetre_squared_per_second"
}
ElseIf (_diffrn_radiation.probe == "electron") {
    _units.code = "electrons_per_angstrom_squared_per_second"
}
Else {
    _units.code = "photons_per_millimetre_squared_per_second"
}
```

## Three Types of dREL Methods

1. **Evaluation methods** - Compute derived values from other data items
2. **Definition methods** - Tailor definitions based on instance data
3. **Validation methods** - Boolean consistency tests

## References

- [dREL Paper](https://pubs.acs.org/doi/10.1021/ci300076w) - Spadaccini et al., J. Chem. Inf. Model. 2012
- [COMCIFS dREL Grammar](https://github.com/COMCIFS/dREL/blob/master/annotated-grammar.rst)
- [DDLm Dictionary Language](https://github.com/COMCIFS/cif_core/blob/master/ddl.dic)
