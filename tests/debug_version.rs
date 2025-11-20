use cif_parser::CifDocument;

#[test]
fn debug_version_detection() {
    let cif = r#"#\\#CIF_2.0
data_molecule
_name 'Water'
"#;

    println!(
        "First line bytes: {:?}",
        cif.lines().next().unwrap().as_bytes()
    );
    println!("First line: {:?}", cif.lines().next().unwrap());
    println!(
        "Starts with #\\#CIF_2.0: {}",
        cif.lines().next().unwrap().starts_with("#\\#CIF_2.0")
    );

    let doc = CifDocument::parse(cif).unwrap();
    println!("Detected version: {:?}", doc.version);
}
