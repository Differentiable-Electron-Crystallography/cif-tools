// tests/test_block_name.rs
use cif_parser::Document;

#[test]
fn test_mixed_case_block_name() {
    let cases = vec![
        ("data_test\n_item value\n", "test"),
        ("DATA_TEST\n_item value\n", "TEST"),
        ("DaTa_MiXeDcAsE\n_item value\n", "MiXeDcAsE"),
        // Note: Empty block names are not allowed per CIF 2.0 spec (was a CIF 1.1 bug)
        // The grammar now correctly requires container_code = { non_blank_char+ }
    ];

    for (input, expected) in cases {
        println!("Testing: {}", input.escape_debug());
        match Document::parse(input) {
            Ok(doc) => {
                assert_eq!(doc.blocks.len(), 1, "Should have exactly one block");
                let actual = &doc.blocks[0].name;
                println!("  Expected: '{}', Got: '{}'", expected, actual);
                assert_eq!(actual, expected, "Block name mismatch for input: {}", input);
            }
            Err(e) => {
                panic!("Failed to parse '{}': {:?}", input, e);
            }
        }
    }
}

#[test]
fn test_empty_block_name_rejected() {
    // CIF 2.0 fixed a CIF 1.1 bug where empty block names were allowed
    // The spec requires at least one non-blank character after data_
    let input = "data_\n_item value\n";
    let result = Document::parse(input);
    assert!(result.is_err(), "Empty block names should be rejected");
}
