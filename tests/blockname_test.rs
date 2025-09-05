// tests/test_block_name.rs
use cif_parser::Document;

#[test]
fn test_mixed_case_block_name() {
    let cases = vec![
        ("data_test\n_item value\n", "test"),
        ("DATA_TEST\n_item value\n", "TEST"),
        ("DaTa_MiXeDcAsE\n_item value\n", "MiXeDcAsE"),
        ("data_\n_item value\n", ""), // Empty block name
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
