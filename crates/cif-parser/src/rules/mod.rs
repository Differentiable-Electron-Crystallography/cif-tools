//! Version-specific parsing rules for CIF 1.1 and CIF 2.0.
//!
//! This module implements the strategy pattern for handling version differences
//! between CIF specifications. Each trait method encapsulates the **full decision**
//! for a construct - both validation and transformation.
//!
//! # Architecture
//!
//! ```text
//! RawDocument (lossless)
//!       │
//!       ▼
//! VersionRules.resolve()
//!       │
//!       ├─ Cif1Rules: permissive, transforms (e.g., unescape doubled quotes)
//!       └─ Cif2Rules: strict, validates + transforms
//!       │
//!       ▼
//! CifDocument (typed)
//! ```

mod cif1;
mod cif2;
mod helpers;

pub use cif1::Cif1Rules;
pub use cif2::Cif2Rules;

use std::fmt;

use crate::ast::{CifBlock, CifDocument, CifFrame, CifLoop, CifValue, Span};
use crate::raw::{
    RawBlock, RawDocument, RawFrame, RawListSyntax, RawLoop, RawQuotedString, RawTableSyntax,
    RawTextField, RawTripleQuoted, RawUnquoted, RawValue,
};

/// Strategy trait for version-specific CIF parsing rules.
///
/// Each method encapsulates the FULL decision for that construct:
/// - **Validation**: reject invalid constructs (return `Err(VersionViolation)`)
/// - **Transformation**: convert to typed representation (return `Ok(...)`)
///
/// # CIF 1.1 vs CIF 2.0
///
/// | Construct | CIF 1.1 | CIF 2.0 |
/// |-----------|---------|---------|
/// | Empty block name | ✅ Allowed | ❌ Error |
/// | Empty frame name | ✅ Allowed | ❌ Error |
/// | Lists `[...]` | → Text | → List |
/// | Tables `{...}` | → Text | → Table |
/// | Triple quotes | → Text | → Text (parsed) |
/// | Doubled quotes `''` | ✅ Unescape | ❌ Error |
pub trait VersionRules {
    /// Resolve a raw document to a typed CifDocument.
    fn resolve(&self, raw: &RawDocument) -> Result<CifDocument, VersionViolation>;

    /// Resolve a raw value to a typed CifValue.
    /// Dispatches to specific methods based on value type.
    fn resolve_value(&self, raw: &RawValue) -> Result<CifValue, VersionViolation>;

    /// Resolve a quoted string.
    /// - CIF 1.1: Unescapes doubled quotes (transformation)
    /// - CIF 2.0: Rejects doubled quotes (validation)
    fn resolve_quoted(&self, raw: &RawQuotedString) -> Result<CifValue, VersionViolation>;

    /// Resolve a triple-quoted string.
    /// - CIF 1.1: Treats as literal text (transformation)
    /// - CIF 2.0: Extracts content (transformation)
    fn resolve_triple_quoted(&self, raw: &RawTripleQuoted) -> Result<CifValue, VersionViolation>;

    /// Resolve a text field (same in both versions).
    fn resolve_text_field(&self, raw: &RawTextField) -> Result<CifValue, VersionViolation>;

    /// Resolve an unquoted value (same in both versions).
    fn resolve_unquoted(&self, raw: &RawUnquoted) -> Result<CifValue, VersionViolation>;

    /// Resolve list syntax.
    /// - CIF 1.1: Returns as literal text (transformation)
    /// - CIF 2.0: Returns as List value (transformation)
    fn resolve_list(&self, raw: &RawListSyntax) -> Result<CifValue, VersionViolation>;

    /// Resolve table syntax.
    /// - CIF 1.1: Returns as literal text (transformation)
    /// - CIF 2.0: Returns as Table value (transformation)
    fn resolve_table(&self, raw: &RawTableSyntax) -> Result<CifValue, VersionViolation>;

    /// Validate a block name.
    /// - CIF 1.1: Allows empty names
    /// - CIF 2.0: Rejects empty names
    fn validate_block_name(&self, name: &str, span: Span) -> Result<(), VersionViolation>;

    /// Validate a frame name.
    /// - CIF 1.1: Allows empty names
    /// - CIF 2.0: Rejects empty names
    fn validate_frame_name(&self, name: &str, span: Span) -> Result<(), VersionViolation>;

    /// Resolve a complete block (validates name, then resolves contents).
    fn resolve_block(&self, raw: &RawBlock) -> Result<CifBlock, VersionViolation>;

    /// Resolve a complete frame (validates name, then resolves contents).
    fn resolve_frame(&self, raw: &RawFrame) -> Result<CifFrame, VersionViolation>;

    /// Resolve a loop structure.
    fn resolve_loop(&self, raw: &RawLoop) -> Result<CifLoop, VersionViolation>;

    /// Collect all violations without failing (for upgrade guidance).
    /// Walks the entire raw AST and returns all rule violations found.
    fn collect_violations(&self, raw: &RawDocument) -> Vec<VersionViolation>;
}

// ===== Version Violation =====

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
pub mod rule_ids {
    /// CIF 2.0 files MUST start with the `#\#CIF_2.0` magic header.
    pub const CIF2_MISSING_MAGIC_HEADER: &str = "cif2-missing-magic-header";

    /// CIF 2.0 does not allow doubled-quote escaping (`''` or `""`).
    pub const CIF2_NO_DOUBLED_QUOTES: &str = "cif2-no-doubled-quotes";

    /// CIF 2.0 requires non-empty data block names.
    pub const CIF2_NO_EMPTY_BLOCK_NAME: &str = "cif2-no-empty-block-name";

    /// CIF 2.0 requires non-empty save frame names.
    pub const CIF2_NO_EMPTY_FRAME_NAME: &str = "cif2-no-empty-frame-name";
}
