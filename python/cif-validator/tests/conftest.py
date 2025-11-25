"""Pytest configuration for cif-validator tests."""

import pytest


@pytest.fixture
def sample_cif_content() -> str:
    """Return sample CIF content for testing."""
    return """
data_test
_cell_length_a  10.000
_cell_length_b  10.000
_cell_length_c  10.000
"""
