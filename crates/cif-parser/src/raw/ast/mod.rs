//! Raw AST types for version-agnostic, lossless CIF parsing.
//!
//! These types preserve all syntactic information from the input,
//! allowing version-specific rules to be applied in a separate resolution pass.

mod block;
mod document;
mod value;

pub use block::*;
pub use document::*;
pub use value::*;
