//! Parsing logic for converting PEST parse trees into AST structures.
//!
//! This module contains all the logic for transforming PEST's generic `Pair<Rule>`
//! structures into our typed AST. The parsing logic is completely decoupled from
//! the AST types themselves.
//!
//! # Architecture
//!
//! **Two-pass parsing approach**:
//! 1. **Pass 1 - Raw parsing** (version-agnostic): `input string` → `RawDocument`
//! 2. **Pass 2 - Resolution** (version-specific): `RawDocument` → `CifDocument`
//!
//! This architecture allows version-specific rules to be applied cleanly via
//! the `VersionRules` trait, with `Cif1Rules` and `Cif2Rules` implementations.
//!
//! # Design Principles
//!
//! - **Standalone functions**: Parsing logic uses free functions, not methods on AST types
//! - **Single responsibility**: Each module handles one AST type
//! - **Error handling**: All parsing functions return `Result<T, CifError>`
//! - **No silent failures**: Unknown rules are logged/handled explicitly
//! - **Lossless intermediate representation**: Raw types preserve all input information
//!
//! # Module Organization
//!
//! - `helpers`: Common utility functions for parse tree traversal
//! - `value`: Parse individual CIF values to Raw types
//! - `loop_parser`: Parse loop structures to Raw types
//! - `block`: Parse data blocks and save frames to Raw types
//! - `document`: Parse complete CIF documents (entry point with two-pass resolution according to 1.1 or 2.0 dialect)
//! - `options`: Parse options and result types

pub mod block;
pub mod document;
pub mod helpers;
pub mod loop_parser;
pub mod options;
pub mod value;

pub use document::parse_file;
pub use options::{ParseOptions, ParseResult};
