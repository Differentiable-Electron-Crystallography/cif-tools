// tests/integration/ccdc_paracetamol.rs
// Integration tests for CIF file from Cambridge Crystallographic Data Centre (CCDC)

use crate::fixture_path;
use cif_parser::Document;

#[test]
fn test_parse_ccdc_paracetamol() {
    let path = fixture_path("ccdc_paracetamol.cif");
    let doc = Document::from_file(&path).expect("Failed to parse CCDC paracetamol CIF");

    assert_eq!(doc.blocks.len(), 1);
    let block = &doc.blocks[0];
    assert_eq!(block.name, "I");
}

#[test]
fn test_ccdc_paracetamol_cell_parameters() {
    let path = fixture_path("ccdc_paracetamol.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    // Orthorhombic crystal system
    assert_eq!(
        block
            .get_item("_symmetry_cell_setting")
            .unwrap()
            .as_string()
            .unwrap(),
        "orthorhombic"
    );

    // Cell lengths
    assert_eq!(
        block
            .get_item("_cell_length_a")
            .unwrap()
            .as_numeric()
            .unwrap(),
        11.76
    );
    assert_eq!(
        block
            .get_item("_cell_length_b")
            .unwrap()
            .as_numeric()
            .unwrap(),
        7.232
    );
    assert_eq!(
        block
            .get_item("_cell_length_c")
            .unwrap()
            .as_numeric()
            .unwrap(),
        17.16
    );

    // Cell angles (all 90 for orthorhombic)
    assert_eq!(
        block
            .get_item("_cell_angle_alpha")
            .unwrap()
            .as_numeric()
            .unwrap(),
        90.0
    );
    assert_eq!(
        block
            .get_item("_cell_angle_beta")
            .unwrap()
            .as_numeric()
            .unwrap(),
        90.0
    );
    assert_eq!(
        block
            .get_item("_cell_angle_gamma")
            .unwrap()
            .as_numeric()
            .unwrap(),
        90.0
    );
}

#[test]
fn test_ccdc_paracetamol_space_group() {
    let path = fixture_path("ccdc_paracetamol.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    assert_eq!(
        block
            .get_item("_symmetry_space_group_name_H-M")
            .unwrap()
            .as_string()
            .unwrap(),
        "P b c a"
    );
    assert_eq!(
        block
            .get_item("_symmetry_Int_Tables_number")
            .unwrap()
            .as_numeric()
            .unwrap(),
        61.0
    );
}

#[test]
fn test_ccdc_paracetamol_symmetry_loop() {
    let path = fixture_path("ccdc_paracetamol.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    // Find the symmetry operations loop
    let symm_loop = block.find_loop("_symmetry_equiv_pos_site_id").unwrap();

    // Pbca has 8 symmetry operations
    assert_eq!(symm_loop.len(), 8);
    assert_eq!(symm_loop.tags.len(), 2);

    // Check first operation is identity
    assert_eq!(
        symm_loop
            .get_by_tag(0, "_symmetry_equiv_pos_as_xyz")
            .unwrap()
            .as_string()
            .unwrap(),
        "x,y,z"
    );
}

#[test]
fn test_ccdc_paracetamol_atom_sites() {
    let path = fixture_path("ccdc_paracetamol.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    let atom_loop = block.find_loop("_atom_site_label").unwrap();

    // Paracetamol has 20 atoms in asymmetric unit
    assert_eq!(atom_loop.len(), 20);

    // Check loop has expected tags
    assert!(atom_loop.tags.contains(&"_atom_site_label".to_string()));
    assert!(atom_loop
        .tags
        .contains(&"_atom_site_type_symbol".to_string()));
    assert!(atom_loop.tags.contains(&"_atom_site_fract_x".to_string()));
    assert!(atom_loop.tags.contains(&"_atom_site_fract_y".to_string()));
    assert!(atom_loop.tags.contains(&"_atom_site_fract_z".to_string()));

    // Check first atom (O1)
    assert_eq!(
        atom_loop
            .get_by_tag(0, "_atom_site_label")
            .unwrap()
            .as_string()
            .unwrap(),
        "O1"
    );
    assert_eq!(
        atom_loop
            .get_by_tag(0, "_atom_site_type_symbol")
            .unwrap()
            .as_string()
            .unwrap(),
        "O"
    );
}
