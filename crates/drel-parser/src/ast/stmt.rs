//! Statement types for dREL AST

use super::expr::Expr;
use super::operator::AssignOp;
use super::span::Span;
use serde::{Deserialize, Serialize};

/// dREL Statement with source location
///
/// Every statement carries a [`Span`] indicating where it appears in the source code.
/// The actual statement variant is stored in [`StmtKind`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stmt {
    /// The kind of statement
    pub kind: StmtKind,
    /// Source location of this statement
    pub span: Span,
}

/// dREL Statement variants
///
/// Statements in dREL include control flow, assignments, and function definitions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StmtKind {
    // === Control Flow ===
    /// If/ElseIf/Else conditional
    If {
        /// Primary condition
        condition: Expr,
        /// Then block
        then_block: Vec<Stmt>,
        /// ElseIf branches (condition, block)
        elseif_blocks: Vec<(Expr, Vec<Stmt>)>,
        /// Optional else block
        else_block: Option<Vec<Stmt>>,
    },

    /// For loop over an iterable
    For {
        /// Loop variable name
        var: String,
        /// Iterable expression
        iterable: Expr,
        /// Loop body
        body: Vec<Stmt>,
    },

    /// Loop over category packets (CIF-specific)
    ///
    /// ```drel
    /// Loop a as atom_site : i Where a.occupancy > 0.5 {
    ///     ...
    /// }
    /// ```
    Loop {
        /// Loop variable name
        var: String,
        /// Category to iterate over
        category: String,
        /// Optional index variable
        index_var: Option<String>,
        /// Optional filter condition
        condition: Option<Expr>,
        /// Loop body
        body: Vec<Stmt>,
    },

    /// Do loop (numeric range iteration)
    ///
    /// ```drel
    /// Do i = 1, 10, 2 {
    ///     ...
    /// }
    /// ```
    Do {
        /// Loop variable name
        var: String,
        /// Start value
        start: Expr,
        /// End value
        end: Expr,
        /// Optional step (default 1)
        step: Option<Expr>,
        /// Loop body
        body: Vec<Stmt>,
    },

    /// Repeat loop (until Break)
    Repeat {
        /// Loop body
        body: Vec<Stmt>,
    },

    /// With statement (local variable binding, maintains packet context)
    ///
    /// ```drel
    /// With t as atom_type {
    ///     ...
    /// }
    /// ```
    With {
        /// Variable name
        var: String,
        /// Value expression
        value: Expr,
        /// Block body
        body: Vec<Stmt>,
    },

    // === Functions ===
    /// Function definition
    FunctionDef {
        /// Function name
        name: String,
        /// Parameter names
        params: Vec<String>,
        /// Function body
        body: Vec<Stmt>,
    },

    // === Loop Control ===
    /// Break out of loop
    Break,

    /// Continue to next iteration
    Next,

    // === Assignment ===
    /// Assignment statement (with various operators)
    Assignment {
        /// Target expression (usually identifier or data name)
        target: Expr,
        /// Assignment operator
        op: AssignOp,
        /// Value expression
        value: Expr,
    },

    // === Expression Statement ===
    /// Expression evaluated for side effects or as final value
    Expr(Expr),
}

impl Stmt {
    /// Create a new statement with the given kind and span
    pub fn new(kind: StmtKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Create a simple assignment statement
    pub fn assign(target: Expr, value: Expr, span: Span) -> Self {
        Self::new(
            StmtKind::Assignment {
                target,
                op: AssignOp::Assign,
                value,
            },
            span,
        )
    }

    /// Create an increment assignment (+=) statement
    pub fn add_assign(target: Expr, value: Expr, span: Span) -> Self {
        Self::new(
            StmtKind::Assignment {
                target,
                op: AssignOp::AddAssign,
                value,
            },
            span,
        )
    }

    /// Create an if statement
    pub fn if_stmt(condition: Expr, then_block: Vec<Stmt>, span: Span) -> Self {
        Self::new(
            StmtKind::If {
                condition,
                then_block,
                elseif_blocks: Vec::new(),
                else_block: None,
            },
            span,
        )
    }

    /// Create a for loop statement
    pub fn for_loop(var: impl Into<String>, iterable: Expr, body: Vec<Stmt>, span: Span) -> Self {
        Self::new(
            StmtKind::For {
                var: var.into(),
                iterable,
                body,
            },
            span,
        )
    }

    /// Create a Loop statement (CIF category iteration)
    pub fn loop_stmt(
        var: impl Into<String>,
        category: impl Into<String>,
        body: Vec<Stmt>,
        span: Span,
    ) -> Self {
        Self::new(
            StmtKind::Loop {
                var: var.into(),
                category: category.into(),
                index_var: None,
                condition: None,
                body,
            },
            span,
        )
    }

    /// Create a Do loop statement
    pub fn do_loop(
        var: impl Into<String>,
        start: Expr,
        end: Expr,
        body: Vec<Stmt>,
        span: Span,
    ) -> Self {
        Self::new(
            StmtKind::Do {
                var: var.into(),
                start,
                end,
                step: None,
                body,
            },
            span,
        )
    }

    /// Create a With statement
    pub fn with_stmt(var: impl Into<String>, value: Expr, body: Vec<Stmt>, span: Span) -> Self {
        Self::new(
            StmtKind::With {
                var: var.into(),
                value,
                body,
            },
            span,
        )
    }

    /// Create a Break statement
    pub fn break_stmt(span: Span) -> Self {
        Self::new(StmtKind::Break, span)
    }

    /// Create a Next statement
    pub fn next_stmt(span: Span) -> Self {
        Self::new(StmtKind::Next, span)
    }

    /// Create a Repeat statement
    pub fn repeat_stmt(body: Vec<Stmt>, span: Span) -> Self {
        Self::new(StmtKind::Repeat { body }, span)
    }

    /// Create an expression statement
    pub fn expr_stmt(expr: Expr, span: Span) -> Self {
        Self::new(StmtKind::Expr(expr), span)
    }

    /// Check if this statement is a control flow statement
    pub fn is_control_flow(&self) -> bool {
        matches!(
            self.kind,
            StmtKind::If { .. }
                | StmtKind::For { .. }
                | StmtKind::Loop { .. }
                | StmtKind::Do { .. }
                | StmtKind::Repeat { .. }
                | StmtKind::With { .. }
                | StmtKind::Break
                | StmtKind::Next
        )
    }

    /// Check if this statement is an assignment
    pub fn is_assignment(&self) -> bool {
        matches!(self.kind, StmtKind::Assignment { .. })
    }
}
