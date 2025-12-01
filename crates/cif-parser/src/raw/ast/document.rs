//! Raw document type for lossless CIF parsing.

use crate::ast::Span;
use crate::raw::RawBlock;

/// A raw CIF document before version-specific resolution.
///
/// Contains all parsed blocks and metadata about the file,
/// ready for version-specific validation and transformation.
#[derive(Debug, Clone)]
pub struct RawDocument {
    /// All data blocks in the document
    pub blocks: Vec<RawBlock>,
    /// Whether the `#\#CIF_2.0` magic comment was present
    pub has_cif2_magic: bool,
    /// Span of the entire document
    pub span: Span,
}

impl RawDocument {
    /// Create a new empty raw document
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            has_cif2_magic: false,
            span: Span::default(),
        }
    }

    /// Create a raw document with the given blocks
    pub fn with_blocks(blocks: Vec<RawBlock>, has_cif2_magic: bool, span: Span) -> Self {
        Self {
            blocks,
            has_cif2_magic,
            span,
        }
    }
}

impl Default for RawDocument {
    fn default() -> Self {
        Self::new()
    }
}
