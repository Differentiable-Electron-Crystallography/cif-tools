//! Parse options and result types for the CIF parser.
//!
//! This module provides the builder pattern for configuring parsing behavior
//! and the result type for advanced parsing operations.

use crate::ast::CifDocument;
use crate::rules::VersionViolation;

/// Options for parsing CIF documents.
///
/// Use the builder pattern to configure parsing behavior:
///
/// ```
/// use cif_parser::ParseOptions;
///
/// let options = ParseOptions::new()
///     .upgrade_guidance(true);
/// ```
#[derive(Debug, Clone, Default)]
pub struct ParseOptions {
    /// Collect upgrade guidance (what would make CIF 1.1 valid CIF 2.0)
    pub upgrade_guidance: bool,
}

impl ParseOptions {
    /// Create new default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable or disable upgrade guidance collection.
    ///
    /// When enabled and parsing a CIF 1.1 file, the parser will also check
    /// what changes would be needed to make it valid CIF 2.0.
    ///
    /// # Example
    ///
    /// ```
    /// use cif_parser::ParseOptions;
    ///
    /// let options = ParseOptions::new().upgrade_guidance(true);
    /// ```
    pub fn upgrade_guidance(mut self, enabled: bool) -> Self {
        self.upgrade_guidance = enabled;
        self
    }
}

/// Result of parsing with options.
///
/// Contains both the parsed document and any upgrade issues found
/// (if `upgrade_guidance` was enabled).
#[derive(Debug)]
pub struct ParseResult {
    /// The parsed CIF document
    pub document: CifDocument,

    /// Upgrade issues found (empty unless `upgrade_guidance` was enabled AND file is CIF 1.1)
    ///
    /// Each issue describes what would need to change to make the file valid CIF 2.0.
    pub upgrade_issues: Vec<VersionViolation>,
}

impl ParseResult {
    /// Create a new parse result.
    pub fn new(document: CifDocument, upgrade_issues: Vec<VersionViolation>) -> Self {
        Self {
            document,
            upgrade_issues,
        }
    }

    /// Check if the document has any upgrade issues.
    pub fn has_upgrade_issues(&self) -> bool {
        !self.upgrade_issues.is_empty()
    }
}
