# Span Documentation

Spans track source locations for CIF values, enabling LSP/IDE features like hover, go-to-definition, and syntax highlighting.

## Structure

| Field | Description |
|-------|-------------|
| `start_line` | Starting line (1-indexed) |
| `start_col` | Starting column (1-indexed) |
| `end_line` | Ending line (1-indexed) |
| `end_col` | Ending column (1-indexed, **exclusive**) |

## Conventions

- **1-indexed**: Lines and columns start at 1, not 0
- **Exclusive end**: `end_col` points to the character *after* the last character of the value
  - A single character at column 5 has `start_col=5, end_col=6`
  - Value length = `end_col - start_col`

## Usage

### Python
```python
import cif_parser

doc = cif_parser.parse_file("structure.cif")
value = doc.first_block().get_item("_cell_length_a")

span = value.span
print(f"Line {span.start_line}, columns {span.start_col}-{span.end_col}")

# Hit testing for hover/click
if span.contains(cursor_line, cursor_col):
    show_hover_info(value)
```

### JavaScript (WASM)
```javascript
import { parse } from '@cif-tools/parser';

const doc = parse(cifContent);
const value = doc.first_block().get_item('_cell_length_a');

const span = value.span;
console.log(`Line ${span.startLine}, columns ${span.startCol}-${span.endCol}`);

// Hit testing for hover/click
if (span.contains(cursorLine, cursorCol)) {
    showHoverInfo(value);
}
```

## API Reference

### Python (`cif_parser.Span`)
- `start_line: int` - Starting line
- `start_col: int` - Starting column
- `end_line: int` - Ending line
- `end_col: int` - Ending column (exclusive)
- `contains(line: int, col: int) -> bool` - Check if position is within span
- Supports `==`, `hash()`, `str()`, `repr()`

### JavaScript (`JsSpan`)
- `startLine: number` - Starting line
- `startCol: number` - Starting column
- `endLine: number` - Ending line
- `endCol: number` - Ending column (exclusive)
- `contains(line: number, col: number): boolean` - Check if position is within span
