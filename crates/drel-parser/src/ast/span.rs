//! Source span information for AST nodes

use serde::{Deserialize, Serialize};
use std::fmt;

/// Source location information tracking where an AST node appears in source code.
///
/// Spans track both the start and end positions, enabling precise error messages
/// and IDE features like go-to-definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Span {
    /// Starting line number (1-indexed)
    pub start_line: usize,
    /// Starting column number (1-indexed)
    pub start_col: usize,
    /// Ending line number (1-indexed)
    pub end_line: usize,
    /// Ending column number (1-indexed)
    pub end_col: usize,
}

impl Span {
    /// Create a new span with explicit start and end positions
    pub fn new(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Self {
        Self {
            start_line,
            start_col,
            end_line,
            end_col,
        }
    }

    /// Create a span representing a single point (start = end)
    pub fn point(line: usize, col: usize) -> Self {
        Self {
            start_line: line,
            start_col: col,
            end_line: line,
            end_col: col,
        }
    }

    /// Merge two spans, taking the start of self and end of other.
    ///
    /// Useful for creating spans that cover compound expressions like `a + b`.
    pub fn merge(self, other: Span) -> Self {
        Self {
            start_line: self.start_line,
            start_col: self.start_col,
            end_line: other.end_line,
            end_col: other.end_col,
        }
    }

    /// Check if this span contains a given line and column
    pub fn contains(&self, line: usize, col: usize) -> bool {
        if line < self.start_line || line > self.end_line {
            return false;
        }
        if line == self.start_line && col < self.start_col {
            return false;
        }
        if line == self.end_line && col > self.end_col {
            return false;
        }
        true
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start_line == self.end_line && self.start_col == self.end_col {
            write!(f, "{}:{}", self.start_line, self.start_col)
        } else if self.start_line == self.end_line {
            write!(f, "{}:{}-{}", self.start_line, self.start_col, self.end_col)
        } else {
            write!(
                f,
                "{}:{}-{}:{}",
                self.start_line, self.start_col, self.end_line, self.end_col
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_new() {
        let span = Span::new(1, 5, 1, 10);
        assert_eq!(span.start_line, 1);
        assert_eq!(span.start_col, 5);
        assert_eq!(span.end_line, 1);
        assert_eq!(span.end_col, 10);
    }

    #[test]
    fn test_span_point() {
        let span = Span::point(3, 7);
        assert_eq!(span.start_line, 3);
        assert_eq!(span.start_col, 7);
        assert_eq!(span.end_line, 3);
        assert_eq!(span.end_col, 7);
    }

    #[test]
    fn test_span_merge() {
        let left = Span::new(1, 1, 1, 5);
        let right = Span::new(1, 9, 1, 15);
        let merged = left.merge(right);

        assert_eq!(merged.start_line, 1);
        assert_eq!(merged.start_col, 1);
        assert_eq!(merged.end_line, 1);
        assert_eq!(merged.end_col, 15);
    }

    #[test]
    fn test_span_display() {
        assert_eq!(format!("{}", Span::point(1, 5)), "1:5");
        assert_eq!(format!("{}", Span::new(1, 5, 1, 10)), "1:5-10");
        assert_eq!(format!("{}", Span::new(1, 5, 3, 10)), "1:5-3:10");
    }

    #[test]
    fn test_span_contains() {
        let span = Span::new(2, 5, 4, 10);

        // Inside
        assert!(span.contains(3, 1));
        assert!(span.contains(2, 5));
        assert!(span.contains(4, 10));

        // Outside
        assert!(!span.contains(1, 1));
        assert!(!span.contains(2, 4));
        assert!(!span.contains(4, 11));
        assert!(!span.contains(5, 1));
    }

    #[test]
    fn test_span_default() {
        let span = Span::default();
        assert_eq!(span.start_line, 0);
        assert_eq!(span.start_col, 0);
    }
}
