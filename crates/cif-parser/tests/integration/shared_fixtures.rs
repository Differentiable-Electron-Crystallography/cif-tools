// tests/integration/shared_fixtures.rs
//
// Comprehensive integration tests using shared fixtures.
// These tests are designed to be ported to Python and JavaScript for test parity.

use crate::fixture_path;
use cif_parser::{CifValue, CifVersion, Document};

// =============================================================================
// simple.cif - Basic CIF with unknown (?) and not-applicable (.) values
// =============================================================================

#[test]
fn test_simple_parse() {
    let path = fixture_path("simple.cif");
    let doc = Document::from_file(&path).expect("Failed to parse simple.cif");

    assert_eq!(doc.blocks.len(), 1);
    assert_eq!(doc.blocks[0].name, "simple");
}

#[test]
fn test_simple_unknown_value() {
    let path = fixture_path("simple.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    let value = block.get_item("_temperature_kelvin").unwrap();
    assert!(matches!(value, CifValue::Unknown));
}

#[test]
fn test_simple_not_applicable_value() {
    let path = fixture_path("simple.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    let value = block.get_item("_pressure").unwrap();
    assert!(matches!(value, CifValue::NotApplicable));
}

#[test]
fn test_simple_text_value() {
    let path = fixture_path("simple.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    let value = block.get_item("_title").unwrap();
    assert_eq!(value.as_string().unwrap(), "Simple Test Structure");
}

#[test]
fn test_simple_numeric_value() {
    let path = fixture_path("simple.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    let value = block.get_item("_cell_length_a").unwrap();
    assert_eq!(value.as_numeric().unwrap(), 10.0);
}

// =============================================================================
// loops.cif - Multiple loops (atom sites, bonds)
// =============================================================================

#[test]
fn test_loops_parse() {
    let path = fixture_path("loops.cif");
    let doc = Document::from_file(&path).expect("Failed to parse loops.cif");

    assert_eq!(doc.blocks.len(), 1);
    assert_eq!(doc.blocks[0].name, "loops");
}

#[test]
fn test_loops_multiple_loops() {
    let path = fixture_path("loops.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    // Should have 2 loops: atom_site and bond
    assert_eq!(block.loops.len(), 2);
}

#[test]
fn test_loops_atom_site_loop() {
    let path = fixture_path("loops.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    let atom_loop = block.find_loop("_atom_site_label").unwrap();
    assert_eq!(atom_loop.len(), 5); // C1, C2, N1, O1, O2

    // Test accessing by tag
    let first_label = atom_loop
        .get_by_tag(0, "_atom_site_label")
        .unwrap()
        .as_string()
        .unwrap();
    assert_eq!(first_label, "C1");

    // Test getting a column
    let x_coords = atom_loop.get_column("_atom_site_fract_x").unwrap();
    assert_eq!(x_coords.len(), 5);
}

#[test]
fn test_loops_bond_loop() {
    let path = fixture_path("loops.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    let bond_loop = block.find_loop("_bond_type").unwrap();
    assert_eq!(bond_loop.len(), 3); // single, double, triple

    let first_type = bond_loop
        .get_by_tag(0, "_bond_type")
        .unwrap()
        .as_string()
        .unwrap();
    assert_eq!(first_type, "single");

    let first_length = bond_loop
        .get_by_tag(0, "_bond_length")
        .unwrap()
        .as_numeric()
        .unwrap();
    assert!((first_length - 1.54).abs() < 0.01);
}

// =============================================================================
// complex.cif - Save frames, multiple blocks
// =============================================================================

#[test]
fn test_complex_parse() {
    let path = fixture_path("complex.cif");
    let doc = Document::from_file(&path).expect("Failed to parse complex.cif");

    // Should have 2 data blocks
    assert_eq!(doc.blocks.len(), 2);
}

#[test]
fn test_complex_multiple_blocks() {
    let path = fixture_path("complex.cif");
    let doc = Document::from_file(&path).unwrap();

    assert_eq!(doc.blocks[0].name, "block1");
    assert_eq!(doc.blocks[1].name, "block2");

    // Access by name
    let block2 = doc.get_block("block2").unwrap();
    assert_eq!(
        block2.get_item("_title").unwrap().as_string().unwrap(),
        "Second Data Block"
    );
}

#[test]
fn test_complex_save_frame() {
    let path = fixture_path("complex.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    // Should have 1 save frame
    assert_eq!(block.frames.len(), 1);
    assert_eq!(block.frames[0].name, "frame1");

    // Access frame items
    let frame = &block.frames[0];
    assert_eq!(
        frame
            .get_item("_frame_category")
            .unwrap()
            .as_string()
            .unwrap(),
        "restraints"
    );
}

// =============================================================================
// pycifrw_xanthine.cif - Uncertainty values (NumericWithUncertainty)
// =============================================================================

#[test]
fn test_xanthine_uncertainty_detection() {
    let path = fixture_path("pycifrw_xanthine.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    // Cell length a has uncertainty: 10.01(11)
    let cell_a = block.get_item("_cell_length_a").unwrap();
    assert!(
        matches!(cell_a, CifValue::NumericWithUncertainty { .. }),
        "Expected NumericWithUncertainty for _cell_length_a"
    );
}

#[test]
fn test_xanthine_uncertainty_value() {
    let path = fixture_path("pycifrw_xanthine.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    // 10.01(11) means value=10.01, uncertainty=0.11
    let cell_a = block.get_item("_cell_length_a").unwrap();
    let value = cell_a.as_numeric().unwrap();
    let uncertainty = cell_a.uncertainty().unwrap();

    assert!((value - 10.01).abs() < 0.001);
    assert!((uncertainty - 0.11).abs() < 0.001);
}

#[test]
fn test_xanthine_uncertainty_as_tuple() {
    let path = fixture_path("pycifrw_xanthine.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    // Test as_numeric_with_uncertainty() returns both
    let cell_a = block.get_item("_cell_length_a").unwrap();
    let (value, uncertainty) = cell_a.as_numeric_with_uncertainty().unwrap();

    assert!((value - 10.01).abs() < 0.001);
    assert!((uncertainty - 0.11).abs() < 0.001);
}

#[test]
fn test_xanthine_multiple_uncertainties() {
    let path = fixture_path("pycifrw_xanthine.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    // Test multiple values with uncertainty
    // _cell_length_b: 18.23(8) -> value=18.23, uncertainty=0.08
    let cell_b = block.get_item("_cell_length_b").unwrap();
    assert!((cell_b.as_numeric().unwrap() - 18.23).abs() < 0.001);
    assert!((cell_b.uncertainty().unwrap() - 0.08).abs() < 0.001);

    // _cell_length_c: 6.93(13) -> value=6.93, uncertainty=0.13
    let cell_c = block.get_item("_cell_length_c").unwrap();
    assert!((cell_c.as_numeric().unwrap() - 6.93).abs() < 0.001);
    assert!((cell_c.uncertainty().unwrap() - 0.13).abs() < 0.001);

    // _cell_angle_beta: 107.5(9) -> value=107.5, uncertainty=0.9
    let beta = block.get_item("_cell_angle_beta").unwrap();
    assert!((beta.as_numeric().unwrap() - 107.5).abs() < 0.1);
    assert!((beta.uncertainty().unwrap() - 0.9).abs() < 0.1);
}

#[test]
fn test_xanthine_plain_numeric_no_uncertainty() {
    let path = fixture_path("pycifrw_xanthine.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    // _cell_angle_alpha is plain 90.0 (no uncertainty)
    let alpha = block.get_item("_cell_angle_alpha").unwrap();
    assert!(matches!(alpha, CifValue::Numeric(_)));
    assert!(alpha.uncertainty().is_none());
}

// =============================================================================
// crystalmaker_LuAG.cif - High precision uncertainty values
// =============================================================================

#[test]
fn test_luag_high_precision_uncertainty() {
    let path = fixture_path("crystalmaker_LuAG.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    // 11.910400(4) -> value=11.9104, uncertainty=0.000004
    let cell_a = block.get_item("_cell_length_a").unwrap();
    let (value, uncertainty) = cell_a.as_numeric_with_uncertainty().unwrap();

    assert!((value - 11.9104).abs() < 0.0001);
    assert!((uncertainty - 0.000004).abs() < 0.0000001);
}

#[test]
fn test_luag_zero_uncertainty() {
    let path = fixture_path("crystalmaker_LuAG.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    // 90.000000(0) -> value=90.0, uncertainty=0.0
    let alpha = block.get_item("_cell_angle_alpha").unwrap();
    let (value, uncertainty) = alpha.as_numeric_with_uncertainty().unwrap();

    assert!((value - 90.0).abs() < 0.0001);
    assert!(uncertainty.abs() < 0.0000001);
}

// =============================================================================
// cif2_lists.cif - CIF 2.0 list syntax
// =============================================================================

#[test]
fn test_cif2_lists_version() {
    let path = fixture_path("cif2_lists.cif");
    let doc = Document::from_file(&path).expect("Failed to parse cif2_lists.cif");

    assert_eq!(doc.version, CifVersion::V2_0);
}

#[test]
fn test_cif2_empty_list() {
    let path = fixture_path("cif2_lists.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    let value = block.get_item("_empty_list").unwrap();
    match value {
        CifValue::List(items) => assert!(items.is_empty()),
        _ => panic!("Expected List, got {:?}", value),
    }
}

#[test]
fn test_cif2_single_item_list() {
    let path = fixture_path("cif2_lists.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    let value = block.get_item("_single_item").unwrap();
    match value {
        CifValue::List(items) => {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].as_numeric().unwrap(), 42.0);
        }
        _ => panic!("Expected List, got {:?}", value),
    }
}

#[test]
fn test_cif2_numeric_list() {
    let path = fixture_path("cif2_lists.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    let value = block.get_item("_numeric_list").unwrap();
    match value {
        CifValue::List(items) => {
            assert_eq!(items.len(), 5);
            for (i, item) in items.iter().enumerate() {
                assert_eq!(item.as_numeric().unwrap(), (i + 1) as f64);
            }
        }
        _ => panic!("Expected List, got {:?}", value),
    }
}

#[test]
fn test_cif2_nested_list() {
    let path = fixture_path("cif2_lists.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    let value = block.get_item("_nested_list").unwrap();
    match value {
        CifValue::List(items) => {
            assert_eq!(items.len(), 2);
            // First nested list [1 2]
            let inner1 = items[0].as_list().unwrap();
            assert_eq!(inner1.len(), 2);
            assert_eq!(inner1[0].as_numeric().unwrap(), 1.0);
            assert_eq!(inner1[1].as_numeric().unwrap(), 2.0);
            // Second nested list [3 4]
            let inner2 = items[1].as_list().unwrap();
            assert_eq!(inner2.len(), 2);
            assert_eq!(inner2[0].as_numeric().unwrap(), 3.0);
            assert_eq!(inner2[1].as_numeric().unwrap(), 4.0);
        }
        _ => panic!("Expected List, got {:?}", value),
    }
}

#[test]
fn test_cif2_list_with_unknown() {
    let path = fixture_path("cif2_lists.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    let value = block.get_item("_mixed_with_unknown").unwrap();
    match value {
        CifValue::List(items) => {
            assert_eq!(items.len(), 4);
            assert_eq!(items[0].as_numeric().unwrap(), 1.0);
            assert_eq!(items[1].as_numeric().unwrap(), 2.0);
            assert!(matches!(items[2], CifValue::Unknown));
            assert_eq!(items[3].as_numeric().unwrap(), 4.0);
        }
        _ => panic!("Expected List, got {:?}", value),
    }
}

// =============================================================================
// cif2_tables.cif - CIF 2.0 table syntax
// =============================================================================

#[test]
fn test_cif2_tables_version() {
    let path = fixture_path("cif2_tables.cif");
    let doc = Document::from_file(&path).expect("Failed to parse cif2_tables.cif");

    assert_eq!(doc.version, CifVersion::V2_0);
}

#[test]
fn test_cif2_empty_table() {
    let path = fixture_path("cif2_tables.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    let value = block.get_item("_empty_table").unwrap();
    match value {
        CifValue::Table(map) => assert!(map.is_empty()),
        _ => panic!("Expected Table, got {:?}", value),
    }
}

#[test]
fn test_cif2_simple_table() {
    let path = fixture_path("cif2_tables.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    let value = block.get_item("_simple_table").unwrap();
    match value {
        CifValue::Table(map) => {
            assert_eq!(map.len(), 2);
            assert_eq!(map.get("a").unwrap().as_numeric().unwrap(), 1.0);
            assert_eq!(map.get("b").unwrap().as_numeric().unwrap(), 2.0);
        }
        _ => panic!("Expected Table, got {:?}", value),
    }
}

#[test]
fn test_cif2_coordinates_table() {
    let path = fixture_path("cif2_tables.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    let value = block.get_item("_coordinates").unwrap();
    match value {
        CifValue::Table(map) => {
            assert_eq!(map.len(), 3);
            assert_eq!(map.get("x").unwrap().as_numeric().unwrap(), 1.5);
            assert_eq!(map.get("y").unwrap().as_numeric().unwrap(), 2.5);
            assert_eq!(map.get("z").unwrap().as_numeric().unwrap(), 3.5);
        }
        _ => panic!("Expected Table, got {:?}", value),
    }
}

#[test]
fn test_cif2_table_with_unknown() {
    let path = fixture_path("cif2_tables.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    let value = block.get_item("_with_unknown").unwrap();
    match value {
        CifValue::Table(map) => {
            assert_eq!(map.len(), 2);
            assert_eq!(map.get("value").unwrap().as_numeric().unwrap(), 42.0);
            assert!(matches!(map.get("error").unwrap(), CifValue::Unknown));
        }
        _ => panic!("Expected Table, got {:?}", value),
    }
}
