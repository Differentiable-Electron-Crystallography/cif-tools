//! Comprehensive tests for CIF 2.0 features
//!
//! Tests cover:
//! - Version detection (magic header)
//! - List parsing (simple, nested, empty)
//! - Table parsing (simple, nested, complex keys)
//! - Triple-quoted strings
//! - Version enforcement (CIF 2.0 features only available with magic header)

use cif_parser::{CifDocument, CifValue, CifVersion};

// ========================================================================
// Version Detection Tests
// ========================================================================

#[test]
fn test_cif2_magic_header_detection() {
    let cif = "#\\#CIF_2.0\ndata_test\n_item value\n";
    let doc = CifDocument::parse(cif).unwrap();
    assert_eq!(doc.version, CifVersion::V2_0);
}

#[test]
fn test_cif1_no_magic_header() {
    let cif = "data_test\n_item value\n";
    let doc = CifDocument::parse(cif).unwrap();
    assert_eq!(doc.version, CifVersion::V1_1);
}

#[test]
fn test_cif1_with_comment_not_magic_header() {
    let cif = "# This is a comment\ndata_test\n_item value\n";
    let doc = CifDocument::parse(cif).unwrap();
    assert_eq!(doc.version, CifVersion::V1_1);
}

// ========================================================================
// List Parsing Tests
// ========================================================================

#[test]
fn test_simple_numeric_list() {
    let cif = "#\\#CIF_2.0\ndata_test\n_coords [1.0 2.0 3.0]\n";
    let doc = CifDocument::parse(cif).unwrap();
    let block = doc.first_block().unwrap();
    let coords = block.items.get("_coords").unwrap();

    let list = coords.as_list().expect("Should be a list");
    assert_eq!(list.len(), 3);
    assert_eq!(list[0].as_numeric(), Some(1.0));
    assert_eq!(list[1].as_numeric(), Some(2.0));
    assert_eq!(list[2].as_numeric(), Some(3.0));
}

#[test]
fn test_simple_text_list() {
    let cif = "#\\#CIF_2.0\ndata_test\n_names ['Alice' 'Bob' 'Charlie']\n";
    let doc = CifDocument::parse(cif).unwrap();
    let block = doc.first_block().unwrap();
    let names = block.items.get("_names").unwrap();

    let list = names.as_list().expect("Should be a list");
    assert_eq!(list.len(), 3);
    assert_eq!(list[0].as_string(), Some("Alice"));
    assert_eq!(list[1].as_string(), Some("Bob"));
    assert_eq!(list[2].as_string(), Some("Charlie"));
}

#[test]
fn test_empty_list() {
    let cif = "#\\#CIF_2.0\ndata_test\n_empty []\n";
    let doc = CifDocument::parse(cif).unwrap();
    let block = doc.first_block().unwrap();
    let empty = block.items.get("_empty").unwrap();

    let list = empty.as_list().expect("Should be a list");
    assert_eq!(list.len(), 0);
}

#[test]
fn test_mixed_type_list() {
    let cif = "#\\#CIF_2.0\ndata_test\n_mixed [1.0 'text' ? .]\n";
    let doc = CifDocument::parse(cif).unwrap();
    let block = doc.first_block().unwrap();
    let mixed = block.items.get("_mixed").unwrap();

    let list = mixed.as_list().expect("Should be a list");
    assert_eq!(list.len(), 4);
    assert!(matches!(list[0], CifValue::Numeric(_)));
    assert!(matches!(list[1], CifValue::Text(_)));
    assert!(matches!(list[2], CifValue::Unknown));
    assert!(matches!(list[3], CifValue::NotApplicable));
}

#[test]
fn test_nested_list() {
    let cif = "#\\#CIF_2.0\ndata_test\n_nested [[1.0 2.0] [3.0 4.0]]\n";
    let doc = CifDocument::parse(cif).unwrap();
    let block = doc.first_block().unwrap();
    let nested = block.items.get("_nested").unwrap();

    let outer_list = nested.as_list().expect("Should be a list");
    assert_eq!(outer_list.len(), 2);

    let inner1 = outer_list[0].as_list().expect("Should be a nested list");
    assert_eq!(inner1.len(), 2);
    assert_eq!(inner1[0].as_numeric(), Some(1.0));
    assert_eq!(inner1[1].as_numeric(), Some(2.0));

    let inner2 = outer_list[1].as_list().expect("Should be a nested list");
    assert_eq!(inner2.len(), 2);
    assert_eq!(inner2[0].as_numeric(), Some(3.0));
    assert_eq!(inner2[1].as_numeric(), Some(4.0));
}

#[test]
fn test_list_with_multiline_whitespace() {
    let cif = "#\\#CIF_2.0\ndata_test\n_coords [\n  1.0\n  2.0\n  3.0\n]\n";
    let doc = CifDocument::parse(cif).unwrap();
    let block = doc.first_block().unwrap();
    let coords = block.items.get("_coords").unwrap();

    let list = coords.as_list().expect("Should be a list");
    assert_eq!(list.len(), 3);
}

// ========================================================================
// Table Parsing Tests
// ========================================================================

#[test]
fn test_simple_table() {
    let cif = "#\\#CIF_2.0\ndata_test\n_point {'x':1.0 'y':2.0 'z':3.0}\n";
    let doc = CifDocument::parse(cif).unwrap();
    let block = doc.first_block().unwrap();
    let point = block.items.get("_point").unwrap();

    let table = point.as_table().expect("Should be a table");
    assert_eq!(table.len(), 3);
    assert_eq!(table.get("x").unwrap().as_numeric(), Some(1.0));
    assert_eq!(table.get("y").unwrap().as_numeric(), Some(2.0));
    assert_eq!(table.get("z").unwrap().as_numeric(), Some(3.0));
}

#[test]
fn test_empty_table() {
    let cif = "#\\#CIF_2.0\ndata_test\n_empty {}\n";
    let doc = CifDocument::parse(cif).unwrap();
    let block = doc.first_block().unwrap();
    let empty = block.items.get("_empty").unwrap();

    let table = empty.as_table().expect("Should be a table");
    assert_eq!(table.len(), 0);
}

#[test]
fn test_table_with_various_value_types() {
    let cif = "#\\#CIF_2.0\ndata_test\n_data {'num':42.0 'text':'hello' 'unknown':? 'na':.}\n";
    let doc = CifDocument::parse(cif).unwrap();
    let block = doc.first_block().unwrap();
    let data = block.items.get("_data").unwrap();

    let table = data.as_table().expect("Should be a table");
    assert_eq!(table.len(), 4);
    assert!(matches!(table.get("num").unwrap(), CifValue::Numeric(_)));
    assert!(matches!(table.get("text").unwrap(), CifValue::Text(_)));
    assert!(matches!(table.get("unknown").unwrap(), CifValue::Unknown));
    assert!(matches!(table.get("na").unwrap(), CifValue::NotApplicable));
}

#[test]
fn test_table_with_triple_quoted_keys() {
    let cif = "#\\#CIF_2.0\ndata_test\n_map {\"\"\"long key\"\"\":1.0 '''another key''':2.0}\n";
    let doc = CifDocument::parse(cif).unwrap();
    let block = doc.first_block().unwrap();
    let map = block.items.get("_map").unwrap();

    let table = map.as_table().expect("Should be a table");
    assert_eq!(table.len(), 2);
    assert_eq!(table.get("long key").unwrap().as_numeric(), Some(1.0));
    assert_eq!(table.get("another key").unwrap().as_numeric(), Some(2.0));
}

#[test]
fn test_nested_table() {
    let cif = "#\\#CIF_2.0\ndata_test\n_outer {'inner':{'a':1.0 'b':2.0}}\n";
    let doc = CifDocument::parse(cif).unwrap();
    let block = doc.first_block().unwrap();
    let outer = block.items.get("_outer").unwrap();

    let outer_table = outer.as_table().expect("Should be a table");
    let inner_table = outer_table
        .get("inner")
        .unwrap()
        .as_table()
        .expect("Should be a nested table");
    assert_eq!(inner_table.len(), 2);
    assert_eq!(inner_table.get("a").unwrap().as_numeric(), Some(1.0));
    assert_eq!(inner_table.get("b").unwrap().as_numeric(), Some(2.0));
}

#[test]
fn test_table_with_list_values() {
    let cif = "#\\#CIF_2.0\ndata_test\n_data {'coords':[1.0 2.0 3.0] 'names':['Alice' 'Bob']}\n";
    let doc = CifDocument::parse(cif).unwrap();
    let block = doc.first_block().unwrap();
    let data = block.items.get("_data").unwrap();

    let table = data.as_table().expect("Should be a table");
    let coords = table
        .get("coords")
        .unwrap()
        .as_list()
        .expect("Should be a list");
    assert_eq!(coords.len(), 3);

    let names = table
        .get("names")
        .unwrap()
        .as_list()
        .expect("Should be a list");
    assert_eq!(names.len(), 2);
}

#[test]
fn test_list_of_tables() {
    let cif = "#\\#CIF_2.0\ndata_test\n_points [{'x':1.0 'y':2.0} {'x':3.0 'y':4.0}]\n";
    let doc = CifDocument::parse(cif).unwrap();
    let block = doc.first_block().unwrap();
    let points = block.items.get("_points").unwrap();

    let list = points.as_list().expect("Should be a list");
    assert_eq!(list.len(), 2);

    let point1 = list[0].as_table().expect("Should be a table");
    assert_eq!(point1.get("x").unwrap().as_numeric(), Some(1.0));
    assert_eq!(point1.get("y").unwrap().as_numeric(), Some(2.0));

    let point2 = list[1].as_table().expect("Should be a table");
    assert_eq!(point2.get("x").unwrap().as_numeric(), Some(3.0));
    assert_eq!(point2.get("y").unwrap().as_numeric(), Some(4.0));
}

// ========================================================================
// Triple-Quoted String Tests
// ========================================================================

#[test]
fn test_triple_quoted_double_quotes() {
    let cif = "#\\#CIF_2.0\ndata_test\n_text \"\"\"This is\nmulti-line\ntext\"\"\"\n";
    let doc = CifDocument::parse(cif).unwrap();
    let block = doc.first_block().unwrap();
    let text = block.items.get("_text").unwrap();

    assert_eq!(text.as_string(), Some("This is\nmulti-line\ntext"));
}

#[test]
fn test_triple_quoted_single_quotes() {
    let cif = "#\\#CIF_2.0\ndata_test\n_text '''This is\nmulti-line\ntext'''\n";
    let doc = CifDocument::parse(cif).unwrap();
    let block = doc.first_block().unwrap();
    let text = block.items.get("_text").unwrap();

    assert_eq!(text.as_string(), Some("This is\nmulti-line\ntext"));
}

#[test]
fn test_triple_quoted_empty() {
    let cif = "#\\#CIF_2.0\ndata_test\n_empty \"\"\"\"\"\"\n";
    let doc = CifDocument::parse(cif).unwrap();
    let block = doc.first_block().unwrap();
    let empty = block.items.get("_empty").unwrap();

    assert_eq!(empty.as_string(), Some(""));
}

#[test]
fn test_triple_quoted_with_single_quotes_inside() {
    let cif = "#\\#CIF_2.0\ndata_test\n_text \"\"\"It's a \"beautiful\" day\"\"\"\n";
    let doc = CifDocument::parse(cif).unwrap();
    let block = doc.first_block().unwrap();
    let text = block.items.get("_text").unwrap();

    assert_eq!(text.as_string(), Some("It's a \"beautiful\" day"));
}

// ========================================================================
// Version Enforcement Tests
// ========================================================================

#[test]
fn test_cif1_treats_brackets_as_text() {
    // CIF 1.1 files with brackets must quote them (brackets are reserved characters)
    // Even without CIF 2.0 magic header, brackets don't create lists
    let cif = "data_test\n_item '[not_a_list]'\n";
    let doc = CifDocument::parse(cif).unwrap();
    let block = doc.first_block().unwrap();
    let item = block.items.get("_item").unwrap();

    // In CIF 1.1 mode, quoted brackets are treated as text (not parsed as lists)
    assert!(matches!(item, CifValue::Text(_)));
    assert_eq!(item.as_string(), Some("[not_a_list]"));
}

#[test]
fn test_cif1_treats_braces_as_text() {
    // CIF 1.1 files with braces must quote them (braces are reserved characters)
    // Even without CIF 2.0 magic header, braces don't create tables
    let cif = "data_test\n_item '{not_a_table}'\n";
    let doc = CifDocument::parse(cif).unwrap();
    let block = doc.first_block().unwrap();
    let item = block.items.get("_item").unwrap();

    // In CIF 1.1 mode, quoted braces are treated as text (not parsed as tables)
    assert!(matches!(item, CifValue::Text(_)));
    assert_eq!(item.as_string(), Some("{not_a_table}"));
}

// ========================================================================
// CifValue Helper Method Tests
// ========================================================================

#[test]
fn test_is_cif2_only() {
    let list = CifValue::List(vec![]);
    assert!(list.is_cif2_only());

    let table = CifValue::Table(std::collections::HashMap::new());
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
    let mut map = std::collections::HashMap::new();
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
    let mut map = std::collections::HashMap::new();
    map.insert("a".to_string(), CifValue::Numeric(1.0));
    map.insert("b".to_string(), CifValue::Numeric(2.0));
    map.insert("c".to_string(), CifValue::Numeric(3.0));
    let table = CifValue::Table(map);

    let mut keys: Vec<&str> = table.as_table_keys().unwrap().collect();
    keys.sort(); // HashMap keys are unordered
    assert_eq!(keys, vec!["a", "b", "c"]);

    let text = CifValue::Text("hello".to_string());
    assert!(text.as_table_keys().is_none());
}

// ========================================================================
// Complex Integration Tests
// ========================================================================

#[test]
fn test_complete_cif2_document() {
    let cif = "#\\#CIF_2.0
data_molecule
_name 'Water'
_formula 'H2O'
_atoms [
    {'element':'H' 'x':0.0 'y':0.0 'z':0.0}
    {'element':'O' 'x':1.0 'y':0.0 'z':0.0}
    {'element':'H' 'x':2.0 'y':0.0 'z':0.0}
]
_properties {
    'mass':18.015
    'state':'liquid'
    'boiling_point':373.15
}
";

    let doc = CifDocument::parse(cif).unwrap();
    assert_eq!(doc.version, CifVersion::V2_0);

    let block = doc.first_block().unwrap();
    assert_eq!(block.name, "molecule");
    assert_eq!(block.items.get("_name").unwrap().as_string(), Some("Water"));
    assert_eq!(
        block.items.get("_formula").unwrap().as_string(),
        Some("H2O")
    );

    let atoms = block.items.get("_atoms").unwrap().as_list().unwrap();
    assert_eq!(atoms.len(), 3);

    let props = block.items.get("_properties").unwrap().as_table().unwrap();
    assert_eq!(props.get("mass").unwrap().as_numeric(), Some(18.015));
    assert_eq!(props.get("state").unwrap().as_string(), Some("liquid"));
}
