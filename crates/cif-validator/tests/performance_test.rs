//! Performance tests for CIF parsing and dictionary loading.
//!
//! These tests verify that parsing large CIF files completes within reasonable time bounds.
//! They help detect performance regressions in the parser.

use std::time::Instant;

const DICT_PATH: &str = "dics/cif_core.dic";

/// Test that dictionary loading completes within reasonable time (< 5 seconds in release mode).
///
/// This test was added to prevent regression after fixing a performance issue
/// where the text_content grammar rule was O(n²) due to negative lookahead.
#[test]
fn test_dictionary_load_performance() {
    let start = Instant::now();
    let dict = cif_validator::load_dictionary_file(DICT_PATH).expect("Failed to load dictionary");
    let elapsed = start.elapsed();

    // Verify dictionary loaded correctly
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

    // Performance assertion - should complete in under 5 seconds in release mode
    // In debug mode this will be slower, so we allow more time
    #[cfg(not(debug_assertions))]
    assert!(
        elapsed < Duration::from_secs(5),
        "Dictionary loading took too long: {:?}. Expected < 5 seconds.",
        elapsed
    );

    println!(
        "Dictionary loaded in {:?} ({} items, {} categories)",
        elapsed,
        dict.items.len(),
        dict.categories.len()
    );
}

/// Benchmark individual steps of dictionary loading for profiling.
#[test]
fn test_dictionary_load_benchmark() {
    println!("\n=== CIF Dictionary Loading Benchmark ===\n");

    // Step 1: Read file
    let start = Instant::now();
    let content = std::fs::read_to_string(DICT_PATH).expect("Failed to read file");
    let read_time = start.elapsed();
    println!("1. File read: {:?} ({} bytes)", read_time, content.len());

    // Step 2: CIF parsing
    let start = Instant::now();
    let doc = cif_parser::CifDocument::parse(&content).expect("Failed to parse CIF");
    let parse_time = start.elapsed();
    println!("2. CIF parse: {:?}", parse_time);
    println!("   Blocks: {}", doc.blocks.len());
    if let Some(block) = doc.first_block() {
        println!("   Frames: {}", block.frames.len());
        println!("   Items: {}", block.items.len());
    }

    // Step 3: Dictionary loading
    let start = Instant::now();
    let dict = cif_validator::dictionary::load_dictionary(&doc).expect("Failed to load dictionary");
    let load_time = start.elapsed();
    println!("3. Dict load: {:?}", load_time);
    println!("   Categories: {}", dict.categories.len());
    println!("   Items: {}", dict.items.len());
    println!("   Aliases: {}", dict.aliases.len());

    let total = read_time + parse_time + load_time;
    println!("\n=== Total: {:?} ===", total);

    // The CIF parsing step should be the dominant cost
    // After optimization, it should be under 1 second in release mode
    #[cfg(not(debug_assertions))]
    {
        assert!(
            parse_time < Duration::from_secs(3),
            "CIF parsing took too long: {:?}. Expected < 3 seconds.",
            parse_time
        );
        assert!(
            load_time < Duration::from_secs(1),
            "Dictionary loading took too long: {:?}. Expected < 1 second.",
            load_time
        );
    }
}
