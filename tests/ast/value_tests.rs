//! CifValue API tests
//!
//! Tests the public API of CifValue enum and its helper methods.
//! These tests focus on value operations (as_list, as_table, type checking, etc.)

use cif_parser::{CifDocument, CifValue};
use std::collections::HashMap;

// ========================================================================
// List Value Tests
// ========================================================================

#[test]
fn test_simple_numeric_list() {
    let cif = "#\\#CIF_2.0\ndata_test\n_coords [1.0 2.0 3.0]\n";
    let doc = CifDocument::parse(cif).unwrap();
    let coords = doc.first_block().unwrap().items.get("_coords").unwrap();

    let list = coords.as_list().expect("Should be a list");
    assert_eq!(list.len(), 3);
    assert_eq!(list[0].as_numeric(), Some(1.0));
    assert_eq!(list[1].as_numeric(), Some(2.0));
    assert_eq!(list[2].as_numeric(), Some(3.0));
}

#[test]
fn test_empty_list() {
    let cif = "#\\#CIF_2.0\ndata_test\n_empty []\n";
    let doc = CifDocument::parse(cif).unwrap();
    let empty = doc.first_block().unwrap().items.get("_empty").unwrap();

    let list = empty.as_list().expect("Should be a list");
    assert_eq!(list.len(), 0);
}

#[test]
fn test_nested_list() {
    let cif = "#\\#CIF_2.0\ndata_test\n_nested [[1.0 2.0] [3.0 4.0]]\n";
    let doc = CifDocument::parse(cif).unwrap();
    let nested = doc.first_block().unwrap().items.get("_nested").unwrap();

    let outer = nested.as_list().expect("Should be a list");
    assert_eq!(outer.len(), 2);

    let inner1 = outer[0].as_list().expect("Should be nested list");
    assert_eq!(inner1.len(), 2);
    assert_eq!(inner1[0].as_numeric(), Some(1.0));
}

// ========================================================================
// Table Value Tests
// ========================================================================

#[test]
fn test_simple_table() {
    let cif = "#\\#CIF_2.0\ndata_test\n_point {'x':1.0 'y':2.0}\n";
    let doc = CifDocument::parse(cif).unwrap();
    let point = doc.first_block().unwrap().items.get("_point").unwrap();

    let table = point.as_table().expect("Should be a table");
    assert_eq!(table.len(), 2);
    assert_eq!(table.get("x").unwrap().as_numeric(), Some(1.0));
    assert_eq!(table.get("y").unwrap().as_numeric(), Some(2.0));
}

#[test]
fn test_empty_table() {
    let cif = "#\\#CIF_2.0\ndata_test\n_empty {}\n";
    let doc = CifDocument::parse(cif).unwrap();
    let empty = doc.first_block().unwrap().items.get("_empty").unwrap();

    let table = empty.as_table().expect("Should be a table");
    assert_eq!(table.len(), 0);
}

#[test]
fn test_nested_table() {
    let cif = "#\\#CIF_2.0\ndata_test\n_outer {'inner':{'a':1.0 'b':2.0}}\n";
    let doc = CifDocument::parse(cif).unwrap();
    let outer = doc.first_block().unwrap().items.get("_outer").unwrap();

    let outer_table = outer.as_table().expect("Should be a table");
    let inner_table = outer_table
        .get("inner")
        .unwrap()
        .as_table()
        .expect("Should be nested table");
    assert_eq!(inner_table.get("a").unwrap().as_numeric(), Some(1.0));
}

// ========================================================================
// CifValue Helper Method Tests
// ========================================================================

#[test]
fn test_is_cif2_only() {
    let list = CifValue::List(vec![]);
    assert!(list.is_cif2_only());

    let table = CifValue::Table(HashMap::new());
    assert!(table.is_cif2_only());

    let text = CifValue::Text("hello".to_string());
    assert!(!text.is_cif2_only());

    let num = CifValue::Numeric(42.0);
    assert!(!num.is_cif2_only());
}

#[test]
fn test_as_list_len() {
    let list = CifValue::List(vec![
        CifValue::Numeric(1.0),
        CifValue::Numeric(2.0),
        CifValue::Numeric(3.0),
    ]);
    assert_eq!(list.as_list_len(), Some(3));

    let text = CifValue::Text("hello".to_string());
    assert_eq!(text.as_list_len(), None);
}

#[test]
fn test_as_table_get() {
    let mut map = HashMap::new();
    map.insert("x".to_string(), CifValue::Numeric(1.0));
    map.insert("y".to_string(), CifValue::Numeric(2.0));
    let table = CifValue::Table(map);

    assert_eq!(table.as_table_get("x").unwrap().as_numeric(), Some(1.0));
    assert_eq!(table.as_table_get("y").unwrap().as_numeric(), Some(2.0));
    assert!(table.as_table_get("z").is_none());

    let text = CifValue::Text("hello".to_string());
    assert!(text.as_table_get("x").is_none());
}

#[test]
fn test_as_table_keys() {
    let mut map = HashMap::new();
    map.insert("a".to_string(), CifValue::Numeric(1.0));
    map.insert("b".to_string(), CifValue::Numeric(2.0));
    map.insert("c".to_string(), CifValue::Numeric(3.0));
    let table = CifValue::Table(map);

    let mut keys: Vec<&str> = table.as_table_keys().unwrap().collect();
    keys.sort();
    assert_eq!(keys, vec!["a", "b", "c"]);

    let text = CifValue::Text("hello".to_string());
    assert!(text.as_table_keys().is_none());
}
