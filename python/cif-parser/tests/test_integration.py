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


# =============================================================================
# Span Tests - Source location tracking for LSP/IDE features
# =============================================================================


def test_span_basic_properties(simple_cif):
    """Test that span has all required properties."""
    doc = cif_parser.parse_file(str(simple_cif))
    block = doc.first_block()

    value = block.get_item("_cell_length_a")
    span = value.span

    # All properties should be accessible
    assert hasattr(span, "start_line")
    assert hasattr(span, "start_col")
    assert hasattr(span, "end_line")
    assert hasattr(span, "end_col")

    # Lines are 1-indexed and should be positive
    assert span.start_line >= 1
    assert span.end_line >= 1
    assert span.start_col >= 1
    assert span.end_col >= 1


def test_span_numeric_value_position(simple_cif):
    """Test span position for numeric value in simple.cif."""
    doc = cif_parser.parse_file(str(simple_cif))
    block = doc.first_block()

    # Line 2: _cell_length_a    10.0
    value = block.get_item("_cell_length_a")
    span = value.span

    assert span.start_line == 2
    assert span.end_line == 2
    # Value "10.0" should span 4 characters (end_col is exclusive)
    assert span.end_col - span.start_col == 4


def test_span_text_value_position(simple_cif):
    """Test span position for text value in simple.cif."""
    doc = cif_parser.parse_file(str(simple_cif))
    block = doc.first_block()

    # Line 8: _title 'Simple Test Structure'
    value = block.get_item("_title")
    span = value.span

    assert span.start_line == 8
    assert span.end_line == 8


def test_span_unknown_value_position(simple_cif):
    """Test span position for unknown value (?) in simple.cif."""
    doc = cif_parser.parse_file(str(simple_cif))
    block = doc.first_block()

    # Line 9: _temperature_kelvin ?
    value = block.get_item("_temperature_kelvin")
    span = value.span

    assert span.start_line == 9
    assert span.end_line == 9
    # Single character '?' (end_col is exclusive)
    assert span.end_col - span.start_col == 1


def test_span_not_applicable_value_position(simple_cif):
    """Test span position for not applicable value (.) in simple.cif."""
    doc = cif_parser.parse_file(str(simple_cif))
    block = doc.first_block()

    # Line 10: _pressure .
    value = block.get_item("_pressure")
    span = value.span

    assert span.start_line == 10
    assert span.end_line == 10
    # Single character '.' (end_col is exclusive)
    assert span.end_col - span.start_col == 1


def test_span_contains_method(simple_cif):
    """Test span.contains() method for hit testing."""
    doc = cif_parser.parse_file(str(simple_cif))
    block = doc.first_block()

    value = block.get_item("_cell_length_a")
    span = value.span

    # Position inside the span should return True
    assert span.contains(span.start_line, span.start_col)
    assert span.contains(span.end_line, span.end_col)

    # Position outside the span should return False
    assert not span.contains(span.start_line - 1, span.start_col)
    assert not span.contains(span.start_line, span.start_col - 1)
    assert not span.contains(span.end_line + 1, span.end_col)
    assert not span.contains(span.end_line, span.end_col + 1)


def test_span_str_representation(simple_cif):
    """Test span string representation."""
    doc = cif_parser.parse_file(str(simple_cif))
    block = doc.first_block()

    value = block.get_item("_cell_length_a")
    span = value.span

    # String representation should be in format "line:col-line:col" or "line:col-col"
    span_str = str(span)
    assert ":" in span_str
    assert span_str  # Not empty


def test_span_repr(simple_cif):
    """Test span repr representation."""
    doc = cif_parser.parse_file(str(simple_cif))
    block = doc.first_block()

    value = block.get_item("_cell_length_a")
    span = value.span

    # Repr should contain all the fields
    span_repr = repr(span)
    assert "start_line" in span_repr
    assert "start_col" in span_repr
    assert "end_line" in span_repr
    assert "end_col" in span_repr


def test_span_equality(simple_cif):
    """Test span equality comparison."""
    doc = cif_parser.parse_file(str(simple_cif))
    block = doc.first_block()

    # Same value should have same span
    value1 = block.get_item("_cell_length_a")
    value2 = block.get_item("_cell_length_a")
    assert value1.span == value2.span

    # Different values should have different spans
    value3 = block.get_item("_cell_length_b")
    assert value1.span != value3.span


def test_span_hashable(simple_cif):
    """Test that span can be used in sets/dicts."""
    doc = cif_parser.parse_file(str(simple_cif))
    block = doc.first_block()

    span1 = block.get_item("_cell_length_a").span
    span2 = block.get_item("_cell_length_b").span

    # Should be hashable
    span_set = {span1, span2}
    assert len(span_set) == 2

    # Same span should hash the same
    span1_again = block.get_item("_cell_length_a").span
    span_set.add(span1_again)
    assert len(span_set) == 2  # No new element added


def test_span_loop_values(loops_cif):
    """Test span for values within loops."""
    doc = cif_parser.parse_file(str(loops_cif))
    block = doc.first_block()

    atom_loop = block.find_loop("_atom_site_label")

    # First row values should be on line 11
    first_label = atom_loop.get_by_tag(0, "_atom_site_label")
    assert first_label.span.start_line == 11

    # Second row values should be on line 12
    second_label = atom_loop.get_by_tag(1, "_atom_site_label")
    assert second_label.span.start_line == 12

    # Different columns on same row should have same line but different columns
    first_x = atom_loop.get_by_tag(0, "_atom_site_fract_x")
    assert first_x.span.start_line == first_label.span.start_line
    assert first_x.span.start_col != first_label.span.start_col


def test_span_loop_column_values(loops_cif):
    """Test that loop column values have distinct spans."""
    doc = cif_parser.parse_file(str(loops_cif))
    block = doc.first_block()

    atom_loop = block.find_loop("_atom_site_label")
    labels = atom_loop.get_column("_atom_site_label")

    # Each label should have a different line
    lines = [label.span.start_line for label in labels]
    assert len(set(lines)) == len(lines)  # All unique lines

    # Lines should be consecutive (11, 12, 13, 14, 15)
    assert lines == [11, 12, 13, 14, 15]


def test_span_multiple_blocks(complex_cif):
    """Test spans across multiple data blocks."""
    doc = cif_parser.parse_file(str(complex_cif))

    block1 = doc.get_block(0)
    block2 = doc.get_block(1)

    # Values in block2 should have higher line numbers than block1
    # Both blocks have _entry_id
    entry1 = block1.get_item("_entry_id")
    entry2 = block2.get_item("_entry_id")

    assert entry2.span.start_line > entry1.span.start_line


def test_span_uncertainty_value(xanthine_cif):
    """Test span for values with uncertainty notation."""
    doc = cif_parser.parse_file(str(xanthine_cif))
    block = doc.first_block()

    # Value with uncertainty like 10.01(11) should have span covering the whole notation
    cell_a = block.get_item("_cell_length_a")
    span = cell_a.span

    assert span.start_line >= 1
    assert span.end_col > span.start_col  # Should span multiple characters


def test_span_cif2_list_values(cif2_lists_cif):
    """Test span for CIF 2.0 list values."""
    doc = cif_parser.parse_file(str(cif2_lists_cif))
    block = doc.first_block()

    # The list itself should have a span
    numeric_list = block.get_item("_numeric_list")
    assert numeric_list.span.start_line >= 1

    # Nested list should also have spans
    nested_list = block.get_item("_nested_list")
    assert nested_list.span.start_line >= 1


def test_span_cif2_table_values(cif2_tables_cif):
    """Test span for CIF 2.0 table values."""
    doc = cif_parser.parse_file(str(cif2_tables_cif))
    block = doc.first_block()

    # The table itself should have a span
    simple_table = block.get_item("_simple_table")
    assert simple_table.span.start_line >= 1

    # Coordinates table should have a span
    coords = block.get_item("_coordinates")
    assert coords.span.start_line >= 1


def test_span_class_exported():
    """Test that Span class is properly exported."""
    from cif_parser import Span

    assert Span is not None
    # Span instances come from values, not constructed directly
    # But the class should be importable for type hints


# =============================================================================
# cif2_comprehensive.cif - Comprehensive CIF 2.0 feature tests
# =============================================================================


def test_cif2_comprehensive_parse(cif2_comprehensive_cif):
    """Test parsing cif2_comprehensive.cif."""
    doc = cif_parser.parse_file(str(cif2_comprehensive_cif))

    assert len(doc) == 1
    assert doc.is_cif2()
    assert doc.first_block().name == "cif2_comprehensive"


def test_cif2_comprehensive_list_text(cif2_comprehensive_cif):
    """Test list with text values (quoted strings)."""
    doc = cif_parser.parse_file(str(cif2_comprehensive_cif))
    block = doc.first_block()

    value = block.get_item("_list_text")
    assert value.is_list
    py_value = value.to_python()
    assert py_value == ["alpha", "beta", "gamma"]


def test_cif2_comprehensive_list_mixed_types(cif2_comprehensive_cif):
    """Test list with mixed types (text and numeric)."""
    doc = cif_parser.parse_file(str(cif2_comprehensive_cif))
    block = doc.first_block()

    value = block.get_item("_list_mixed_types")
    assert value.is_list
    py_value = value.to_python()
    assert py_value == ["label1", 1.5, "label2", 2.5]


def test_cif2_comprehensive_list_deeply_nested(cif2_comprehensive_cif):
    """Test deeply nested lists."""
    doc = cif_parser.parse_file(str(cif2_comprehensive_cif))
    block = doc.first_block()

    value = block.get_item("_list_deeply_nested")
    assert value.is_list
    py_value = value.to_python()
    assert py_value == [[[1.0, 2.0], [3.0, 4.0]], [[5.0, 6.0], [7.0, 8.0]]]


def test_cif2_comprehensive_table_text(cif2_comprehensive_cif):
    """Test table with text values."""
    doc = cif_parser.parse_file(str(cif2_comprehensive_cif))
    block = doc.first_block()

    value = block.get_item("_table_text")
    assert value.is_table
    py_value = value.to_python()
    assert py_value["name"] == "test"
    assert py_value["type"] == "example"


def test_cif2_comprehensive_table_nested(cif2_comprehensive_cif):
    """Test nested tables."""
    doc = cif_parser.parse_file(str(cif2_comprehensive_cif))
    block = doc.first_block()

    value = block.get_item("_table_nested")
    assert value.is_table
    py_value = value.to_python()
    assert py_value["outer"]["inner"] == 1.0
    assert py_value["outer"]["value"] == 2.0


def test_cif2_comprehensive_table_with_list(cif2_comprehensive_cif):
    """Test table containing a list."""
    doc = cif_parser.parse_file(str(cif2_comprehensive_cif))
    block = doc.first_block()

    value = block.get_item("_table_with_list")
    assert value.is_table
    py_value = value.to_python()
    assert py_value["name"] == "vector"
    assert py_value["components"] == [1.0, 2.0, 3.0]


def test_cif2_comprehensive_list_of_tables(cif2_comprehensive_cif):
    """Test list containing tables."""
    doc = cif_parser.parse_file(str(cif2_comprehensive_cif))
    block = doc.first_block()

    value = block.get_item("_list_of_tables")
    assert value.is_list
    py_value = value.to_python()
    assert len(py_value) == 2
    assert py_value[0]["x"] == 1.0
    assert py_value[0]["y"] == 2.0
    assert py_value[1]["x"] == 3.0
    assert py_value[1]["y"] == 4.0


def test_cif2_comprehensive_complex_nested(cif2_comprehensive_cif):
    """Test complex nested structure (table with list of tables)."""
    doc = cif_parser.parse_file(str(cif2_comprehensive_cif))
    block = doc.first_block()

    value = block.get_item("_complex_nested")
    assert value.is_table
    py_value = value.to_python()
    assert py_value["count"] == 2.0
    assert len(py_value["points"]) == 2
    assert py_value["points"][0]["x"] == 0.0
    assert py_value["points"][1]["y"] == 1.0


def test_cif2_comprehensive_triple_quoted(cif2_comprehensive_cif):
    """Test triple-quoted strings."""
    doc = cif_parser.parse_file(str(cif2_comprehensive_cif))
    block = doc.first_block()

    # Single-quote triple-quoted
    value = block.get_item("_triple_single_line")
    assert value.to_python() == "This is a triple-quoted string"

    # Double-quote triple-quoted
    value = block.get_item("_triple_double_line")
    assert value.to_python() == "This is also triple-quoted"

    # With embedded quotes
    value = block.get_item("_triple_with_quotes")
    assert value.to_python() == "String with 'embedded' quotes"

    value = block.get_item("_triple_with_double_quotes")
    assert value.to_python() == 'String with "embedded" quotes'


def test_cif2_comprehensive_triple_quoted_multiline(cif2_comprehensive_cif):
    """Test multi-line triple-quoted string."""
    doc = cif_parser.parse_file(str(cif2_comprehensive_cif))
    block = doc.first_block()

    value = block.get_item("_triple_multiline")
    py_value = value.to_python()
    assert "Line one" in py_value
    assert "Line two" in py_value
    assert "Line three" in py_value


def test_cif2_comprehensive_list_with_triple(cif2_comprehensive_cif):
    """Test list containing triple-quoted strings."""
    doc = cif_parser.parse_file(str(cif2_comprehensive_cif))
    block = doc.first_block()

    value = block.get_item("_list_with_triple")
    assert value.is_list
    py_value = value.to_python()
    assert py_value == ["first", "second"]


def test_cif2_comprehensive_unicode(cif2_comprehensive_cif):
    """Test Unicode values."""
    doc = cif_parser.parse_file(str(cif2_comprehensive_cif))
    block = doc.first_block()

    # Greek letters
    value = block.get_item("_unicode_greek")
    assert value.to_python() == "αβγδεζηθ"

    # Mathematical symbols
    value = block.get_item("_unicode_math")
    assert value.to_python() == "∑∏∫∂∇"

    # Angstrom and degree symbols
    value = block.get_item("_unicode_units")
    assert "Å" in value.to_python()
    assert "°" in value.to_python()

    # Accented characters
    value = block.get_item("_unicode_accents")
    py_value = value.to_python()
    assert "Müller" in py_value
    assert "Böhm" in py_value
    assert "Señor" in py_value


def test_cif2_comprehensive_unicode_in_list(cif2_comprehensive_cif):
    """Test Unicode values in a list."""
    doc = cif_parser.parse_file(str(cif2_comprehensive_cif))
    block = doc.first_block()

    value = block.get_item("_list_unicode")
    assert value.is_list
    py_value = value.to_python()
    assert py_value == ["α", "β", "γ"]


def test_cif2_comprehensive_unicode_in_table(cif2_comprehensive_cif):
    """Test Unicode in table keys."""
    doc = cif_parser.parse_file(str(cif2_comprehensive_cif))
    block = doc.first_block()

    value = block.get_item("_table_unicode")
    assert value.is_table
    py_value = value.to_python()
    assert py_value["α"] == 1.0
    assert py_value["β"] == 2.0
    assert py_value["γ"] == 3.0


def test_cif2_comprehensive_loop_with_cif2_values(cif2_comprehensive_cif):
    """Test loop containing CIF 2.0 values (lists and tables)."""
    doc = cif_parser.parse_file(str(cif2_comprehensive_cif))
    block = doc.first_block()

    assert block.num_loops == 1
    loop = block.find_loop("_atom_label")
    assert len(loop) == 4

    # Check first row
    label = loop.get_by_tag(0, "_atom_label")
    assert label.text == "C1"

    coords = loop.get_by_tag(0, "_atom_coords")
    assert coords.is_list
    assert coords.to_python() == [0.1, 0.2, 0.3]

    props = loop.get_by_tag(0, "_atom_properties")
    assert props.is_table
    props_dict = props.to_python()
    assert props_dict["element"] == "C"
    assert props_dict["mass"] == 12.0

    # Check third row (nitrogen)
    label = loop.get_by_tag(2, "_atom_label")
    assert label.text == "N1"

    props = loop.get_by_tag(2, "_atom_properties")
    props_dict = props.to_python()
    assert props_dict["element"] == "N"
    assert props_dict["mass"] == 14.0
