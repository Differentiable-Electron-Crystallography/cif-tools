// tests/integration_tests.rs
// Integration tests for complete CIF file parsing

use cif_parser::{Document, Value};
use std::path::PathBuf;

// Helper to get test fixtures path
fn fixture_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(name);
    path
}

#[test]
fn test_parse_simple_cif() {
    let cif_content = r#"
# Simple CIF file for testing
data_simple
_name          'Test Structure'
_temperature   293.15
_pressure      101.325
"#;

    let doc = Document::parse(cif_content).expect("Failed to parse simple CIF");
    assert_eq!(doc.blocks.len(), 1);

    let block = &doc.blocks[0];
    assert_eq!(block.name, "simple");
    assert_eq!(block.items.len(), 3);

    // Check values
    assert_eq!(
        block.get_item("_name").unwrap().as_string().unwrap(),
        "Test Structure"
    );
    assert_eq!(
        block
            .get_item("_temperature")
            .unwrap()
            .as_numeric()
            .unwrap(),
        293.15
    );
}

#[test]
fn test_parse_loop_structure() {
    let cif_content = r#"
data_atoms
loop_
_atom_site_label
_atom_site_type_symbol
_atom_site_fract_x
_atom_site_fract_y
_atom_site_fract_z
C1   C   0.1234   0.5678   0.9012
N1   N   0.2345   0.6789   0.0123
O1   O   0.3456   0.7890   0.1234
H1   H   0.4567   0.8901   0.2345
"#;

    let doc = Document::parse(cif_content).expect("Failed to parse loop");
    let block = &doc.blocks[0];

    assert_eq!(block.loops.len(), 1);
    let loop_ = &block.loops[0];

    assert_eq!(loop_.tags.len(), 5);
    assert_eq!(loop_.len(), 4); // 4 atoms

    // Check first atom
    assert_eq!(
        loop_
            .get_by_tag(0, "_atom_site_label")
            .unwrap()
            .as_string()
            .unwrap(),
        "C1"
    );
    assert_eq!(
        loop_
            .get_by_tag(0, "_atom_site_fract_x")
            .unwrap()
            .as_numeric()
            .unwrap(),
        0.1234
    );
}

#[test]
fn test_parse_text_fields() {
    let cif_content = r#"
data_text
_abstract
;This is a multi-line
text field that can contain
any characters including "quotes" and 'apostrophes'
and even semicolons ; in the middle
;
_formula 'C12 H22 O11'
"#;

    let doc = Document::parse(cif_content).expect("Failed to parse text fields");
    let block = &doc.blocks[0];

    let r#abstract = block.get_item("_abstract").unwrap().as_string().unwrap();
    assert!(r#abstract.contains("multi-line"));
    assert!(r#abstract.contains("semicolons ; in the middle"));
}

#[test]
fn test_parse_multiple_blocks() {
    let cif_content = r#"
data_first
_item1 value1

data_second
_item2 value2

global_
_global_item global_value

data_third
_item3 value3
"#;

    let doc = Document::parse(cif_content).expect("Failed to parse multiple blocks");
    assert_eq!(doc.blocks.len(), 4);

    assert_eq!(doc.blocks[0].name, "first");
    assert_eq!(doc.blocks[1].name, "second");
    assert_eq!(doc.blocks[2].name, ""); // Global block has empty name
    assert_eq!(doc.blocks[3].name, "third");
}

#[test]
fn test_parse_save_frames() {
    let cif_content = r#"
data_with_frames
_main_item main_value

save_frame1
_frame_item1 frame_value1
loop_
_loop_tag1
_loop_tag2
val1 val2
val3 val4
save_

save_frame2
_frame_item2 frame_value2
save_

_another_main_item another_value
"#;

    let doc = Document::parse(cif_content).expect("Failed to parse save frames");
    let block = &doc.blocks[0];

    assert_eq!(block.frames.len(), 2);
    assert_eq!(block.frames[0].name, "frame1");
    assert_eq!(block.frames[1].name, "frame2");

    // Check frame content
    let frame1 = &block.frames[0];
    assert!(frame1.items.contains_key("_frame_item1"));
    assert_eq!(frame1.loops.len(), 1);
}

#[test]
fn test_parse_special_values() {
    let cif_content = r#"
data_special
_unknown_value      ?
_not_applicable     .
_normal_value       42
_quoted_question    '?'
_quoted_dot         '.'
"#;

    let doc = Document::parse(cif_content).expect("Failed to parse special values");
    let block = &doc.blocks[0];

    assert_eq!(*block.get_item("_unknown_value").unwrap(), Value::Unknown);
    assert_eq!(
        *block.get_item("_not_applicable").unwrap(),
        Value::NotApplicable
    );
    assert_eq!(
        *block.get_item("_normal_value").unwrap(),
        Value::Numeric(42.0)
    );
    assert_eq!(
        *block.get_item("_quoted_question").unwrap(),
        Value::Text("?".to_string())
    );
    assert_eq!(
        *block.get_item("_quoted_dot").unwrap(),
        Value::Text(".".to_string())
    );
}

#[test]
fn test_parse_comments() {
    let cif_content = r#"
# File header comment
# Another header comment

data_test  # inline comment after data block name

# Comment before item
_item1 value1  # inline comment after value

# Multi-line comment block
# Line 1
# Line 2
# Line 3

_item2 value2
"#;

    // Comments should be ignored, parsing should succeed
    let doc = Document::parse(cif_content).expect("Failed to parse with comments");
    let block = &doc.blocks[0];

    assert_eq!(block.name, "test");
    assert_eq!(block.items.len(), 2);
}

#[test]
fn test_parse_complex_loop() {
    let cif_content = r#"
data_complex_loop
loop_
_id
_string_unquoted
_string_single
_string_double
_numeric
_unknown
_not_applicable
1  simple  'single quoted'  "double quoted"  123.456  ?  .
2  C6H12O6  'O''Brien'      "She said \"Hi\""  -45.67  .  ?
"#;

    let doc = Document::parse(cif_content).expect("Failed to parse complex loop");
    let block = &doc.blocks[0];
    let loop_ = &block.loops[0];

    assert_eq!(loop_.len(), 2);

    // Check first row
    let row0_id = loop_.get(0, 0).unwrap();
    assert_eq!(row0_id.as_numeric().unwrap(), 1.0);

    let row0_unknown = loop_.get(0, 5).unwrap();
    assert_eq!(*row0_unknown, Value::Unknown);

    // Check second row
    let row1_string = loop_.get(1, 1).unwrap();
    assert_eq!(row1_string.as_string().unwrap(), "C6H12O6");
}

#[test]
fn test_parse_case_insensitive_keywords() {
    let cif_content = r#"
DaTa_MiXeDcAsE
_item1 value1

LoOp_
_tag1
_tag2
val1 val2

SAVE_MyFrame
_frame_item value
SaVe_

GLOBAL_
_global value
"#;

    let doc = Document::parse(cif_content).expect("Failed to parse mixed case");
    assert_eq!(doc.blocks.len(), 2);
    // The block name keeps the original case after the keyword
    assert_eq!(doc.blocks[0].name, "MiXeDcAsE");
}

#[test]
fn test_parse_empty_values() {
    let cif_content = r#"
data_empty
_empty_single ''
_empty_double ""
_empty_text
;
;
"#;

    let doc = Document::parse(cif_content).expect("Failed to parse empty values");
    let block = &doc.blocks[0];

    assert_eq!(
        block
            .get_item("_empty_single")
            .unwrap()
            .as_string()
            .unwrap(),
        ""
    );
    assert_eq!(
        block
            .get_item("_empty_double")
            .unwrap()
            .as_string()
            .unwrap(),
        ""
    );
    // Empty text field should also work
    let empty_text = block.get_item("_empty_text").unwrap().as_string().unwrap();
    assert!(empty_text.is_empty() || empty_text.trim().is_empty());
}

#[test]
fn test_parse_numeric_formats() {
    let cif_content = r#"
data_numbers
_integer        42
_float          3.14159
_negative       -273.15
_scientific1    1.23e-4
_scientific2    6.022E+23
_decimal        0.0001
"#;

    let doc = Document::parse(cif_content).expect("Failed to parse numbers");
    let block = &doc.blocks[0];

    assert_eq!(
        block.get_item("_integer").unwrap().as_numeric().unwrap(),
        42.0
    );
    assert_eq!(
        block.get_item("_float").unwrap().as_numeric().unwrap(),
        3.14159
    );
    assert_eq!(
        block.get_item("_negative").unwrap().as_numeric().unwrap(),
        -273.15
    );
    assert!(
        (block
            .get_item("_scientific1")
            .unwrap()
            .as_numeric()
            .unwrap()
            - 0.000123)
            .abs()
            < 1e-9
    );
    assert_eq!(
        block
            .get_item("_scientific2")
            .unwrap()
            .as_numeric()
            .unwrap(),
        6.022e23
    );
}

#[test]
fn test_malformed_loop_handling() {
    // Test loop with wrong number of values (should fail)
    let bad_loop = r#"
data_bad
loop_
_tag1
_tag2
value1 value2
value3  # Missing value4!
"#;

    let result = Document::parse(bad_loop);
    assert!(result.is_err(), "Should fail on malformed loop");
}

#[test]
fn test_nested_quotes() {
    let cif_content = r#"
data_quotes
_single_with_double  'He said "Hello"'
_double_with_single  "It's working"
_complex             'O'"'"'Brien'
"#;

    let doc = Document::parse(cif_content).expect("Failed to parse nested quotes");
    let block = &doc.blocks[0];

    assert_eq!(
        block
            .get_item("_single_with_double")
            .unwrap()
            .as_string()
            .unwrap(),
        "He said \"Hello\""
    );
    assert_eq!(
        block
            .get_item("_double_with_single")
            .unwrap()
            .as_string()
            .unwrap(),
        "It's working"
    );
}

// Test parsing from file (requires fixture files)
#[test]
#[ignore] // Remove ignore when you have fixture files
fn test_parse_from_file() {
    let path = fixture_path("simple.cif");
    if path.exists() {
        let doc = Document::from_file(&path).expect("Failed to parse file");
        assert!(!doc.blocks.is_empty());
    }
}

// Performance test for large files
#[test]
#[ignore] // This is a performance test, run manually
fn test_parse_large_file() {
    // Create a large CIF content
    let mut large_cif = String::from("data_large\n");

    // Add many items
    for i in 0..1000 {
        large_cif.push_str(&format!("_item_{} value_{}\n", i, i));
    }

    // Add a large loop
    large_cif.push_str("loop_\n_id\n_value\n");
    for i in 0..10000 {
        large_cif.push_str(&format!("{} val_{}\n", i, i));
    }

    let start = std::time::Instant::now();
    let doc = Document::parse(&large_cif).expect("Failed to parse large file");
    let duration = start.elapsed();

    println!("Parsed large file in {:?}", duration);
    assert_eq!(doc.blocks[0].items.len(), 1000);
    assert_eq!(doc.blocks[0].loops[0].len(), 10000);
}

#[test]
fn test_error_span_tracking() {
    use cif_parser::CifError;

    // Test 1: Loop with mismatched values should include line/column
    let cif_with_bad_loop = r#"
data_test
loop_
_col1
_col2
value1
"#;

    let result = Document::parse(cif_with_bad_loop);
    assert!(result.is_err());

    if let Err(err) = result {
        // Check that error contains location info
        if let CifError::InvalidStructure { message, location } = &err {
            assert!(message.contains("Loop has 2 tags but 1 values"));
            assert!(location.is_some());
            let (line, col) = location.unwrap();
            assert_eq!(line, 3); // loop_ starts on line 3
            assert!(col > 0);
        } else {
            panic!("Expected InvalidStructure error");
        }

        // Test that error message formatting includes location
        let error_message = format!("{}", err);
        assert!(error_message.contains("Error at line 3"));
        assert!(error_message.contains("column"));
    } else {
        panic!("Expected error");
    }
}
