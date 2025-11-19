"""
Type stubs for cif_parser

CIF (Crystallographic Information File) parser with Python bindings.

Note on typing: CIF files are dynamically typed - tag values can be text, numeric,
or special values (unknown/not applicable). This is reflected in the type system
through runtime type checking using properties like is_numeric, is_text, etc.

Example usage with type checking:
    import cif_parser

    doc = cif_parser.parse(cif_content)
    block = doc.first_block()

    if block is not None:
        value = block.get_item("_cell_length_a")
        if value is not None and value.is_numeric:
            length = value.numeric
            if length is not None:
                print(f"Cell length: {length}")
"""

from typing import Iterator, overload

__version__: str
__author__: str

class Value:
    """
    Represents a single value in a CIF file with runtime type detection.

    CIF values can be text, numeric, or special values
    (unknown '?' or not applicable '.').
    Use the type-checking properties to determine the actual type at runtime.

    Type checking pattern:
        if value.is_numeric:
            num = value.numeric  # Optional[float]
            if num is not None:
                # Use num as float
        elif value.is_text:
            text = value.text  # Optional[str]
            if text is not None:
                # Use text as str
    """

    @property
    def is_text(self) -> bool:
        """Returns True if this value contains text."""
        ...

    @property
    def is_numeric(self) -> bool:
        """Returns True if this value is numeric."""
        ...

    @property
    def is_unknown(self) -> bool:
        """Returns True if this is the special value '?' (unknown)."""
        ...

    @property
    def is_not_applicable(self) -> bool:
        """Returns True if this is the special value '.' (not applicable)."""
        ...

    @property
    def text(self) -> str | None:
        """
        Get the text value if is_text is True, otherwise None.

        Returns:
            The text content, or None if this is not a text value.
        """
        ...

    @property
    def numeric(self) -> float | None:
        """
        Get the numeric value if is_numeric is True, otherwise None.

        Returns:
            The numeric value as float, or None if this is not numeric.
        """
        ...

    @property
    def value_type(self) -> str:
        """
        Get a string representation of the value type.

        Returns:
            One of: "Text", "Numeric", "Unknown", "NotApplicable"
        """
        ...

    def to_python(self) -> str | float | None:
        """
        Convert to native Python type.

        Returns:
            - str if text value
            - float if numeric value
            - None if unknown or not applicable
        """
        ...

    def __str__(self) -> str:
        """String representation of the value."""
        ...

    def __repr__(self) -> str:
        """Debug representation of the value."""
        ...

    def __eq__(self, other: object) -> bool:
        """Check equality with another Value."""
        ...

class Loop:
    """
    Represents a loop structure (tabular data) in a CIF file.

    Loops contain rows of values organized by column tags. Use iteration
    to process rows as dictionaries, or use positional access methods.

    Example:
        for row in loop:
            label = row["_atom_site_label"]
            if label.is_text:
                print(label.text)

        # Or access by position
        value = loop.get(row=0, col=1)
    """

    @property
    def tags(self) -> list[str]:
        """Get the column tags (headers)."""
        ...

    @property
    def num_columns(self) -> int:
        """Get the number of columns."""
        ...

    def is_empty(self) -> bool:
        """Check if the loop has no rows."""
        ...

    def get(self, row: int, col: int) -> Value | None:
        """
        Get a value by row and column index.

        Args:
            row: Row index (0-based)
            col: Column index (0-based)

        Returns:
            The value at the specified position, or None if out of bounds.
        """
        ...

    def get_by_tag(self, row: int, tag: str) -> Value | None:
        """
        Get a value by row index and tag name.

        Args:
            row: Row index (0-based)
            tag: Column tag name (e.g., "_atom_site_label")

        Returns:
            The value at the specified row for the given tag, or None if not found.
        """
        ...

    def get_column(self, tag: str) -> list[Value] | None:
        """
        Get all values for a specific column tag.

        Args:
            tag: Column tag name

        Returns:
            List of values for the column, or None if tag doesn't exist.
        """
        ...

    def get_row_dict(self, row: int) -> dict[str, Value] | None:
        """
        Get a row as a dictionary mapping tags to values.

        Args:
            row: Row index (0-based)

        Returns:
            Dictionary of {tag: value}, or None if row doesn't exist.
        """
        ...

    def rows(self) -> list[list[Value]]:
        """
        Get all rows as lists of values.

        Returns:
            List of rows, where each row is a list of values.
        """
        ...

    def __len__(self) -> int:
        """Get the number of rows."""
        ...

    def __iter__(self) -> Iterator[dict[str, Value]]:
        """
        Iterate over rows as dictionaries.

        Yields:
            Dictionary mapping tag names to values for each row.

        Example:
            for row in loop:
                label = row["_atom_site_label"]
                x_coord = row["_atom_site_fract_x"]
        """
        ...

    def __str__(self) -> str:
        """String representation."""
        ...

    def __repr__(self) -> str:
        """Debug representation."""
        ...

class Frame:
    """
    Represents a save frame in a CIF file.

    Save frames are named sub-containers within data blocks that group
    related items and loops.
    """

    @property
    def name(self) -> str:
        """Get the frame name."""
        ...

    @property
    def item_keys(self) -> list[str]:
        """Get all item tag names in this frame."""
        ...

    @property
    def num_loops(self) -> int:
        """Get the number of loops in this frame."""
        ...

    def get_item(self, key: str) -> Value | None:
        """
        Get a data item value by tag name.

        Args:
            key: Tag name (e.g., "_frame_category")

        Returns:
            The value for the tag, or None if not found.
        """
        ...

    def items(self) -> dict[str, Value]:
        """
        Get all data items as a dictionary.

        Returns:
            Dictionary mapping tag names to values.
        """
        ...

    def get_loop(self, index: int) -> Loop | None:
        """
        Get a loop by index.

        Args:
            index: Loop index (0-based)

        Returns:
            The loop at the specified index, or None if out of bounds.
        """
        ...

    def loops(self) -> list[Loop]:
        """
        Get all loops in this frame.

        Returns:
            List of all loops.
        """
        ...

    def __str__(self) -> str:
        """String representation."""
        ...

    def __repr__(self) -> str:
        """Debug representation."""
        ...

class Block:
    """
    Represents a data block in a CIF file.

    Data blocks are the primary organizational unit, containing items,
    loops, and save frames.
    """

    @property
    def name(self) -> str:
        """Get the block name (from 'data_name' header)."""
        ...

    @property
    def item_keys(self) -> list[str]:
        """Get all item tag names in this block."""
        ...

    @property
    def num_loops(self) -> int:
        """Get the number of loops in this block."""
        ...

    @property
    def num_frames(self) -> int:
        """Get the number of save frames in this block."""
        ...

    def get_item(self, key: str) -> Value | None:
        """
        Get a data item value by tag name.

        Args:
            key: Tag name (e.g., "_cell_length_a")

        Returns:
            The value for the tag, or None if not found.
        """
        ...

    def items(self) -> dict[str, Value]:
        """
        Get all data items as a dictionary.

        Returns:
            Dictionary mapping tag names to values.
        """
        ...

    def get_loop(self, index: int) -> Loop | None:
        """
        Get a loop by index.

        Args:
            index: Loop index (0-based)

        Returns:
            The loop at the specified index, or None if out of bounds.
        """
        ...

    def find_loop(self, tag: str) -> Loop | None:
        """
        Find the first loop containing a specific tag.

        Args:
            tag: Tag name to search for

        Returns:
            The first loop containing the tag, or None if not found.
        """
        ...

    def get_frame(self, index: int) -> Frame | None:
        """
        Get a save frame by index.

        Args:
            index: Frame index (0-based)

        Returns:
            The frame at the specified index, or None if out of bounds.
        """
        ...

    def get_loop_tags(self) -> list[str]:
        """
        Get all loop tags from all loops in this block.

        Returns:
            List of all tag names used in loops.
        """
        ...

    def __str__(self) -> str:
        """String representation."""
        ...

    def __repr__(self) -> str:
        """Debug representation."""
        ...

class Document:
    """
    Represents a complete CIF document (root container).

    A document contains one or more data blocks. This is the entry point
    for parsing CIF files.

    Example:
        doc = Document.parse(cif_string)
        # or
        doc = Document.from_file("structure.cif")

        for block in doc:
            print(block.name)
    """

    @staticmethod
    def parse(content: str) -> Document:
        """
        Parse a CIF document from a string.

        Args:
            content: CIF file content as string

        Returns:
            Parsed document

        Raises:
            ValueError: If parsing fails due to invalid CIF syntax
        """
        ...

    @staticmethod
    def from_file(path: str) -> Document:
        """
        Parse a CIF document from a file.

        Args:
            path: Path to CIF file

        Returns:
            Parsed document

        Raises:
            IOError: If file cannot be read
            ValueError: If parsing fails due to invalid CIF syntax
        """
        ...

    @property
    def blocks(self) -> list[Block]:
        """Get all data blocks in this document."""
        ...

    @property
    def block_names(self) -> list[str]:
        """Get the names of all blocks in this document."""
        ...

    def get_block(self, index: int) -> Block | None:
        """
        Get a block by index.

        Args:
            index: Block index (0-based)

        Returns:
            The block at the specified index, or None if out of bounds.
        """
        ...

    def get_block_by_name(self, name: str) -> Block | None:
        """
        Get a block by name.

        Args:
            name: Block name (without 'data_' prefix)

        Returns:
            The block with the specified name, or None if not found.
        """
        ...

    def first_block(self) -> Block | None:
        """
        Get the first block (common for single-block CIF files).

        Returns:
            The first block, or None if document is empty.
        """
        ...

    def __len__(self) -> int:
        """Get the number of blocks."""
        ...

    @overload
    def __getitem__(self, key: int) -> Block: ...
    @overload
    def __getitem__(self, key: str) -> Block: ...
    def __iter__(self) -> Iterator[Block]:
        """
        Iterate over all blocks.

        Yields:
            Each block in the document.
        """
        ...

    def __str__(self) -> str:
        """String representation."""
        ...

    def __repr__(self) -> str:
        """Debug representation."""
        ...

# Module-level convenience functions

def parse(content: str) -> Document:
    """
    Parse a CIF document from a string.

    This is a convenience function equivalent to Document.parse().

    Args:
        content: CIF file content as string

    Returns:
        Parsed document

    Raises:
        ValueError: If parsing fails due to invalid CIF syntax

    Example:
        import cif_parser

        cif_content = '''
        data_example
        _cell_length_a  10.000
        _title 'My Structure'
        '''

        doc = cif_parser.parse(cif_content)
        block = doc.first_block()
    """
    ...

def parse_file(path: str) -> Document:
    """
    Parse a CIF document from a file.

    This is a convenience function equivalent to Document.from_file().

    Args:
        path: Path to CIF file

    Returns:
        Parsed document

    Raises:
        IOError: If file cannot be read
        ValueError: If parsing fails due to invalid CIF syntax

    Example:
        import cif_parser

        doc = cif_parser.parse_file("structure.cif")
        for block in doc:
            print(f"Block: {block.name}")
    """
    ...
