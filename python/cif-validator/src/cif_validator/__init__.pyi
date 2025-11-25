"""
Type stubs for cif_validator

DDLm-based CIF validation library with Python bindings.

Example usage:
    import cif_validator

    errors = cif_validator.validate(cif_content, dictionary_content)
    for error in errors:
        print(error)
"""

__version__: str
__author__: str

def validate(cif_content: str, dictionary: str) -> list[str]:
    """
    Validate a CIF document against a DDLm dictionary.

    Args:
        cif_content: CIF file content as string
        dictionary: DDLm dictionary content as string

    Returns:
        List of validation error messages. Empty list if validation passes.

    Raises:
        ValueError: If CIF content or dictionary cannot be parsed

    Example:
        import cif_validator

        cif_content = '''
        data_example
        _cell_length_a  10.000
        '''

        errors = cif_validator.validate(cif_content, dictionary_content)
        if errors:
            for error in errors:
                print(f"Validation error: {error}")
        else:
            print("Validation passed!")
    """
    ...
