//! Raw parsing logic for Pass 1: PEST parse tree → RawDocument.
//!
//! This module handles version-agnostic parsing that produces lossless
//! intermediate representations (Raw* types). All syntactic information
//! is preserved for version-specific rules to process in Pass 2.
//!
//! # Architecture
//!
//! ```text
//! Input String
//!     │
//!     ▼
//! PEST Grammar (cif.pest)
//!     │
//!     ▼
//! raw::parser::parse_raw() → RawDocument
//! ```
//!
//! # Module Organization
//!
//! - `helpers`: Line index and span extraction utilities
//! - `value`: Parse CIF values to RawValue variants
//! - `loop_parser`: Parse loop structures to RawLoop
//! - `block`: Parse data blocks and save frames to RawBlock/RawFrame

pub(crate) mod block;
pub(crate) mod helpers;
pub(crate) mod loop_parser;
pub(crate) mod value;

use crate::ast::CifVersion;
use crate::error::CifError;
use crate::raw::RawDocument;
use crate::{CIFParser, Rule};
use block::parse_datablock_raw;
use helpers::{clear_line_index, extract_span, init_line_index};
use pest::Parser;

/// Detect CIF version from input by scanning for magic comment.
///
/// CIF 2.0 files MUST start with `#\#CIF_2.0` magic comment (after optional BOM).
/// Files without this comment are treated as CIF 1.1.
pub(crate) fn detect_version(input: &str) -> CifVersion {
    let trimmed = input.trim_start_matches('\u{FEFF}'); // Remove BOM if present
    let first_line = trimmed.lines().next().unwrap_or("");

    if first_line.trim_start().starts_with("#\\#CIF_2.0") {
        CifVersion::V2_0
    } else {
        CifVersion::V1_1
    }
}

/// Parse input to raw AST (Pass 1 - version-agnostic).
///
/// This is the main entry point for raw parsing. It produces a `RawDocument`
/// that preserves all syntactic information from the input.
pub(crate) fn parse_raw(input: &str) -> Result<RawDocument, CifError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_version_cif2() {
        assert_eq!(detect_version("#\\#CIF_2.0\ndata_test\n"), CifVersion::V2_0);
    }

    #[test]
    fn test_detect_version_cif1() {
        assert_eq!(detect_version("data_test\n"), CifVersion::V1_1);
    }

    #[test]
    fn test_parse_raw_empty_block() {
        let cif = "data_test\n";
        let raw = parse_raw(cif).unwrap();
        assert_eq!(raw.blocks.len(), 1);
        assert_eq!(raw.blocks[0].name, "test");
    }

    #[test]
    fn test_parse_raw_multiple_blocks() {
        let cif = "data_first\n_item1 val1\ndata_second\n_item2 val2\n";
        let raw = parse_raw(cif).unwrap();
        assert_eq!(raw.blocks.len(), 2);
        assert_eq!(raw.blocks[0].name, "first");
        assert_eq!(raw.blocks[1].name, "second");
    }
}
