//! Expression types for dREL AST

use super::operator::{BinaryOperator, UnaryOperator};
use super::span::Span;
use serde::{Deserialize, Serialize};

/// dREL Expression with source location
///
/// Every expression carries a [`Span`] indicating where it appears in the source code.
/// The actual expression variant is stored in [`ExprKind`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Expr {
    /// The kind of expression
    pub kind: ExprKind,
    /// Source location of this expression
    pub span: Span,
}

/// dREL Expression variants
///
/// Expressions in dREL can be literals, references, operators, or composite values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExprKind {
    // === Literals ===
    /// Integer literal (decimal, hex, octal, or binary)
    Integer(i64),

    /// Floating-point literal
    Float(f64),

    /// Imaginary number literal (e.g., 3.14j)
    Imaginary {
        /// The imaginary coefficient
        value: f64,
    },

    /// String literal (single, double, or triple quoted)
    String(String),

    /// Null literal
    Null,

    /// Missing value literal
    Missing,

    // === References ===
    /// Simple identifier (variable name)
    Identifier(String),

    /// CIF data name reference (e.g., _cell.length_a)
    DataName {
        /// Category name (e.g., "cell")
        category: String,
        /// Object name (e.g., "length_a")
        object: String,
    },

    // === Operators ===
    /// Binary operation (e.g., a + b, x * y)
    BinaryOp {
        /// Left operand
        left: Box<Expr>,
        /// Operator
        op: BinaryOperator,
        /// Right operand
        right: Box<Expr>,
    },

    /// Unary operation (e.g., -x, not y)
    UnaryOp {
        /// Operator
        op: UnaryOperator,
        /// Operand
        operand: Box<Expr>,
    },

    // === Access ===
    /// Subscription/indexing (e.g., list[0], matrix[i,j], category[.key=value])
    Subscription {
        /// Target being subscripted
        target: Box<Expr>,
        /// Subscript arguments
        subscripts: Vec<Subscript>,
    },

    /// Attribute reference (e.g., obj.attr)
    AttributeRef {
        /// Target object
        target: Box<Expr>,
        /// Attribute name
        attribute: String,
    },

    /// Function call (e.g., Sqrt(x), Sin(angle))
    FunctionCall {
        /// Function being called
        function: Box<Expr>,
        /// Arguments
        args: Vec<Expr>,
    },

    // === Composite ===
    /// List literal (e.g., [1, 2, 3])
    List(Vec<Expr>),

    /// Table literal (e.g., {"key": value})
    Table(Vec<(String, Expr)>),
}

impl Expr {
    /// Create a new expression with the given kind and span
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Create an integer literal expression
    pub fn integer(value: i64, span: Span) -> Self {
        Self::new(ExprKind::Integer(value), span)
    }

    /// Create a float literal expression
    pub fn float(value: f64, span: Span) -> Self {
        Self::new(ExprKind::Float(value), span)
    }

    /// Create an imaginary literal expression
    pub fn imaginary(value: f64, span: Span) -> Self {
        Self::new(ExprKind::Imaginary { value }, span)
    }

    /// Create a string literal expression
    pub fn string(value: impl Into<String>, span: Span) -> Self {
        Self::new(ExprKind::String(value.into()), span)
    }

    /// Create a null literal expression
    pub fn null(span: Span) -> Self {
        Self::new(ExprKind::Null, span)
    }

    /// Create a missing literal expression
    pub fn missing(span: Span) -> Self {
        Self::new(ExprKind::Missing, span)
    }

    /// Create an identifier expression
    pub fn identifier(name: impl Into<String>, span: Span) -> Self {
        Self::new(ExprKind::Identifier(name.into()), span)
    }

    /// Create a data name reference expression
    pub fn data_name(category: impl Into<String>, object: impl Into<String>, span: Span) -> Self {
        Self::new(
            ExprKind::DataName {
                category: category.into(),
                object: object.into(),
            },
            span,
        )
    }

    /// Create a binary operation expression
    ///
    /// The span is computed by merging the left and right operand spans.
    pub fn binary(left: Expr, op: BinaryOperator, right: Expr) -> Self {
        let span = left.span.merge(right.span);
        Self::new(
            ExprKind::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            },
            span,
        )
    }

    /// Create a binary operation expression with explicit span
    pub fn binary_with_span(left: Expr, op: BinaryOperator, right: Expr, span: Span) -> Self {
        Self::new(
            ExprKind::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            },
            span,
        )
    }

    /// Create a unary operation expression
    pub fn unary(op: UnaryOperator, operand: Expr, span: Span) -> Self {
        Self::new(
            ExprKind::UnaryOp {
                op,
                operand: Box::new(operand),
            },
            span,
        )
    }

    /// Create a function call expression
    pub fn call(function: Expr, args: Vec<Expr>, span: Span) -> Self {
        Self::new(
            ExprKind::FunctionCall {
                function: Box::new(function),
                args,
            },
            span,
        )
    }

    /// Create a subscription expression
    pub fn subscript(target: Expr, subscripts: Vec<Subscript>, span: Span) -> Self {
        Self::new(
            ExprKind::Subscription {
                target: Box::new(target),
                subscripts,
            },
            span,
        )
    }

    /// Create an attribute reference expression
    pub fn attr(target: Expr, attribute: impl Into<String>, span: Span) -> Self {
        Self::new(
            ExprKind::AttributeRef {
                target: Box::new(target),
                attribute: attribute.into(),
            },
            span,
        )
    }

    /// Create a list expression
    pub fn list(items: Vec<Expr>, span: Span) -> Self {
        Self::new(ExprKind::List(items), span)
    }

    /// Create a table expression
    pub fn table(entries: Vec<(String, Expr)>, span: Span) -> Self {
        Self::new(ExprKind::Table(entries), span)
    }

    /// Check if this expression is a literal
    pub fn is_literal(&self) -> bool {
        matches!(
            self.kind,
            ExprKind::Integer(_)
                | ExprKind::Float(_)
                | ExprKind::Imaginary { .. }
                | ExprKind::String(_)
                | ExprKind::Null
                | ExprKind::Missing
        )
    }

    /// Check if this expression is a reference (identifier or data name)
    pub fn is_reference(&self) -> bool {
        matches!(
            self.kind,
            ExprKind::Identifier(_) | ExprKind::DataName { .. }
        )
    }

    /// Check if this is a data name reference
    pub fn is_data_name(&self) -> bool {
        matches!(self.kind, ExprKind::DataName { .. })
    }
}

/// Subscript types for array/list/table access
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Subscript {
    /// Simple index (e.g., list[0])
    Index(Expr),

    /// Slice (e.g., list[1:3], list[::2])
    Slice {
        /// Start index (None for beginning)
        start: Option<Box<Expr>>,
        /// Stop index (None for end)
        stop: Option<Box<Expr>>,
        /// Step (None for 1)
        step: Option<Box<Expr>>,
    },

    /// Key match for category lookup (e.g., category[.key = value])
    KeyMatch {
        /// Key name
        key: String,
        /// Value to match
        value: Box<Expr>,
    },
}

impl Subscript {
    /// Create a simple index subscript
    pub fn index(expr: Expr) -> Self {
        Self::Index(expr)
    }

    /// Create a slice subscript
    pub fn slice(start: Option<Expr>, stop: Option<Expr>, step: Option<Expr>) -> Self {
        Self::Slice {
            start: start.map(Box::new),
            stop: stop.map(Box::new),
            step: step.map(Box::new),
        }
    }

    /// Create a key match subscript
    pub fn key_match(key: impl Into<String>, value: Expr) -> Self {
        Self::KeyMatch {
            key: key.into(),
            value: Box::new(value),
        }
    }
}
