//! Abstract Syntax Tree types for dREL
//!
//! This module defines the AST representation for dREL programs,
//! including expressions, statements, and operators.
//!
//! All AST nodes carry [`Span`] information tracking their source location.

mod expr;
mod operator;
mod span;
mod stmt;

pub use expr::{Expr, ExprKind, Subscript};
pub use operator::{AssignOp, BinaryOperator, UnaryOperator};
pub use span::Span;
pub use stmt::{Stmt, StmtKind};

use serde::{Deserialize, Serialize};

/// A complete dREL program consisting of multiple statements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    /// The statements in the program
    pub statements: Vec<Stmt>,
}

impl Program {
    /// Create a new program from statements
    pub fn new(statements: Vec<Stmt>) -> Self {
        Self { statements }
    }

    /// Check if the program is empty
    pub fn is_empty(&self) -> bool {
        self.statements.is_empty()
    }

    /// Get the number of statements
    pub fn len(&self) -> usize {
        self.statements.len()
    }
}

impl From<Vec<Stmt>> for Program {
    fn from(statements: Vec<Stmt>) -> Self {
        Self::new(statements)
    }
}
