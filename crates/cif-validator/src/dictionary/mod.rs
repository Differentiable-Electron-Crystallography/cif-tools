//! Dictionary loading and representation for DDLm dictionaries.
//!
//! This module provides types and functions for:
//! - Representing DDLm dictionary structures (categories, data items, types)
//! - Loading dictionaries from CIF 2.0 files
//! - Validating dictionary internal consistency (dREL references)
//! - Multi-dictionary composition

mod loader;
mod types;
mod validator;

pub use loader::load_dictionary;
pub use types::*;
pub use validator::validate_dictionary;
