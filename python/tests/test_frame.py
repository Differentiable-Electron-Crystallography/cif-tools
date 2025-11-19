"""Tests for the Frame class (save frames)."""

import pytest

import cif_parser


@pytest.fixture
def frame1(complex_doc):
    """Fixture providing frame1 from complex.cif."""
    block = complex_doc.get_block_by_name("block1")
    assert block is not None
    frame = block.get_frame(0)
    assert frame is not None
    return frame


class TestFrameProperties:
    """Test frame properties."""

    def test_name_property(self, frame1):
        """Test name property."""
        assert frame1.name == "frame1"

    def test_item_keys_property(self, frame1):
        """Test item_keys property."""
        keys = frame1.item_keys
        assert isinstance(keys, list)
        assert len(keys) > 0

        # Check for known items from complex.cif save frame
        assert "_frame_category" in keys
        assert "_frame_id" in keys
        assert "_restraint_type" in keys

    def test_num_loops_property(self, frame1):
        """Test num_loops property."""
        # frame1 has one loop (_restraint_atom1, _restraint_atom2, _restraint_distance)
        assert frame1.num_loops == 1

    def test_num_loops_zero(self, complex_doc):
        """Test num_loops when frame has no loops."""
        # Create a test to verify behavior if we had a frame without loops
        # For now, we know frame1 has 1 loop, so this just documents the API
        block = complex_doc.get_block_by_name("block1")
        assert block is not None
        frame = block.get_frame(0)
        assert frame is not None
        assert frame.num_loops >= 0  # Should always be non-negative


class TestItemAccess:
    """Test item access methods."""

    def test_get_item(self, frame1):
        """Test get_item() method."""
        # Get text values
        category = frame1.get_item("_frame_category")
        assert category is not None
        assert category.is_text
        assert category.text == "restraints"

        frame_id = frame1.get_item("_frame_id")
        assert frame_id is not None
        assert frame_id.is_text
        assert frame_id.text == "frame1"

        restraint_type = frame1.get_item("_restraint_type")
        assert restraint_type is not None
        assert restraint_type.is_text
        assert restraint_type.text == "distance"

    def test_get_item_not_found(self, frame1):
        """Test get_item() returns None for non-existent key."""
        result = frame1.get_item("_nonexistent_tag")
        assert result is None

    def test_items_method(self, frame1):
        """Test items() method returns dictionary."""
        items_dict = frame1.items()
        assert isinstance(items_dict, dict)
        assert len(items_dict) > 0

        # Check that keys match item_keys
        assert set(items_dict.keys()) == set(frame1.item_keys)

        # Check values are Value objects
        for value in items_dict.values():
            assert isinstance(value, cif_parser.Value)


class TestLoopAccess:
    """Test loop access methods."""

    def test_get_loop_by_index(self, frame1):
        """Test get_loop() method."""
        loop = frame1.get_loop(0)
        assert loop is not None
        assert loop.num_columns == 3
        assert "_restraint_atom1" in loop.tags
        assert "_restraint_atom2" in loop.tags
        assert "_restraint_distance" in loop.tags

    def test_get_loop_invalid_index(self, frame1):
        """Test get_loop() with invalid index returns None."""
        assert frame1.get_loop(999) is None

    def test_loops_method(self, frame1):
        """Test loops property returns list of loops."""
        loops = frame1.loops
        assert isinstance(loops, list)
        assert len(loops) == 1

        loop = loops[0]
        assert isinstance(loop, cif_parser.Loop)
        assert loop.num_columns == 3


class TestFrameLoopContent:
    """Test accessing content within frame loops."""

    def test_loop_iteration(self, frame1):
        """Test iterating over loop within frame."""
        loop = frame1.get_loop(0)
        assert loop is not None

        rows = list(loop)
        assert len(rows) == 2  # Two restraints in the loop

        # First restraint: C1-C2, 1.54
        assert rows[0]["_restraint_atom1"].text == "C1"
        assert rows[0]["_restraint_atom2"].text == "C2"
        assert rows[0]["_restraint_distance"].numeric == pytest.approx(1.54)

        # Second restraint: C2-C3, 1.54
        assert rows[1]["_restraint_atom1"].text == "C2"
        assert rows[1]["_restraint_atom2"].text == "C3"
        assert rows[1]["_restraint_distance"].numeric == pytest.approx(1.54)

    def test_loop_column_access(self, frame1):
        """Test accessing columns in frame loop."""
        loop = frame1.get_loop(0)
        assert loop is not None

        atom1_col = loop.get_column("_restraint_atom1")
        assert atom1_col is not None
        assert len(atom1_col) == 2
        assert atom1_col[0].text == "C1"
        assert atom1_col[1].text == "C2"

        distance_col = loop.get_column("_restraint_distance")
        assert distance_col is not None
        assert len(distance_col) == 2
        assert all(v.is_numeric for v in distance_col)


class TestFrameStringRepresentation:
    """Test string representation methods."""

    def test_str(self, frame1):
        """Test __str__ method."""
        s = str(frame1)
        assert isinstance(s, str)
        assert len(s) > 0

    def test_repr(self, frame1):
        """Test __repr__ method."""
        r = repr(frame1)
        assert isinstance(r, str)
        assert len(r) > 0
        # repr should contain useful debug info
        assert "Frame" in r or "frame" in r.lower()


class TestFrameIntegration:
    """Test frame integration with parent block."""

    def test_frame_accessed_through_block(self, complex_doc):
        """Test accessing frame through its parent block."""
        block = complex_doc.get_block_by_name("block1")
        assert block is not None

        # Block should report having 1 frame
        assert block.num_frames == 1

        # Access frame
        frame = block.get_frame(0)
        assert frame is not None
        assert frame.name == "frame1"

        # Verify frame content
        assert frame.num_loops == 1
        assert len(frame.item_keys) > 0

    def test_block_without_frames(self, complex_doc):
        """Test block without save frames."""
        block2 = complex_doc.get_block_by_name("block2")
        assert block2 is not None
        assert block2.num_frames == 0
        assert block2.get_frame(0) is None
