"""Tests for CIF Value type detection and conversion."""

import pytest

import cif_parser


def test_text_value(simple_doc):
    """Test text value detection and access."""
    block = simple_doc.first_block()
    value = block.get_item("_title")

    assert value is not None
    assert value.is_text
    assert not value.is_numeric
    assert not value.is_unknown
    assert not value.is_not_applicable

    assert value.text == "Simple Test Structure"
    assert value.numeric is None


def test_numeric_value(simple_doc):
    """Test numeric value detection and access."""
    block = simple_doc.first_block()
    value = block.get_item("_cell_length_a")

    assert value is not None
    assert value.is_numeric
    assert not value.is_text
    assert not value.is_unknown
    assert not value.is_not_applicable

    assert value.numeric == 10.0
    assert value.text is None


def test_unknown_value(simple_doc):
    """Test unknown value ('?') detection."""
    block = simple_doc.first_block()
    value = block.get_item("_temperature_kelvin")

    assert value is not None
    assert value.is_unknown
    assert not value.is_text
    assert not value.is_numeric
    assert not value.is_not_applicable

    assert value.text is None
    assert value.numeric is None


def test_not_applicable_value(simple_doc):
    """Test not applicable value ('.') detection."""
    block = simple_doc.first_block()
    value = block.get_item("_pressure")

    assert value is not None
    assert value.is_not_applicable
    assert not value.is_text
    assert not value.is_numeric
    assert not value.is_unknown

    assert value.text is None
    assert value.numeric is None


def test_to_python_text():
    """Test to_python() conversion for text values."""
    cif = "data_test\n_item 'hello'"
    doc = cif_parser.parse(cif)
    value = doc.first_block().get_item("_item")

    python_value = value.to_python()
    assert isinstance(python_value, str)
    assert python_value == "hello"


def test_to_python_numeric():
    """Test to_python() conversion for numeric values."""
    cif = "data_test\n_item 42.5"
    doc = cif_parser.parse(cif)
    value = doc.first_block().get_item("_item")

    python_value = value.to_python()
    assert isinstance(python_value, float)
    assert python_value == 42.5


def test_to_python_unknown():
    """Test to_python() conversion for unknown values."""
    cif = "data_test\n_item ?"
    doc = cif_parser.parse(cif)
    value = doc.first_block().get_item("_item")

    python_value = value.to_python()
    assert python_value is None


def test_to_python_not_applicable():
    """Test to_python() conversion for not applicable values."""
    cif = "data_test\n_item ."
    doc = cif_parser.parse(cif)
    value = doc.first_block().get_item("_item")

    python_value = value.to_python()
    assert python_value is None


def test_value_equality():
    """Test Value equality comparison."""
    cif = "data_test\n_item1 42.0\n_item2 42.0\n_item3 43.0"
    doc = cif_parser.parse(cif)
    block = doc.first_block()

    value1 = block.get_item("_item1")
    value2 = block.get_item("_item2")
    value3 = block.get_item("_item3")

    assert value1 == value2
    assert value1 != value3


def test_value_string_representation():
    """Test __str__ and __repr__ methods."""
    cif = "data_test\n_item 42.0"
    doc = cif_parser.parse(cif)
    value = doc.first_block().get_item("_item")

    str_repr = str(value)
    assert str_repr  # Should not be empty

    repr_str = repr(value)
    assert repr_str  # Should not be empty


def test_scientific_notation():
    """Test numeric values in scientific notation."""
    cif = "data_test\n_item 1.23e-4"
    doc = cif_parser.parse(cif)
    value = doc.first_block().get_item("_item")

    assert value.is_numeric
    assert value.numeric == pytest.approx(0.000123)


def test_quoted_string():
    """Test quoted string values."""
    cif = "data_test\n_item 'quoted string'"
    doc = cif_parser.parse(cif)
    value = doc.first_block().get_item("_item")

    assert value.is_text
    assert value.text == "quoted string"


def test_double_quoted_string():
    """Test double-quoted string values."""
    cif = 'data_test\n_item "double quoted"'
    doc = cif_parser.parse(cif)
    value = doc.first_block().get_item("_item")

    assert value.is_text
    assert value.text == "double quoted"


def test_unquoted_string():
    """Test unquoted string values."""
    cif = "data_test\n_item unquoted_value"
    doc = cif_parser.parse(cif)
    value = doc.first_block().get_item("_item")

    assert value.is_text
    assert value.text == "unquoted_value"


def test_value_type_property():
    """Test value_type property returns correct string."""
    cif = """
    data_test
    _text 'hello'
    _numeric 42.0
    _unknown ?
    _not_applicable .
    """
    doc = cif_parser.parse(cif)
    block = doc.first_block()

    assert block.get_item("_text").value_type == "text"
    assert block.get_item("_numeric").value_type == "numeric"
    assert block.get_item("_unknown").value_type == "unknown"
    assert block.get_item("_not_applicable").value_type == "not_applicable"
