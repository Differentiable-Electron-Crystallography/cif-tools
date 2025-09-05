// Run with: cargo run --example basic_usage

use cif_parser::Document;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Example 1: Parse a simple CIF file
    let cif_content = r#"
data_example
_cell.length_a   10.000
_cell.length_b   10.000
_cell.length_c   15.000
_cell.angle_alpha 90.00
_cell.angle_beta  90.00
_cell.angle_gamma 90.00

loop_
_atom_site.label
_atom_site.type_symbol
_atom_site.fract_x
_atom_site.fract_y
_atom_site.fract_z
C1  C  0.1234  0.5678  0.9012
N1  N  0.2345  0.6789  0.0123
O1  O  0.3456  0.7890  0.1234
"#;

    // Parse the CIF content
    let doc = Document::parse(cif_content)?;

    // Access the first (and only) data block
    let block = doc.first_block().expect("No data blocks found");
    println!("Data block name: {}", block.name);

    // Access individual data items
    if let Some(cell_a) = block.get_item("_cell.length_a") {
        if let Some(value) = cell_a.as_numeric() {
            println!("Cell length a: {:.3}", value);
        }
    }

    // Access loop data
    if let Some(atom_loop) = block.find_loop("_atom_site.label") {
        println!("\nAtom sites:");
        println!("Number of atoms: {}", atom_loop.len());

        // Iterate through all atoms
        for i in 0..atom_loop.len() {
            let label = atom_loop
                .get_by_tag(i, "_atom_site.label")
                .and_then(|v| v.as_string())
                .unwrap_or("?");
            let x = atom_loop
                .get_by_tag(i, "_atom_site.fract_x")
                .and_then(|v| v.as_numeric())
                .unwrap_or(0.0);
            let y = atom_loop
                .get_by_tag(i, "_atom_site.fract_y")
                .and_then(|v| v.as_numeric())
                .unwrap_or(0.0);
            let z = atom_loop
                .get_by_tag(i, "_atom_site.fract_z")
                .and_then(|v| v.as_numeric())
                .unwrap_or(0.0);

            println!("  {}: ({:.4}, {:.4}, {:.4})", label, x, y, z);
        }
    }

    Ok(())
}
