# Test Fixtures

Shared CIF test fixtures used across Rust, Python, and JavaScript integration tests.

## Files

### Basic CIF Files
- `simple.cif` - Basic CIF with unknown (`?`) and not-applicable (`.`) values
- `simple_with_loop.cif` - Basic CIF with a loop and space group info
- `loops.cif` - Multiple loops (atom sites, bonds)
- `complex.cif` - Save frames, multiple data blocks

### Real-World Structures
- `ccdc_paracetamol.cif` - Cambridge Crystallographic Data Centre structure
- `cod_urea.cif` - Crystallography Open Database example
- `crystalmaker_LuAG.cif` - High-precision uncertainty values (e.g., `11.910400(4)`)
- `pycifrw_xanthine.cif` - Uncertainty values (e.g., `10.01(11)`)

### CIF 2.0 Features
- `cif2_lists.cif` - CIF 2.0 list syntax: empty, single-item, numeric, nested lists
- `cif2_tables.cif` - CIF 2.0 table syntax: empty, simple, coordinate tables

### Additional Examples
- `example_cifs/` - Collection of additional CIF examples

## Test Coverage

These fixtures are used to test:
1. **Value types**: Text, Numeric, Unknown (`?`), NotApplicable (`.`)
2. **Numeric with uncertainty**: `10.01(11)` -> value=10.01, uncertainty=0.11
3. **High-precision uncertainty**: `11.910400(4)` -> value=11.9104, uncertainty=0.000004
4. **Loops**: Multiple loops, column access, row iteration
5. **Save frames**: Frame access within blocks
6. **Multiple blocks**: Document with multiple data blocks
7. **CIF 2.0 lists**: `[]`, `[42]`, `[1 2 3]`, `[[1 2] [3 4]]`
8. **CIF 2.0 tables**: `{}`, `{'a':1 'b':2}`, `{'x':1.5 'y':2.5 'z':3.5}`
