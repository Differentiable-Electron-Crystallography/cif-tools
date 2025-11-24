//! CifDocument API tests
//!
//! Tests document-level operations, multi-block handling, and integration

use cif_parser::{CifDocument, CifVersion};

#[test]
fn test_multiple_blocks() {
    let cif = "data_first\n_item1 value1\n\ndata_second\n_item2 value2\n";
    let doc = CifDocument::parse(cif).unwrap();

    assert_eq!(doc.blocks.len(), 2);
    assert_eq!(doc.blocks[0].name, "first");
    assert_eq!(doc.blocks[1].name, "second");
}

#[test]
fn test_complete_cif2_document() {
    let cif = "#\\#CIF_2.0
data_molecule
_name 'Water'
_atoms [
    {'element':'H' 'x':0.0}
    {'element':'O' 'x':1.0}
]
";

    let doc = CifDocument::parse(cif).unwrap();
    assert_eq!(doc.version, CifVersion::V2_0);

    let block = doc.first_block().unwrap();
    assert_eq!(block.name, "molecule");
    assert_eq!(block.items.get("_name").unwrap().as_string(), Some("Water"));

    let atoms = block.items.get("_atoms").unwrap().as_list().unwrap();
    assert_eq!(atoms.len(), 2);
}

#[test]
fn test_first_block() {
    let cif = "data_test\n_item value\n";
    let doc = CifDocument::parse(cif).unwrap();
    assert!(doc.first_block().is_some());
    assert_eq!(doc.first_block().unwrap().name, "test");
}
