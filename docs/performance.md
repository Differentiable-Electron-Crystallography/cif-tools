# Performance Notes

This document describes performance considerations and optimizations in the cif-tools parsers.

## CIF Parser Performance

### Line/Column Calculation Optimization (CRITICAL)

**Issue:** PEST's `line_col()` function is O(n) - it counts newlines from the start of the file each time. When called for every AST node (~1.2 million nodes for `cif_core.dic`), this results in O(n²) total time.

**Solution:** Pre-compute a line index (newline byte positions) O(n) once, then use binary search for O(log n) lookups.

```rust
// Build once O(n), lookup O(log n)
pub struct LineIndex {
    newlines: Vec<usize>,  // Byte offsets of newlines
}

impl LineIndex {
    pub fn line_col(&self, offset: usize) -> (usize, usize) {
        // Binary search for line number
        let line = match self.newlines.binary_search(&offset) {
            Ok(i) | Err(i) => i + 1,
        };
        let line_start = if line == 1 { 0 } else { self.newlines[line - 2] + 1 };
        (line, offset - line_start + 1)
    }
}
```

**Impact:** Parsing of `cif_core.dic` (29,000 lines) improved from **54 seconds to 40 milliseconds** - a **1350x speedup**.

### Grammar Optimizations

Minor optimizations applied to grammar rules:

1. **Text field content** - Optimized negative lookahead pattern
2. **Triple-quoted strings** - Optimized negative lookahead pattern
3. **Block content alternatives** - Reordered for faster dispatch

## Benchmarking

Performance tests are located in `crates/cif-validator/tests/performance_test.rs`.

Run benchmarks with:

```bash
# Full benchmark including PEST internals
cargo run --release -p cif-parser --bin bench_pest_only -- crates/cif-validator/dics/cif_core.dic

# Quick parsing benchmark
cargo run --release -p cif-parser --bin bench_parse -- crates/cif-validator/dics/cif_core.dic

# Validator performance tests
cargo test -p cif-validator --release -- performance --nocapture
```

Expected times for `cif_core.dic` (release mode):
- File read: < 1ms
- PEST parse: ~30ms
- AST building: ~10ms
- Dictionary load: < 2ms
- **Total: ~40ms**

## General PEG Parser Performance Tips

When writing PEST grammar rules, avoid these patterns:

1. **Negative lookahead in loops:** `(!delimiter ~ ANY)*` can be slow
   - Instead, match characters that are definitely not part of the delimiter

2. **Avoid PEST's `line_col()` in hot paths:** It's O(n)
   - Pre-compute a line index and use binary search

3. **Long alternatives in choice expressions:** Each alternative is tried in order
   - Put most common cases first

4. **Deep recursion:** Can cause stack overflow
   - Consider iterative patterns where possible

## dREL Parser

The dREL parser has not been optimized for performance as dREL parsing only happens during dictionary validation (optional), not during normal CIF file validation.

If dREL parsing becomes a bottleneck, similar optimizations could be applied.
