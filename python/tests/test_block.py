"""Tests for the Block class."""

import cif_parser


class TestBlockProperties:
    """Test block properties."""

    def test_name_property(self, simple_doc, loops_doc):
        """Test block name property."""
        simple_block = simple_doc.first_block()
        assert simple_block is not None
        assert simple_block.name == "simple"

        loops_block = loops_doc.first_block()
        assert loops_block is not None
        assert loops_block.name == "loops"

    def test_item_keys_property(self, simple_doc):
        """Test item_keys property returns list of tag names."""
        block = simple_doc.first_block()
        assert block is not None

        keys = block.item_keys
        assert isinstance(keys, list)
        assert len(keys) > 0

        # Check for known items from simple.cif
        assert "_cell_length_a" in keys
        assert "_cell_length_b" in keys
        assert "_title" in keys

    def test_num_loops_property(self, loops_doc):
        """Test num_loops property."""
        block = loops_doc.first_block()
        assert block is not None
        # loops.cif has 2 loops
        assert block.num_loops == 2

    def test_num_loops_zero(self, simple_doc):
        """Test num_loops when block has no loops."""
        block = simple_doc.first_block()
        assert block is not None
        assert block.num_loops == 0

    def test_num_frames_property(self, complex_doc):
        """Test num_frames property."""
        # block1 in complex.cif has 1 save frame
        block1 = complex_doc.get_block_by_name("block1")
        assert block1 is not None
        assert block1.num_frames == 1

        # block2 has no save frames
        block2 = complex_doc.get_block_by_name("block2")
        assert block2 is not None
        assert block2.num_frames == 0


class TestItemAccess:
    """Test item access methods."""

    def test_get_item(self, simple_doc):
        """Test get_item() method."""
        block = simple_doc.first_block()
        assert block is not None

        # Get numeric value
        cell_a = block.get_item("_cell_length_a")
        assert cell_a is not None
        assert cell_a.is_numeric
        assert cell_a.numeric == 10.0

        # Get text value
        title = block.get_item("_title")
        assert title is not None
        assert title.is_text
        assert title.text == "Simple Test Structure"

        # Get unknown value
        temp = block.get_item("_temperature_kelvin")
        assert temp is not None
        assert temp.is_unknown

    def test_get_item_not_found(self, simple_doc):
        """Test get_item() returns None for non-existent key."""
        block = simple_doc.first_block()
        assert block is not None

        result = block.get_item("_nonexistent_tag")
        assert result is None

    def test_items_method(self, simple_doc):
        """Test items() method returns dictionary."""
        block = simple_doc.first_block()
        assert block is not None

        items_dict = block.items()
        assert isinstance(items_dict, dict)
        assert len(items_dict) > 0

        # Check that keys match item_keys
        assert set(items_dict.keys()) == set(block.item_keys)

        # Check values are Value objects
        for value in items_dict.values():
            assert isinstance(value, cif_parser.Value)


class TestLoopAccess:
    """Test loop access methods."""

    def test_get_loop_by_index(self, loops_doc):
        """Test get_loop() with valid index."""
        block = loops_doc.first_block()
        assert block is not None

        # Get first loop (atom_site loop with 6 columns)
        loop0 = block.get_loop(0)
        assert loop0 is not None
        assert loop0.num_columns == 6
        assert "_atom_site_label" in loop0.tags

        # Get second loop (bond loop with 2 columns)
        loop1 = block.get_loop(1)
        assert loop1 is not None
        assert loop1.num_columns == 2
        assert "_bond_type" in loop1.tags

    def test_get_loop_invalid_index(self, loops_doc):
        """Test get_loop() with invalid index returns None."""
        block = loops_doc.first_block()
        assert block is not None

        assert block.get_loop(999) is None

    def test_get_loop_no_loops(self, simple_doc):
        """Test get_loop() on block without loops."""
        block = simple_doc.first_block()
        assert block is not None

        assert block.get_loop(0) is None

    def test_find_loop(self, loops_doc):
        """Test find_loop() finds loop containing tag."""
        block = loops_doc.first_block()
        assert block is not None

        # Find atom_site loop
        atom_loop = block.find_loop("_atom_site_label")
        assert atom_loop is not None
        assert "_atom_site_label" in atom_loop.tags
        assert "_atom_site_fract_x" in atom_loop.tags

        # Find bond loop
        bond_loop = block.find_loop("_bond_type")
        assert bond_loop is not None
        assert "_bond_type" in bond_loop.tags
        assert "_bond_length" in bond_loop.tags

    def test_find_loop_not_found(self, loops_doc):
        """Test find_loop() returns None when tag not in any loop."""
        block = loops_doc.first_block()
        assert block is not None

        result = block.find_loop("_nonexistent_tag")
        assert result is None

    def test_find_loop_no_loops(self, simple_doc):
        """Test find_loop() on block without loops."""
        block = simple_doc.first_block()
        assert block is not None

        result = block.find_loop("_cell_length_a")
        assert result is None

    def test_get_loop_tags(self, loops_doc):
        """Test get_loop_tags() returns all tags from all loops."""
        block = loops_doc.first_block()
        assert block is not None

        all_tags = block.get_loop_tags()
        assert isinstance(all_tags, list)

        # Should have tags from both loops
        assert "_atom_site_label" in all_tags
        assert "_atom_site_fract_x" in all_tags
        assert "_bond_type" in all_tags
        assert "_bond_length" in all_tags

    def test_get_loop_tags_no_loops(self, simple_doc):
        """Test get_loop_tags() on block without loops."""
        block = simple_doc.first_block()
        assert block is not None

        tags = block.get_loop_tags()
        assert isinstance(tags, list)
        assert len(tags) == 0


class TestFrameAccess:
    """Test save frame access methods."""

    def test_get_frame(self, complex_doc):
        """Test get_frame() method."""
        block = complex_doc.get_block_by_name("block1")
        assert block is not None

        frame = block.get_frame(0)
        assert frame is not None
        assert frame.name == "frame1"

    def test_get_frame_invalid_index(self, complex_doc):
        """Test get_frame() with invalid index returns None."""
        block = complex_doc.get_block_by_name("block1")
        assert block is not None

        assert block.get_frame(999) is None

    def test_get_frame_no_frames(self, simple_doc):
        """Test get_frame() on block without frames."""
        block = simple_doc.first_block()
        assert block is not None

        assert block.get_frame(0) is None


class TestBlockStringRepresentation:
    """Test string representation methods."""

    def test_str(self, simple_doc):
        """Test __str__ method."""
        block = simple_doc.first_block()
        assert block is not None

        s = str(block)
        assert isinstance(s, str)
        assert len(s) > 0

    def test_repr(self, simple_doc):
        """Test __repr__ method."""
        block = simple_doc.first_block()
        assert block is not None

        r = repr(block)
        assert isinstance(r, str)
        assert len(r) > 0
        # repr should contain useful debug info
        assert "Block" in r or "block" in r.lower()
