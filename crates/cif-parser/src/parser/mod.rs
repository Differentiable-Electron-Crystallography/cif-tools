//! Parsing logic for converting PEST parse trees into AST structures.
//!
//! This module contains all the logic for transforming PEST's generic `Pair<Rule>`
//! structures into our typed AST. The parsing logic is completely decoupled from
//! the AST types themselves.
//!
//! # Architecture
//!
//! **Two-stage parsing approach**:
//! 1. **Grammar parsing** (PEST): `input string` → `Pair<Rule>` parse tree
//! 2. **AST construction** (this module): `Pair<Rule>` → typed AST structures
//!
//! # Design Principles
//!
//! - **Standalone functions**: Parsing logic uses free functions, not methods on AST types
//! - **Single responsibility**: Each module handles one AST type
//! - **Error handling**: All parsing functions return `Result<T, CifError>`
//! - **No silent failures**: Unknown rules are logged/handled explicitly
//!
//! # Module Organization
//!
//! - `helpers`: Common utility functions for parse tree traversal
//! - `value`: Parse individual CIF values
//! - `loop_parser`: Parse loop structures
//! - `block`: Parse data blocks and save frames
//! - `document`: Parse complete CIF documents (entry point)

pub mod block;
pub mod document;
pub mod helpers;
pub mod loop_parser;
pub mod value;

pub use document::parse_file;
