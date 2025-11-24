//! Utility functions for traversing PEST parse trees.
//!
//! This module provides helper functions that reduce boilerplate when working
//! with PEST's `Pair<Rule>` structures. These helpers preserve span information
//! for error reporting while simplifying parse tree traversal.

use crate::Rule;
use pest::iterators::Pair;

/// Extract the string content from a parse tree node
///
/// Use this when you need the text but want to keep the `Pair` around
/// for span information in case of errors.
///
/// Note: Currently unused but available for future parser refactorings.
#[allow(dead_code)]
#[inline]
pub(crate) fn extract_text(pair: &Pair<Rule>) -> String {
    pair.as_str().to_string()
}

/// Find the first pair matching a specific rule in an iterator
///
/// Returns the `Pair` itself (which includes span info), not just the text.
/// This allows extracting location information when constructing errors.
///
/// Note: Currently unused but available for future parser refactorings.
#[allow(dead_code)]
pub(crate) fn find_first<'a>(
    pairs: impl Iterator<Item = Pair<'a, Rule>>,
    rule: Rule,
) -> Option<Pair<'a, Rule>> {
    pairs.into_iter().find(|p| p.as_rule() == rule)
}

/// Collect all pairs matching a specific rule
///
/// Note: Currently unused but available for future parser refactorings.
#[allow(dead_code)]
pub(crate) fn collect_rule<'a>(
    pairs: impl Iterator<Item = Pair<'a, Rule>>,
    rule: Rule,
) -> Vec<Pair<'a, Rule>> {
    pairs.filter(|p| p.as_rule() == rule).collect()
}

/// Extract (line, column) location from a parse tree node
///
/// Returns 1-indexed line and column numbers.
pub(crate) fn extract_location(pair: &Pair<Rule>) -> (usize, usize) {
    pair.as_span().start_pos().line_col()
}
