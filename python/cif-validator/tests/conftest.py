"""Pytest configuration and fixtures for cif_validator integration tests."""

from pathlib import Path

import pytest


@pytest.fixture
def fixtures_dir():
    """Return path to shared fixtures directory at project root."""
    # conftest.py -> tests/ -> cif-validator/ -> python/ -> cif-tools/ -> fixtures/
    return Path(__file__).parent.parent.parent.parent / "fixtures"


@pytest.fixture
def validation_fixtures_dir(fixtures_dir):
    """Return path to validation fixtures subdirectory."""
    return fixtures_dir / "validation"


@pytest.fixture
def validation_dict_path(validation_fixtures_dir):
    """Return path to test_validation.dic."""
    return validation_fixtures_dir / "test_validation.dic"


@pytest.fixture
def valid_cif_path(validation_fixtures_dir):
    """Return path to valid_structure.cif."""
    return validation_fixtures_dir / "valid_structure.cif"


@pytest.fixture
def invalid_cif_path(validation_fixtures_dir):
    """Return path to invalid_structure.cif."""
    return validation_fixtures_dir / "invalid_structure.cif"


@pytest.fixture
def validation_dict_content(validation_dict_path):
    """Return contents of test_validation.dic."""
    return validation_dict_path.read_text()


@pytest.fixture
def valid_cif_content(valid_cif_path):
    """Return contents of valid_structure.cif."""
    return valid_cif_path.read_text()


@pytest.fixture
def invalid_cif_content(invalid_cif_path):
    """Return contents of invalid_structure.cif."""
    return invalid_cif_path.read_text()


@pytest.fixture
def sample_cif_content() -> str:
    """Return sample CIF content for testing."""
    return """
data_test
_cell_length_a  10.000
_cell_length_b  10.000
_cell_length_c  10.000
"""
