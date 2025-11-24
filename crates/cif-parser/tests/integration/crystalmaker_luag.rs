// tests/integration/crystalmaker_luag.rs
// Integration tests for CIF file from CrystalMaker software
// Structure: Lutetium Aluminum Garnet (LuAG - Lu3Al5O12)

use crate::fixture_path;
use cif_parser::Document;

#[test]
fn test_parse_crystalmaker_luag() {
    let path = fixture_path("crystalmaker_LuAG.cif");
    let doc = Document::from_file(&path).expect("Failed to parse CrystalMaker LuAG CIF");

    assert_eq!(doc.blocks.len(), 1);
    let block = &doc.blocks[0];
    assert_eq!(block.name, "I");
}

#[test]
fn test_crystalmaker_luag_audit() {
    let path = fixture_path("crystalmaker_LuAG.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    // CrystalMaker adds audit creation method
    let audit = block
        .get_item("_audit_creation_method")
        .unwrap()
        .as_string()
        .unwrap();

    assert!(audit.contains("CrystalMaker"));
}

#[test]
fn test_crystalmaker_luag_cell_parameters() {
    let path = fixture_path("crystalmaker_LuAG.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    // Cubic garnet structure (all lengths equal)
    // CrystalMaker outputs values with uncertainty notation: "11.910400(4)"
    // These are now correctly parsed as NumericWithUncertainty
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

    // Should all be approximately 11.9104
    assert!((a - 11.9104).abs() < 0.001);
    assert!((b - 11.9104).abs() < 0.001);
    assert!((c - 11.9104).abs() < 0.001);

    // Cubic: all angles 90 (also with uncertainty notation)
    assert_eq!(
        block
            .get_item("_cell_angle_alpha")
            .unwrap()
            .as_numeric()
            .unwrap(),
        90.0
    );
}

#[test]
fn test_crystalmaker_luag_space_group() {
    let path = fixture_path("crystalmaker_LuAG.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    assert_eq!(
        block
            .get_item("_symmetry_cell_setting")
            .unwrap()
            .as_string()
            .unwrap(),
        "cubic"
    );

    // Ia-3d space group (garnet structure)
    assert_eq!(
        block
            .get_item("_symmetry_space_group_name_H-M")
            .unwrap()
            .as_string()
            .unwrap(),
        "I a -3 d"
    );
    assert_eq!(
        block
            .get_item("_symmetry_Int_Tables_number")
            .unwrap()
            .as_numeric()
            .unwrap(),
        230.0
    );
}

#[test]
fn test_crystalmaker_luag_symmetry_operations() {
    let path = fixture_path("crystalmaker_LuAG.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    let symm_loop = block.find_loop("_symmetry_equiv_pos_as_xyz").unwrap();

    // Ia-3d has 96 symmetry operations (body-centered cubic with glides)
    assert_eq!(symm_loop.len(), 96);

    // First operation - note CrystalMaker quotes the values
    let first_op = symm_loop
        .get_by_tag(0, "_symmetry_equiv_pos_as_xyz")
        .unwrap()
        .as_string()
        .unwrap();
    assert_eq!(first_op, "+x,+y,+z");
}

#[test]
fn test_crystalmaker_luag_atom_sites() {
    let path = fixture_path("crystalmaker_LuAG.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    let atom_loop = block.find_loop("_atom_site_label").unwrap();

    // LuAG asymmetric unit: Al2, Al3, Lu1, O1 = 4 sites
    assert_eq!(atom_loop.len(), 4);

    // Check tags include occupancy (common in CrystalMaker output)
    assert!(atom_loop.tags.contains(&"_atom_site_occupancy".to_string()));

    // Check atom labels
    let labels: Vec<&str> = (0..4)
        .map(|i| {
            atom_loop
                .get_by_tag(i, "_atom_site_label")
                .unwrap()
                .as_string()
                .unwrap()
        })
        .collect();

    assert!(labels.contains(&"Lu1"));
    assert!(labels.contains(&"Al2"));
    assert!(labels.contains(&"Al3"));
    assert!(labels.contains(&"O1"));
}

#[test]
fn test_crystalmaker_luag_aniso_displacement() {
    let path = fixture_path("crystalmaker_LuAG.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    // CrystalMaker includes anisotropic displacement parameters
    let aniso_loop = block.find_loop("_atom_site_aniso_label").unwrap();

    assert_eq!(aniso_loop.len(), 4);

    // Check aniso tags
    assert!(aniso_loop
        .tags
        .contains(&"_atom_site_aniso_U_11".to_string()));
    assert!(aniso_loop
        .tags
        .contains(&"_atom_site_aniso_U_22".to_string()));
    assert!(aniso_loop
        .tags
        .contains(&"_atom_site_aniso_U_33".to_string()));
    assert!(aniso_loop
        .tags
        .contains(&"_atom_site_aniso_U_12".to_string()));
    assert!(aniso_loop
        .tags
        .contains(&"_atom_site_aniso_U_13".to_string()));
    assert!(aniso_loop
        .tags
        .contains(&"_atom_site_aniso_U_23".to_string()));
}

#[test]
fn test_crystalmaker_luag_multiple_loops() {
    let path = fixture_path("crystalmaker_LuAG.cif");
    let doc = Document::from_file(&path).unwrap();
    let block = &doc.blocks[0];

    // Should have 3 loops: symmetry ops, atom sites, aniso params
    assert_eq!(block.loops.len(), 3);
}
