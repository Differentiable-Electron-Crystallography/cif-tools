//! Version violation types for error reporting and upgrade guidance.

use crate::ast::Span;
use std::fmt;

/// A violation of version-specific rules.
///
/// Contains structured metadata for:
/// - Error reporting (span, message)
/// - Upgrade guidance (suggestion, rule_id)
#[derive(Debug, Clone)]
pub struct VersionViolation {
    /// Source location of the violation
    pub span: Span,
    /// Human-readable error message
    pub message: String,
    /// Suggested fix (for upgrade guidance)
    pub suggestion: Option<String>,
    /// Machine-readable rule identifier
    pub rule_id: &'static str,
}

impl VersionViolation {
    /// Create a new violation with the given span, message, and rule ID.
    pub fn new(span: Span, message: impl Into<String>, rule_id: &'static str) -> Self {
        Self {
            span,
            message: message.into(),
            suggestion: None,
            rule_id,
        }
    }

    /// Add a suggestion for how to fix the violation.
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

impl fmt::Display for VersionViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} at line {}, column {}",
            self.rule_id, self.message, self.span.start_line, self.span.start_col
        )?;
        if let Some(ref suggestion) = self.suggestion {
            write!(f, " (suggestion: {})", suggestion)?;
        }
        Ok(())
    }
}

impl std::error::Error for VersionViolation {}

/// Machine-readable rule identifiers for version violations.
///
/// These can be used for:
/// - Filtering specific violations
/// - Programmatic handling
/// - Documentation references
pub mod rule_ids {
    /// CIF 2.0 does not allow doubled-quote escaping (`''` or `""`).
    /// Use triple-quoted strings instead.
    pub const CIF2_NO_DOUBLED_QUOTES: &str = "cif2-no-doubled-quotes";

    /// CIF 2.0 requires non-empty data block names.
    pub const CIF2_NO_EMPTY_BLOCK_NAME: &str = "cif2-no-empty-block-name";

    /// CIF 2.0 requires non-empty save frame names.
    pub const CIF2_NO_EMPTY_FRAME_NAME: &str = "cif2-no-empty-frame-name";

    // Note: Lists, tables, and triple-quotes in CIF 1.1 are handled via
    // transformations (silent degradation to text), not violations.
}
