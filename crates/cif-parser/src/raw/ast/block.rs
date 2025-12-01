//! Raw block and frame types for lossless CIF parsing.

use crate::ast::Span;
use crate::raw::RawValue;

/// A raw data block before version-specific validation.
///
/// Preserves all information including potentially invalid constructs
/// (like empty names in CIF 2.0 mode).
#[derive(Debug, Clone)]
pub struct RawBlock {
    /// Block name (may be empty - valid in CIF 1.1, invalid in CIF 2.0 unless global)
    pub name: String,
    /// Whether this is a global_ block (empty name is allowed even in CIF 2.0)
    pub is_global: bool,
    /// Span of just the name portion (for error reporting on empty names)
    pub name_span: Span,
    /// Data items (tag-value pairs)
    pub items: Vec<RawDataItem>,
    /// Loop structures
    pub loops: Vec<RawLoop>,
    /// Save frames
    pub frames: Vec<RawFrame>,
    /// Span of the entire block
    pub span: Span,
}

/// A raw data item (tag-value pair).
#[derive(Debug, Clone)]
pub struct RawDataItem {
    /// The tag name (including leading underscore)
    pub tag: String,
    /// Span of just the tag
    pub tag_span: Span,
    /// The value
    pub value: RawValue,
    /// Span of the entire item
    pub span: Span,
}

/// A raw loop structure.
#[derive(Debug, Clone)]
pub struct RawLoop {
    /// Column tags
    pub tags: Vec<RawLoopTag>,
    /// All values in order (row-major)
    pub values: Vec<RawValue>,
    /// Span of the entire loop
    pub span: Span,
}

/// A loop tag with its span.
#[derive(Debug, Clone)]
pub struct RawLoopTag {
    /// The tag name
    pub name: String,
    /// Source location
    pub span: Span,
}

/// A raw save frame before version-specific validation.
#[derive(Debug, Clone)]
pub struct RawFrame {
    /// Frame name (may be empty - valid in CIF 1.1, invalid in CIF 2.0)
    pub name: String,
    /// Span of just the name portion
    pub name_span: Span,
    /// Data items
    pub items: Vec<RawDataItem>,
    /// Loop structures
    pub loops: Vec<RawLoop>,
    /// Span of the entire frame
    pub span: Span,
}
