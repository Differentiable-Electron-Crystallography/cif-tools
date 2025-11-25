# CIF Parser Grammar Behavior Notes

## Overview

This document describes the current behavior of the CIF parser grammar (`src/cif.pest`) and notes areas where the grammar is intentionally permissive. The test suite documents this **actual behavior**, maintaining fidelity to the grammar as implemented.

## Permissive Behavior

The CIF 1.1 grammar implementation has several permissive behaviors that accept malformed input without raising errors:

### 1. Optional Content in Files

**Rule:** `file = { SOI ~ whitespace? ~ (!EOI ~ content ~ EOI)? }`

**Behavior:** The `?` makes content optional. When parsing fails partway through a data block, the parser returns an empty document instead of raising an error.

**Example:**
```cif
data_test
loop_
_tag1
# Missing loop values - parsing fails here
```

**Result:** Returns empty `Document` (0 blocks) instead of `ValueError`

**Impact:**
- Silent failures - users don't get error messages for malformed CIF
- Difficult to debug - no indication of what went wrong
- May mask data loss

**Why It's This Way:** The grammar prioritizes robustness over strictness. An empty document indicates "nothing parseable" rather than "syntax error".

### 2. Empty Loops Allowed

**Rule:** `loop_block = { str_loop ~ whitespace ~ (loop_tag ~ whitespace)+ ~ (loop_values | &(keyword | EOI)) ~ loop_end }`

**Behavior:** The `| &(keyword | EOI)` alternative allows loops to have tags but no values.

**Example:**
```cif
data_test
loop_
_tag1
_tag2
# No values provided
data_next_block
```

**Result:** Parses successfully, creating a loop with 2 tags and 0 rows

**Impact:**
- Accepts incomplete loops that may represent data entry errors
- CIF files can have structurally valid but semantically empty loops

**Why It's This Way:** Allows graceful handling when encountering the next keyword (another data block or save frame) immediately after loop tags.

### 3. Unclosed Quotes Treated as Unquoted Strings

**Rule:** `unquoted = { !keyword ~ !(^"_" | "$" | "#") ~ nonblank_ch+ }`

**Behavior:** Quote characters (`'` and `"`) are not excluded from the start of unquoted strings.

**Example:**
```cif
data_test
_item 'unclosed quote
```

**Result:** Parses successfully, value is `'unclosed` (includes the quote character)

**Impact:**
- Unclosed quoted strings don't raise errors
- The opening quote becomes part of the value
- Users may not realize their quotes are malformed

**Why It's This Way:** The `nonblank_ch` rule allows any printable non-blank character, including quotes. The quote rules only match complete quoted strings, so a failed quote match falls through to unquoted.

### 4. Loop Without Tags

**Rule:** Loop requires at least one tag: `(loop_tag ~ whitespace)+`

**Behavior:** If `loop_` keyword appears but no valid tags follow, parsing fails and returns empty document.

**Example:**
```cif
data_test
loop_
value1 value2 value3
```

**Result:** Returns empty `Document` (0 blocks) - same as case #1

**Impact:** Same as optional content - silent failure

**Why It's This Way:** The tags requirement is strict, but the optional content rule means failure results in empty document.

## Comparison: Permissive vs Strict

| Scenario | Current (Permissive) | Strict Alternative |
|----------|---------------------|-------------------|
| Unclosed quote | Parses as unquoted string with quote char | `ValueError`: "Unclosed quote at line X" |
| Loop without values | Creates empty loop | `ValueError`: "Loop has no data rows" |
| Incomplete data block | Returns empty document | `ValueError`: "Unexpected end of block" |
| Loop without tags | Returns empty document | `ValueError`: "Loop keyword without tags" |

## Trade-offs

### Advantages of Permissive Grammar

1. **Robustness**: Parser doesn't crash on slightly malformed input
2. **Partial recovery**: Can extract valid blocks even if others fail
3. **Flexibility**: Accepts variations in CIF formatting
4. **Backward compatibility**: May handle older/non-conformant files

### Disadvantages of Permissive Grammar

1. **Silent failures**: Errors don't produce helpful messages
2. **Data quality**: Accepts malformed data that may be incorrect
3. **Debugging difficulty**: Users don't know what's wrong
4. **Unexpected behavior**: Quote characters in unquoted strings is confusing

## Future Considerations

If strictness becomes a priority, consider:

### Option 1: Post-Parse Validation

Add validation after successful parse:
```rust
pub fn parse_strict(input: &str) -> Result<CifDocument, CifError> {
    let doc = parse(input)?;
    validate_document(&doc)?; // Check for empty loops, etc.
    Ok(doc)
}
```

**Pros:**
- Keeps grammar unchanged
- Easy to make optional (strict mode flag)
- Can provide custom error messages

**Cons:**
- Can't provide precise line/column information
- Validation logic separate from grammar

### Option 2: Stricter Grammar Rules

Modify grammar rules:
1. Remove `?` from file content: `file = { SOI ~ whitespace? ~ content ~ whitespace? ~ EOI }`
2. Require loop values: `loop_block = { str_loop ~ whitespace ~ (loop_tag ~ whitespace)+ ~ loop_values ~ loop_end }`
3. Exclude quotes from unquoted: `unquoted = { !keyword ~ !(^"_" | "$" | "#" | "'" | "\"" | ";") ~ nonblank_ch+ }`

**Pros:**
- Precise error messages with line/column from Pest
- Fails fast at parse time
- Grammar rules match CIF spec more closely

**Cons:**
- Breaking change for existing users
- Less robust to variations
- May reject some real-world CIF files

### Option 3: Configurable Strictness

Provide both permissive and strict parsers:
```rust
pub fn parse(input: &str) -> Result<CifDocument, CifError>  // Permissive
pub fn parse_strict(input: &str) -> Result<CifDocument, CifError>  // Strict
```

**Pros:**
- Best of both worlds
- Users can choose based on their needs
- Gradual migration path

**Cons:**
- Maintenance burden of two grammars/parsers
- Complexity in codebase

## Current Approach: Document Actual Behavior

For now, the test suite **documents the permissive behavior as-is**:

- Tests verify what the grammar **actually does**
- Docstrings explain **why** it behaves this way
- This file provides **context** for future decisions

This maintains **fidelity to the grammar** while preparing for potential future changes.

## Testing Philosophy

The test suite follows these principles:

1. **Document, don't prescribe**: Tests show what happens, not what "should" happen
2. **Explain behavior**: Docstrings explain permissive behavior
3. **Reference this file**: Tests link to `docs/grammar-notes.md` for context
4. **Enable future changes**: Clear documentation makes it easier to modify behavior later

## References

- CIF 1.1 Specification: https://www.iucr.org/resources/cif/spec/version1.1/cifsyntax
- Grammar implementation: `src/cif.pest`
- Test suite: `python/tests/test_errors.py`, `python/tests/test_document.py`
