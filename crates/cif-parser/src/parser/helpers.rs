//! Utility functions for traversing PEST parse trees.
//!
//! This module provides helper functions that reduce boilerplate when working
//! with PEST's `Pair<Rule>` structures. These helpers preserve span information
//! for error reporting while simplifying parse tree traversal.

use crate::ast::Span;
use crate::Rule;
use pest::iterators::Pair;
use std::cell::RefCell;

// Thread-local line index for fast byte-offset to line/column conversion
// Built once per parse, then used for O(log n) lookups
thread_local! {
    static LINE_INDEX: RefCell<Option<LineIndex>> = const { RefCell::new(None) };
}

/// Index of newline positions for fast line/column lookup
#[derive(Clone)]
pub(crate) struct LineIndex {
    /// Byte offsets of each newline character
    newlines: Vec<usize>,
}

impl LineIndex {
    /// Build a line index from input text
    pub fn new(input: &str) -> Self {
        let newlines: Vec<usize> = input
            .bytes()
            .enumerate()
            .filter(|(_, b)| *b == b'\n')
            .map(|(i, _)| i)
            .collect();
        Self { newlines }
    }

    /// Convert byte offset to (line, column), both 1-indexed
    pub fn line_col(&self, offset: usize) -> (usize, usize) {
        // Binary search to find line number
        let line = match self.newlines.binary_search(&offset) {
            Ok(i) => i + 1,  // Exact match on newline
            Err(i) => i + 1, // Between newlines
        };

        // Calculate column: offset - start of this line
        let line_start = if line == 1 {
            0
        } else {
            self.newlines[line - 2] + 1 // +1 to skip the newline char
        };

        let col = offset - line_start + 1; // 1-indexed
        (line, col)
    }
}

/// Initialize the thread-local line index for the current parse
pub(crate) fn init_line_index(input: &str) {
    LINE_INDEX.with(|idx| {
        *idx.borrow_mut() = Some(LineIndex::new(input));
    });
}

/// Clear the thread-local line index after parsing
pub(crate) fn clear_line_index() {
    LINE_INDEX.with(|idx| {
        *idx.borrow_mut() = None;
    });
}

/// Convert byte offset to line/column using the cached line index
fn offset_to_line_col(offset: usize) -> (usize, usize) {
    LINE_INDEX.with(|idx| {
        if let Some(index) = idx.borrow().as_ref() {
            index.line_col(offset)
        } else {
            // Fallback if no index (shouldn't happen during normal parsing)
            (offset, 0)
        }
    })
}

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
/// Returns 1-indexed (line, column) using the cached line index.
/// PERFORMANCE: O(log n) lookup using pre-built line index.
pub(crate) fn extract_location(pair: &Pair<Rule>) -> (usize, usize) {
    let offset = pair.as_span().start();
    offset_to_line_col(offset)
}

/// Extract a full [`Span`] from a PEST pair.
///
/// Returns a [`Span`] with start and end line/column positions (1-indexed).
/// PERFORMANCE: O(log n) per lookup using pre-built line index.
pub(crate) fn extract_span(pair: &Pair<Rule>) -> Span {
    let pest_span = pair.as_span();
    let (start_line, start_col) = offset_to_line_col(pest_span.start());
    let (end_line, end_col) = offset_to_line_col(pest_span.end());
    Span::new(start_line, start_col, end_line, end_col)
}
