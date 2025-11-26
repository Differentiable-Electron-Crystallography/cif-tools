"""Pytest configuration and fixtures for cif_parser integration tests."""

from pathlib import Path

import pytest


@pytest.fixture
def fixtures_dir():
    """Return path to shared fixtures directory at project root."""
    # conftest.py -> tests/ -> cif-parser/ -> python/ -> cif-tools/ -> fixtures/
    return Path(__file__).parent.parent.parent.parent / "fixtures"


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
def xanthine_cif(fixtures_dir):
    """Return path to pycifrw_xanthine.cif test file."""
    return fixtures_dir / "pycifrw_xanthine.cif"


@pytest.fixture
def luag_cif(fixtures_dir):
    """Return path to crystalmaker_LuAG.cif test file."""
    return fixtures_dir / "crystalmaker_LuAG.cif"


@pytest.fixture
def cif2_lists_cif(fixtures_dir):
    """Return path to cif2_lists.cif test file."""
    return fixtures_dir / "cif2_lists.cif"


@pytest.fixture
def cif2_tables_cif(fixtures_dir):
    """Return path to cif2_tables.cif test file."""
    return fixtures_dir / "cif2_tables.cif"


@pytest.fixture
def cif2_comprehensive_cif(fixtures_dir):
    """Return path to cif2_comprehensive.cif test file with all CIF 2.0 features."""
    return fixtures_dir / "cif2_comprehensive.cif"
