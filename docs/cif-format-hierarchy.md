# CIF Format: Hierarchy and Structure Reference

## Overview

CIF (Crystallographic Information File) is a standard file format maintained by the International Union of Crystallography (IUCr) for exchanging crystallographic data. The format uses a structured, human-readable text format based on the STAR (Self-defining Text Archive and Retrieval) file standard.

---

## Hierarchical Structure

The CIF format has a clear hierarchical organization, shown here from top to bottom:

```
CifDocument (Root)
│
├── DataBlock (data_name or global_)
│   ├── DataItem (tag-value pairs)
│   │   └── Value (Text/Numeric/Special)
│   ├── Loop (tabular data)
│   │   ├── Tags (column headers)
│   │   └── Values (rows of data)
│   └── SaveFrame (save_name ... save_)
│       ├── DataItem
│       └── Loop
```

### Key Relationships

- A **Document** contains one or more **DataBlocks**
- **DataBlocks** can contain **DataItems**, **Loops**, and **SaveFrames**
- **SaveFrames** can contain **DataItems** and **Loops** (but NOT other SaveFrames)
- **DataItems** link a **Tag** (identifier) to a **Value**
- **Loops** organize multiple tags with multiple rows of values in tabular form

---

## Component Details

### 1. Document (CifDocument)

**Grammar Location:** `src/cif.pest` lines 248-264

**Purpose:** The root container for an entire CIF file. Represents the complete parsed document.

**Structure:**
```cif
# Optional comments
data_structure1
_item value

data_structure2
_item value
```

**Use Cases:**
- **Single-structure files:** One crystal structure's complete data
- **Multi-structure files:** Multiple related structures (e.g., different temperature measurements)
- **Database exports:** Multiple structures from a crystallographic database

**Real-World Example:**
```cif
# Protein structure at different pH values
data_protein_pH7
_cell_length_a 50.123
_cell_length_b 60.456

data_protein_pH5
_cell_length_a 50.789
_cell_length_b 60.234
```

---

### 2. DataBlock (CifBlock)

**Grammar Location:** `src/cif.pest` lines 240-245

**Purpose:** The primary organizational unit that groups all data for a single entity (typically one crystal structure, one experiment, or one dataset).

**Types:**
- **Regular Block:** `data_name` - Contains structure-specific data
- **Global Block:** `global_` - Contains settings that apply to all subsequent blocks

**Structure:**
```cif
data_blockname     # Block header
_tag1 value1       # Data items
_tag2 value2

loop_              # Loop structures
_col1 _col2
val1  val2

save_frame1        # Save frames
_frame_tag value
save_
```

**Block Name Extraction:**
- `data_protein` → block name is `"protein"`
- `DATA_STRUCTURE` → name is `"STRUCTURE"` (case-insensitive)
- `global_` → name is `""` (empty string)

**Use Cases:**
- **Single structure:** All crystallographic data for one compound
- **Multiple polymorphs:** Different crystal forms of the same molecule
- **Time series:** Same structure measured at different times/conditions
- **Global settings:** Shared metadata (instrument details, contact info)

**Real-World Example:**
```cif
data_aspirin_form_I
_chemical_name_common 'Aspirin form I'
_cell_length_a    11.430
_cell_length_b    6.591
_cell_length_c    11.395
_space_group_name_H-M_alt  'P 21/c'

loop_
_atom_site_label
_atom_site_fract_x
_atom_site_fract_y
_atom_site_fract_z
C1  0.1234  0.5678  0.9012
O1  0.2345  0.6789  0.0123
```

---

### 3. DataItem

**Grammar Location:** `src/cif.pest` lines 208-213

**Purpose:** Represents a single piece of data as a key-value pair. The fundamental unit of data storage in CIF.

**Structure:**
```cif
_tag_name value
```

**Components:**
- **Tag:** Identifier starting with underscore (`_`), can contain dots for hierarchical naming
- **Value:** Can be text, numeric, quoted, or special values

**Tag Naming Conventions:**
- `_cell_length_a` - underscore-separated
- `_atom_site.label` - dot-separated for hierarchical organization (category.item)

**Use Cases:**
- **Scalar measurements:** Single values like cell parameters, volume, density
- **Identifiers:** Sample names, compound IDs
- **Metadata:** Author information, dates, software versions
- **Flags:** Binary or categorical settings

**Real-World Examples:**
```cif
# Unit cell parameters
_cell_length_a    10.523
_cell_length_b    15.678
_cell_angle_alpha 90.00

# Chemical information
_chemical_formula_sum 'C8 H10 N4 O2'
_chemical_formula_weight 194.19

# Special values
_refine_ls_R_factor_obs  0.0234
_disorder_flag           ?      # Unknown
_absorption_correction   .      # Not applicable

# Text field (multi-line)
_chemical_name_systematic
;
2-acetoxy-benzoic acid
commonly known as aspirin
;
```

---

### 4. Loop

**Grammar Location:** `src/cif.pest` lines 224-231

**Purpose:** Represents tabular data with named columns and multiple rows. Essential for representing lists of related data like atomic coordinates, bond distances, or measurement series.

**Structure:**
```cif
loop_
_column1_tag
_column2_tag
_column3_tag
value1_1  value1_2  value1_3    # Row 1
value2_1  value2_2  value2_3    # Row 2
value3_1  value3_2  value3_3    # Row 3
```

**Key Features:**
- **Column headers:** CIF tags that define what each column represents
- **Row organization:** Values are read sequentially and distributed across columns
- **Type flexibility:** Each column can have its own data type
- **Validation:** Number of values must be divisible by number of tags

**Use Cases:**

#### 1. Atomic Coordinates (Most Common)
```cif
loop_
_atom_site_label          # Atom identifier
_atom_site_type_symbol    # Element symbol
_atom_site_fract_x        # Fractional x coordinate
_atom_site_fract_y        # Fractional y coordinate
_atom_site_fract_z        # Fractional z coordinate
_atom_site_U_iso_or_equiv # Thermal parameter
C1   C   0.1234  0.5678  0.9012  0.0234
C2   C   0.2345  0.6789  0.0123  0.0245
N1   N   0.3456  0.7890  0.1234  0.0256
O1   O   0.4567  0.8901  0.2345  0.0267
```

#### 2. Bond Distances
```cif
loop_
_geom_bond_atom_site_label_1
_geom_bond_atom_site_label_2
_geom_bond_distance
C1  C2  1.524
C1  H1  1.089
N1  C2  1.456
```

#### 3. Anisotropic Displacement Parameters
```cif
loop_
_atom_site_aniso_label
_atom_site_aniso_U_11
_atom_site_aniso_U_22
_atom_site_aniso_U_33
C1  0.0234  0.0245  0.0256
C2  0.0245  0.0256  0.0267
```

#### 4. Bibliography/Citations
```cif
loop_
_citation_id
_citation_journal_abbrev
_citation_year
_citation_page_first
1  'Acta Cryst.'  2024  1234
2  'J. Am. Chem. Soc.'  2023  5678
```

#### 5. Reflection Data (X-ray Intensities)
```cif
loop_
_refln_index_h
_refln_index_k
_refln_index_l
_refln_F_squared_meas
1  0  0  1234.5
1  0  1  567.8
1  1  0  890.1
```

---

### 5. SaveFrame

**Grammar Location:** `src/cif.pest` lines 233-238

**Purpose:** Named sub-containers within data blocks that group related items and loops. Used for organizing reusable or logically grouped data that's distinct from the main block content.

**Structure:**
```cif
data_main_block
_main_item value

save_frame_name
_frame_item1 value1
_frame_item2 value2
loop_
_tag1 _tag2
val1  val2
save_              # Closing delimiter (no name)
```

**Key Characteristics:**
- Bounded by `save_name` (opening) and `save_` (closing)
- Can contain DataItems and Loops
- **Cannot** contain other SaveFrames (no nesting)
- Must be contained within a DataBlock
- Optional - not all CIF files use them

**Use Cases:**

#### 1. Molecular Fragments / Residue Definitions
```cif
data_protein
_entry_id 1ABC

save_residue_ALA
_chem_comp.id                 ALA
_chem_comp.name              'ALANINE'
_chem_comp.type              'L-peptide linking'
_chem_comp.formula           'C3 H7 N O2'
loop_
_chem_comp_atom.atom_id
_chem_comp_atom.type_symbol
N    N
CA   C
C    C
O    O
CB   C
save_

save_residue_GLY
_chem_comp.id                 GLY
_chem_comp.name              'GLYCINE'
# ... more glycine details
save_
```

#### 2. Restraint Definitions (for Refinement)
```cif
data_refinement
_refine_ls_R_factor 0.0234

save_restraint_set_1
_restraint_type 'bond_distance'
loop_
_restraint_atom_1
_restraint_atom_2
_restraint_target
_restraint_sigma
C1  C2  1.524  0.020
C2  C3  1.524  0.020
save_

save_restraint_set_2
_restraint_type 'angle'
# ... angle restraints
save_
```

#### 3. Template Definitions (mmCIF/PDBx)
```cif
data_dictionary

save_category_atom_site
_category.description
;
Atom site information including coordinates,
occupancy, and thermal parameters
;
loop_
_category_key.name
'_atom_site.id'
save_

save_item_atom_site_label
_item.name              '_atom_site.label'
_item.category_id       'atom_site'
_item_type.code         'code'
save_
```

#### 4. Ligand Libraries
```cif
data_compound_library

save_ligand_ATP
_chem_comp.id     ATP
_chem_comp.name   'ADENOSINE-5-TRIPHOSPHATE'
_chem_comp.formula 'C10 H16 N5 O13 P3'
loop_
_chem_comp_atom.atom_id
_chem_comp_atom.type_symbol
_chem_comp_atom.charge
# ... ATP atom definitions
save_

save_ligand_GTP
_chem_comp.id     GTP
_chem_comp.name   'GUANOSINE-5-TRIPHOSPHATE'
# ... GTP definitions
save_
```

**Why Use SaveFrames?**
- **Organization:** Logical grouping of related data
- **Reusability:** Define once, reference multiple times
- **Clarity:** Separate concerns (e.g., main structure vs. restraint definitions)
- **Standards:** Required by some CIF dictionaries (especially mmCIF/PDBx)

---

### 6. Value (CifValue)

**Grammar Location:** `src/cif.pest` lines 172-177

**Purpose:** Represents individual data values with automatic type detection and handling of special cases.

**Value Types:**

#### 1. Text (String)
- Unquoted: `simple_value`, `C12H22O11`
- Single-quoted: `'value with spaces'`, `'He said "hello"'`
- Double-quoted: `"value with spaces"`, `"It's fine"`
- Text fields (multi-line):
  ```cif
  ;This is a long
  multi-line description
  that can contain special characters
  ;
  ```

#### 2. Numeric (f64)
- Integers: `123`, `-456`
- Floats: `123.456`, `-789.012`
- Scientific notation: `1.23e-4`, `5.67E+8`
- With uncertainty: `1.234(5)` stored as `1.234`

#### 3. Special Values
- `?` - Unknown or missing data
- `.` - Not applicable or inapplicable

**Type Detection Strategy:**
1. Check for special values (`?`, `.`) first
2. Remove quotes/extract text field content
3. Attempt numeric parsing
4. Fall back to text if parsing fails

**Real-World Examples:**
```cif
# Text values
_chemical_name_common         aspirin
_space_group_name_H-M_alt    'P 21/c'
_chemical_formula_sum         'C9 H8 O4'

# Numeric values
_cell_length_a                10.523
_cell_angle_alpha             90.00
_refine_ls_R_factor_obs       0.0234
_diffrn_ambient_temperature   293.0

# Scientific notation
_atom_site_occupancy          1.00
_refine_ls_shift/su_max       3.45e-4

# Special values
_refine_diff_density_max      ?      # Unknown
_twin_individual_mass_fraction_refined  .  # Not applicable

# Text fields
_publ_section_abstract
;
The crystal structure of aspirin (2-acetoxybenzoic acid)
has been determined at 123 K using single-crystal X-ray
diffraction. The compound crystallizes in the monoclinic
space group P 21/c.
;
```

---

## Parsing Order and Logical Flow

When parsing a CIF file, the structure is processed in this order:

1. **Document Level:** Initialize document container
2. **Block Headers:** Encounter `data_name` or `global_`
3. **Block Content:** Parse items, loops, and save frames
4. **Item Processing:** Tag followed by value
5. **Loop Processing:**
   - Parse `loop_` keyword
   - Collect all tags
   - Collect all values
   - Organize into rows (values ÷ tags = rows)
6. **SaveFrame Processing:**
   - Parse `save_name` (opening)
   - Process contained items/loops
   - Parse `save_` (closing)
7. **Repeat:** Continue until end of file

---

## Common Patterns and Best Practices

### Single vs. Multiple Blocks

**Single Block (Most Common):**
```cif
data_compound_X
# All data for one structure
```

**Multiple Blocks:**
```cif
data_powder_pattern_1
# Powder diffraction data

data_single_crystal_1
# Single crystal structure

data_computed_structure_1
# DFT-optimized geometry
```

### When to Use Loops vs. Items

**Use DataItems for:**
- Single scalar values
- Identifiers and names
- Unique properties

**Use Loops for:**
- Lists of atoms, bonds, angles
- Multiple measurements
- Tabular data with common properties

### SaveFrame Usage

**Optional but useful for:**
- mmCIF/PDBx files (often required)
- Complex structures with multiple components
- Defining reusable data templates
- Organizing restraints/constraints

**Not needed for:**
- Simple small-molecule structures
- Most routine crystallographic data

---

## Implementation References

### Grammar Definition
- **File:** `src/cif.pest`
- **Lines 1-266:** Complete CIF 1.1 grammar specification

### AST Structures
- **`src/ast/document.rs`:** CifDocument structure
- **`src/ast/block.rs`:** CifBlock structure
- **`src/ast/frame.rs`:** CifFrame (SaveFrame) structure
- **`src/ast/loop_struct.rs`:** CifLoop structure
- **`src/ast/value.rs`:** CifValue type definitions

### Examples
- **`examples/advanced_features.rs`:** SaveFrame examples
- **`examples/basic_usage.rs`:** DataItems and Loops
- **`tests/fixtures/simple.cif`:** Real CIF example

---

## Summary

The CIF format uses a clear hierarchical structure optimized for crystallographic data:

| Component | Contains | Purpose |
|-----------|----------|---------|
| **Document** | DataBlocks | Multiple experimental datasets |
| **DataBlock** | Items, Loops, SaveFrames | Data for one structure/experiment |
| **DataItem** | Tag-Value pair | Single scalar values |
| **Loop** | Tags + Values | Tabular/columnar data |
| **SaveFrame** | Items, Loops | Logical grouping and reusability |
| **Value** | Text/Numeric/Special | Typed data with auto-detection |

This design makes CIF both human-readable and machine-parseable, suitable for:
- Crystallographic databases (Cambridge Structural Database)
- Structure repositories (Protein Data Bank)
- Routine laboratory data exchange
- Long-term data archival

---

## External Resources

- [IUCr CIF Specification](https://www.iucr.org/resources/cif)
- [CIF Core Dictionary](https://www.iucr.org/resources/cif/dictionaries/cif_core)
- [mmCIF Dictionary](https://mmcif.wwpdb.org/)
- [STAR File Format](https://www.iucr.org/resources/cif/spec/version1.1/cifsyntax)
