// Example for parsing mmCIF/PDBx files

use cif_parser::Document;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Example of parsing a PDBx/mmCIF file
    let mmcif_content = r#"
data_1ABC
_entry.id   1ABC
_struct.title 'EXAMPLE PROTEIN STRUCTURE'

loop_
_atom_site.group_PDB
_atom_site.id
_atom_site.type_symbol
_atom_site.label_atom_id
_atom_site.label_comp_id
_atom_site.label_asym_id
_atom_site.label_seq_id
_atom_site.Cartn_x
_atom_site.Cartn_y
_atom_site.Cartn_z
ATOM   1   N  N   MET A 1   10.000  20.000  30.000
ATOM   2   C  CA  MET A 1   11.000  21.000  31.000
ATOM   3   C  C   MET A 1   12.000  22.000  32.000
ATOM   4   O  O   MET A 1   13.000  23.000  33.000
"#;

    let doc = Document::parse(mmcif_content)?;
    let block = doc.first_block().expect("No data blocks found");

    // Get entry information
    if let Some(entry_id) = block.get_item("_entry.id") {
        println!("PDB ID: {}", entry_id.as_string().unwrap_or("?"));
    }

    if let Some(title) = block.get_item("_struct.title") {
        println!("Title: {}", title.as_string().unwrap_or("?"));
    }

    // Parse atom coordinates
    if let Some(atom_loop) = block.find_loop("_atom_site.id") {
        println!("\nAtom coordinates:");

        for i in 0..atom_loop.len().min(5) {
            // Show first 5 atoms
            let atom_id = atom_loop
                .get_by_tag(i, "_atom_site.id")
                .and_then(|v| v.as_numeric())
                .unwrap_or(0.0) as i32;
            let atom_name = atom_loop
                .get_by_tag(i, "_atom_site.label_atom_id")
                .and_then(|v| v.as_string())
                .unwrap_or("?");
            let res_name = atom_loop
                .get_by_tag(i, "_atom_site.label_comp_id")
                .and_then(|v| v.as_string())
                .unwrap_or("?");
            let x = atom_loop
                .get_by_tag(i, "_atom_site.Cartn_x")
                .and_then(|v| v.as_numeric())
                .unwrap_or(0.0);
            let y = atom_loop
                .get_by_tag(i, "_atom_site.Cartn_y")
                .and_then(|v| v.as_numeric())
                .unwrap_or(0.0);
            let z = atom_loop
                .get_by_tag(i, "_atom_site.Cartn_z")
                .and_then(|v| v.as_numeric())
                .unwrap_or(0.0);

            println!(
                "  Atom {}: {} in {} at ({:.3}, {:.3}, {:.3})",
                atom_id, atom_name, res_name, x, y, z
            );
        }
    }

    Ok(())
}
