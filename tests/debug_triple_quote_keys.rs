use cif_parser::CifDocument;

#[test]
fn debug_triple_quoted_keys() {
    let cif = "#\\#CIF_2.0\ndata_test\n_map {\"\"\"long key\"\"\":1.0 '''another key''':2.0}\n";
    let doc = CifDocument::parse(cif).unwrap();
    let block = doc.first_block().unwrap();
    let map = block.items.get("_map").unwrap();

    let table = map.as_table().expect("Should be a table");

    // Print all keys to see what we actually got
    println!("Table keys:");
    for key in table.keys() {
        println!("  '{}'", key);
    }

    // Try to get with and without quotes
    println!("\nTrying to get 'long key': {:?}", table.get("long key"));
    println!(
        "Trying to get '\"\"\"long key\"\"\"': {:?}",
        table.get("\"\"\"long key\"\"\"")
    );
}
