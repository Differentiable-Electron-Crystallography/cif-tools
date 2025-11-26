//! Parser module for converting PEST parse trees to AST
//!
//! This module handles the conversion from PEST's parse tree representation
//! to our typed AST structures.

use crate::ast::{Expr, Stmt};
use crate::error::DrelError;
use crate::Rule;
use pest::iterators::Pairs;

mod expr;
mod helpers;
mod stmt;

/// Parse a complete program from PEST pairs
pub fn parse_program(pairs: Pairs<Rule>) -> Result<Vec<Stmt>, DrelError> {
    let mut statements = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::program => {
                for inner in pair.into_inner() {
                    if inner.as_rule() != Rule::EOI {
                        statements.push(stmt::parse_stmt(inner)?);
                    }
                }
            }
            Rule::EOI => {}
            _ => {
                statements.push(stmt::parse_stmt(pair)?);
            }
        }
    }

    Ok(statements)
}

/// Parse a single expression from PEST pairs
pub fn parse_expression(pairs: Pairs<Rule>) -> Result<Expr, DrelError> {
    for pair in pairs {
        match pair.as_rule() {
            Rule::expression => {
                return expr::parse_expr(pair);
            }
            _ => continue,
        }
    }

    Err(DrelError::invalid_structure("No expression found", 0, 0))
}
