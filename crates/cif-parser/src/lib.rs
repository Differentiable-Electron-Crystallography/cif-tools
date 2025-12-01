//! # CIF Parser Library
//!
//! A comprehensive parser for Crystallographic Information Framework (CIF) files,
//! implementing the CIF 1.1 and CIF2.0 specifications.
//!
//! ## What is CIF?
//!
//! CIF is a standard file format used in crystallography and chemistry to store
//! structured data about crystal structures, molecular information, and related
//! metadata. Files contain data blocks with key-value pairs, loops (tables), and
//! text fields.
//!
//! ## Key Parsing Challenges
//!
//! CIF parsing presents several unique challenges that this library addresses:
//!
//! ### 1. Case-Insensitive Keywords
//! Keywords like `data_`, `loop_`, and `global_` are case-insensitive, so
//! `DATA_BLOCK`, `data_block`, and `DaTa_BlOcK` are all valid.
//!
//! ### 2. Complex Value Types
//! Values can be:
//! - Unquoted strings: `value`
//! - Quoted strings: `'value'` or `"value"`
//! - Multi-line text fields: `;text\nfield;`
//! - Special values: `?` (unknown) and `.` (not applicable)
//! - Numbers: `123.45`, `-1.5e-3`
//!
//! ### 3. Loop State Management
//! Loops can be interrupted by other elements, requiring careful state management:
//! ```text
//! loop_
//! _atom.id _atom.type
//! _other_item other_value    # Interrupts the loop!
//! 1 C
//! 2 N
//! ```
//!
//! ### 4. Text Field Parsing
//! Text fields use semicolons at the beginning of lines as delimiters, requiring
//! special handling in the grammar and parsing logic.
//!
//! ## Architecture
//!
//! The parser uses a **two-stage approach**:
//! 1. **Grammar parsing** with PEST PEG parser (defined in `cif.pest`)
//! 2. **AST construction** that builds typed data structures from parse trees
//!
//! This architecture is **decoupled**: AST types don't know how to parse themselves,
//! and parsing logic is separate from data representation. This follows Rust best
//! practices and makes the code easier to test and maintain.
//!
//! ## Why PEST (Not Parser Combinators)?
//!
//! CIF uses PEST rather than parser combinators (like `nom`) because:
//! - CIF has a formal specification that maps naturally to PEG grammars
//! - The grammar file (`cif.pest`) serves as living documentation
//! - Declarative grammars are easier to review and verify against the spec
//! - Grammar changes don't require recompiling Rust code
//!
//! Parser combinators would be appropriate for binary formats or simpler grammars,
//! but PEST is the right tool for CIF's complexity.
//!
//! ## Why Not Pratt Parsing?
//!
//! Pratt parsers handle operator precedence in expression languages (like `a + b * c`).
//! CIF is a **data format**, not an expression language - it has no operators or
//! precedence rules. Pratt parsing doesn't apply to this domain.
//!
//! ## Module Organization
//!
//! - [`ast`] - Abstract Syntax Tree types (final, typed representation)
//! - [`raw`] - Raw AST types and Pass 1 parsing (lossless, version-agnostic)
//! - [`rules`] - Pass 2 resolution (version-specific: CIF 1.1 vs 2.0)
//! - [`error`] - Error types
//!
//! ## Examples
//!
//! ### Basic Usage
//! ```
//! use cif_parser::{Document, CifError};
//!
//! let cif_content = r#"
//! data_example
//! _cell_length_a  10.000
//! _cell_length_b  20.000
//! _title 'My Structure'
//! "#;
//!
//! let doc = Document::parse(cif_content)?;
//! let block = doc.first_block().unwrap();
//!
//! assert_eq!(block.name, "example");
//! assert_eq!(block.get_item("_cell_length_a").unwrap().as_numeric(), Some(10.0));
//! # Ok::<(), CifError>(())
//! ```
//!
//! ### Working with Loops
//! ```
//! use cif_parser::Document;
//!
//! let cif_content = r#"
//! data_atoms
//! loop_
//! _atom_site_label
//! _atom_site_type_symbol
//! _atom_site_fract_x
//! C1  C  0.1234
//! N1  N  0.5678
//! O1  O  0.9012
//! "#;
//!
//! let doc = Document::parse(cif_content).unwrap();
//! let block = doc.first_block().unwrap();
//! let loop_ = &block.loops[0];
//!
//! assert_eq!(loop_.len(), 3); // 3 rows
//! assert_eq!(loop_.tags.len(), 3); // 3 columns
//!
//! // Access by row and tag name
//! let atom_type = loop_.get_by_tag(0, "_atom_site_type_symbol").unwrap();
//! assert_eq!(atom_type.as_string().unwrap(), "C");
//! ```

use pest_derive::Parser;
use std::path::Path;

// ===== Core Modules =====

pub mod ast;
pub mod error;
pub mod raw;
pub mod rules;

// ===== PEST Parser =====

#[derive(Parser)]
#[grammar = "grammar/cif.pest"]
pub struct CIFParser;

// ===== Re-exports =====

// AST types
pub use ast::{CifBlock, CifDocument, CifFrame, CifLoop, CifValue, CifValueKind, CifVersion, Span};

// Error types
pub use error::CifError;

// Rules and violations
pub use rules::{Cif1Rules, Cif2Rules, VersionRules, VersionViolation};

// Convenient type aliases (matching old API)
pub use CifBlock as Block;
pub use CifDocument as Document;
pub use CifFrame as Frame;
pub use CifLoop as Loop;
pub use CifValue as Value;
pub use CifValueKind as ValueKind;
pub use CifVersion as Version;

// ===== Parse Options and Results =====

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

// ===== Public Convenience Functions =====

/// Parse a CIF file from a path
///
/// # Examples
/// ```no_run
/// use cif_parser::parse_file;
///
/// let doc = parse_file("structure.cif").unwrap();
/// ```
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<CifDocument, CifError> {
    CifDocument::from_file(path)
}

/// Parse a CIF string
///
/// # Examples
/// ```
/// use cif_parser::parse_string;
///
/// let doc = parse_string("data_test\n_item value\n").unwrap();
/// ```
pub fn parse_string(input: &str) -> Result<CifDocument, CifError> {
    CifDocument::parse(input)
}

/// Parse a CIF string with options.
///
/// This is the main entry point for parsing with advanced options like upgrade guidance.
///
/// # Example
///
/// ```
/// use cif_parser::{ParseOptions, parse_string_with_options};
///
/// let input = "data_test\n_item value\n";
/// let result = parse_string_with_options(input, ParseOptions::new().upgrade_guidance(true))?;
///
/// println!("Document: {:?}", result.document);
/// for issue in &result.upgrade_issues {
///     println!("Upgrade issue: {}", issue);
/// }
/// # Ok::<(), cif_parser::CifError>(())
/// ```
pub fn parse_string_with_options(
    input: &str,
    options: ParseOptions,
) -> Result<ParseResult, CifError> {
    // Pass 1: Parse to raw AST (version-agnostic)
    let raw_doc = raw::parser::parse_raw(input)?;

    // Detect version from magic comment (stored in raw_doc)
    let version = if raw_doc.has_cif2_magic {
        CifVersion::V2_0
    } else {
        CifVersion::V1_1
    };

    // Pass 2: Resolve with version rules
    let document = match version {
        CifVersion::V1_1 => Cif1Rules.resolve(&raw_doc).map_err(violation_to_error)?,
        CifVersion::V2_0 => Cif2Rules.resolve(&raw_doc).map_err(violation_to_error)?,
    };

    // Collect upgrade issues if requested AND file is CIF 1.1
    let upgrade_issues = if options.upgrade_guidance && version == CifVersion::V1_1 {
        Cif2Rules.collect_violations(&raw_doc)
    } else {
        vec![]
    };

    Ok(ParseResult::new(document, upgrade_issues))
}

/// Convert a VersionViolation to CifError.
fn violation_to_error(violation: VersionViolation) -> CifError {
    CifError::InvalidStructure {
        message: format!(
            "[{}] {}{}",
            violation.rule_id,
            violation.message,
            violation
                .suggestion
                .map(|s| format!(" ({})", s))
                .unwrap_or_default()
        ),
        location: Some((violation.span.start_line, violation.span.start_col)),
    }
}

// ===== Re-export for internal use =====
pub use pest::iterators::Pair;
// Rule enum is automatically public via the #[derive(Parser)] macro

// ===== Conditional Compilation Modules =====

// WASM bindings module (conditionally compiled)
#[cfg(target_arch = "wasm32")]
pub mod wasm;

// Python bindings module (conditionally compiled)
#[cfg(feature = "python")]
pub mod python;
