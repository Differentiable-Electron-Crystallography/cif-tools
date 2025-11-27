# CIF Parser Performance Optimization - COMPLETED

## Problem Statement

The cif-validator integration tests were hanging when parsing `cif_core.dic` (~29,000 lines, 1222 save frames). The parsing took **55 seconds** in release mode, which is unacceptable.

## Solution Summary

**Root Cause**: PEST's `line_col()` function is O(n) - it counts newlines from the start of the file each time it's called. With ~1.2 million parse tree nodes, this resulted in O(n²) total time.

**Fix**: Implemented a `LineIndex` that pre-computes newline positions O(n) once, then uses binary search for O(log n) lookups.

**Result**: Parse time reduced from **54 seconds to 40 milliseconds** - a **1350x speedup**.

## Root Cause Analysis

### Step 1: Isolating the Bottleneck

Initial benchmarking showed the issue was in AST building, not PEST parsing:

```
1. PEST parse (lazy): 32ms
2. Count all pairs recursively: 17ms (1.18 million pairs)
3. Full parse with AST: 54 seconds  ← BOTTLENECK

Delta (AST building): 54 seconds
```

### Step 2: Scalability Testing

Created test files with varying numbers of save frames to identify O(n²) pattern:

| Frames | Lines | Parse Time | Time/Frame |
|--------|-------|------------|------------|
| 25     | 593   | 27ms       | 1.1ms      |
| 50     | 1133  | 84ms       | 1.7ms      |
| 100    | 2303  | 368ms      | 3.7ms      |
| 200    | 4339  | 1.3s       | 6.5ms      |
| 1222   | 28966 | 55s        | 45ms       |

Time per frame growing linearly with total frames confirmed O(n²) behavior.

### Step 3: Finding the Real Cause

After many grammar optimization attempts failed, discovered that `extract_span()` and `extract_location()` called PEST's `line_col()` for every AST node:

```rust
// Before - O(n) per call
pub(crate) fn extract_span(pair: &Pair<Rule>) -> Span {
    let pest_span = pair.as_span();
    let (start_line, start_col) = pest_span.start_pos().line_col();  // O(n)!
    let (end_line, end_col) = pest_span.end_pos().line_col();        // O(n)!
    Span::new(start_line, start_col, end_line, end_col)
}
```

With ~1.2 million AST nodes, this was O(n²) total.

## Solution Implemented

### LineIndex for O(log n) Lookups

```rust
// Build once O(n), lookup O(log n)
pub(crate) struct LineIndex {
    newlines: Vec<usize>,  // Byte offsets of newlines
}

impl LineIndex {
    pub fn line_col(&self, offset: usize) -> (usize, usize) {
        // Binary search for line number
        let line = match self.newlines.binary_search(&offset) {
            Ok(i) => i + 1,
            Err(i) => i + 1,
        };
        // Calculate column from line start
        let line_start = if line == 1 { 0 } else { self.newlines[line - 2] + 1 };
        (line, offset - line_start + 1)
    }
}
```

### Thread-Local Caching

```rust
thread_local! {
    static LINE_INDEX: RefCell<Option<LineIndex>> = const { RefCell::new(None) };
}

// Initialize at start of parse
pub(crate) fn init_line_index(input: &str) {
    LINE_INDEX.with(|idx| {
        *idx.borrow_mut() = Some(LineIndex::new(input));
    });
}
```

## Final Performance

```
=== PEST Iteration Benchmark ===

File: crates/cif-validator/dics/cif_core.dic
File size: 905010 bytes, 28966 lines

1. PEST parse (lazy): 30ms
2. Count all pairs recursively: 15ms (1,184,723 pairs)
3. Full parse with AST: 41ms

Delta (AST building): 11ms
```

## Files Modified

1. `crates/cif-parser/src/parser/helpers.rs` - Added LineIndex and fast lookups
2. `crates/cif-parser/src/parser/document.rs` - Initialize/clear line index
3. `crates/cif-parser/src/cif.pest` - Minor grammar optimizations (reordering)
4. `crates/cif-validator/tests/performance_test.rs` - Performance tests (new)
5. `crates/cif-parser/src/bin/bench_parse.rs` - Parsing benchmark (new)
6. `crates/cif-parser/src/bin/bench_pest_only.rs` - PEST-only benchmark (new)
7. `docs/performance.md` - Performance documentation (new)

## Test Commands

```bash
# Run performance benchmark
cargo run --release -p cif-parser --bin bench_pest_only -- crates/cif-validator/dics/cif_core.dic

# Run all cif-parser tests
cargo test -p cif-parser --release

# Run cif-validator tests
cargo test -p cif-validator --release
```

## Lessons Learned

1. **Profile before optimizing grammar**: The PEST grammar was not the bottleneck.
2. **Check stdlib/library function complexity**: `line_col()` being O(n) was not documented.
3. **Test with realistic data**: The issue only manifested with large files (1000+ frames).
4. **Binary search is your friend**: Pre-computing indices enables fast lookups.

## References

- PEST PEG parser: https://pest.rs/
- CIF 2.0 EBNF: https://www.iucr.org/__data/assets/text_file/0009/112131/CIF2-ENBF.txt
