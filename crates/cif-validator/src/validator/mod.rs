//! CIF validation engine.
//!
//! This module provides the core validation logic for checking CIF documents
//! against DDLm dictionaries.

mod engine;

pub use engine::{ValidationEngine, ValidationMode};
