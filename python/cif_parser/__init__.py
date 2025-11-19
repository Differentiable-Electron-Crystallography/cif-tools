"""CIF Parser - Fast CIF file parsing in Python using Rust.

This package provides Python bindings for a high-performance CIF (Crystallographic
Information File) parser written in Rust. It supports the full CIF 1.1 specification
including mmCIF/PDBx files.

Basic usage:
    >>> import cif_parser
    >>>
    >>> # Parse CIF content from string
    >>> doc = cif_parser.parse(cif_content)
    >>>
    >>> # Parse CIF file
    >>> doc = cif_parser.parse_file('structure.cif')
    >>>
    >>> # Access data
    >>> block = doc.first_block()
    >>> cell_a = block.get_item('_cell_length_a')
    >>> if cell_a and cell_a.is_numeric:
    ...     print(f"Cell a: {cell_a.numeric}")

Classes:
    Document: Root container for CIF data
    Block: Data block containing items, loops, and frames
    Loop: Tabular data structure
    Frame: Save frame container
    Value: Individual CIF value with type information

Functions:
    parse(content): Parse CIF content from string
    parse_file(path): Parse CIF file
"""

from ._cif_parser import (
    Block,
    Document,
    Frame,
    Loop,
    Value,
    __version__,
    parse,
    parse_file,
)

__all__ = [
    "Document",
    "Block",
    "Loop",
    "Frame",
    "Value",
    "parse",
    "parse_file",
    "__version__",
]

# Package metadata
__author__ = "Iain Maitland"
__email__ = "iain@iainmaitland.com"
__license__ = "MIT OR Apache-2.0"
