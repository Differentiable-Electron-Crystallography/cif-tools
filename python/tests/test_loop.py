"""Tests for the Loop class."""

import pytest

import cif_parser


@pytest.fixture
def atom_loop(loops_doc):
    """Fixture providing the atom_site loop from loops.cif."""
    block = loops_doc.first_block()
    assert block is not None
    loop = block.find_loop("_atom_site_label")
    assert loop is not None
    return loop


@pytest.fixture
def bond_loop(loops_doc):
    """Fixture providing the bond loop from loops.cif."""
    block = loops_doc.first_block()
    assert block is not None
    loop = block.find_loop("_bond_type")
    assert loop is not None
    return loop


class TestLoopProperties:
    """Test loop properties."""

    def test_tags_property(self, atom_loop):
        """Test tags property returns list of column tags."""
        tags = atom_loop.tags
        assert isinstance(tags, list)
        assert len(tags) == 6
        assert "_atom_site_label" in tags
        assert "_atom_site_type_symbol" in tags
        assert "_atom_site_fract_x" in tags
        assert "_atom_site_fract_y" in tags
        assert "_atom_site_fract_z" in tags
        assert "_atom_site_occupancy" in tags

    def test_num_columns_property(self, atom_loop, bond_loop):
        """Test num_columns property."""
        assert atom_loop.num_columns == 6
        assert bond_loop.num_columns == 2

    def test_is_empty_false(self, atom_loop):
        """Test is_empty() returns False for non-empty loop."""
        assert not atom_loop.is_empty()

    def test_len(self, atom_loop, bond_loop):
        """Test __len__ returns number of rows."""
        # atom_site loop has 5 rows
        assert len(atom_loop) == 5
        # bond loop has 3 rows
        assert len(bond_loop) == 3


class TestPositionalAccess:
    """Test positional access methods."""

    def test_get_by_row_and_col(self, atom_loop):
        """Test get() method with row and column indices."""
        # First row, first column (label)
        val = atom_loop.get(0, 0)
        assert val is not None
        assert val.is_text
        assert val.text == "C1"

        # Second row, second column (type_symbol)
        val = atom_loop.get(1, 1)
        assert val is not None
        assert val.is_text
        assert val.text == "C"

        # Third row, third column (fract_x)
        val = atom_loop.get(2, 2)
        assert val is not None
        assert val.is_numeric
        assert val.numeric == pytest.approx(0.3456)

    def test_get_out_of_bounds(self, atom_loop):
        """Test get() returns None for out of bounds indices."""
        # Invalid row
        assert atom_loop.get(999, 0) is None

        # Invalid column
        assert atom_loop.get(0, 999) is None

    def test_get_by_tag(self, atom_loop):
        """Test get_by_tag() method."""
        # First row, label column
        val = atom_loop.get_by_tag(0, "_atom_site_label")
        assert val is not None
        assert val.text == "C1"

        # Third row (N1), occupancy column
        val = atom_loop.get_by_tag(2, "_atom_site_occupancy")
        assert val is not None
        assert val.is_numeric
        assert val.numeric == pytest.approx(0.95)

        # Last row, type_symbol column
        val = atom_loop.get_by_tag(4, "_atom_site_type_symbol")
        assert val is not None
        assert val.text == "O"

    def test_get_by_tag_not_found(self, atom_loop):
        """Test get_by_tag() returns None for non-existent tag."""
        result = atom_loop.get_by_tag(0, "_nonexistent_tag")
        assert result is None

    def test_get_by_tag_invalid_row(self, atom_loop):
        """Test get_by_tag() returns None for invalid row."""
        assert atom_loop.get_by_tag(999, "_atom_site_label") is None


class TestColumnAccess:
    """Test column access methods."""

    def test_get_column(self, atom_loop):
        """Test get_column() method."""
        labels = atom_loop.get_column("_atom_site_label")
        assert labels is not None
        assert len(labels) == 5
        assert all(isinstance(v, cif_parser.Value) for v in labels)

        # Check values
        assert labels[0].text == "C1"
        assert labels[1].text == "C2"
        assert labels[2].text == "N1"
        assert labels[3].text == "O1"
        assert labels[4].text == "O2"

    def test_get_column_numeric(self, atom_loop):
        """Test get_column() with numeric values."""
        x_coords = atom_loop.get_column("_atom_site_fract_x")
        assert x_coords is not None
        assert len(x_coords) == 5

        assert x_coords[0].numeric == pytest.approx(0.1234)
        assert x_coords[1].numeric == pytest.approx(0.2345)
        assert x_coords[2].numeric == pytest.approx(0.3456)

    def test_get_column_not_found(self, atom_loop):
        """Test get_column() returns None for non-existent tag."""
        result = atom_loop.get_column("_nonexistent_tag")
        assert result is None


class TestRowAccess:
    """Test row access methods."""

    def test_get_row_dict(self, atom_loop):
        """Test get_row_dict() method."""
        row0 = atom_loop.get_row_dict(0)
        assert row0 is not None
        assert isinstance(row0, dict)
        assert len(row0) == 6

        # Check keys
        assert "_atom_site_label" in row0
        assert "_atom_site_type_symbol" in row0
        assert "_atom_site_fract_x" in row0

        # Check values
        assert row0["_atom_site_label"].text == "C1"
        assert row0["_atom_site_type_symbol"].text == "C"
        assert row0["_atom_site_fract_x"].numeric == pytest.approx(0.1234)

    def test_get_row_dict_invalid_row(self, atom_loop):
        """Test get_row_dict() returns None for invalid row."""
        assert atom_loop.get_row_dict(999) is None

    def test_rows_method(self, bond_loop):
        """Test rows() method returns list of lists."""
        rows = bond_loop.rows()
        assert isinstance(rows, list)
        assert len(rows) == 3

        # Each row should be a list of values
        assert all(isinstance(row, list) for row in rows)
        assert all(len(row) == 2 for row in rows)

        # Check first row
        assert rows[0][0].text == "single"
        assert rows[0][1].numeric == pytest.approx(1.54)

        # Check second row
        assert rows[1][0].text == "double"
        assert rows[1][1].numeric == pytest.approx(1.34)


class TestLoopIteration:
    """Test loop iteration."""

    def test_iteration_basic(self, bond_loop):
        """Test basic iteration over loop."""
        rows_list = []
        for row in bond_loop:
            rows_list.append(row)

        assert len(rows_list) == 3
        # Each row should be a dict
        assert all(isinstance(row, dict) for row in rows_list)

    def test_iteration_values(self, bond_loop):
        """Test iteration yields correct values."""
        bond_types = []
        bond_lengths = []

        for row in bond_loop:
            bond_types.append(row["_bond_type"].text)
            bond_lengths.append(row["_bond_length"].numeric)

        assert bond_types == ["single", "double", "triple"]
        assert bond_lengths[0] == pytest.approx(1.54)
        assert bond_lengths[1] == pytest.approx(1.34)
        assert bond_lengths[2] == pytest.approx(1.20)

    def test_iteration_large_loop(self, atom_loop):
        """Test iteration over larger loop."""
        count = 0
        for row in atom_loop:
            assert isinstance(row, dict)
            assert len(row) == 6
            count += 1

        assert count == 5

    def test_multiple_iterations(self, bond_loop):
        """Test that loop can be iterated multiple times."""
        # First iteration
        first_pass = list(bond_loop)
        assert len(first_pass) == 3

        # Second iteration should work the same
        second_pass = list(bond_loop)
        assert len(second_pass) == 3

        # Values should be the same
        for i in range(3):
            assert first_pass[i]["_bond_type"].text == second_pass[i]["_bond_type"].text


class TestLoopStringRepresentation:
    """Test string representation methods."""

    def test_str(self, atom_loop):
        """Test __str__ method."""
        s = str(atom_loop)
        assert isinstance(s, str)
        assert len(s) > 0

    def test_repr(self, atom_loop):
        """Test __repr__ method."""
        r = repr(atom_loop)
        assert isinstance(r, str)
        assert len(r) > 0
        # repr should contain useful debug info
        assert "Loop" in r or "loop" in r.lower()
