#!/usr/bin/env python3
"""
Example demonstrating CIF version detection and CIF 2.0 features.

This shows how to check the version of a CIF document and work with
CIF 2.0-specific features like lists and tables.
"""

from cif_parser import Document

# CIF 1.1 example (no magic header)
cif1_content = """
data_test
_chemical_name 'Example Compound'
_cell_length_a 10.0
"""

# CIF 2.0 example (with magic header)
cif2_content = """#\#CIF_2.0
data_test
_chemical_name 'Example Compound'
_coordinates [1.0 2.0 3.0]
_properties {x:1.0 y:2.0 z:3.0}
"""


def main():
    print("=" * 60)
    print("CIF Version Detection Example")
    print("=" * 60)

    # Parse CIF 1.1 document
    doc1 = Document.parse(cif1_content)
    print(f"\nCIF 1.1 Document:")
    print(f"  Version: {doc1.version}")
    print(f"  Is CIF 1.1: {doc1.is_cif1()}")
    print(f"  Is CIF 2.0: {doc1.is_cif2()}")

    # Parse CIF 2.0 document
    doc2 = Document.parse(cif2_content)
    print(f"\nCIF 2.0 Document:")
    print(f"  Version: {doc2.version}")
    print(f"  Is CIF 1.1: {doc2.is_cif1()}")
    print(f"  Is CIF 2.0: {doc2.is_cif2()}")

    # Check version properties
    print(f"\nVersion Properties:")
    print(f"  CIF 1.1 version.is_cif1: {doc1.version.is_cif1}")
    print(f"  CIF 2.0 version.is_cif2: {doc2.version.is_cif2}")

    # Working with values
    block2 = doc2.first_block()
    if block2:
        # Check for CIF 2.0 list values
        coords = block2.get_item("_coordinates")
        if coords:
            print(f"\nCoordinates value:")
            print(f"  Type: {coords.value_type}")
            print(f"  Is list: {coords.is_list}")
            print(f"  String representation: {coords}")

        # Check for CIF 2.0 table values
        props = block2.get_item("_properties")
        if props:
            print(f"\nProperties value:")
            print(f"  Type: {props.value_type}")
            print(f"  Is table: {props.is_table}")
            print(f"  String representation: {props}")

    print("\n" + "=" * 60)
    print("Summary:")
    print("  - CIF version is auto-detected from magic header")
    print("  - CIF 2.0 adds List and Table value types")
    print("  - Use doc.version to check which CIF version")
    print("  - Use value.is_list and value.is_table to check types")
    print("=" * 60)


if __name__ == "__main__":
    main()
