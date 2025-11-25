# How the AST is Built Systematically

The CIF parser builds the Abstract Syntax Tree (AST) through a **two-stage architecture** that cleanly separates grammar parsing from AST construction.

## Two-Stage Architecture

```
Input String → [PEST Parser] → Parse Tree → [AST Builder] → Typed AST
```

## Stage 1: Grammar Parsing (PEST)

**Input:** Raw CIF string
**Output:** Parse tree (`Pair<Rule>`)
**Location:** `src/cif.pest`

PEST uses the PEG grammar defined in `cif.pest` to parse the input string into a generic parse tree. The grammar defines 40+ rules including:

- **Character sets**: `ordinary_char`, `nonblank_ch`, `anyprint_ch`
- **Keywords**: `data_`, `loop_`, `global_`, `save_` (case-insensitive)
- **Values**: `singlequoted`, `doublequoted`, `textfield`, `unquoted`
- **Structures**: `datablock`, `loop_block`, `frame`, `dataitem`

The PEST parser produces a tree of `Pair<Rule>` nodes, where each node has:
- A rule type (e.g., `Rule::datablock`, `Rule::loop_tag`)
- A string span (source text + location info)
- Child nodes

## Stage 2: AST Construction

**Input:** Parse tree from PEST
**Output:** Typed AST structures
**Location:** `src/parser/` modules

### Entry Point: Document Parsing

**File:** `src/parser/document.rs:22-48`

```rust
parse_file(input: &str) -> Result<CifDocument, CifError>
```

**Process:**
1. Calls `CIFParser::parse(Rule::file, input)` to get PEST parse tree
2. Creates empty `CifDocument`
3. Walks parse tree looking for `Rule::file` → `Rule::content` → `Rule::datablock`
4. For each datablock, calls `parse_datablock()` and adds to document

### Data Block Parsing

**File:** `src/parser/block.rs:12-40`

```rust
parse_datablock(pair: Pair<Rule>) -> Result<CifBlock, CifError>
```

**Process:**
1. Creates `BlockBuilder` (state management helper)
2. Iterates through child nodes:
   - **`Rule::datablockheading`** → Extract block name (e.g., `data_protein` → `"protein"`)
   - **`Rule::dataitem`** → Parse tag-value pair via `parse_dataitem()`
   - **`Rule::loop_block`** → Parse loop via `parse_loop()`
   - **`Rule::frame`** → Parse save frame via `parse_frame()`
3. Returns `builder.finish()` which finalizes any pending loop

**Key feature:** Case-insensitive keyword parsing while preserving name case:
- `DATA_MyProtein` → name is `"MyProtein"`
- `data_MyProtein` → name is `"MyProtein"`

### Data Item Parsing

**File:** `src/parser/block.rs:42-67`

```rust
parse_dataitem(pair: Pair<Rule>) -> Result<(String, CifValue), CifError>
```

**Process:**
1. Extracts location info for error reporting: `extract_location(&pair)`
2. Collects child nodes into vector
3. Finds tag node (`Rule::item_tag` or `Rule::tag`)
4. Finds value node (`Rule::item_value` or `Rule::value`)
5. Extracts tag string: `extract_text(tag_pair)`
6. Parses value: `CifValue::parse_value(value_str)`
7. Returns `(tag, value)` tuple

**Error handling:** If tag is missing, returns `CifError` with precise line/column location

### Value Parsing

**File:** `src/ast/value.rs:73-86`

```rust
CifValue::parse_value(s: &str) -> CifValue
```

**Systematic value type detection:**

1. **Check special values:**
   - `"?"` → `CifValue::Unknown`
   - `"."` → `CifValue::NotApplicable`

2. **Extract content** (remove delimiters):
   - Single/double quotes: `'text'` → `text`
   - Text fields: `;text\n;` → `text` (trimmed)
   - Unquoted: kept as-is

3. **Type detection:**
   - Try `parse::<f64>()` → success = `CifValue::Numeric(f64)`
   - Fall back → `CifValue::Text(String)`

### Loop Parsing

**File:** `src/parser/loop_parser.rs:26-68`

```rust
parse_loop(pair: Pair<Rule>) -> Result<CifLoop, CifError>
```

**Process:**

1. **Extract tags:**
   - Filter for `Rule::loop_tag` or `Rule::tag` nodes
   - Validate at least one tag exists (error if none)
   - Extract tag strings into `Vec<String>`

2. **Collect values:**
   - Process `Rule::loop_values` → recurse into values
   - Process `Rule::loop_value` or `Rule::value` → parse value
   - Store in flat `Vec<CifValue>`

3. **Organize into rows:**
   - Validate `values.len() % tags.len() == 0` (complete rows)
   - Use `values.chunks(tag_count)` to split into rows
   - Each row = `Vec<CifValue>` with one value per tag

**Example:**
```text
loop_
_col1 _col2 _col3     # 3 tags
v1 v2 v3 v4 v5 v6     # 6 values

Result:
Row 0: [v1, v2, v3]
Row 1: [v4, v5, v6]
```

### Save Frame Parsing

**File:** `src/parser/block.rs:69-105`

```rust
parse_frame(pair: Pair<Rule>) -> Result<CifFrame, CifError>
```

Similar to block parsing but simpler (no nested frames):

**Process:**
1. Extract frame name from `Rule::framename`
2. Create `CifFrame::new(name)`
3. Iterate child nodes:
   - `Rule::dataitem` → Parse and add to `frame.items`
   - `Rule::loop_block` → Parse and add to `frame.loops`
4. Return completed frame

## State Management: The BlockBuilder

**File:** `src/builder.rs:72-123`

### The Problem: Loop Interruption

CIF allows loops to be interrupted by other elements:

```text
loop_
_atom.id _atom.type
_other_item other_value    # Interrupts the loop!
1 C
2 N
```

### The Solution: Pending Loop State

`BlockBuilder` manages "pending loop" state:

```rust
struct BlockBuilder {
    block: CifBlock,
    pending_loop: Option<CifLoop>,  // Current incomplete loop
}
```

**Operations:**
- `add_item()` → Finalize pending loop, then add item
- `start_loop()` → Finalize pending loop, then start new one
- `add_frame()` → Finalize pending loop, then add frame
- `finish()` → Finalize any remaining pending loop

**Automatic finalization** ensures no loops are lost or incorrectly mixed with other data.

## Error Handling & Location Tracking

Throughout parsing, the code preserves **span information** from PEST nodes:

```rust
extract_location(&pair) -> (usize, usize)  // (line, column)
```

When errors occur, they include precise source locations:

```rust
CifError::invalid_structure("Loop has no tags")
    .at_location(line, column)
```

This enables error messages like:
```
Error at line 42, column 5: Loop block has no tags
```

## Summary: The Complete Flow

```
Input String
    ↓
[PEST Grammar Parser] (cif.pest)
    ↓
Generic Parse Tree (Pair<Rule>)
    ↓
[Document Parser] (parser/document.rs)
    ↓
[Block Parser] (parser/block.rs) ← uses BlockBuilder
    ├→ [Data Item Parser]
    ├→ [Loop Parser] (parser/loop_parser.rs)
    └→ [Frame Parser]
        ↓
[Value Parser] (ast/value.rs)
    ↓
Typed AST Structures
    └→ CifDocument
        └→ CifBlock
            ├→ items: HashMap<String, CifValue>
            ├→ loops: Vec<CifLoop>
            └→ frames: Vec<CifFrame>
```

## Key Design Principles

1. **Decoupling**: AST types don't know how to parse themselves
2. **Single Responsibility**: Each parser handles one AST type
3. **Error Recovery**: Precise location tracking for all errors
4. **State Management**: BlockBuilder handles complex loop interruption cases
5. **Type Safety**: Strong typing throughout (no `Any` or untyped nodes)

This architecture makes the parser easy to test, maintain, and extend while handling all the edge cases in the CIF specification.

## Related Documentation

- [CIF 1.1 Specification](https://www.iucr.org/resources/cif/spec/version1.1/cifsyntax)
- [PEST Parser Documentation](https://pest.rs/)
- API Documentation: Run `cargo doc --open` to view detailed API docs
