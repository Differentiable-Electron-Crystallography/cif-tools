use cif_parser::CifDocument;

#[test]
fn debug_empty_list() {
    let cif = "#\\#CIF_2.0\ndata_test\n_empty []\n";
    let doc = CifDocument::parse(cif).unwrap();
    let block = doc.first_block().unwrap();
    let empty = block.items.get("_empty").unwrap();

    println!("Parsed value: {:?}", empty);
    println!(
        "Is list: {}",
        matches!(empty, cif_parser::CifValue::List(_))
    );

    // Try to get as list
    match empty.as_list() {
        Some(list) => println!("List length: {}", list.len()),
        None => println!("NOT A LIST! Value is: {:?}", empty),
    }
}
