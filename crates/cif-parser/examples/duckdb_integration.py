#!/usr/bin/env python3
"""
Example: Using CIF parser with DuckDB for SQL-like queries

This demonstrates how the loop iterator enables easy integration with DuckDB
for powerful SQL-based analysis of CIF data.

Requirements:
    pip install duckdb pandas
"""

import cif_parser

# Sample CIF with atomic coordinates
cif_content = """
data_crystal_structure
_cell_length_a    10.000
_cell_length_b    10.000
_cell_length_c    10.000

loop_
_atom_site_label
_atom_site_type_symbol
_atom_site_fract_x
_atom_site_fract_y
_atom_site_fract_z
_atom_site_occupancy
C1   C   0.1234  0.2345  0.3456  1.00
C2   C   0.2345  0.3456  0.4567  1.00
N1   N   0.3456  0.4567  0.5678  0.95
O1   O   0.4567  0.5678  0.6789  1.00
O2   O   0.5678  0.6789  0.7890  0.90
H1   H   0.6789  0.7890  0.8901  1.00
H2   H   0.7890  0.8901  0.9012  1.00
"""

# Parse the CIF
doc = cif_parser.parse(cif_content)
block = doc.first_block()
loop = block.get_loop(0)

print("=" * 60)
print("CIF Parser + DuckDB Integration Example")
print("=" * 60)

# Method 1: Simple iteration and filtering (no DuckDB needed)
print("\n1. Simple Python filtering (without DuckDB):")
print("-" * 60)
carbon_atoms = [row for row in loop if row["_atom_site_type_symbol"].text == "C"]
print(f"Found {len(carbon_atoms)} carbon atoms:")
for atom in carbon_atoms:
    label = atom["_atom_site_label"].text
    x = atom["_atom_site_fract_x"].numeric
    print(f"  {label}: x={x}")

# Method 2: Using pandas (for more complex analysis)
print("\n2. Using pandas DataFrame:")
print("-" * 60)
try:
    import pandas as pd

    # Convert loop to list of dicts, then extract values
    rows = []
    for row in loop:
        # Convert PyValue objects to native Python types
        row_data = {}
        for tag, value in row.items():
            if value.is_numeric:
                row_data[tag] = value.numeric
            elif value.is_text:
                row_data[tag] = value.text
            else:
                row_data[tag] = None
        rows.append(row_data)

    df = pd.DataFrame(rows)
    print(df)

    print("\n  Summary statistics for occupancy:")
    print(df["_atom_site_occupancy"].describe())

except ImportError:
    print("  (pandas not installed, skipping)")

# Method 3: Using DuckDB for SQL queries
print("\n3. Using DuckDB for SQL queries:")
print("-" * 60)
try:
    import duckdb

    # Same conversion as above
    rows = []
    for row in loop:
        row_data = {}
        for tag, value in row.items():
            if value.is_numeric:
                row_data[tag] = value.numeric
            elif value.is_text:
                row_data[tag] = value.text
            else:
                row_data[tag] = None
        rows.append(row_data)

    # Query with DuckDB
    print("\n  Query: SELECT type, COUNT(*), AVG(occupancy)")
    result = duckdb.query(
        """
        SELECT
            _atom_site_type_symbol as type,
            COUNT(*) as count,
            AVG(_atom_site_occupancy) as avg_occupancy,
            MIN(_atom_site_fract_z) as min_z,
            MAX(_atom_site_fract_z) as max_z
        FROM rows
        GROUP BY _atom_site_type_symbol
        ORDER BY count DESC
    """
    ).to_df()
    print(result)

    print("\n  Query: Find atoms with fractional z > 0.5")
    high_z = duckdb.query(
        """
        SELECT
            _atom_site_label,
            _atom_site_type_symbol,
            _atom_site_fract_z
        FROM rows
        WHERE _atom_site_fract_z > 0.5
        ORDER BY _atom_site_fract_z
    """
    ).to_df()
    print(high_z)

    print("\n  Query: Atoms with occupancy < 1.0")
    partial = duckdb.query(
        """
        SELECT *
        FROM rows
        WHERE _atom_site_occupancy < 1.0
    """
    ).to_df()
    print(partial)

except ImportError:
    print("  (duckdb not installed, skipping)")

print("\n" + "=" * 60)
print("Key Takeaway:")
print("=" * 60)
print(
    """
The loop iterator enables:
  1. Simple Python iteration: for row in loop
  2. Easy conversion to pandas DataFrame
  3. SQL queries via DuckDB on CIF data
  4. Powerful data analysis workflows

All with just: list(loop)
"""
)
