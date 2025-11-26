# Performance Learnings: How We Got a 1350x Speedup

This document explains a major performance bug we found and fixed, written so anyone can understand the problem and apply similar fixes to their own code.

## The Problem

Our CIF parser was taking **54 seconds** to parse a 29,000-line file. After fixing a single function, it took **40 milliseconds**. That's 1350 times faster.

## The Symptom

Integration tests were "hanging" - they'd run for minutes without completing. The test output showed:

```
test test_load_cif_core_dictionary has been running for over 60 seconds
```

## Finding the Bottleneck

### Step 1: Measure, Don't Guess

We added timing to each step:

```rust
let start = Instant::now();
let content = std::fs::read_to_string(path)?;
println!("File read: {:?}", start.elapsed());  // 0.5ms

let start = Instant::now();
let doc = parse(&content)?;
println!("Parse: {:?}", start.elapsed());  // 54 seconds!
```

This told us the problem was in parsing, not file I/O.

### Step 2: Isolate Further

We use a library called PEST for parsing. We tested if PEST itself was slow:

```rust
// Just run PEST, don't build our data structures
let pairs = PestParser::parse(Rule::file, &content)?;
println!("PEST parse: {:?}", elapsed);  // 32ms

// Now build our data structures from the parse tree
let doc = build_ast(pairs)?;
println!("AST build: {:?}", elapsed);  // 54 seconds!
```

PEST was fast (32ms). Our code that processed PEST's output was slow (54 seconds).

### Step 3: Find the O(n²) Pattern

We tested with different file sizes:

| File Size | Time | Time per Line |
|-----------|------|---------------|
| 500 lines | 27ms | 0.05ms |
| 2000 lines | 368ms | 0.18ms |
| 10000 lines | 7s | 0.7ms |
| 29000 lines | 54s | 1.9ms |

Notice how "time per line" keeps growing? That's the signature of O(n²) - the algorithm gets slower as the input gets bigger, even per-item.

## The Root Cause

We were calling a function called `line_col()` that converts a byte position in the file to a line number and column. For example, byte 1000 might be line 45, column 12.

```rust
fn extract_span(pair: &Pair) -> Span {
    let (start_line, start_col) = pair.start_pos().line_col();  // <- THE PROBLEM
    let (end_line, end_col) = pair.end_pos().line_col();
    Span::new(start_line, start_col, end_line, end_col)
}
```

This looks innocent. But `line_col()` works by counting newlines from the **beginning of the file** every time you call it:

```rust
// Pseudocode for how line_col() works internally
fn line_col(text: &str, byte_offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, char) in text.chars().enumerate() {
        if i == byte_offset {
            return (line, col);
        }
        if char == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}
```

For a position near the end of a 29,000-line file, this function has to count through ~29,000 newlines. That's O(n) per call.

We called this function for every node in our parse tree - about 1.2 million times. So:
- 1.2 million calls × O(n) per call = O(n²) total
- That's why 29,000 lines took 54 seconds

## The Fix

Instead of counting newlines every time, count them **once** and remember where they are:

```rust
struct LineIndex {
    // Byte positions of every newline in the file
    // For "hello\nworld\n", this would be [5, 11]
    newlines: Vec<usize>,
}

impl LineIndex {
    fn new(text: &str) -> Self {
        let newlines = text
            .bytes()
            .enumerate()
            .filter(|(_, b)| *b == b'\n')
            .map(|(i, _)| i)
            .collect();
        Self { newlines }
    }

    fn line_col(&self, offset: usize) -> (usize, usize) {
        // Binary search to find which line this offset is on
        let line = match self.newlines.binary_search(&offset) {
            Ok(i) | Err(i) => i + 1,
        };

        // Calculate column from start of this line
        let line_start = if line == 1 {
            0
        } else {
            self.newlines[line - 2] + 1
        };

        let col = offset - line_start + 1;
        (line, col)
    }
}
```

Now:
- Building the index: O(n) - done once
- Each lookup: O(log n) - binary search
- 1.2 million lookups: O(n log n) total

For 29,000 lines: O(n²) ≈ 841 million operations → O(n log n) ≈ 430,000 operations. That's why we got a 1350x speedup.

## How to Apply This to Your Code

### 1. Look for Functions Called in Loops

If you have code like this:

```rust
for item in huge_list {
    let result = some_function(item);  // Is this O(n)?
}
```

Check if `some_function` is O(n). If so, you have O(n²).

### 2. Check Library Function Complexity

Library functions aren't always O(1). Common traps:
- String operations that scan from the start
- List operations that search linearly
- Anything that "counts" or "finds" from the beginning

### 3. Pre-compute When Possible

If you're computing the same information repeatedly:

```rust
// BAD: O(n) every time
for i in 0..n {
    let line = count_newlines_up_to(text, positions[i]);
}

// GOOD: O(n) once, then O(1) or O(log n) per lookup
let index = build_newline_index(text);  // O(n) once
for i in 0..n {
    let line = index.lookup(positions[i]);  // O(log n) each
}
```

### 4. Use Binary Search for Sorted Data

If you have sorted data (like newline positions in increasing order), binary search gives you O(log n) lookups instead of O(n).

```rust
// O(n) - linear search
fn find_line(newlines: &[usize], offset: usize) -> usize {
    for (i, &pos) in newlines.iter().enumerate() {
        if pos >= offset {
            return i;
        }
    }
    newlines.len()
}

// O(log n) - binary search
fn find_line(newlines: &[usize], offset: usize) -> usize {
    match newlines.binary_search(&offset) {
        Ok(i) | Err(i) => i,
    }
}
```

## Key Takeaways

1. **Measure first**: Don't guess where the slowness is. Add timing.

2. **Test with different sizes**: O(n²) problems only show up with large inputs. Test with 100, 1000, 10000 items.

3. **Watch for "time per item" growing**: If processing 10x more items takes 100x longer (not 10x), you have O(n²).

4. **Check library functions**: Just because you didn't write it doesn't mean it's fast.

5. **Pre-compute repeated work**: If you're computing the same thing many times, compute it once and cache it.

6. **Binary search is your friend**: For sorted data, it turns O(n) into O(log n).

## The Numbers

```
Before: 54,000 ms (54 seconds)
After:     40 ms

Speedup: 1,350x

Lines of code changed: ~50
Time to find the bug: 2 hours
Time to fix: 30 minutes
```

Sometimes the biggest performance wins come from the smallest changes.
