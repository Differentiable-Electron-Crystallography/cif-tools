"""Tests for the Document class and parsing functions."""

import pytest

import cif_parser


class TestParsing:
    """Test parsing functions."""

    def test_parse_function(self, simple_cif_content):
        """Test module-level parse() function."""
        doc = cif_parser.parse(simple_cif_content)
        assert doc is not None
        assert len(doc) == 1

    def test_document_parse_static_method(self, simple_cif_content):
        """Test Document.parse() static method."""
        doc = cif_parser.Document.parse(simple_cif_content)
        assert doc is not None
        assert len(doc) == 1

    def test_parse_file_function(self, simple_cif):
        """Test module-level parse_file() function."""
        doc = cif_parser.parse_file(str(simple_cif))
        assert doc is not None
        assert len(doc) == 1
        block = doc.first_block()
        assert block is not None
        assert block.name == "simple"

    def test_document_from_file_static_method(self, simple_cif):
        """Test Document.from_file() static method."""
        doc = cif_parser.Document.from_file(str(simple_cif))
        assert doc is not None
        assert len(doc) == 1
        block = doc.first_block()
        assert block is not None
        assert block.name == "simple"

    def test_parse_multiblock_document(self, complex_cif):
        """Test parsing a document with multiple blocks."""
        doc = cif_parser.parse_file(str(complex_cif))
        assert len(doc) == 2
        assert doc.block_names == ["block1", "block2"]

    def test_parse_empty_string(self):
        """Test parsing an empty string."""
        # Empty CIF should parse but have no blocks
        try:
            doc = cif_parser.parse("")
            assert len(doc) == 0
        except ValueError:
            # If parser doesn't allow empty, that's also acceptable
            pass

    def test_parse_invalid_cif_syntax(self):
        """Test parsing invalid CIF syntax.

        Current behavior: Incomplete loops (tags without values) create empty loops.
        The grammar is permissive - the loop_block rule has `| &(keyword | EOI)`
        which allows loops to end without values when EOF or next keyword is reached.
        """
        invalid_cif = """
        data_invalid
        loop_
        _tag1
        # Missing loop values
        """
        # Grammar allows this - creates a loop with 1 tag and 0 rows
        doc = cif_parser.parse(invalid_cif)
        assert len(doc) == 1
        block = doc.first_block()
        assert block is not None
        assert block.num_loops == 1
        loop = block.get_loop(0)
        assert loop is not None
        assert len(loop.tags) == 1
        assert loop.tags[0] == "_tag1"
        assert len(loop) == 0  # Empty loop - no data rows

    def test_parse_file_not_found(self):
        """Test that parsing non-existent file raises IOError."""
        with pytest.raises((IOError, FileNotFoundError)):
            cif_parser.parse_file("/nonexistent/path/file.cif")


class TestBlockAccess:
    """Test block access methods."""

    def test_first_block(self, simple_doc):
        """Test first_block() method."""
        block = simple_doc.first_block()
        assert block is not None
        assert block.name == "simple"

    def test_first_block_empty_document(self):
        """Test first_block() on empty document."""
        try:
            doc = cif_parser.parse("")
            assert doc.first_block() is None
        except ValueError:
            # If empty document not allowed, skip this test
            pass

    def test_get_block_by_index(self, complex_doc):
        """Test get_block() with valid index."""
        block0 = complex_doc.get_block(0)
        assert block0 is not None
        assert block0.name == "block1"

        block1 = complex_doc.get_block(1)
        assert block1 is not None
        assert block1.name == "block2"

    def test_get_block_invalid_index(self, simple_doc):
        """Test get_block() with invalid index returns None."""
        assert simple_doc.get_block(999) is None

    def test_get_block_by_name(self, complex_doc):
        """Test get_block_by_name() method."""
        block1 = complex_doc.get_block_by_name("block1")
        assert block1 is not None
        assert block1.name == "block1"

        block2 = complex_doc.get_block_by_name("block2")
        assert block2 is not None
        assert block2.name == "block2"

    def test_get_block_by_name_not_found(self, simple_doc):
        """Test get_block_by_name() with non-existent name."""
        assert simple_doc.get_block_by_name("nonexistent") is None

    def test_getitem_by_index(self, complex_doc):
        """Test __getitem__ with integer index."""
        block0 = complex_doc[0]
        assert block0.name == "block1"

        block1 = complex_doc[1]
        assert block1.name == "block2"

    def test_getitem_by_index_out_of_bounds(self, simple_doc):
        """Test __getitem__ with out of bounds index raises IndexError."""
        with pytest.raises(IndexError):
            _ = simple_doc[999]

        with pytest.raises(IndexError):
            _ = simple_doc[-999]

    def test_getitem_by_name(self, complex_doc):
        """Test __getitem__ with string key (block name)."""
        block1 = complex_doc["block1"]
        assert block1.name == "block1"

        block2 = complex_doc["block2"]
        assert block2.name == "block2"

    def test_getitem_by_name_not_found(self, simple_doc):
        """Test __getitem__ with non-existent name raises KeyError."""
        with pytest.raises(KeyError):
            _ = simple_doc["nonexistent"]

    def test_getitem_invalid_type(self, simple_doc):
        """Test __getitem__ with invalid key type raises TypeError."""
        with pytest.raises(TypeError):
            _ = simple_doc[3.14]


class TestDocumentIteration:
    """Test document iteration and properties."""

    def test_len(self, simple_doc, complex_doc):
        """Test __len__ method."""
        assert len(simple_doc) == 1
        assert len(complex_doc) == 2

    def test_blocks_property(self, complex_doc):
        """Test blocks property returns list of blocks."""
        blocks = complex_doc.blocks
        assert len(blocks) == 2
        assert blocks[0].name == "block1"
        assert blocks[1].name == "block2"

    def test_block_names_property(self, complex_doc):
        """Test block_names property."""
        names = complex_doc.block_names
        assert names == ["block1", "block2"]

    def test_iteration(self, complex_doc):
        """Test iterating over document blocks."""
        block_names = []
        for block in complex_doc:
            block_names.append(block.name)

        assert block_names == ["block1", "block2"]

    def test_iteration_single_block(self, simple_doc):
        """Test iteration over single-block document."""
        blocks = list(simple_doc)
        assert len(blocks) == 1
        assert blocks[0].name == "simple"


class TestDocumentStringRepresentation:
    """Test string representation methods."""

    def test_str(self, simple_doc):
        """Test __str__ method."""
        s = str(simple_doc)
        assert isinstance(s, str)
        assert len(s) > 0

    def test_repr(self, simple_doc):
        """Test __repr__ method."""
        r = repr(simple_doc)
        assert isinstance(r, str)
        assert len(r) > 0
        # repr should be more detailed than str
        assert "Document" in r or "document" in r.lower()
