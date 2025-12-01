//! Raw value types for lossless CIF parsing.
//!
//! These types preserve all syntactic information from the input, allowing
//! version-specific rules to make decisions during the resolution phase.

use crate::ast::Span;

/// Lossless representation of a CIF value before version resolution.
///
/// Each variant preserves the raw input, allowing version-specific rules
/// to perform both validation and transformation in the resolution phase.
#[derive(Debug, Clone)]
pub enum RawValue {
    /// Quoted string: `'content'` or `"content"`
    QuotedString(RawQuotedString),

    /// Triple-quoted string: `'''content'''` or `"""content"""`
    TripleQuotedString(RawTripleQuoted),

    /// Text field: `;content;` (semicolon-delimited multi-line)
    TextField(RawTextField),

    /// Unquoted string (could be number, special value, or text)
    Unquoted(RawUnquoted),

    /// List syntax: `[value1 value2]`
    /// - CIF 1.1: resolves to literal text
    /// - CIF 2.0: resolves to List value
    ListSyntax(RawListSyntax),

    /// Table syntax: `{key:value}`
    /// - CIF 1.1: resolves to literal text
    /// - CIF 2.0: resolves to Table value
    TableSyntax(RawTableSyntax),
}

/// A quoted string with metadata for version-specific processing.
///
/// Preserves:
/// - The raw content including quotes
/// - Which quote character was used
/// - Whether doubled-quote escaping is present (valid in CIF 1.1, invalid in CIF 2.0)
#[derive(Debug, Clone)]
pub struct RawQuotedString {
    /// Full string including quotes (e.g., `'O''Brien'`)
    pub raw_content: String,
    /// The quote character used: `'` or `"`
    pub quote_char: char,
    /// Whether the content contains doubled quotes (`''` or `""`)
    /// CIF 1.1: unescape these; CIF 2.0: reject as invalid
    pub has_doubled_quotes: bool,
    /// Source location
    pub span: Span,
}

/// A triple-quoted string (CIF 2.0 feature).
///
/// In CIF 1.1, this would be parsed as literal text.
/// In CIF 2.0, the triple quotes are removed and content is extracted.
#[derive(Debug, Clone)]
pub struct RawTripleQuoted {
    /// Full string including triple quotes
    pub raw_content: String,
    /// The quote character used: `'` or `"`
    pub quote_char: char,
    /// Source location
    pub span: Span,
}

/// A text field (semicolon-delimited multi-line string).
///
/// Text fields are the same in both CIF 1.1 and CIF 2.0.
#[derive(Debug, Clone)]
pub struct RawTextField {
    /// Content with semicolon delimiters removed
    pub content: String,
    /// Source location
    pub span: Span,
}

/// An unquoted value (whitespace-delimited).
///
/// Could be:
/// - A number (with optional uncertainty notation)
/// - A special value (`?` or `.`)
/// - Plain text
#[derive(Debug, Clone)]
pub struct RawUnquoted {
    /// The raw text (trimmed)
    pub text: String,
    /// Source location
    pub span: Span,
}

/// List syntax with both raw text and parsed elements.
///
/// Preserves both representations for version-specific resolution:
/// - CIF 1.1: uses `raw_text` as literal text value
/// - CIF 2.0: uses `elements` as actual list
#[derive(Debug, Clone)]
pub struct RawListSyntax {
    /// Original `[...]` text for CIF 1.1 fallback
    pub raw_text: String,
    /// Parsed elements for CIF 2.0
    pub elements: Vec<RawValue>,
    /// Source location
    pub span: Span,
}

/// Table syntax with both raw text and parsed entries.
///
/// Preserves both representations for version-specific resolution:
/// - CIF 1.1: uses `raw_text` as literal text value
/// - CIF 2.0: uses `entries` as actual table
#[derive(Debug, Clone)]
pub struct RawTableSyntax {
    /// Original `{...}` text for CIF 1.1 fallback
    pub raw_text: String,
    /// Parsed entries for CIF 2.0
    pub entries: Vec<RawTableEntry>,
    /// Source location
    pub span: Span,
}

/// A single table entry (key-value pair).
#[derive(Debug, Clone)]
pub struct RawTableEntry {
    /// The key (must be a quoted or triple-quoted string)
    pub key: RawTableKey,
    /// The value
    pub value: RawValue,
}

/// Table key can be a quoted or triple-quoted string.
#[derive(Debug, Clone)]
pub enum RawTableKey {
    /// Regular quoted string key
    Quoted(RawQuotedString),
    /// Triple-quoted string key (CIF 2.0)
    TripleQuoted(RawTripleQuoted),
}

impl RawTableKey {
    /// Get the span of this key
    pub fn span(&self) -> Span {
        match self {
            RawTableKey::Quoted(q) => q.span,
            RawTableKey::TripleQuoted(t) => t.span,
        }
    }

    /// Get the raw content of this key
    pub fn raw_content(&self) -> &str {
        match self {
            RawTableKey::Quoted(q) => &q.raw_content,
            RawTableKey::TripleQuoted(t) => &t.raw_content,
        }
    }
}
