//! Raw parsing types for version-agnostic, lossless CIF parsing.
//!
//! This module contains intermediate representations that preserve all syntactic
//! information from the input, allowing version-specific rules to be applied
//! in a separate resolution pass.
//!
//! # Module Organization
//!
//! - `ast` - Raw AST types (RawValue, RawBlock, RawDocument, etc.)

pub mod ast;

// Re-export AST types for convenience
pub use ast::*;
