// Reading from files

use cif_parser;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Parse from file
    // let doc = cif_parser::parse_file("path/to/file.cif")?;

    // Or use the Document type directly
    // let doc = cif_parser::Document::from_file("path/to/file.cif")?;

    // For this example, we'll use a string
    let doc = cif_parser::parse_string("data_test\n_item value")?;

    println!("Successfully parsed {} data blocks", doc.blocks.len());

    Ok(())
}
