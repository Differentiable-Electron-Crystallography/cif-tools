#!/usr/bin/env python3
"""
Example demonstrating type checking with cif_parser type stubs.

This file can be checked with mypy or pyright to verify type correctness:
    mypy examples/type_checking_example.py
    pyright examples/type_checking_example.py

The type stubs reflect the dynamic nature of CIF files while providing
IDE autocomplete and static type checking.
"""

import cif_parser
from typing import Dict, List, Optional, Union

# Sample CIF content
cif_content = """
data_crystal
_cell_length_a    10.5
_cell_length_b    10.5
_cell_length_c    15.2
_cell_angle_alpha 90.0
_title 'Example Crystal Structure'
_temperature_kelvin ?

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
"""


def main() -> None:
    """Demonstrate type-safe CIF parsing."""

    # Parse document - type checker knows this returns Document
    doc: cif_parser.Document = cif_parser.parse(cif_content)

    # Get first block - type checker knows this is Optional[Block]
    block: Optional[cif_parser.Block] = doc.first_block()

    if block is None:
        print("No blocks found!")
        return

    # Type checker knows block.name is str
    print(f"Block name: {block.name}")

    # Example 1: Type-safe numeric value access
    print("\n1. Accessing numeric values:")
    cell_a: Optional[cif_parser.Value] = block.get_item("_cell_length_a")

    if cell_a is not None:
        # Type checker knows is_numeric is bool
        if cell_a.is_numeric:
            # Type checker knows numeric is Optional[float]
            length: Optional[float] = cell_a.numeric
            if length is not None:
                # Now type checker knows length is definitely float
                doubled: float = length * 2.0
                print(f"  Cell length a: {length}")
                print(f"  Doubled: {doubled}")

    # Example 2: Type-safe text value access
    print("\n2. Accessing text values:")
    title: Optional[cif_parser.Value] = block.get_item("_title")

    if title is not None and title.is_text:
        text: Optional[str] = title.text
        if text is not None:
            # Type checker knows text is str
            upper: str = text.upper()
            print(f"  Title: {text}")
            print(f"  Uppercase: {upper}")

    # Example 3: Handling special values (unknown/not applicable)
    print("\n3. Handling special values:")
    temp: Optional[cif_parser.Value] = block.get_item("_temperature_kelvin")

    if temp is not None:
        if temp.is_unknown:
            print("  Temperature: Unknown (?)")
        elif temp.is_not_applicable:
            print("  Temperature: Not applicable (.)")
        elif temp.is_numeric and temp.numeric is not None:
            print(f"  Temperature: {temp.numeric} K")

    # Example 4: Type-safe loop iteration
    print("\n4. Iterating over loop rows:")
    loop: Optional[cif_parser.Loop] = block.get_loop(0)

    if loop is not None:
        # Type checker knows loop.tags is List[str]
        tags: List[str] = loop.tags
        print(f"  Loop has {len(tags)} columns: {tags[:3]}...")

        # Type checker knows iteration yields Dict[str, Value]
        carbon_count: int = 0
        for row in loop:
            # row is Dict[str, Value]
            atom_type: cif_parser.Value = row["_atom_site_type_symbol"]
            if atom_type.is_text and atom_type.text == "C":
                carbon_count += 1

        print(f"  Found {carbon_count} carbon atoms")

    # Example 5: Type-safe value conversion for DuckDB/pandas
    print("\n5. Converting to Python types (for DuckDB/pandas):")
    if loop is not None:
        # Build list with proper typing
        rows: List[Dict[str, Union[str, float, None]]] = []

        for row in loop:
            # Convert Values to Python types
            python_row: Dict[str, Union[str, float, None]] = {}

            for tag, value in row.items():
                # to_python() returns Union[str, float, None]
                python_value: Union[str, float, None] = value.to_python()
                python_row[tag] = python_value

            rows.append(python_row)

        print(f"  Converted {len(rows)} rows to Python types")
        print(f"  First row keys: {list(rows[0].keys())[:3]}...")

    # Example 6: Document iteration
    print("\n6. Iterating over blocks:")
    block_count: int = len(doc)
    print(f"  Document has {block_count} block(s)")

    for blk in doc:
        # Type checker knows blk is Block
        name: str = blk.name
        num_items: int = len(blk.item_keys)
        print(f"    - Block '{name}' with {num_items} items")

    # Example 7: Dictionary-style access with type checking
    print("\n7. Dictionary-style block access:")

    # Access by index - type checker knows this can raise IndexError
    try:
        first_block: cif_parser.Block = doc[0]
        print(f"  Block 0: {first_block.name}")
    except IndexError:
        print("  No block at index 0")

    # Access by name - type checker knows this can raise KeyError
    try:
        named_block: cif_parser.Block = doc["crystal"]
        print(f"  Block 'crystal': {named_block.name}")
    except KeyError:
        print("  Block 'crystal' not found")

    print("\n✅ All type checking examples completed!")


if __name__ == "__main__":
    main()
