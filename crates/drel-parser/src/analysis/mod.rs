//! Static analysis utilities for dREL AST
//!
//! This module provides functions for analyzing dREL programs without
//! evaluating them, including:
//! - Extracting item references (data names, categories)
//! - Building dependency graphs
//! - Validating references against dictionaries

mod dependencies;
mod references;

pub use dependencies::{build_dependency_graph, DependencyGraph};
pub use references::{extract_references, ItemReference, ReferenceKind};
