# CIF Parser

A high-performance, multi-dialect parser for the Crystallographic Information File (CIF) format.

## Overview

The CIF parser transforms CIF files into a typed Abstract Syntax Tree (AST) while supporting multiple CIF versions/dialects.

**Key capabilities:**
- Parse CIF 1.1 and CIF 2.0 files with a single grammar
- Preserve source locations (spans) for IDE integration
- Fast - 40ms parse time for 29,000-line files 
- Extensible dialect system for future CIF versions

---

## Architecture: The Two-Pass Design

```
Input String
    │
    ▼
┌─────────────────────────────────┐
│  PEST Grammar (cif.pest)        │  ← Syntactic superset
│  Parses ALL valid CIF syntax    │
└─────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────┐
│  Pass 1: Raw Parser             │
│  Produces dialect-agnostic AST  │
└─────────────────────────────────┘
    │
    ▼
Raw AST (lossless intermediate representation)
    │
    ▼
┌─────────────────────────────────┐
│  Pass 2: Dialect Resolution     │
│  Applies CIF 1.1 or 2.0 rules   │
└─────────────────────────────────┘
    │
    ▼
Typed AST (CifDocument)
```

### Why Two Passes?

The CIF format has multiple versions with different rules. Rather than maintaining separate grammars, we use:

1. **Superset Grammar**: `cif.pest` accepts everything that *could* be valid CIF syntax in any dialect
2. **Raw AST**: A lossless intermediate representation preserving all syntactic information
3. **Dialect Rules**: Version-specific logic that interprets, transforms, or rejects constructs

This separation provides:
- **Single source of truth** for CIF syntax
- **Clean dialect extensibility** without grammar changes
- **Upgrade guidance** by checking one dialect's AST against another's rules

### Syntax vs Semantics

The architecture cleanly separates two kinds of rules:

**Syntactic rules** (in `cif.pest`):
- Is this structurally well-formed?
- Can it be tokenized and parsed?
- Example: Are quotes matched? Is nesting correct?

**Semantic rules** (in dialect modules):
- Is this construct allowed in this dialect?
- How should it be interpreted?
- Example: "Triple-quoted strings are CIF 2.0 only"

---

## Dialect Resolution Strategies

When the dialect resolver encounters a Raw AST node, it applies one of three strategies:

### 1. Pass-through

Use the raw value directly—no dialect-specific handling needed.

```
Text fields     → same in both dialects
Unquoted values → same parsing logic
```

### 2. Transform

Apply dialect-specific interpretation to produce typed values.

| Construct | CIF 1.1 Transform | CIF 2.0 Transform |
|-----------|-------------------|-------------------|
| `'it''s'` | Text: `it''s` (preserve doubled quotes) | — |
| `'''text'''` | Text: `'''text'''` (literal string) | Text: `text` (extract content) |
| `[a, b, c]` | Text: `[a, b, c]` (literal string) | List: [a, b, c] |
| `{"k": v}` | Text: `{"k": v}` (literal string) | Table: {k → v} |

### 3. Reject

Return a violation error for constructs not allowed in this dialect.

| Construct | CIF 1.1 | CIF 2.0 |
|-----------|---------|---------|
| `'it''s'` (doubled quotes) | Allowed | **VIOLATION** — use triple-quotes |
| `data_` (empty name) | Allowed | **VIOLATION** — name required |
| Missing `#\#CIF_2.0` header | N/A | **VIOLATION** — header required |

### Complete Example

Input: `'it''s complicated'`

**CIF 1.1 Resolution:**
```
Raw: QuotedString { raw: "'it''s complicated'", has_doubled_quotes: true }
Strategy: Transform (preserve escapes)
Result: CifValue::Text("it''s complicated")
```

**CIF 2.0 Resolution:**
```
Raw: QuotedString { raw: "'it''s complicated'", has_doubled_quotes: true }
Strategy: Reject
Result: VersionViolation("Doubled-quote escaping not allowed in CIF 2.0")
        Suggestion: "Use triple-quoted strings: '''...'''"
```

### Dialect Asymmetry

CIF 2.0 is **stricter** in some ways but **richer** in others:

| Aspect | CIF 1.1 | CIF 2.0 |
|--------|---------|---------|
| Empty block names | Allowed | Forbidden |
| Doubled-quote escapes | Allowed | Forbidden |
| Triple-quoted strings | Degraded to text | Supported |
| Lists `[...]` | Degraded to text | Supported |
| Tables `{...}` | Degraded to text | Supported |
| Character set | ASCII | Unicode |

The dialect system handles this asymmetry cleanly by allowing each dialect to define its own resolution strategy per construct.

---

## The Raw AST: Dual Representation

A key design insight: Raw AST nodes carry **both** the raw text and parsed structure.

```rust
pub struct RawTableSyntax {
    pub raw_text: String,           // Original: {"key": value}
    pub entries: Vec<RawTableEntry>, // Parsed: [(key, value), ...]
    pub span: Span,
}
```

This enables:
- **CIF 1.1**: Use `raw_text` for degradation → `"{"key": value}"`
- **CIF 2.0**: Use `entries` for transformation → `Table {key: value}`

Without dual representation, CIF 1.1 couldn't gracefully degrade structured syntax to text.

---

## CIF Format Structure

### Hierarchy

```
CifDocument
 └─ CifBlock[]
     ├─ name: String
     ├─ items: HashMap<String, CifValue>
     ├─ loops: Vec<CifLoop>
     └─ frames: Vec<CifFrame>
          ├─ name: String
          ├─ items: HashMap<String, CifValue>
          └─ loops: Vec<CifLoop>
```

### Data Blocks

The primary organizational unit. Identified by `data_name`:

```cif
data_myprotein
_cell.length_a  10.5
_cell.length_b  20.3
```

Keywords are case-insensitive but names preserve case:
- `DATA_MyProtein` → block name is `"MyProtein"`
- `data_MyProtein` → block name is `"MyProtein"`

### Loops (Tabular Data)

```cif
loop_
_atom.id _atom.type _atom.x
1 C 0.0
2 N 1.5
3 O 2.0
```

Structure:
- Tags define columns
- Values fill rows in order
- Row count = total values ÷ tag count

### Values

```rust
pub enum CifValueKind {
    Text(String),
    Numeric(f64),
    NumericWithUncertainty { value: f64, uncertainty: f64 },
    Unknown,        // ?
    NotApplicable,  // .
    List(Vec<CifValue>),           // CIF 2.0
    Table(HashMap<String, CifValue>), // CIF 2.0
}
```

Type inference:
1. `?` → Unknown
2. `.` → NotApplicable
3. Numeric string → Numeric
4. `7.470(6)` → NumericWithUncertainty (value: 7.470, uncertainty: 0.006)
5. Otherwise → Text

### Save Frames

Nested containers within blocks for grouping related definitions:

```cif
data_dictionary
save_atom_site.label
    _definition.id  '_atom_site.label'
    _type.contents  Text
save_
```

---

## Source Location Tracking (Spans)

Every AST node carries a span:

```rust
pub struct Span {
    pub start_line: usize,  // 1-indexed
    pub start_col: usize,   // 1-indexed
    pub end_line: usize,
    pub end_col: usize,     // inclusive
}
```

This enables:
- **Error messages**: "Type error at line 42, col 5"
- **IDE hover**: Look up definition at cursor position
- **Go-to-definition**: Jump from usage to source
- **Syntax highlighting**: Map tokens to source ranges

---

## Performance

### The 1350x Speedup

**Problem**: Parsing a 29,000-line file took 54 seconds.

**Root cause**: PEST's `line_col()` function is O(n)—it counts newlines from the start of the file. With ~1.2 million AST nodes, this was O(n²) total.

**Solution**: Pre-compute a line index once O(n), then use binary search for O(log n) lookups.

```
Before: 54,000 ms (54 seconds)
After:     40 ms

Speedup: 1,350x
```

### How It Works

```rust
struct LineIndex {
    newlines: Vec<usize>,  // Byte positions of all '\n' characters
}

impl LineIndex {
    fn line_col(&self, offset: usize) -> (usize, usize) {
        // Binary search: O(log n) instead of O(n)
        let line = self.newlines.binary_search(&offset);
        // ... calculate column from line start
    }
}
```

### Benchmark Reference

**Hardware**: MacBook Pro (Nov 2023), M3 Max, 48GB RAM

**File**: [`cif_core.dic`](https://github.com/COMCIFS/cif_core) (~29,000 lines)

| Stage | Time |
|-------|------|
| File read | < 1ms |
| PEST parse | ~30ms |
| AST building | ~10ms |
| **Total** | **~40ms** |

---

## Design Decisions

### Permissive Grammar

The grammar is intentionally permissive—it accepts some malformed input rather than failing:

| Scenario | Behavior |
|----------|----------|
| Unclosed quote | Parses as unquoted string with quote char |
| Loop without values | Creates empty loop (0 rows) |
| Incomplete block | Returns empty document |

**Rationale**: Robustness over strictness. Real-world CIF files often have minor issues.

**Trade-off**: Silent failures can mask problems. Consider post-parse validation for strict mode.

### Why PEST?

- PEG grammars map naturally to the CIF specification
- Grammar file (`cif.pest`) serves as executable documentation
- Declarative rules are easier to maintain than hand-written parsers

### Thread-Local Line Index

The line index is stored in thread-local storage:

```rust
thread_local! {
    static LINE_INDEX: RefCell<Option<LineIndex>> = ...;
}
```

**Rationale**: Avoids threading the index through every function while remaining thread-safe. Initialized at parse start, cleared at end.

---

## Extensibility

The dialect architecture supports evolution:

- **New CIF versions**: Add a new dialect module without changing the grammar
- **Software-specific variants**: Formalize quirks as custom dialects
- **Upgrade tooling**: Check files against target dialect, collect violations

---

## API Quick Reference

### Rust

```rust
use cif_parser::{CifDocument, CifVersion};

// Parse with auto-detected dialect
let doc = CifDocument::parse(content)?;

// Parse with specific dialect
let doc = CifDocument::parse_as(content, CifVersion::V1_1)?;

// Access data
let block = doc.first_block().unwrap();
let value = block.get_item("_cell.length_a");
```

### Python

```python
import cif_parser

doc = cif_parser.parse(content)
block = doc.first_block()
value = block.get_item("_cell.length_a")
print(f"Value at line {value.span.start_line}")
```

### JavaScript (WASM)

```javascript
import { parse } from '@anthropic/cif-parser';

const doc = parse(content);
const block = doc.first_block();
const value = block.get_item('_cell.length_a');
```

---

## References

- [CIF 1.1 Specification](https://www.iucr.org/resources/cif/spec/version1.1/cifsyntax)
- [CIF 2.0 Specification](https://www.iucr.org/resources/cif/cif2)
- [PEST Parser Documentation](https://pest.rs/)
- [CIF Core Dictionary](https://github.com/COMCIFS/cif_core)

---

## Paper Breadcrumbs

Key contributions for potential academic publication:

1. **Two-pass architecture with superset grammar**: Single grammar serves multiple format versions through dialect-specific semantic resolution

2. **Three-strategy dialect resolution**: Pass-through, transform, and reject/degrade provide a complete taxonomy for handling version differences

3. **Dual representation in Raw AST**: Preserving both raw text and parsed structure enables graceful degradation

4. **O(n²) → O(n log n) optimization**: Pre-computed line index with binary search achieved 1350x speedup

5. **Practical extensibility**: Architecture supports future dialects without grammar modification
