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

### Validation Fixtures
- `validation/test_validation.dic` - DDLm dictionary for validation testing
- `validation/valid_structure.cif` - CIF file that passes validation (0 errors)
- `validation/invalid_structure.cif` - CIF file with intentional errors (9 errors)

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
9. **Validation**: DDLm dictionary validation with range, type, and enumeration checks

## Validation Test Details

The `validation/` directory contains fixtures for testing cif-validator:

### Dictionary (`test_validation.dic`)
Defines data items with constraints:
- **Cell parameters**: `_cell.length_*` (range: 0.1-1000), `_cell.angle_*` (range: 0-180)
- **Crystal system**: `_symmetry.crystal_system` (enum: triclinic, monoclinic, orthorhombic, tetragonal, trigonal, hexagonal, cubic)
- **Atom sites**: `_atom_site.fract_*` (range: 0-1), `_atom_site.occupancy` (range: 0-1)

### Valid Structure (`valid_structure.cif`)
All values within valid ranges and using correct enumerations. Expected: 0 errors.

### Invalid Structure (`invalid_structure.cif`)
Contains 9 intentional validation errors:
1. `_cell.length_a = -5.0` - negative cell length
2. `_cell.length_b = 5000.0` - cell length exceeds maximum
3. `_cell.angle_alpha = 270.0` - angle exceeds 180°
4. `_cell.angle_beta = -45.0` - negative angle
5. `_symmetry.crystal_system = dodecahedral` - invalid enumeration
6. `_atom_site.fract_x = 1.5` - fractional coordinate > 1
7. `_atom_site.fract_y = -0.1` - negative fractional coordinate
8. `_atom_site.occupancy = 2.5` - occupancy > 1
9. `_atom_site.occupancy = -0.5` - negative occupancy
