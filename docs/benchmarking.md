# Benchmarking

This document describes how to run benchmarks and performance considerations in cif-tools.

## Running Benchmarks

Benchmarks use [Criterion.rs](https://github.com/bheisler/criterion.rs) and are located in:
- `crates/cif-parser/benches/parsing.rs` - CIF parsing benchmarks
- `crates/cif-validator/benches/dictionary_loading.rs` - Dictionary loading benchmarks

### Using just

```bash
just bench           # Run all benchmarks
just bench-parser    # Run cif-parser benchmarks only
just bench-validator # Run cif-validator benchmarks only
```

### Using cargo directly

```bash
cargo bench                      # All benchmarks
cargo bench -p cif-parser        # Parser benchmarks
cargo bench -p cif-validator     # Validator benchmarks
cargo bench -- <filter>          # Run specific benchmark by name
```

### GitHub Actions

Benchmarks can be run manually from the GitHub Actions tab:
1. Go to Actions → "Benchmarks"
2. Click "Run workflow"
3. Select which crate to benchmark (all, cif-parser, or cif-validator)

Results are uploaded as artifacts and retained for 30 days.

## Benchmark Descriptions

### cif-parser

| Benchmark | Description |
|-----------|-------------|
| `pest_parse_lazy` | PEST parsing only (lazy, no tree traversal) |
| `pest_full_traversal` | PEST parsing + full recursive tree traversal |
| `full_ast_parse` | Complete parse including AST construction |

### cif-validator

| Benchmark | Description |
|-----------|-------------|
| `cif_parse` | CIF document parsing (same as parser) |
| `dictionary_load` | Dictionary construction from parsed CIF |
| `full_dictionary_load` | End-to-end: file read + parse + dictionary load |

## Expected Performance

For `cif_core.dic` (~29,000 lines) in release mode:

| Stage | Time |
|-------|------|
| File read | < 1ms |
| PEST parse | ~30ms |
| AST building | ~10ms |
| Dictionary load | < 2ms |
| **Total** | **~40ms** |

## Performance Notes

### Line/Column Calculation Optimization

**Issue:** PEST's `line_col()` function is O(n) - it counts newlines from the start of the file each time. When called for every AST node (~1.2 million nodes for `cif_core.dic`), this results in O(n²) total time.

**Solution:** Pre-compute a line index (newline byte positions) O(n) once, then use binary search for O(log n) lookups.

```rust
pub struct LineIndex {
    newlines: Vec<usize>,  // Byte offsets of newlines
}

impl LineIndex {
    pub fn line_col(&self, offset: usize) -> (usize, usize) {
        let line = match self.newlines.binary_search(&offset) {
            Ok(i) | Err(i) => i + 1,
        };
        let line_start = if line == 1 { 0 } else { self.newlines[line - 2] + 1 };
        (line, offset - line_start + 1)
    }
}
```

**Impact:** Parsing of `cif_core.dic` improved from **54 seconds to 40 milliseconds** - a **1350x speedup**.

### Grammar Optimizations

Minor optimizations applied to grammar rules:

1. **Text field content** - Optimized negative lookahead pattern
2. **Triple-quoted strings** - Optimized negative lookahead pattern
3. **Block content alternatives** - Reordered for faster dispatch

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
