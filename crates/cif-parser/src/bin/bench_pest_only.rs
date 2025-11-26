//! Benchmark for PEST parsing only (no AST building)

use cif_parser::{CIFParser, Rule};
use pest::Parser;
use std::time::Instant;

fn count_pairs(pair: pest::iterators::Pair<Rule>) -> usize {
    let mut count = 1;
    for inner in pair.into_inner() {
        count += count_pairs(inner);
    }
    count
}

fn main() {
    let dict_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "crates/cif-validator/dics/cif_core.dic".to_string());

    println!("=== PEST Iteration Benchmark ===\n");
    println!("File: {}\n", dict_path);

    // Read file
    let content = std::fs::read_to_string(&dict_path).expect("Failed to read file");
    println!(
        "File size: {} bytes, {} lines",
        content.len(),
        content.lines().count()
    );

    // PEST parse only (lazy - doesn't build full tree yet)
    let start = Instant::now();
    let pairs = CIFParser::parse(Rule::file, &content).expect("Failed to parse");
    let pest_time = start.elapsed();
    println!("1. PEST parse (lazy): {:?}", pest_time);

    // Iterate all pairs recursively (forces full tree traversal)
    let start = Instant::now();
    let mut total_pairs = 0;
    for pair in pairs {
        total_pairs += count_pairs(pair);
    }
    let iter_time = start.elapsed();
    println!(
        "2. Count all pairs recursively: {:?} ({} pairs)",
        iter_time, total_pairs
    );

    // Parse again and build AST
    let start = Instant::now();
    let _doc = cif_parser::CifDocument::parse(&content).expect("Failed to parse");
    let full_time = start.elapsed();
    println!("3. Full parse with AST: {:?}", full_time);

    println!("\nDelta (AST building): {:?}", full_time - pest_time);
}
