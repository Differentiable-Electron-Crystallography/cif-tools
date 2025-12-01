//! Raw parsing types for version-agnostic, lossless CIF parsing according to the cif.pest superset grammar.
//!
//! This module contains intermediate representations that preserve all syntactic
//! information from the input, allowing version-specific rules to be applied
//! in a separate resolution pass.
//!
//! # Module Organization
//!
//! - `ast` - Raw AST types (RawValue, RawBlock, RawDocument, etc.)
//! - `parser` - Pass 1 parsing logic (PEST → Raw AST) [internal]

pub mod ast;
pub(crate) mod parser;

// Re-export AST types for convenience
pub use ast::*;
