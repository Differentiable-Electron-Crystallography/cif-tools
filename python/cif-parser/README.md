# cif-parser

A fast CIF (Crystallographic Information File) parser for Python, powered by Rust.

## Installation

```bash
pip install cif-parser
```

## Features

- Full CIF 1.1 and CIF 2.0 syntax support
- Automatic version detection
- Type-safe value access with automatic numeric parsing
- High performance Rust core with Python bindings

## Quick Start

```python
import cif_parser

# Parse from file
doc = cif_parser.parse_file('structure.cif')

# Or parse from string
doc = cif_parser.parse("""
data_example
_cell.length_a 10.000
_cell.length_b 20.000
_cell.length_c 30.000
""")

# Access data blocks
block = doc.first_block()
print(f"Block name: {block.name}")

# Get single values
cell_a = block.get_item('_cell.length_a')
print(f"Cell a: {cell_a.numeric}")  # 10.0

# Iterate over all items
for name, value in block.items():
    print(f"{name}: {value}")
```

## API Reference

### Parsing Functions

- `parse(text: str) -> CifDocument` - Parse CIF content from a string
- `parse_file(path: str) -> CifDocument` - Parse CIF content from a file

### CifDocument

- `blocks` - List of all data blocks
- `first_block()` - Get the first data block (or None)
- `get_block(name: str)` - Get a block by name (or None)

### DataBlock

- `name` - The block name (without `data_` prefix)
- `get_item(name: str)` - Get a single item by name
- `get_loop(category: str)` - Get a loop by category
- `items()` - Iterate over all items
- `loops()` - Iterate over all loops

### CifValue

- `raw` - The raw string value
- `numeric` - Parse as float (or None)
- `integer` - Parse as int (or None)
- `is_missing` - True if value is `.` (inapplicable)
- `is_unknown` - True if value is `?` (unknown)

## Examples

Interactive Jupyter notebooks are provided in the `examples/` directory:

- **version_detection.ipynb** - CIF version detection and CIF 2.0 features
- **type_checking_example.ipynb** - Type-safe parsing with IDE autocomplete
- **duckdb_integration.ipynb** - SQL queries on CIF data with pandas/DuckDB

### Running the Examples

Install the optional example dependencies:

```bash
# Using uv (recommended)
uv sync --extra examples
uv run jupyter lab examples/

# Or using pip
pip install cif-parser[examples]
jupyter lab examples/
```

## CIF Version Support

The parser automatically detects CIF version based on file content:

- **CIF 1.1**: Standard crystallographic files
- **CIF 2.0**: Extended syntax with triple-quoted strings, list/table values

## License

Licensed under either of Apache License 2.0 or MIT license at your option.

## Links

- [GitHub Repository](https://github.com/Differentiable-Electron-Crystallography/cif-tools)
- [CIF Specification](https://www.iucr.org/resources/cif)
