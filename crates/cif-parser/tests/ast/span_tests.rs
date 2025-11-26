//! Span tracking tests
//!
//! Verifies that source locations (spans) are correctly captured for all AST nodes
//! during parsing. This enables precise error reporting and IDE features.

use cif_parser::{CifDocument, Span};

// ========================================================================
// Document Span Tests
// ========================================================================

#[test]
fn test_document_span() {
    let cif = "data_test\n_item value\n";
    let doc = CifDocument::parse(cif).unwrap();

    // Document should span from start to end of input
    assert_eq!(doc.span.start_line, 1);
    assert_eq!(doc.span.start_col, 1);
    // End position depends on parser behavior, just verify it's set
    assert!(doc.span.end_line >= 1);
}

#[test]
fn test_multiline_document_span() {
    let cif = "data_test\n_item1 value1\n_item2 value2\n_item3 value3\n";
    let doc = CifDocument::parse(cif).unwrap();

    assert_eq!(doc.span.start_line, 1);
    assert_eq!(doc.span.start_col, 1);
    // Should span multiple lines
    assert!(doc.span.end_line >= 4);
}

// ========================================================================
// Block Span Tests
// ========================================================================

#[test]
fn test_block_span() {
    let cif = "data_test\n_item value\n";
    let doc = CifDocument::parse(cif).unwrap();
    let block = doc.first_block().unwrap();

    // Block starts at data_test on line 1
    assert_eq!(block.span.start_line, 1);
    assert_eq!(block.span.start_col, 1);
}

#[test]
fn test_multiple_blocks_span() {
    let cif = "data_first\n_item1 val1\n\ndata_second\n_item2 val2\n";
    let doc = CifDocument::parse(cif).unwrap();

    let first = &doc.blocks[0];
    let second = &doc.blocks[1];

    // First block starts at line 1
    assert_eq!(first.span.start_line, 1);

    // Second block starts at line 4 (after empty line)
    assert_eq!(second.span.start_line, 4);
}

// ========================================================================
// Loop Span Tests
// ========================================================================

#[test]
fn test_loop_span() {
    let cif = "data_test\nloop_\n_col1\n_col2\nval1 val2\n";
    let doc = CifDocument::parse(cif).unwrap();
    let loop_ = &doc.blocks[0].loops[0];

    // Loop starts at "loop_" on line 2
    assert_eq!(loop_.span.start_line, 2);
    assert_eq!(loop_.span.start_col, 1);
}

#[test]
fn test_loop_value_spans() {
    let cif = "data_test\nloop_\n_col1\nval1\nval2\nval3\n";
    let doc = CifDocument::parse(cif).unwrap();
    let loop_ = &doc.blocks[0].loops[0];

    // First value "val1" is on line 4
    let val1 = loop_.get(0, 0).unwrap();
    assert_eq!(val1.span.start_line, 4);

    // Second value "val2" is on line 5
    let val2 = loop_.get(1, 0).unwrap();
    assert_eq!(val2.span.start_line, 5);

    // Third value "val3" is on line 6
    let val3 = loop_.get(2, 0).unwrap();
    assert_eq!(val3.span.start_line, 6);
}

// ========================================================================
// Frame Span Tests
// ========================================================================

#[test]
fn test_frame_span() {
    let cif = "data_test\nsave_frame1\n_frame_item value\nsave_\n";
    let doc = CifDocument::parse(cif).unwrap();
    let frame = &doc.blocks[0].frames[0];

    // Frame starts at "save_frame1" on line 2
    assert_eq!(frame.span.start_line, 2);
    assert_eq!(frame.span.start_col, 1);
}

// ========================================================================
// Item Value Span Tests
// ========================================================================

#[test]
fn test_item_value_span() {
    let cif = "data_test\n_item value\n";
    let doc = CifDocument::parse(cif).unwrap();
    let item = doc.blocks[0].get_item("_item").unwrap();

    // Value "value" is on line 2, after "_item "
    assert_eq!(item.span.start_line, 2);
}

#[test]
fn test_numeric_value_span() {
    let cif = "data_test\n_number 42.5\n";
    let doc = CifDocument::parse(cif).unwrap();
    let item = doc.blocks[0].get_item("_number").unwrap();

    assert_eq!(item.span.start_line, 2);
    assert_eq!(item.as_numeric(), Some(42.5));
}

#[test]
fn test_quoted_value_span() {
    let cif = "data_test\n_text 'hello world'\n";
    let doc = CifDocument::parse(cif).unwrap();
    let item = doc.blocks[0].get_item("_text").unwrap();

    assert_eq!(item.span.start_line, 2);
    assert_eq!(item.as_string(), Some("hello world"));
}

// ========================================================================
// Text Field Span Tests
// ========================================================================

#[test]
fn test_text_field_span() {
    let cif = "data_test\n_text\n;Line 1\nLine 2\n;\n";
    let doc = CifDocument::parse(cif).unwrap();
    let text = doc.blocks[0].get_item("_text").unwrap();

    // Text field starts at the opening semicolon on line 3
    assert_eq!(text.span.start_line, 3);
    // Text field should end at or after line 5 (closing semicolon)
    assert!(text.span.end_line >= 5);
}

// ========================================================================
// CIF 2.0 List Span Tests
// ========================================================================

#[test]
fn test_cif2_list_span() {
    let cif = "#\\#CIF_2.0\ndata_test\n_list [1 2 3]\n";
    let doc = CifDocument::parse(cif).unwrap();
    let list = doc.blocks[0].get_item("_list").unwrap();

    // List starts on line 3
    assert_eq!(list.span.start_line, 3);
    assert!(list.is_list());
}

#[test]
fn test_cif2_nested_list_item_spans() {
    let cif = "#\\#CIF_2.0\ndata_test\n_list [1.0 2.0 3.0]\n";
    let doc = CifDocument::parse(cif).unwrap();
    let list = doc.blocks[0].get_item("_list").unwrap();
    let items = list.as_list().unwrap();

    // All items in the list should have spans on line 3
    for item in items {
        assert_eq!(item.span.start_line, 3);
    }
}

// ========================================================================
// CIF 2.0 Table Span Tests
// ========================================================================

#[test]
fn test_cif2_table_span() {
    let cif = "#\\#CIF_2.0\ndata_test\n_table {'x':1.0 'y':2.0}\n";
    let doc = CifDocument::parse(cif).unwrap();
    let table = doc.blocks[0].get_item("_table").unwrap();

    // Table starts on line 3
    assert_eq!(table.span.start_line, 3);
    assert!(table.is_table());
}

// ========================================================================
// Span Display and Utility Tests
// ========================================================================

#[test]
fn test_span_display_single_point() {
    let span = Span::point(4, 7);
    assert_eq!(format!("{}", span), "4:7");
}

#[test]
fn test_span_display_single_line() {
    let span = Span::new(2, 3, 2, 8);
    assert_eq!(format!("{}", span), "2:3-8");
}

#[test]
fn test_span_display_multi_line() {
    let span = Span::new(1, 5, 3, 10);
    assert_eq!(format!("{}", span), "1:5-3:10");
}

#[test]
fn test_span_merge() {
    let left = Span::new(1, 1, 1, 5);
    let right = Span::new(1, 10, 2, 3);
    let merged = left.merge(right);

    assert_eq!(merged.start_line, 1);
    assert_eq!(merged.start_col, 1);
    assert_eq!(merged.end_line, 2);
    assert_eq!(merged.end_col, 3);
}

#[test]
fn test_span_contains() {
    let span = Span::new(2, 5, 4, 10);

    // Points inside the span
    assert!(span.contains(3, 1)); // Middle line
    assert!(span.contains(2, 5)); // Start position
    assert!(span.contains(4, 10)); // End position
    assert!(span.contains(2, 10)); // Start line, after start col
    assert!(span.contains(4, 5)); // End line, before end col

    // Points outside the span
    assert!(!span.contains(1, 1)); // Before start line
    assert!(!span.contains(5, 1)); // After end line
    assert!(!span.contains(2, 4)); // Start line, before start col
    assert!(!span.contains(4, 11)); // End line, after end col
}
