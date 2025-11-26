//! CifLoop API tests
//!
//! Tests loop structure access, row/column iteration, and value retrieval

use cif_parser::CifDocument;

// ========================================================================
// Loop Basic Properties
// ========================================================================

#[test]
fn test_loop_len() {
    let cif = "data_test\nloop_\n_col1\n_col2\nv1 v2\nv3 v4\nv5 v6\n";
    let doc = CifDocument::parse(cif).unwrap();
    let loop_ = &doc.first_block().unwrap().loops[0];

    assert_eq!(loop_.len(), 3);
}

#[test]
fn test_loop_is_empty() {
    // Loop with values
    let cif = "data_test\nloop_\n_col\nval\n";
    let doc = CifDocument::parse(cif).unwrap();
    let loop_ = &doc.first_block().unwrap().loops[0];
    assert!(!loop_.is_empty());

    // Empty loop (tags only, no values)
    let cif_empty = "data_test\nloop_\n_col\ndata_next\n";
    let doc_empty = CifDocument::parse(cif_empty).unwrap();
    let loop_empty = &doc_empty.first_block().unwrap().loops[0];
    assert!(loop_empty.is_empty());
}

#[test]
fn test_loop_tags() {
    let cif = "data_test\nloop_\n_atom_id\n_atom_type\n_atom_x\n1 C 0.5\n";
    let doc = CifDocument::parse(cif).unwrap();
    let loop_ = &doc.first_block().unwrap().loops[0];

    assert_eq!(loop_.tags.len(), 3);
    assert_eq!(loop_.tags[0], "_atom_id");
    assert_eq!(loop_.tags[1], "_atom_type");
    assert_eq!(loop_.tags[2], "_atom_x");
}

// ========================================================================
// Value Access by Index
// ========================================================================

#[test]
fn test_loop_get() {
    let cif = "data_test\nloop_\n_a\n_b\n1 2\n3 4\n";
    let doc = CifDocument::parse(cif).unwrap();
    let loop_ = &doc.first_block().unwrap().loops[0];

    // Row 0
    assert_eq!(loop_.get(0, 0).unwrap().as_numeric(), Some(1.0));
    assert_eq!(loop_.get(0, 1).unwrap().as_numeric(), Some(2.0));

    // Row 1
    assert_eq!(loop_.get(1, 0).unwrap().as_numeric(), Some(3.0));
    assert_eq!(loop_.get(1, 1).unwrap().as_numeric(), Some(4.0));

    // Out of bounds
    assert!(loop_.get(2, 0).is_none());
    assert!(loop_.get(0, 2).is_none());
}

#[test]
fn test_loop_get_by_tag() {
    let cif = "data_test\nloop_\n_name\n_value\nalpha 100\nbeta 200\n";
    let doc = CifDocument::parse(cif).unwrap();
    let loop_ = &doc.first_block().unwrap().loops[0];

    assert_eq!(
        loop_.get_by_tag(0, "_name").unwrap().as_string(),
        Some("alpha")
    );
    assert_eq!(
        loop_.get_by_tag(0, "_value").unwrap().as_numeric(),
        Some(100.0)
    );
    assert_eq!(
        loop_.get_by_tag(1, "_name").unwrap().as_string(),
        Some("beta")
    );

    // Nonexistent tag
    assert!(loop_.get_by_tag(0, "_nonexistent").is_none());
}

// ========================================================================
// Column Access
// ========================================================================

#[test]
fn test_loop_get_column() {
    let cif = "data_test\nloop_\n_id\n_val\n1 a\n2 b\n3 c\n";
    let doc = CifDocument::parse(cif).unwrap();
    let loop_ = &doc.first_block().unwrap().loops[0];

    let ids = loop_.get_column("_id").unwrap();
    assert_eq!(ids.len(), 3);
    assert_eq!(ids[0].as_numeric(), Some(1.0));
    assert_eq!(ids[1].as_numeric(), Some(2.0));
    assert_eq!(ids[2].as_numeric(), Some(3.0));

    let vals = loop_.get_column("_val").unwrap();
    assert_eq!(vals[0].as_string(), Some("a"));
    assert_eq!(vals[1].as_string(), Some("b"));
    assert_eq!(vals[2].as_string(), Some("c"));

    // Nonexistent column
    assert!(loop_.get_column("_nonexistent").is_none());
}

// ========================================================================
// Iteration
// ========================================================================

#[test]
fn test_loop_rows() {
    let cif = "data_test\nloop_\n_a\n_b\n1 2\n3 4\n";
    let doc = CifDocument::parse(cif).unwrap();
    let loop_ = &doc.first_block().unwrap().loops[0];

    let rows: Vec<_> = loop_.rows().collect();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].len(), 2);
    assert_eq!(rows[1].len(), 2);
}

#[test]
fn test_loop_tags_iter() {
    let cif = "data_test\nloop_\n_x\n_y\n_z\n1 2 3\n";
    let doc = CifDocument::parse(cif).unwrap();
    let loop_ = &doc.first_block().unwrap().loops[0];

    let tags: Vec<_> = loop_.tags_iter().collect();
    assert_eq!(tags, vec!["_x", "_y", "_z"]);
}

// ========================================================================
// Mixed Value Types
// ========================================================================

#[test]
fn test_loop_mixed_values() {
    let cif = "data_test
loop_
_label
_number
_text
A 1.5 'hello world'
B 2.5 \"quoted\"
C ? .
";
    let doc = CifDocument::parse(cif).unwrap();
    let loop_ = &doc.first_block().unwrap().loops[0];

    // Row with text, number, quoted string
    assert_eq!(
        loop_.get_by_tag(0, "_label").unwrap().as_string(),
        Some("A")
    );
    assert_eq!(
        loop_.get_by_tag(0, "_number").unwrap().as_numeric(),
        Some(1.5)
    );
    assert_eq!(
        loop_.get_by_tag(0, "_text").unwrap().as_string(),
        Some("hello world")
    );

    // Row with special values
    assert!(loop_.get_by_tag(2, "_number").unwrap().is_unknown());
    assert!(loop_.get_by_tag(2, "_text").unwrap().is_not_applicable());
}

// ========================================================================
// Edge Cases
// ========================================================================

#[test]
fn test_loop_single_column() {
    let cif = "data_test\nloop_\n_single\na\nb\nc\n";
    let doc = CifDocument::parse(cif).unwrap();
    let loop_ = &doc.first_block().unwrap().loops[0];

    assert_eq!(loop_.tags.len(), 1);
    assert_eq!(loop_.len(), 3);
    assert_eq!(loop_.get(0, 0).unwrap().as_string(), Some("a"));
}

#[test]
fn test_loop_single_row() {
    let cif = "data_test\nloop_\n_a\n_b\n_c\n1 2 3\n";
    let doc = CifDocument::parse(cif).unwrap();
    let loop_ = &doc.first_block().unwrap().loops[0];

    assert_eq!(loop_.tags.len(), 3);
    assert_eq!(loop_.len(), 1);
}

#[test]
fn test_multiple_loops_in_block() {
    let cif = "data_test
loop_
_first_a
_first_b
1 2

loop_
_second_x
_second_y
_second_z
a b c
";
    let doc = CifDocument::parse(cif).unwrap();
    let block = doc.first_block().unwrap();

    assert_eq!(block.loops.len(), 2);
    assert_eq!(block.loops[0].tags.len(), 2);
    assert_eq!(block.loops[1].tags.len(), 3);
}
