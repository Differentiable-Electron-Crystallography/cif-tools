"""Integration tests for cif_parser using shared fixtures.

These tests mirror the Rust integration tests in shared_fixtures.rs
for test parity across Rust, Python, and JavaScript.
"""

import cif_parser
import pytest

# =============================================================================
# simple.cif - Basic CIF with unknown (?) and not-applicable (.) values
# =============================================================================


def test_simple_parse(simple_cif):
    """Test parsing simple.cif."""
    doc = cif_parser.parse_file(str(simple_cif))

    assert len(doc) == 1
    assert doc.first_block().name == "simple"


def test_simple_unknown_value(simple_cif):
    """Test unknown value (?) detection."""
    doc = cif_parser.parse_file(str(simple_cif))
    block = doc.first_block()

    value = block.get_item("_temperature_kelvin")
    assert value.is_unknown


def test_simple_not_applicable_value(simple_cif):
    """Test not applicable value (.) detection."""
    doc = cif_parser.parse_file(str(simple_cif))
    block = doc.first_block()

    value = block.get_item("_pressure")
    assert value.is_not_applicable


def test_simple_text_value(simple_cif):
    """Test text value access."""
    doc = cif_parser.parse_file(str(simple_cif))
    block = doc.first_block()

    value = block.get_item("_title")
    assert value.text == "Simple Test Structure"


def test_simple_numeric_value(simple_cif):
    """Test numeric value access."""
    doc = cif_parser.parse_file(str(simple_cif))
    block = doc.first_block()

    value = block.get_item("_cell_length_a")
    assert value.numeric == 10.0


# =============================================================================
# loops.cif - Multiple loops (atom sites, bonds)
# =============================================================================


def test_loops_parse(loops_cif):
    """Test parsing loops.cif."""
    doc = cif_parser.parse_file(str(loops_cif))

    assert len(doc) == 1
    assert doc.first_block().name == "loops"


def test_loops_multiple_loops(loops_cif):
    """Test multiple loops in a block."""
    doc = cif_parser.parse_file(str(loops_cif))
    block = doc.first_block()

    # Should have 2 loops: atom_site and bond
    assert block.num_loops == 2


def test_loops_atom_site_loop(loops_cif):
    """Test atom site loop access."""
    doc = cif_parser.parse_file(str(loops_cif))
    block = doc.first_block()

    atom_loop = block.find_loop("_atom_site_label")
    assert len(atom_loop) == 5  # C1, C2, N1, O1, O2

    # Test accessing by tag
    first_label = atom_loop.get_by_tag(0, "_atom_site_label")
    assert first_label.text == "C1"

    # Test getting a column
    x_coords = atom_loop.get_column("_atom_site_fract_x")
    assert len(x_coords) == 5


def test_loops_bond_loop(loops_cif):
    """Test bond loop access."""
    doc = cif_parser.parse_file(str(loops_cif))
    block = doc.first_block()

    bond_loop = block.find_loop("_bond_type")
    assert len(bond_loop) == 3  # single, double, triple

    first_type = bond_loop.get_by_tag(0, "_bond_type")
    assert first_type.text == "single"

    first_length = bond_loop.get_by_tag(0, "_bond_length")
    assert first_length.numeric == pytest.approx(1.54, abs=0.01)


# =============================================================================
# complex.cif - Save frames, multiple blocks
# =============================================================================


def test_complex_parse(complex_cif):
    """Test parsing complex.cif."""
    doc = cif_parser.parse_file(str(complex_cif))

    # Should have 2 data blocks
    assert len(doc) == 2


def test_complex_multiple_blocks(complex_cif):
    """Test multiple block access."""
    doc = cif_parser.parse_file(str(complex_cif))

    assert doc.get_block(0).name == "block1"
    assert doc.get_block(1).name == "block2"

    # Access by name
    block2 = doc.get_block_by_name("block2")
    assert block2.get_item("_title").text == "Second Data Block"


def test_complex_save_frame(complex_cif):
    """Test save frame access."""
    doc = cif_parser.parse_file(str(complex_cif))
    block = doc.first_block()

    # Should have 1 save frame
    assert block.num_frames == 1
    frame = block.get_frame(0)
    assert frame.name == "frame1"

    # Access frame items
    assert frame.get_item("_frame_category").text == "restraints"


# =============================================================================
# pycifrw_xanthine.cif - Uncertainty values (NumericWithUncertainty)
# =============================================================================


def test_xanthine_uncertainty_detection(xanthine_cif):
    """Test numeric with uncertainty type detection."""
    doc = cif_parser.parse_file(str(xanthine_cif))
    block = doc.first_block()

    # Cell length a has uncertainty: 10.01(11)
    cell_a = block.get_item("_cell_length_a")
    assert cell_a.is_numeric_with_uncertainty


def test_xanthine_uncertainty_value(xanthine_cif):
    """Test uncertainty value extraction."""
    doc = cif_parser.parse_file(str(xanthine_cif))
    block = doc.first_block()

    # 10.01(11) means value=10.01, uncertainty=0.11
    cell_a = block.get_item("_cell_length_a")
    assert cell_a.numeric == pytest.approx(10.01, abs=0.001)
    assert cell_a.uncertainty == pytest.approx(0.11, abs=0.001)


def test_xanthine_uncertainty_value_type(xanthine_cif):
    """Test value_type for numeric with uncertainty."""
    doc = cif_parser.parse_file(str(xanthine_cif))
    block = doc.first_block()

    cell_a = block.get_item("_cell_length_a")
    assert cell_a.value_type == "numeric_with_uncertainty"


def test_xanthine_multiple_uncertainties(xanthine_cif):
    """Test multiple values with uncertainty."""
    doc = cif_parser.parse_file(str(xanthine_cif))
    block = doc.first_block()

    # _cell_length_b: 18.23(8) -> value=18.23, uncertainty=0.08
    cell_b = block.get_item("_cell_length_b")
    assert cell_b.numeric == pytest.approx(18.23, abs=0.001)
    assert cell_b.uncertainty == pytest.approx(0.08, abs=0.001)

    # _cell_length_c: 6.93(13) -> value=6.93, uncertainty=0.13
    cell_c = block.get_item("_cell_length_c")
    assert cell_c.numeric == pytest.approx(6.93, abs=0.001)
    assert cell_c.uncertainty == pytest.approx(0.13, abs=0.001)

    # _cell_angle_beta: 107.5(9) -> value=107.5, uncertainty=0.9
    beta = block.get_item("_cell_angle_beta")
    assert beta.numeric == pytest.approx(107.5, abs=0.1)
    assert beta.uncertainty == pytest.approx(0.9, abs=0.1)


def test_xanthine_plain_numeric_no_uncertainty(xanthine_cif):
    """Test plain numeric has no uncertainty."""
    doc = cif_parser.parse_file(str(xanthine_cif))
    block = doc.first_block()

    # _cell_angle_alpha is plain 90.0 (no uncertainty)
    alpha = block.get_item("_cell_angle_alpha")
    assert alpha.is_numeric
    assert not alpha.is_numeric_with_uncertainty
    assert alpha.uncertainty is None


# =============================================================================
# crystalmaker_LuAG.cif - High precision uncertainty values
# =============================================================================


def test_luag_high_precision_uncertainty(luag_cif):
    """Test high-precision uncertainty values."""
    doc = cif_parser.parse_file(str(luag_cif))
    block = doc.first_block()

    # 11.910400(4) -> value=11.9104, uncertainty=0.000004
    cell_a = block.get_item("_cell_length_a")
    assert cell_a.numeric == pytest.approx(11.9104, abs=0.0001)
    assert cell_a.uncertainty == pytest.approx(0.000004, abs=0.0000001)


def test_luag_zero_uncertainty(luag_cif):
    """Test zero uncertainty values."""
    doc = cif_parser.parse_file(str(luag_cif))
    block = doc.first_block()

    # 90.000000(0) -> value=90.0, uncertainty=0.0
    alpha = block.get_item("_cell_angle_alpha")
    assert alpha.numeric == pytest.approx(90.0, abs=0.0001)
    assert alpha.uncertainty == pytest.approx(0.0, abs=0.0000001)


# =============================================================================
# cif2_lists.cif - CIF 2.0 list syntax
# =============================================================================


def test_cif2_lists_version(cif2_lists_cif):
    """Test CIF 2.0 version detection for lists file."""
    doc = cif_parser.parse_file(str(cif2_lists_cif))

    assert doc.is_cif2()


def test_cif2_empty_list(cif2_lists_cif):
    """Test empty list parsing."""
    doc = cif_parser.parse_file(str(cif2_lists_cif))
    block = doc.first_block()

    value = block.get_item("_empty_list")
    assert value.is_list
    py_value = value.to_python()
    assert py_value == []


def test_cif2_single_item_list(cif2_lists_cif):
    """Test single-item list parsing."""
    doc = cif_parser.parse_file(str(cif2_lists_cif))
    block = doc.first_block()

    value = block.get_item("_single_item")
    assert value.is_list
    py_value = value.to_python()
    assert len(py_value) == 1
    assert py_value[0] == 42.0


def test_cif2_numeric_list(cif2_lists_cif):
    """Test numeric list parsing."""
    doc = cif_parser.parse_file(str(cif2_lists_cif))
    block = doc.first_block()

    value = block.get_item("_numeric_list")
    assert value.is_list
    py_value = value.to_python()
    assert len(py_value) == 5
    for i, item in enumerate(py_value):
        assert item == float(i + 1)


def test_cif2_nested_list(cif2_lists_cif):
    """Test nested list parsing."""
    doc = cif_parser.parse_file(str(cif2_lists_cif))
    block = doc.first_block()

    value = block.get_item("_nested_list")
    assert value.is_list
    py_value = value.to_python()
    assert len(py_value) == 2
    # First nested list [1 2]
    assert py_value[0] == [1.0, 2.0]
    # Second nested list [3 4]
    assert py_value[1] == [3.0, 4.0]


def test_cif2_list_with_unknown(cif2_lists_cif):
    """Test list with unknown value."""
    doc = cif_parser.parse_file(str(cif2_lists_cif))
    block = doc.first_block()

    value = block.get_item("_mixed_with_unknown")
    assert value.is_list
    py_value = value.to_python()
    assert len(py_value) == 4
    assert py_value[0] == 1.0
    assert py_value[1] == 2.0
    assert py_value[2] is None  # Unknown converts to None
    assert py_value[3] == 4.0


# =============================================================================
# cif2_tables.cif - CIF 2.0 table syntax
# =============================================================================


def test_cif2_tables_version(cif2_tables_cif):
    """Test CIF 2.0 version detection for tables file."""
    doc = cif_parser.parse_file(str(cif2_tables_cif))

    assert doc.is_cif2()


def test_cif2_empty_table(cif2_tables_cif):
    """Test empty table parsing."""
    doc = cif_parser.parse_file(str(cif2_tables_cif))
    block = doc.first_block()

    value = block.get_item("_empty_table")
    assert value.is_table
    py_value = value.to_python()
    assert py_value == {}


def test_cif2_simple_table(cif2_tables_cif):
    """Test simple table parsing."""
    doc = cif_parser.parse_file(str(cif2_tables_cif))
    block = doc.first_block()

    value = block.get_item("_simple_table")
    assert value.is_table
    py_value = value.to_python()
    assert len(py_value) == 2
    assert py_value["a"] == 1.0
    assert py_value["b"] == 2.0


def test_cif2_coordinates_table(cif2_tables_cif):
    """Test coordinates table parsing."""
    doc = cif_parser.parse_file(str(cif2_tables_cif))
    block = doc.first_block()

    value = block.get_item("_coordinates")
    assert value.is_table
    py_value = value.to_python()
    assert len(py_value) == 3
    assert py_value["x"] == 1.5
    assert py_value["y"] == 2.5
    assert py_value["z"] == 3.5


def test_cif2_table_with_unknown(cif2_tables_cif):
    """Test table with unknown value."""
    doc = cif_parser.parse_file(str(cif2_tables_cif))
    block = doc.first_block()

    value = block.get_item("_with_unknown")
    assert value.is_table
    py_value = value.to_python()
    assert len(py_value) == 2
    assert py_value["value"] == 42.0
    assert py_value["error"] is None  # Unknown converts to None
