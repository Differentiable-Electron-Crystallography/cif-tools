//! # dREL Parser
//!
//! A parser for dREL (dictionary Relational Expression Language), the methods
//! language used in DDLm dictionaries for expressing data relationships in CIF.
//!
//! dREL is designed to express complex data relationships in a simple, canonical
//! form that is readable by non-programmers and can be machine-parsed for
//! validation and analysis.
//!
//! ## Overview
//!
//! From the [dREL paper](https://pubs.acs.org/doi/10.1021/ci300076w):
//! - dREL provides *reference implementations* for validation, not optimized computation
//! - Expressions are canonical and reverse-translatable to typographical formulas
//! - Each script is a pure function with no side effects
//! - Missing values trigger recursive method execution
//!
//! ## Usage
//!
//! ```rust,ignore
//! use drel_parser::{parse, parse_expr, extract_references};
//!
//! // Parse a simple expression
//! let expr = parse_expr("_cell.length_a * _cell.length_b").unwrap();
//!
//! // Parse a full program (multiple statements)
//! let stmts = parse(r#"
//!     Loop t as atom_type {
//!         _cell.atomic_mass += t.number_in_cell * t.atomic_mass
//!     }
//! "#).unwrap();
//!
//! // Extract item references for validation
//! let refs = extract_references(&stmts);
//! ```
//!
//! ## Three Types of dREL Methods
//!
//! 1. **Evaluation methods**: Compute derived values
//! 2. **Definition methods**: Tailor definitions based on instance data
//! 3. **Validation methods**: Boolean consistency tests

pub mod analysis;
pub mod ast;
pub mod error;
mod parser;

// Re-export main types
pub use ast::{
    AssignOp, BinaryOperator, Expr, ExprKind, Program, Span, Stmt, StmtKind, Subscript,
    UnaryOperator,
};
pub use error::DrelError;

// Re-export analysis types
pub use analysis::{
    build_dependency_graph, extract_references, DependencyGraph, ItemReference, ReferenceKind,
};

use pest::Parser;

// PEST generates a Rule enum without docs, so we suppress the warning
#[allow(missing_docs)]
mod pest_parser {
    #[derive(pest_derive::Parser)]
    #[grammar = "drel.pest"]
    pub struct DrelParser;
}

use pest_parser::{DrelParser, Rule};

/// Parse a dREL program (multiple statements)
///
/// # Example
///
/// ```rust,ignore
/// let stmts = drel_parser::parse(r#"
///     _crystal.density = 1.6605 * _cell.atomic_mass / _cell.volume
/// "#)?;
/// ```
pub fn parse(source: &str) -> Result<Vec<Stmt>, DrelError> {
    let pairs = DrelParser::parse(Rule::program, source)?;
    parser::parse_program(pairs)
}

/// Parse a single dREL expression
///
/// # Example
///
/// ```rust,ignore
/// let expr = drel_parser::parse_expr("_cell.length_a * _cell.length_b")?;
/// ```
pub fn parse_expr(source: &str) -> Result<Expr, DrelError> {
    let pairs = DrelParser::parse(Rule::expression, source)?;
    parser::parse_expression(pairs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_assignment() {
        let result = parse("x = 5");
        assert!(result.is_ok(), "Failed to parse: {:?}", result);
    }

    #[test]
    fn test_parse_data_name() {
        let result = parse_expr("_cell.length_a");
        assert!(result.is_ok(), "Failed to parse: {:?}", result);
    }
}
