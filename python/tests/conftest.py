"""Pytest configuration and fixtures for cif_parser tests."""

from pathlib import Path

import pytest

import cif_parser


@pytest.fixture
def fixtures_dir():
    """Return path to fixtures directory."""
    return Path(__file__).parent / "fixtures"


@pytest.fixture
def simple_cif(fixtures_dir):
    """Return path to simple.cif test file."""
    return fixtures_dir / "simple.cif"


@pytest.fixture
def loops_cif(fixtures_dir):
    """Return path to loops.cif test file."""
    return fixtures_dir / "loops.cif"


@pytest.fixture
def complex_cif(fixtures_dir):
    """Return path to complex.cif test file."""
    return fixtures_dir / "complex.cif"


@pytest.fixture
def simple_doc(simple_cif):
    """Return parsed Document from simple.cif."""
    return cif_parser.parse_file(str(simple_cif))


@pytest.fixture
def loops_doc(loops_cif):
    """Return parsed Document from loops.cif."""
    return cif_parser.parse_file(str(loops_cif))


@pytest.fixture
def complex_doc(complex_cif):
    """Return parsed Document from complex.cif."""
    return cif_parser.parse_file(str(complex_cif))


@pytest.fixture
def simple_cif_content():
    """Return simple CIF content as string."""
    return """
data_test
_cell_length_a  10.0
_title 'Test Structure'
_temperature ?
_pressure .
"""


@pytest.fixture
def loop_cif_content():
    """Return CIF with loop as string."""
    return """
data_test
loop_
_atom_site_label
_atom_site_type_symbol
_atom_site_fract_x
C1  C  0.123
N1  N  0.456
O1  O  0.789
"""
