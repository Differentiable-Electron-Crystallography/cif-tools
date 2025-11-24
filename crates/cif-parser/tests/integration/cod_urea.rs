// tests/integration/cod_urea.rs
// Integration tests for CIF file from Crystallography Open Database (COD)
// Note: This file contains Silicon structure data (COD entry 9011998)

use crate::fixture_path;
use cif_parser::Document;

#[test]
fn test_parse_cod_urea() {
    let path = fixture_path("cod_urea.cif");
    let doc = Document::from_file(&path).expect("Failed to parse COD CIF");

    assert_eq!(doc.blocks.len(), 1);
    let block = &doc.blocks[0];
    assert_eq!(block.name, "9011998");
}

#[test]
fn test_cod_urea_cell_parameters() {
    let path = fixture_path("cod_urea.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    // Cubic cell (all lengths equal)
    let a = block
        .get_item("_cell_length_a")
        .unwrap()
        .as_numeric()
        .unwrap();
    let b = block
        .get_item("_cell_length_b")
        .unwrap()
        .as_numeric()
        .unwrap();
    let c = block
        .get_item("_cell_length_c")
        .unwrap()
        .as_numeric()
        .unwrap();

    assert_eq!(a, 5.430941);
    assert_eq!(b, 5.430941);
    assert_eq!(c, 5.430941);

    // All angles 90 for cubic
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
fn test_cod_urea_space_group() {
    let path = fixture_path("cod_urea.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    assert_eq!(
        block
            .get_item("_symmetry_space_group_name_H-M")
            .unwrap()
            .as_string()
            .unwrap(),
        "F d -3 m"
    );
    assert_eq!(
        block
            .get_item("_space_group_IT_number")
            .unwrap()
            .as_numeric()
            .unwrap(),
        227.0
    );
}

#[test]
fn test_cod_urea_symmetry_operations() {
    let path = fixture_path("cod_urea.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    let symm_loop = block.find_loop("_space_group_symop_operation_xyz").unwrap();

    // Fd-3m has 192 symmetry operations (face-centered cubic with diamond glide)
    assert_eq!(symm_loop.len(), 192);

    // First operation should be identity
    assert_eq!(
        symm_loop
            .get_by_tag(0, "_space_group_symop_operation_xyz")
            .unwrap()
            .as_string()
            .unwrap(),
        "x,y,z"
    );
}

#[test]
fn test_cod_urea_atom_sites() {
    let path = fixture_path("cod_urea.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    let atom_loop = block.find_loop("_atom_site_label").unwrap();

    // Silicon structure has 1 atom in asymmetric unit
    assert_eq!(atom_loop.len(), 1);

    assert_eq!(
        atom_loop
            .get_by_tag(0, "_atom_site_label")
            .unwrap()
            .as_string()
            .unwrap(),
        "Si"
    );

    // Check fractional coordinates (origin for diamond structure)
    assert_eq!(
        atom_loop
            .get_by_tag(0, "_atom_site_fract_x")
            .unwrap()
            .as_numeric()
            .unwrap(),
        0.0
    );
    assert_eq!(
        atom_loop
            .get_by_tag(0, "_atom_site_fract_y")
            .unwrap()
            .as_numeric()
            .unwrap(),
        0.0
    );
    assert_eq!(
        atom_loop
            .get_by_tag(0, "_atom_site_fract_z")
            .unwrap()
            .as_numeric()
            .unwrap(),
        0.0
    );
}

#[test]
fn test_cod_urea_bibliographic_info() {
    let path = fixture_path("cod_urea.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    // Check publication info is parsed correctly
    assert_eq!(
        block
            .get_item("_journal_name_full")
            .unwrap()
            .as_string()
            .unwrap(),
        "Journal of Applied Crystallography"
    );
    assert_eq!(
        block
            .get_item("_journal_volume")
            .unwrap()
            .as_numeric()
            .unwrap(),
        8.0
    );
    assert_eq!(
        block
            .get_item("_journal_year")
            .unwrap()
            .as_numeric()
            .unwrap(),
        1975.0
    );

    // Author loop
    let author_loop = block.find_loop("_publ_author_name").unwrap();
    assert_eq!(author_loop.len(), 3);
}

#[test]
fn test_cod_urea_text_field() {
    let path = fixture_path("cod_urea.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    // Publication title is in a text field (semicolon delimited)
    let title = block
        .get_item("_publ_section_title")
        .unwrap()
        .as_string()
        .unwrap();

    assert!(title.contains("lattice constants"));
    assert!(title.contains("silicon"));
}
