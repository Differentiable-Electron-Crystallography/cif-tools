//! Document-level parsing logic (entry point for two-pass parsing).
//!
//! # Architecture
//!
//! ```text
//! Input String
//!     │
//!     ▼
//! Pass 1: parse_raw() → RawDocument (version-agnostic, lossless)
//!     │
//!     ▼
//! Pass 2: VersionRules.resolve() → CifDocument
//!     │
//!     ├─ Cif1Rules: permissive, transforms
//!     └─ Cif2Rules: strict, validates + transforms
//! ```

use crate::ast::{CifDocument, CifVersion};
use crate::error::CifError;
use crate::parser::block::parse_datablock_raw;
use crate::parser::helpers::{clear_line_index, extract_span, init_line_index};
use crate::parser::options::{ParseOptions, ParseResult};
use crate::raw::RawDocument;
use crate::rules::{Cif1Rules, Cif2Rules, VersionRules, VersionViolation};
use crate::{CIFParser, Rule};
use pest::Parser;

/// Detect CIF version from input by scanning for magic comment.
///
/// CIF 2.0 files MUST start with `#\#CIF_2.0` magic comment (after optional BOM).
/// Files without this comment are treated as CIF 1.1.
pub fn detect_version(input: &str) -> CifVersion {
    let trimmed = input.trim_start_matches('\u{FEFF}'); // Remove BOM if present
    let first_line = trimmed.lines().next().unwrap_or("");

    if first_line.trim_start().starts_with("#\\#CIF_2.0") {
        CifVersion::V2_0
    } else {
        CifVersion::V1_1
    }
}

/// Parse a complete CIF file from a string (auto-detects version).
///
/// This is the main entry point for parsing. It uses a two-pass approach:
/// 1. Parse to raw AST (version-agnostic)
/// 2. Resolve with version-specific rules
pub fn parse_file(input: &str) -> Result<CifDocument, CifError> {
    let result = parse_file_with_options(input, ParseOptions::default())?;
    Ok(result.document)
}

/// Parse a CIF file with options.
///
/// # Example
///
/// ```
/// use cif_parser::{ParseOptions, parser::document::parse_file_with_options};
///
/// let input = "data_test\n_item value\n";
/// let result = parse_file_with_options(input, ParseOptions::new().upgrade_guidance(true))?;
///
/// println!("Document: {:?}", result.document);
/// for issue in &result.upgrade_issues {
///     println!("Upgrade issue: {}", issue);
/// }
/// # Ok::<(), cif_parser::CifError>(())
/// ```
pub fn parse_file_with_options(
    input: &str,
    options: ParseOptions,
) -> Result<ParseResult, CifError> {
    // Pass 1: Parse to raw AST (version-agnostic)
    let raw_doc = parse_raw(input)?;

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

/// Parse input to raw AST (Pass 1 - version-agnostic).
pub fn parse_raw(input: &str) -> Result<RawDocument, CifError> {
    // Detect version for metadata (but don't use it for parsing decisions)
    let has_cif2_magic = detect_version(input) == CifVersion::V2_0;

    // Build line index for fast line/column lookups
    init_line_index(input);

    // Parse with PEST
    let pairs = CIFParser::parse(Rule::file, input)?;

    // Build raw AST
    let mut raw_doc = RawDocument::new();
    raw_doc.has_cif2_magic = has_cif2_magic;

    for pair in pairs {
        if pair.as_rule() == Rule::file {
            raw_doc.span = extract_span(&pair);
            parse_file_content_raw(pair, &mut raw_doc)?;
        }
    }

    // Clean up line index
    clear_line_index();

    Ok(raw_doc)
}

/// Parse the content of a file rule to raw blocks.
fn parse_file_content_raw(
    pair: pest::iterators::Pair<Rule>,
    raw_doc: &mut RawDocument,
) -> Result<(), CifError> {
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::datablock => {
                let block = parse_datablock_raw(inner_pair)?;
                raw_doc.blocks.push(block);
            }
            Rule::content => {
                // Legacy: content rule contains datablocks
                for content_pair in inner_pair.into_inner() {
                    if content_pair.as_rule() == Rule::datablock {
                        let block = parse_datablock_raw(content_pair)?;
                        raw_doc.blocks.push(block);
                    }
                }
            }
            _ => {
                // Skip other rules (file_heading, wspace, etc.)
            }
        }
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_block() {
        let cif = "data_test\n";
        let doc = parse_file(cif).unwrap();
        assert_eq!(doc.blocks.len(), 1);
        assert_eq!(doc.blocks[0].name, "test");
    }

    #[test]
    fn test_parse_multiple_blocks() {
        let cif = "data_first\n_item1 val1\ndata_second\n_item2 val2\n";
        let doc = parse_file(cif).unwrap();
        assert_eq!(doc.blocks.len(), 2);
        assert_eq!(doc.blocks[0].name, "first");
        assert_eq!(doc.blocks[1].name, "second");
    }

    #[test]
    fn test_parse_empty_file() {
        let cif = "   \n  # comment\n  ";
        let result = parse_file(cif);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().blocks.len(), 0);
    }

    #[test]
    fn test_detect_version_cif2() {
        assert_eq!(detect_version("#\\#CIF_2.0\ndata_test\n"), CifVersion::V2_0);
    }

    #[test]
    fn test_detect_version_cif1() {
        assert_eq!(detect_version("data_test\n"), CifVersion::V1_1);
    }

    #[test]
    fn test_parse_with_upgrade_guidance() {
        let cif = "data_test\n_item 'O''Brien'\n"; // CIF 1.1 with doubled quotes
        let result =
            parse_file_with_options(cif, ParseOptions::new().upgrade_guidance(true)).unwrap();

        assert_eq!(result.document.version, CifVersion::V1_1);
        assert!(!result.upgrade_issues.is_empty());
        assert_eq!(
            result.upgrade_issues[0].rule_id,
            crate::rules::rule_ids::CIF2_NO_DOUBLED_QUOTES
        );
    }
}
