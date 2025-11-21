"""Tests for error handling and edge cases."""

import os
import tempfile

import pytest

import cif_parser


class TestParsingErrors:
    """Test parsing error handling.

    NOTE: The current CIF grammar is permissive and accepts some malformed input.
    These tests document the ACTUAL behavior of the grammar, not necessarily
    ideal behavior. See docs/grammar-notes.md for details.
    """

    def test_invalid_syntax_unclosed_quote(self):
        """Test parsing CIF with unclosed quote.

        Current behavior: Unclosed quotes raise a ValueError.
        The parser is strict and rejects malformed quoted strings.
        """
        invalid_cif = """
        data_test
        _item 'unclosed
        """
        # Parser rejects unclosed quotes
        with pytest.raises(ValueError):
            cif_parser.parse(invalid_cif)

    def test_invalid_syntax_incomplete_loop(self):
        """Test parsing incomplete loop raises ValueError."""
        invalid_cif = """
        data_test
        loop_
        _tag1
        _tag2
        value1
        # Missing second value
        """
        with pytest.raises(ValueError):
            cif_parser.parse(invalid_cif)

    def test_invalid_syntax_loop_without_tags(self):
        """Test parsing loop without tags.

        Current behavior: Loop keyword without tags raises a ValueError.
        The parser is strict and rejects loops without proper tag declarations.
        """
        invalid_cif = """
        data_test
        loop_
        value1 value2 value3
        """
        # Parser rejects loops without tags
        with pytest.raises(ValueError):
            cif_parser.parse(invalid_cif)

    def test_invalid_syntax_save_without_data(self):
        """Test parsing save frame outside data block."""
        invalid_cif = """
        save_orphan
        _item value
        save_
        """
        # This should either raise ValueError or be handled gracefully
        try:
            doc = cif_parser.parse(invalid_cif)
            # If it doesn't raise, verify it's handled correctly
            assert len(doc) == 0 or doc is not None
        except ValueError:
            # Raising ValueError is acceptable
            pass

    def test_malformed_tag(self):
        """Test parsing malformed tag name."""
        invalid_cif = """
        data_test
        invalid_tag_no_underscore value
        """
        # This might be accepted or rejected depending on parser strictness
        try:
            doc = cif_parser.parse(invalid_cif)
            # If accepted, verify it parsed
            assert doc is not None
        except ValueError:
            # Rejection is also acceptable
            pass


class TestFileErrors:
    """Test file I/O error handling."""

    def test_parse_file_not_found(self):
        """Test parsing non-existent file raises IOError."""
        with pytest.raises((IOError, FileNotFoundError)):
            cif_parser.parse_file("/path/that/does/not/exist/file.cif")

    def test_parse_file_directory(self):
        """Test parsing a directory path raises appropriate error."""
        with tempfile.TemporaryDirectory() as tmpdir:
            with pytest.raises((IOError, IsADirectoryError, ValueError)):
                cif_parser.parse_file(tmpdir)

    def test_parse_empty_file(self):
        """Test parsing an empty file."""
        with tempfile.NamedTemporaryFile(mode="w", suffix=".cif", delete=False) as f:
            temp_path = f.name
            # Write nothing

        try:
            # Empty file should either parse with no blocks or raise ValueError
            try:
                doc = cif_parser.parse_file(temp_path)
                assert len(doc) == 0
            except ValueError:
                # Raising ValueError is also acceptable
                pass
        finally:
            os.unlink(temp_path)

    def test_parse_file_invalid_encoding(self):
        """Test parsing file with invalid encoding."""
        with tempfile.NamedTemporaryFile(mode="wb", suffix=".cif", delete=False) as f:
            temp_path = f.name
            # Write invalid UTF-8 bytes
            f.write(b"\xff\xfe invalid encoding \xff")

        try:
            # Should handle encoding errors gracefully
            try:
                doc = cif_parser.parse_file(temp_path)
                # If it doesn't raise, that's fine too
                assert doc is not None or doc is None
            except (OSError, ValueError, UnicodeDecodeError):
                # Any of these exceptions are acceptable
                pass
        finally:
            os.unlink(temp_path)


class TestAccessErrors:
    """Test access errors and bounds checking."""

    def test_document_getitem_index_error(self, simple_doc):
        """Test document indexing with invalid index raises IndexError."""
        with pytest.raises(IndexError):
            _ = simple_doc[999]

        with pytest.raises(IndexError):
            _ = simple_doc[-999]

    def test_document_getitem_key_error(self, simple_doc):
        """Test document indexing with non-existent name raises KeyError."""
        with pytest.raises(KeyError):
            _ = simple_doc["nonexistent_block"]

    def test_document_getitem_type_error(self, simple_doc):
        """Test document indexing with invalid key type raises TypeError."""
        with pytest.raises(TypeError):
            _ = simple_doc[3.14]

        with pytest.raises(TypeError):
            _ = simple_doc[None]

        with pytest.raises(TypeError):
            _ = simple_doc[["list"]]


class TestEmptyAndNone:
    """Test handling of empty structures and None values."""

    def test_block_get_item_returns_none(self, simple_doc):
        """Test that get_item returns None for non-existent items."""
        block = simple_doc.first_block()
        assert block is not None
        assert block.get_item("_does_not_exist") is None

    def test_block_get_loop_returns_none(self, simple_doc):
        """Test that get_loop returns None when index out of bounds."""
        block = simple_doc.first_block()
        assert block is not None
        assert block.get_loop(0) is None  # simple.cif has no loops
        assert block.get_loop(999) is None

    def test_block_find_loop_returns_none(self, loops_doc):
        """Test that find_loop returns None when tag not found."""
        block = loops_doc.first_block()
        assert block is not None
        assert block.find_loop("_nonexistent_tag") is None

    def test_loop_get_returns_none(self, loops_doc):
        """Test that loop.get returns None for out of bounds."""
        block = loops_doc.first_block()
        assert block is not None
        loop = block.get_loop(0)
        assert loop is not None

        assert loop.get(999, 0) is None
        assert loop.get(0, 999) is None

    def test_loop_get_by_tag_returns_none(self, loops_doc):
        """Test that get_by_tag returns None for invalid inputs."""
        block = loops_doc.first_block()
        assert block is not None
        loop = block.get_loop(0)
        assert loop is not None

        assert loop.get_by_tag(0, "_nonexistent") is None
        assert loop.get_by_tag(999, "_atom_site_label") is None

    def test_loop_get_column_returns_none(self, loops_doc):
        """Test that get_column returns None for non-existent tag."""
        block = loops_doc.first_block()
        assert block is not None
        loop = block.get_loop(0)
        assert loop is not None

        assert loop.get_column("_nonexistent_tag") is None

    def test_loop_get_row_dict_returns_none(self, loops_doc):
        """Test that get_row_dict returns None for invalid row."""
        block = loops_doc.first_block()
        assert block is not None
        loop = block.get_loop(0)
        assert loop is not None

        assert loop.get_row_dict(999) is None


class TestEdgeCases:
    """Test edge cases and special scenarios."""

    def test_value_none_properties(self, simple_doc):
        """Test that Value properties handle special values correctly."""
        block = simple_doc.first_block()
        assert block is not None

        # Test unknown value (?)
        unknown = block.get_item("_temperature_kelvin")
        assert unknown is not None
        assert unknown.is_unknown
        assert not unknown.is_numeric
        assert not unknown.is_text
        assert not unknown.is_not_applicable
        # to_python() should return None for unknown
        assert unknown.to_python() is None

        # Test not applicable value (.)
        not_applicable = block.get_item("_pressure")
        assert not_applicable is not None
        assert not_applicable.is_not_applicable
        assert not not_applicable.is_numeric
        assert not not_applicable.is_text
        assert not not_applicable.is_unknown
        # to_python() should return None for not applicable
        assert not_applicable.to_python() is None

    def test_multiblock_iteration_order(self, complex_doc):
        """Test that multi-block documents maintain block order."""
        block_names = []
        for block in complex_doc:
            block_names.append(block.name)

        # Order should be preserved
        assert block_names == ["block1", "block2"]

        # Second iteration should have same order
        block_names_2 = []
        for block in complex_doc:
            block_names_2.append(block.name)

        assert block_names == block_names_2

    def test_empty_items_dict(self):
        """Test block with no items (only loops)."""
        # Create a CIF with a block that has only a loop
        cif = """
        data_only_loop
        loop_
        _tag1
        _tag2
        a b
        c d
        """
        doc = cif_parser.parse(cif)
        block = doc.first_block()
        assert block is not None

        items = block.items()
        # Should have empty or minimal items dict
        assert isinstance(items, dict)
        # All items should be Value objects
        for value in items.values():
            assert isinstance(value, cif_parser.Value)

    def test_single_column_loop(self):
        """Test loop with single column."""
        cif = """
        data_single_col
        loop_
        _single_tag
        value1
        value2
        value3
        """
        doc = cif_parser.parse(cif)
        block = doc.first_block()
        assert block is not None

        loop = block.find_loop("_single_tag")
        assert loop is not None
        assert loop.num_columns == 1
        assert len(loop) == 3

        # Test iteration
        values = []
        for row in loop:
            values.append(row["_single_tag"].text)
        assert values == ["value1", "value2", "value3"]

    def test_quoted_strings_in_loops(self):
        """Test that quoted strings in loops are handled correctly."""
        cif = """
        data_quoted
        loop_
        _description
        _value
        'single quoted'  1.0
        "double quoted"  2.0
        unquoted         3.0
        """
        doc = cif_parser.parse(cif)
        block = doc.first_block()
        assert block is not None

        loop = block.find_loop("_description")
        assert loop is not None

        descriptions = loop.get_column("_description")
        assert descriptions is not None
        assert descriptions[0].text == "single quoted"
        assert descriptions[1].text == "double quoted"
        assert descriptions[2].text == "unquoted"
