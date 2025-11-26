//! Operator types for dREL expressions and statements

use serde::{Deserialize, Serialize};

/// Binary operators for dREL expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BinaryOperator {
    // Arithmetic
    /// Addition (+)
    Add,
    /// Subtraction (-)
    Sub,
    /// Multiplication (*), also dot product for vectors
    Mul,
    /// Division (/)
    Div,
    /// Power (**)
    Power,
    /// Cross product (^) for vectors
    Cross,

    // Comparison
    /// Equal (==)
    Eq,
    /// Not equal (!=)
    Ne,
    /// Less than (<)
    Lt,
    /// Greater than (>)
    Gt,
    /// Less than or equal (<=)
    Le,
    /// Greater than or equal (>=)
    Ge,
    /// Membership test (in)
    In,
    /// Negated membership test (not in)
    NotIn,

    // Logical
    /// Logical AND (and, &&)
    And,
    /// Logical OR (or, ||)
    Or,
}

impl BinaryOperator {
    /// Returns true if this is an arithmetic operator
    pub fn is_arithmetic(&self) -> bool {
        matches!(
            self,
            Self::Add | Self::Sub | Self::Mul | Self::Div | Self::Power | Self::Cross
        )
    }

    /// Returns true if this is a comparison operator
    pub fn is_comparison(&self) -> bool {
        matches!(
            self,
            Self::Eq
                | Self::Ne
                | Self::Lt
                | Self::Gt
                | Self::Le
                | Self::Ge
                | Self::In
                | Self::NotIn
        )
    }

    /// Returns true if this is a logical operator
    pub fn is_logical(&self) -> bool {
        matches!(self, Self::And | Self::Or)
    }

    /// Get the operator symbol as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
            Self::Power => "**",
            Self::Cross => "^",
            Self::Eq => "==",
            Self::Ne => "!=",
            Self::Lt => "<",
            Self::Gt => ">",
            Self::Le => "<=",
            Self::Ge => ">=",
            Self::In => "in",
            Self::NotIn => "not in",
            Self::And => "and",
            Self::Or => "or",
        }
    }
}

/// Unary operators for dREL expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnaryOperator {
    /// Unary positive (+)
    Pos,
    /// Unary negative (-)
    Neg,
    /// Logical NOT (not, !)
    Not,
}

impl UnaryOperator {
    /// Get the operator symbol as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pos => "+",
            Self::Neg => "-",
            Self::Not => "not",
        }
    }
}

/// Assignment operators for dREL statements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssignOp {
    /// Simple assignment (=)
    Assign,
    /// Add and assign (+=)
    AddAssign,
    /// Subtract and assign (-=)
    SubAssign,
    /// Multiply and assign (*=)
    MulAssign,
    /// Append to list/matrix (++=)
    AppendAssign,
    /// Prepend to list/matrix (--=)
    PrependAssign,
}

impl AssignOp {
    /// Get the operator symbol as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Assign => "=",
            Self::AddAssign => "+=",
            Self::SubAssign => "-=",
            Self::MulAssign => "*=",
            Self::AppendAssign => "++=",
            Self::PrependAssign => "--=",
        }
    }

    /// Returns true if this is a compound assignment (not simple =)
    pub fn is_compound(&self) -> bool {
        !matches!(self, Self::Assign)
    }
}
