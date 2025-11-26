//! Integration tests using the real cif_core.dic dictionary

use cif_parser::CifDocument;
use cif_validator::{load_dictionary_file, ValidationMode, Validator};

const DICT_PATH: &str = "dics/cif_core.dic";

#[test]
fn test_load_cif_core_dictionary() {
    let dict = load_dictionary_file(DICT_PATH).expect("Failed to load cif_core.dic");

    // Check metadata
    assert_eq!(dict.metadata.title, Some("CIF_CORE".to_string()));
    assert!(dict.metadata.version.is_some());

    // Should have many items
    assert!(
        dict.items.len() > 1000,
        "Expected many items, got {}",
        dict.items.len()
    );
    assert!(
        dict.categories.len() > 50,
        "Expected many categories, got {}",
        dict.categories.len()
    );

    // Check some known items exist
    assert!(dict.has_item("_cell.length_a"), "Missing _cell.length_a");
    assert!(
        dict.has_item("_atom_site.label"),
        "Missing _atom_site.label"
    );
    assert!(
        dict.has_item("_diffrn.ambient_temperature"),
        "Missing _diffrn.ambient_temperature"
    );

    // Check alias resolution
    // Legacy name _cell_length_a should resolve to _cell.length_a
    let canonical = dict.resolve_name("_cell_length_a");
    assert!(
        canonical == "_cell.length_a" || dict.has_item(&canonical),
        "Alias resolution failed for _cell_length_a, got {}",
        canonical
    );
}

#[test]
fn test_validate_simple_cif() {
    let validator = Validator::new()
        .with_dictionary_file(DICT_PATH)
        .expect("Failed to load dictionary")
        .with_mode(ValidationMode::Lenient); // Lenient mode for real-world CIFs

    let cif_content = r#"
data_test_structure
_cell.length_a 10.5
_cell.length_b 10.5
_cell.length_c 15.0
_cell.angle_alpha 90
_cell.angle_beta 90
_cell.angle_gamma 90
"#;

    let doc = CifDocument::parse(cif_content).expect("Failed to parse CIF");
    let result = validator.validate(&doc).expect("Validation failed");

    // In lenient mode, this should pass (even if some items are unknown)
    println!(
        "Validation result: {} errors, {} warnings",
        result.errors.len(),
        result.warnings.len()
    );

    for error in &result.errors {
        println!("  Error: {}", error);
    }
}

#[test]
fn test_validate_atom_site_loop() {
    let validator = Validator::new()
        .with_dictionary_file(DICT_PATH)
        .expect("Failed to load dictionary")
        .with_mode(ValidationMode::Lenient);

    let cif_content = r#"
data_test_structure
loop_
_atom_site.label
_atom_site.type_symbol
_atom_site.fract_x
_atom_site.fract_y
_atom_site.fract_z
Si1 Si 0.0 0.0 0.0
O1  O  0.25 0.25 0.25
"#;

    let doc = CifDocument::parse(cif_content).expect("Failed to parse CIF");
    let result = validator.validate(&doc).expect("Validation failed");

    println!(
        "Atom site validation: {} errors, {} warnings",
        result.errors.len(),
        result.warnings.len()
    );

    for error in &result.errors {
        println!("  Error: {}", error);
    }
    for warning in &result.warnings {
        println!("  Warning: {}", warning);
    }
}

#[test]
fn test_validated_cif_definition_lookup() {
    let validator = Validator::new()
        .with_dictionary_file(DICT_PATH)
        .expect("Failed to load dictionary");

    // Use _diffrn.ambient_pressure which has its type info defined directly in the dictionary
    // (not via _import.get like _cell.length_a)
    let cif_content = r#"
data_test
_diffrn.ambient_pressure 101.325
"#;

    let doc = CifDocument::parse(cif_content).expect("Failed to parse CIF");
    let validated = validator
        .validate_typed(doc)
        .expect("Failed to create ValidatedCif");

    // Get the block and check the definition
    let block = validated.first_block().expect("No block found");
    let (value, def) = block
        .get_with_def("_diffrn.ambient_pressure")
        .expect("Item not found");

    assert!(value.is_numeric());
    assert!(def.is_some());

    let definition = def.unwrap();
    println!("Definition for _diffrn.ambient_pressure:");
    println!("  Name: {}", definition.name);
    println!("  Category: {}", definition.category);
    println!("  Type: {:?}", definition.type_info.contents);
    println!(
        "  Description: {}",
        definition.description.as_deref().unwrap_or("(none)")
    );

    // Check type info - this item has _type.contents Real defined directly
    assert_eq!(
        definition.type_info.contents,
        cif_validator::ContentType::Real
    );
}
