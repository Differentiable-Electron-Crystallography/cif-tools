//! Benchmark for CIF parsing only (no validator)

use std::time::Instant;

fn main() {
    let dict_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "../cif-validator/dics/cif_core.dic".to_string());

    println!("=== CIF Parsing Benchmark ===\n");
    println!("File: {}\n", dict_path);

    // Read file
    let start = Instant::now();
    let content = std::fs::read_to_string(&dict_path).expect("Failed to read file");
    let read_time = start.elapsed();
    println!(
        "File read: {:?} ({} bytes, {} lines)",
        read_time,
        content.len(),
        content.lines().count()
    );

    // Parse
    let start = Instant::now();
    let doc = cif_parser::CifDocument::parse(&content).expect("Failed to parse CIF");
    let parse_time = start.elapsed();

    println!("CIF parse: {:?}", parse_time);
    println!("  Version: {:?}", doc.version);
    println!("  Blocks: {}", doc.blocks.len());

    if let Some(block) = doc.first_block() {
        println!("  Block name: {}", block.name);
        println!("  Items: {}", block.items.len());
        println!("  Loops: {}", block.loops.len());
        println!("  Frames: {}", block.frames.len());
    }
}
