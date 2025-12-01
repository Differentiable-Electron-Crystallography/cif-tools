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
mod violation;

pub use cif1::Cif1Rules;
pub use cif2::Cif2Rules;
pub use violation::{rule_ids, VersionViolation};

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
