//! CifFrame API tests
//!
//! Tests save frame creation, item/loop access, and tag iteration

use cif_parser::CifDocument;

// ========================================================================
// Frame Creation and Basic Access
// ========================================================================

#[test]
fn test_frame_name() {
    let cif = "data_test\nsave_myframe\n_item value\nsave_\n";
    let doc = CifDocument::parse(cif).unwrap();
    let block = doc.first_block().unwrap();

    assert_eq!(block.frames.len(), 1);
    assert_eq!(block.frames[0].name, "myframe");
}

#[test]
fn test_frame_name_case_preservation() {
    let cif = "data_test\nsave_MyFrame\n_item value\nsave_\n";
    let doc = CifDocument::parse(cif).unwrap();
    let frame = &doc.first_block().unwrap().frames[0];

    assert_eq!(frame.name, "MyFrame");
}

#[test]
fn test_multiple_frames() {
    let cif = "data_test\nsave_frame1\n_a 1\nsave_\nsave_frame2\n_b 2\nsave_\n";
    let doc = CifDocument::parse(cif).unwrap();
    let block = doc.first_block().unwrap();

    assert_eq!(block.frames.len(), 2);
    assert_eq!(block.frames[0].name, "frame1");
    assert_eq!(block.frames[1].name, "frame2");
}

// ========================================================================
// Frame Item Access
// ========================================================================

#[test]
fn test_frame_get_item() {
    let cif = "data_test\nsave_frame\n_item1 value1\n_item2 42\nsave_\n";
    let doc = CifDocument::parse(cif).unwrap();
    let frame = &doc.first_block().unwrap().frames[0];

    assert!(frame.get_item("_item1").is_some());
    assert_eq!(
        frame.get_item("_item1").unwrap().as_string(),
        Some("value1")
    );
    assert_eq!(frame.get_item("_item2").unwrap().as_numeric(), Some(42.0));
    assert!(frame.get_item("_nonexistent").is_none());
}

#[test]
fn test_frame_items_map() {
    let cif = "data_test\nsave_frame\n_a 1\n_b 2\n_c 3\nsave_\n";
    let doc = CifDocument::parse(cif).unwrap();
    let frame = &doc.first_block().unwrap().frames[0];

    assert_eq!(frame.items.len(), 3);
    assert!(frame.items.contains_key("_a"));
    assert!(frame.items.contains_key("_b"));
    assert!(frame.items.contains_key("_c"));
}

// ========================================================================
// Frame Loop Access
// ========================================================================

#[test]
fn test_frame_find_loop() {
    let cif = "data_test\nsave_frame\nloop_\n_col1\n_col2\nval1 val2\nsave_\n";
    let doc = CifDocument::parse(cif).unwrap();
    let frame = &doc.first_block().unwrap().frames[0];

    let loop_ = frame.find_loop("_col1");
    assert!(loop_.is_some());
    assert_eq!(loop_.unwrap().tags.len(), 2);

    assert!(frame.find_loop("_nonexistent").is_none());
}

#[test]
fn test_frame_multiple_loops() {
    let cif = "data_test
save_frame
loop_
_a1
_a2
v1 v2

loop_
_b1
_b2
_b3
x1 x2 x3
save_
";
    let doc = CifDocument::parse(cif).unwrap();
    let frame = &doc.first_block().unwrap().frames[0];

    assert_eq!(frame.loops.len(), 2);
    assert!(frame.find_loop("_a1").is_some());
    assert!(frame.find_loop("_b3").is_some());
}

// ========================================================================
// Frame Tag Iteration
// ========================================================================

#[test]
fn test_frame_all_tags_items_only() {
    let cif = "data_test\nsave_frame\n_x 1\n_y 2\n_z 3\nsave_\n";
    let doc = CifDocument::parse(cif).unwrap();
    let frame = &doc.first_block().unwrap().frames[0];

    let tags: Vec<&str> = frame.all_tags().collect();
    assert_eq!(tags.len(), 3);
    assert!(tags.contains(&"_x"));
    assert!(tags.contains(&"_y"));
    assert!(tags.contains(&"_z"));
}

#[test]
fn test_frame_all_tags_with_loops() {
    let cif = "data_test\nsave_frame\n_item val\nloop_\n_col1\n_col2\nv1 v2\nsave_\n";
    let doc = CifDocument::parse(cif).unwrap();
    let frame = &doc.first_block().unwrap().frames[0];

    let tags: Vec<&str> = frame.all_tags().collect();
    assert_eq!(tags.len(), 3);
    assert!(tags.contains(&"_item"));
    assert!(tags.contains(&"_col1"));
    assert!(tags.contains(&"_col2"));
}

#[test]
fn test_frame_empty() {
    let cif = "data_test\nsave_empty\nsave_\n";
    let doc = CifDocument::parse(cif).unwrap();
    let frame = &doc.first_block().unwrap().frames[0];

    assert_eq!(frame.name, "empty");
    assert!(frame.items.is_empty());
    assert!(frame.loops.is_empty());
    assert_eq!(frame.all_tags().count(), 0);
}
