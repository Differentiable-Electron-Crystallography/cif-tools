//! Document-level parsing logic (entry point for parsing).

use crate::ast::CifDocument;
use crate::error::CifError;
use crate::parser::block::parse_datablock;
use crate::{CIFParser, Rule};
use pest::Parser;

/// Parse a complete CIF file from a string.
///
/// This is the main entry point for parsing. It:
/// 1. Uses PEST to parse the input string according to the grammar
/// 2. Converts the parse tree into a typed AST
///
/// # Examples
/// ```
/// # use cif_parser::parser::parse_file;
/// let cif = "data_test\n_item value\n";
/// let doc = parse_file(cif).unwrap();
/// assert_eq!(doc.blocks.len(), 1);
/// ```
pub fn parse_file(input: &str) -> Result<CifDocument, CifError> {
    let pairs = CIFParser::parse(Rule::file, input)?;
    let mut doc = CifDocument::new();

    for pair in pairs {
        if pair.as_rule() == Rule::file {
            parse_file_content(pair, &mut doc)?;
        }
    }

    Ok(doc)
}

/// Parse the content of a file rule
fn parse_file_content(
    pair: pest::iterators::Pair<Rule>,
    doc: &mut CifDocument,
) -> Result<(), CifError> {
    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::content {
            for content_pair in inner_pair.into_inner() {
                if content_pair.as_rule() == Rule::datablock {
                    let block = parse_datablock(content_pair)?;
                    doc.blocks.push(block);
                }
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
