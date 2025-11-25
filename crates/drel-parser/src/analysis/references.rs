//! Extract item references from dREL AST
//!
//! This module provides functions for extracting all references to CIF
//! data items and categories from a dREL program.

use crate::ast::{Expr, ExprKind, Span, Stmt, StmtKind, Subscript};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// The kind of reference found in dREL code
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReferenceKind {
    /// Full data name reference (e.g., _cell.length_a)
    DataName,
    /// Category reference (e.g., atom_site in "Loop a as atom_site")
    Category,
    /// Identifier that may be a category or local variable
    Identifier,
}

/// A reference to a CIF item found in dREL code
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemReference {
    /// The kind of reference
    pub kind: ReferenceKind,
    /// Category name (for DataName: the category; for Category: the category name)
    pub category: String,
    /// Object name (only for DataName references)
    pub object: Option<String>,
    /// Source location where this reference appears
    pub span: Span,
}

impl ItemReference {
    /// Create a data name reference
    pub fn data_name(category: impl Into<String>, object: impl Into<String>, span: Span) -> Self {
        Self {
            kind: ReferenceKind::DataName,
            category: category.into(),
            object: Some(object.into()),
            span,
        }
    }

    /// Create a category reference
    pub fn category(name: impl Into<String>, span: Span) -> Self {
        Self {
            kind: ReferenceKind::Category,
            category: name.into(),
            object: None,
            span,
        }
    }

    /// Create an identifier reference
    pub fn identifier(name: impl Into<String>, span: Span) -> Self {
        Self {
            kind: ReferenceKind::Identifier,
            category: name.into(),
            object: None,
            span,
        }
    }

    /// Get the full name of the reference
    ///
    /// For data names: "_category.object"
    /// For categories/identifiers: just the name
    pub fn full_name(&self) -> String {
        match &self.object {
            Some(obj) => format!("_{}.{}", self.category, obj),
            None => self.category.clone(),
        }
    }

    /// Check if this is a data name reference
    pub fn is_data_name(&self) -> bool {
        matches!(self.kind, ReferenceKind::DataName)
    }

    /// Check if this is a category reference
    pub fn is_category(&self) -> bool {
        matches!(self.kind, ReferenceKind::Category)
    }
}

/// Extract all item references from a dREL program
///
/// This function walks the AST and collects all references to:
/// - Data names (e.g., _cell.length_a)
/// - Categories (e.g., atom_site in Loop statements)
///
/// # Example
///
/// ```rust,ignore
/// use drel_parser::{parse, analysis::extract_references};
///
/// let stmts = parse("_crystal.density = _cell.atomic_mass / _cell.volume")?;
/// let refs = extract_references(&stmts);
/// // refs contains: _crystal.density, _cell.atomic_mass, _cell.volume
/// ```
pub fn extract_references(stmts: &[Stmt]) -> Vec<ItemReference> {
    let mut refs = HashSet::new();
    let mut collector = ReferenceCollector::new(&mut refs);

    for stmt in stmts {
        collector.visit_stmt(stmt);
    }

    refs.into_iter().collect()
}

/// Internal visitor for collecting references
struct ReferenceCollector<'a> {
    refs: &'a mut HashSet<ItemReference>,
    /// Track local variables to avoid treating them as references
    local_vars: HashSet<String>,
}

impl<'a> ReferenceCollector<'a> {
    fn new(refs: &'a mut HashSet<ItemReference>) -> Self {
        Self {
            refs,
            local_vars: HashSet::new(),
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::If {
                condition,
                then_block,
                elseif_blocks,
                else_block,
            } => {
                self.visit_expr(condition);
                for s in then_block {
                    self.visit_stmt(s);
                }
                for (cond, block) in elseif_blocks {
                    self.visit_expr(cond);
                    for s in block {
                        self.visit_stmt(s);
                    }
                }
                if let Some(block) = else_block {
                    for s in block {
                        self.visit_stmt(s);
                    }
                }
            }
            StmtKind::For {
                var,
                iterable,
                body,
            } => {
                self.visit_expr(iterable);
                // Add var to local scope
                self.local_vars.insert(var.clone());
                for s in body {
                    self.visit_stmt(s);
                }
                self.local_vars.remove(var);
            }
            StmtKind::Loop {
                var,
                category,
                index_var,
                condition,
                body,
            } => {
                // Category is a reference - use the statement's span for the category
                // (ideally we'd have a separate span for just the category token)
                self.refs
                    .insert(ItemReference::category(category.clone(), stmt.span));

                // Add loop variables to local scope
                self.local_vars.insert(var.clone());
                if let Some(idx) = index_var {
                    self.local_vars.insert(idx.clone());
                }

                if let Some(cond) = condition {
                    self.visit_expr(cond);
                }
                for s in body {
                    self.visit_stmt(s);
                }

                self.local_vars.remove(var);
                if let Some(idx) = index_var {
                    self.local_vars.remove(idx);
                }
            }
            StmtKind::Do {
                var,
                start,
                end,
                step,
                body,
            } => {
                self.visit_expr(start);
                self.visit_expr(end);
                if let Some(s) = step {
                    self.visit_expr(s);
                }
                self.local_vars.insert(var.clone());
                for s in body {
                    self.visit_stmt(s);
                }
                self.local_vars.remove(var);
            }
            StmtKind::Repeat { body } => {
                for s in body {
                    self.visit_stmt(s);
                }
            }
            StmtKind::With { var, value, body } => {
                self.visit_expr(value);
                self.local_vars.insert(var.clone());
                for s in body {
                    self.visit_stmt(s);
                }
                self.local_vars.remove(var);
            }
            StmtKind::FunctionDef { params, body, .. } => {
                for p in params {
                    self.local_vars.insert(p.clone());
                }
                for s in body {
                    self.visit_stmt(s);
                }
                for p in params {
                    self.local_vars.remove(p);
                }
            }
            StmtKind::Assignment { target, value, .. } => {
                self.visit_expr(target);
                self.visit_expr(value);
            }
            StmtKind::Expr(expr) => {
                self.visit_expr(expr);
            }
            StmtKind::Break | StmtKind::Next => {}
        }
    }

    fn visit_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::DataName { category, object } => {
                self.refs.insert(ItemReference::data_name(
                    category.clone(),
                    object.clone(),
                    expr.span,
                ));
            }
            ExprKind::Identifier(name) => {
                // Only add if not a local variable
                if !self.local_vars.contains(name) {
                    // Could be a category reference or a builtin function
                    // We mark it as Identifier for later resolution
                    self.refs
                        .insert(ItemReference::identifier(name.clone(), expr.span));
                }
            }
            ExprKind::BinaryOp { left, right, .. } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }
            ExprKind::UnaryOp { operand, .. } => {
                self.visit_expr(operand);
            }
            ExprKind::Subscription {
                target, subscripts, ..
            } => {
                self.visit_expr(target);
                for sub in subscripts {
                    self.visit_subscript(sub);
                }
            }
            ExprKind::AttributeRef { target, .. } => {
                self.visit_expr(target);
            }
            ExprKind::FunctionCall { function, args } => {
                self.visit_expr(function);
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            ExprKind::List(items) => {
                for item in items {
                    self.visit_expr(item);
                }
            }
            ExprKind::Table(entries) => {
                for (_, value) in entries {
                    self.visit_expr(value);
                }
            }
            // Literals don't contain references
            ExprKind::Integer(_)
            | ExprKind::Float(_)
            | ExprKind::Imaginary { .. }
            | ExprKind::String(_)
            | ExprKind::Null
            | ExprKind::Missing => {}
        }
    }

    fn visit_subscript(&mut self, sub: &Subscript) {
        match sub {
            Subscript::Index(expr) => {
                self.visit_expr(expr);
            }
            Subscript::Slice { start, stop, step } => {
                if let Some(e) = start {
                    self.visit_expr(e);
                }
                if let Some(e) = stop {
                    self.visit_expr(e);
                }
                if let Some(e) = step {
                    self.visit_expr(e);
                }
            }
            Subscript::KeyMatch { value, .. } => {
                self.visit_expr(value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn test_extract_data_names() {
        let stmts = parse("_crystal.density = _cell.atomic_mass / _cell.volume").unwrap();
        let refs = extract_references(&stmts);

        assert!(refs.iter().any(|r| r.full_name() == "_crystal.density"));
        assert!(refs.iter().any(|r| r.full_name() == "_cell.atomic_mass"));
        assert!(refs.iter().any(|r| r.full_name() == "_cell.volume"));
    }

    #[test]
    fn test_extract_loop_category() {
        let stmts = parse(
            r#"
            Loop t as atom_type {
                x += t.atomic_mass
            }
        "#,
        )
        .unwrap();
        let refs = extract_references(&stmts);

        // atom_type should be a category reference
        assert!(refs
            .iter()
            .any(|r| r.category == "atom_type" && r.is_category()));
        // t should NOT be in refs (it's a local variable)
        assert!(!refs.iter().any(|r| r.category == "t"));
    }

    #[test]
    fn test_references_have_spans() {
        let stmts = parse("_cell.volume").unwrap();
        let refs = extract_references(&stmts);

        let cell_ref = refs.iter().find(|r| r.full_name() == "_cell.volume");
        assert!(cell_ref.is_some());
        let cell_ref = cell_ref.unwrap();
        // Should have a valid span (not default 0,0)
        assert!(cell_ref.span.start_line > 0 || cell_ref.span.start_col > 0);
    }
}
