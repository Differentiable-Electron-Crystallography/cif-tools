//! CifBlock API tests
//!
//! Tests block name handling, case preservation, and item/loop/frame access

use cif_parser::Document;

#[test]
fn test_block_name_case_preservation() {
    // Keywords are case-insensitive, but names preserve original case
    let cases = vec![
        ("data_test\n_item value\n", "test"),
        ("DATA_TEST\n_item value\n", "TEST"),
        ("DaTa_MiXeDcAsE\n_item value\n", "MiXeDcAsE"),
    ];

    for (input, expected) in cases {
        let doc = Document::parse(input).unwrap();
        assert_eq!(doc.blocks.len(), 1);
        assert_eq!(doc.blocks[0].name, expected);
    }
}

#[test]
fn test_global_block_empty_name() {
    let cif = "global_\n_item value\n";
    let doc = Document::parse(cif).unwrap();
    assert_eq!(doc.blocks[0].name, "");
}

#[test]
fn test_block_item_access() {
    let cif = "data_test\n_item1 value1\n_item2 42\n";
    let doc = Document::parse(cif).unwrap();
    let block = &doc.blocks[0];

    assert_eq!(block.items.len(), 2);
    assert!(block.items.contains_key("_item1"));
    assert!(block.items.contains_key("_item2"));
}
