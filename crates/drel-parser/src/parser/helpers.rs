//! Helper functions for parsing

use crate::ast::Span;
use crate::Rule;
use pest::iterators::Pair;

/// Extract a full span from a PEST pair
///
/// Returns a [`Span`] with start and end line/column positions.
pub fn span(pair: &Pair<Rule>) -> Span {
    let pest_span = pair.as_span();
    let (start_line, start_col) = pest_span.start_pos().line_col();
    let (end_line, end_col) = pest_span.end_pos().line_col();
    Span::new(start_line, start_col, end_line, end_col)
}

/// Extract line and column from a PEST pair (start position only)
///
/// For full span information, use [`span()`] instead.
pub fn location(pair: &Pair<Rule>) -> (usize, usize) {
    let pest_span = pair.as_span();
    pest_span.start_pos().line_col()
}

/// Get the text content of a pair
pub fn text<'a>(pair: &Pair<'a, Rule>) -> &'a str {
    pair.as_str()
}
