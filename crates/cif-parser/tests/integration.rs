//! Integration tests for CIF parsing
//!
//! - Real-world CIF files: ccdc_paracetamol, cod_urea, crystalmaker_luag, pycifrw_xanthine
//! - Synthetic tests: inline CIF content testing parser features

use std::path::PathBuf;

/// Helper to get test fixtures path from project root fixtures/ directory
pub fn fixture_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // crates/cif-parser -> crates/
    path.pop(); // crates/ -> project root
    path.push("fixtures");
    path.push(name);
    path
}

mod integration {
    // Real-world CIF file tests
    pub mod ccdc_paracetamol;
    pub mod cod_urea;
    pub mod crystalmaker_luag;
    pub mod pycifrw_xanthine;

    // Synthetic inline CIF tests
    pub mod synthetic_tests;

    // Shared fixtures tests (designed for parity with Python/JS)
    pub mod shared_fixtures;
}
