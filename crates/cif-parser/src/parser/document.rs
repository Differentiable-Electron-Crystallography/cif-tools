//! Document-level parsing logic (entry point for parsing).

use crate::ast::{CifDocument, CifVersion};
use crate::error::CifError;
use crate::parser::block::parse_datablock;
use crate::{CIFParser, Rule};
use pest::Parser;

/// Detect CIF version from input by scanning for magic comment.
///
/// CIF 2.0 files MUST start with `#\#CIF_2.0` magic comment (after optional BOM).
/// Files without this comment are treated as CIF 1.1.
///
/// This is a fast, lightweight check that scans only the beginning of the file.
///
/// # Examples
/// ```
/// # use cif_parser::parser::document::detect_version;
/// # use cif_parser::CifVersion;
/// assert_eq!(detect_version("#\\#CIF_2.0\ndata_test\n"), CifVersion::V2_0);
/// assert_eq!(detect_version("data_test\n"), CifVersion::V1_1);
/// ```
pub fn detect_version(input: &str) -> CifVersion {
    // CIF 2.0 EBNF: file-heading = [ ?U+FEFF? ], magic-code, { inline-wspace }
    // magic-code = '#\#CIF_2.0'
    // Note: File content is literally: # \ # C I F _ 2 . 0 (with backslash)

    let trimmed = input.trim_start_matches('\u{FEFF}'); // Remove BOM if present
    let first_line = trimmed.lines().next().unwrap_or("");

    // Check if first line starts with magic comment: #\#CIF_2.0
    if first_line.trim_start().starts_with("#\\#CIF_2.0") {
        CifVersion::V2_0
    } else {
        CifVersion::V1_1
    }
}

/// Parse a complete CIF file from a string (auto-detects version).
///
/// This is the main entry point for parsing. It:
/// 1. Detects CIF version by scanning for `#\#CIF_2.0` magic comment
/// 2. Uses PEST to parse the input string according to the grammar
/// 3. Converts the parse tree into a typed AST with version-aware parsing
///
/// # Examples
/// ```
/// # use cif_parser::parser::parse_file;
/// let cif = "data_test\n_item value\n";
/// let doc = parse_file(cif).unwrap();
/// assert_eq!(doc.blocks.len(), 1);
/// ```
pub fn parse_file(input: &str) -> Result<CifDocument, CifError> {
    // Detect version from magic comment
    let version = detect_version(input);

    // Parse with PEST
    let pairs = CIFParser::parse(Rule::file, input)?;

    // Build AST with detected version
    let mut doc = CifDocument::new_with_version(version);

    for pair in pairs {
        if pair.as_rule() == Rule::file {
            parse_file_content(pair, &mut doc, version)?;
        }
    }

    Ok(doc)
}

/// Parse the content of a file rule
fn parse_file_content(
    pair: pest::iterators::Pair<Rule>,
    doc: &mut CifDocument,
    version: CifVersion,
) -> Result<(), CifError> {
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            // file rule can contain datablock directly or through content rule
            Rule::datablock => {
                let block = parse_datablock(inner_pair, version)?;
                doc.blocks.push(block);
            }
            Rule::content => {
                // Legacy: content rule contains datablocks
                for content_pair in inner_pair.into_inner() {
                    if content_pair.as_rule() == Rule::datablock {
                        let block = parse_datablock(content_pair, version)?;
                        doc.blocks.push(block);
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
        // CIF grammar allows empty files (just whitespace/comments)
        let cif = "   \n  # comment\n  ";
        let result = parse_file(cif);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().blocks.len(), 0);
    }
}
