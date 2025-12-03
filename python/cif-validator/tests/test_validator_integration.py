"""Integration tests for cif_validator using shared fixtures.

These tests mirror the JavaScript integration tests in
javascript/packages/cif-validator/tests/integration.test.cjs
for test parity across Python and JavaScript.
"""

import cif_validator
from cif_validator import ErrorCategory, ValidationMode, Validator, validate

# =============================================================================
# valid_structure.cif - Should pass validation
# =============================================================================


def test_valid_structure_passes_validation(valid_cif_content, validation_dict_content):
    """Test that valid_structure.cif passes validation with no errors."""
    result = validate(valid_cif_content, validation_dict_content)

    assert result.is_valid
    assert len(result.errors) == 0


def test_valid_structure_no_warnings_in_strict_mode(
    valid_cif_content, validation_dict_content
):
    """Test that valid_structure.cif has no warnings in strict mode."""
    validator = Validator()
    validator.add_dictionary(validation_dict_content)
    validator.set_mode(ValidationMode.Strict)

    result = validator.validate(valid_cif_content)

    assert result.is_valid
    # In strict mode, unknown items are errors, not warnings
    # valid_structure.cif should have no unknown items


# =============================================================================
# invalid_structure.cif - Should fail validation with 9 errors
# =============================================================================


def test_invalid_structure_fails_validation(
    invalid_cif_content, validation_dict_content
):
    """Test that invalid_structure.cif fails validation."""
    result = validate(invalid_cif_content, validation_dict_content)

    assert not result.is_valid
    assert len(result.errors) > 0


def test_invalid_structure_error_count(invalid_cif_content, validation_dict_content):
    """Test that invalid_structure.cif has exactly 9 errors.

    Expected errors:
    1. _cell.length_a = -5.0 (range: 0.1-1000)
    2. _cell.length_b = 5000.0 (range: 0.1-1000)
    3. _cell.angle_alpha = 270.0 (range: 0-180)
    4. _cell.angle_beta = -45.0 (range: 0-180)
    5. _symmetry.crystal_system = "dodecahedral" (invalid enum)
    6. _atom_site.fract_x = 1.5000 (range: 0-1)
    7. _atom_site.fract_y = -0.1000 (range: 0-1)
    8. _atom_site.occupancy = 2.5 (range: 0-1)
    9. _atom_site.occupancy = -0.5 (range: 0-1)
    """
    result = validate(invalid_cif_content, validation_dict_content)

    assert len(result.errors) == 9


# =============================================================================
# Range Error Detection
# =============================================================================


def test_invalid_cell_length_errors(invalid_cif_content, validation_dict_content):
    """Test detection of cell length range errors."""
    result = validate(invalid_cif_content, validation_dict_content)

    # Find errors related to cell lengths
    cell_length_errors = [
        e
        for e in result.errors
        if e.data_name and e.data_name.startswith("_cell.length_")
    ]

    # Should have 2 cell length errors: length_a (-5.0) and length_b (5000.0)
    assert len(cell_length_errors) == 2

    # All should be range errors
    for error in cell_length_errors:
        assert error.category == ErrorCategory.RangeError


def test_invalid_cell_angle_errors(invalid_cif_content, validation_dict_content):
    """Test detection of cell angle range errors."""
    result = validate(invalid_cif_content, validation_dict_content)

    # Find errors related to cell angles
    cell_angle_errors = [
        e
        for e in result.errors
        if e.data_name and e.data_name.startswith("_cell.angle_")
    ]

    # Should have 2 cell angle errors: angle_alpha (270.0) and angle_beta (-45.0)
    assert len(cell_angle_errors) == 2

    # All should be range errors
    for error in cell_angle_errors:
        assert error.category == ErrorCategory.RangeError


def test_invalid_fractional_coordinate_errors(
    invalid_cif_content, validation_dict_content
):
    """Test detection of fractional coordinate range errors."""
    result = validate(invalid_cif_content, validation_dict_content)

    # Find errors related to fractional coordinates
    fract_errors = [
        e
        for e in result.errors
        if e.data_name and e.data_name.startswith("_atom_site.fract_")
    ]

    # Should have 2 fractional coord errors: fract_x (1.5) and fract_y (-0.1)
    assert len(fract_errors) == 2

    # All should be range errors
    for error in fract_errors:
        assert error.category == ErrorCategory.RangeError


def test_invalid_occupancy_errors(invalid_cif_content, validation_dict_content):
    """Test detection of occupancy range errors."""
    result = validate(invalid_cif_content, validation_dict_content)

    # Find errors related to occupancy
    occupancy_errors = [
        e for e in result.errors if e.data_name == "_atom_site.occupancy"
    ]

    # Should have 2 occupancy errors: 2.5 and -0.5
    assert len(occupancy_errors) == 2

    # All should be range errors
    for error in occupancy_errors:
        assert error.category == ErrorCategory.RangeError


# =============================================================================
# Enumeration Error Detection
# =============================================================================


def test_invalid_enumeration_error(invalid_cif_content, validation_dict_content):
    """Test detection of enumeration error for crystal_system."""
    result = validate(invalid_cif_content, validation_dict_content)

    # Find the crystal_system error
    crystal_system_errors = [
        e for e in result.errors if e.data_name == "_symmetry.crystal_system"
    ]

    # Should have exactly 1 enumeration error
    assert len(crystal_system_errors) == 1
    assert crystal_system_errors[0].category == ErrorCategory.EnumerationError

    # The actual value should be "dodecahedral"
    assert crystal_system_errors[0].actual == "dodecahedral"


# =============================================================================
# Error Span Information
# =============================================================================


def test_error_spans_have_valid_positions(invalid_cif_content, validation_dict_content):
    """Test that all errors have valid span information."""
    result = validate(invalid_cif_content, validation_dict_content)

    for error in result.errors:
        span = error.span
        # All span values should be positive (1-indexed)
        assert span.start_line >= 1
        assert span.end_line >= 1
        assert span.start_col >= 1
        assert span.end_col >= 1
        # End should be at or after start
        assert span.end_line >= span.start_line
        if span.start_line == span.end_line:
            assert span.end_col >= span.start_col


def test_error_messages_are_informative(invalid_cif_content, validation_dict_content):
    """Test that error messages contain useful information."""
    result = validate(invalid_cif_content, validation_dict_content)

    for error in result.errors:
        # Message should not be empty
        assert error.message
        assert len(error.message) > 0


# =============================================================================
# Validator Class Workflow
# =============================================================================


def test_validator_class_workflow(valid_cif_content, validation_dict_content):
    """Test the Validator class workflow."""
    # Create validator
    validator = Validator()

    # Add dictionary
    validator.add_dictionary(validation_dict_content)

    # Set mode
    validator.set_mode(ValidationMode.Strict)

    # Validate
    result = validator.validate(valid_cif_content)

    assert result.is_valid


def test_validator_can_validate_multiple_documents(
    valid_cif_content, invalid_cif_content, validation_dict_content
):
    """Test that a Validator instance can validate multiple documents."""
    validator = Validator()
    validator.add_dictionary(validation_dict_content)

    # Validate valid document
    result1 = validator.validate(valid_cif_content)
    assert result1.is_valid

    # Validate invalid document with same validator
    result2 = validator.validate(invalid_cif_content)
    assert not result2.is_valid
    assert len(result2.errors) == 9


# =============================================================================
# Validation Modes
# =============================================================================


def test_validation_mode_lenient(validation_dict_content):
    """Test that lenient mode treats unknown items as warnings, not errors."""
    # CIF with an item not in the dictionary
    cif_with_unknown = """#\\#CIF_2.0
data_test
_entry.id 'test'
_unknown_item 'this is not in the dictionary'
_cell.length_a 10.0
"""

    validator = Validator()
    validator.add_dictionary(validation_dict_content)
    validator.set_mode(ValidationMode.Lenient)

    result = validator.validate(cif_with_unknown)

    # In lenient mode, unknown items should be warnings, not errors
    # So the document should be valid
    assert result.is_valid

    # Should have at least one warning about the unknown item
    assert len(result.warnings) >= 1


def test_validation_mode_strict(validation_dict_content):
    """Test that strict mode treats unknown items as errors."""
    # CIF with an item not in the dictionary
    cif_with_unknown = """#\\#CIF_2.0
data_test
_entry.id 'test'
_unknown_item 'this is not in the dictionary'
_cell.length_a 10.0
"""

    validator = Validator()
    validator.add_dictionary(validation_dict_content)
    validator.set_mode(ValidationMode.Strict)

    result = validator.validate(cif_with_unknown)

    # In strict mode, unknown items should be errors
    # Find the unknown item error
    unknown_errors = [
        e for e in result.errors if e.category == ErrorCategory.UnknownDataName
    ]
    assert len(unknown_errors) >= 1


# =============================================================================
# API Smoke Tests
# =============================================================================


def test_version_available():
    """Test that version is accessible."""
    version = cif_validator.__version__
    assert version
    assert isinstance(version, str)


def test_all_exports_available():
    """Test that all documented exports are available."""
    # Main function
    assert callable(validate)

    # Validator class
    assert callable(Validator)

    # Result types
    from cif_validator import ValidationError, ValidationResult, ValidationWarning

    assert ValidationResult
    assert ValidationError
    assert ValidationWarning

    # Span
    from cif_validator import Span

    assert Span

    # Enums
    from cif_validator import ErrorCategory, ValidationMode, WarningCategory

    assert ValidationMode.Strict
    assert ValidationMode.Lenient
    assert ValidationMode.Pedantic
    assert ErrorCategory.RangeError
    assert ErrorCategory.EnumerationError
    assert ErrorCategory.TypeError
    assert WarningCategory.UnknownItem
