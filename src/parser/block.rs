//! Data block and save frame parsing logic.

use crate::ast::{CifBlock, CifFrame, CifValue};
use crate::builder::BlockBuilder;
use crate::error::CifError;
use crate::parser::helpers::{extract_location, extract_text};
use crate::parser::loop_parser::parse_loop;
use crate::Rule;
use pest::iterators::Pair;

/// Parse a data block from the parse tree
pub(crate) fn parse_datablock(pair: Pair<Rule>) -> Result<CifBlock, CifError> {
    let mut builder = BlockBuilder::new(String::new());

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::datablockheading => {
                let name = extract_block_name(inner_pair.as_str());
                builder.block_mut().name = name;
            }
            Rule::dataitem => {
                let (tag, value) = parse_dataitem(inner_pair)?;
                builder.add_item(tag, value);
            }
            Rule::loop_block => {
                let loop_ = parse_loop(inner_pair)?;
                builder.start_loop(loop_);
            }
            Rule::frame => {
                let frame = parse_frame(inner_pair)?;
                builder.add_frame(frame);
            }
            _rule => {
                // Unknown rule - safely ignored
            }
        }
    }

    Ok(builder.finish())
}

/// Parse a data item (tag-value pair) from the parse tree
pub(crate) fn parse_dataitem(pair: Pair<Rule>) -> Result<(String, CifValue), CifError> {
    let item_location = extract_location(&pair);
    let inner: Vec<_> = pair.into_inner().collect();

    // Find tag pair (preserves location for better error messages)
    let tag_pair = inner
        .iter()
        .find(|p| p.as_rule() == Rule::item_tag || p.as_rule() == Rule::tag)
        .ok_or_else(|| {
            CifError::invalid_structure("Data item missing tag")
                .at_location(item_location.0, item_location.1)
        })?;

    // Find value pair
    let value_pair = inner
        .iter()
        .find(|p| p.as_rule() == Rule::item_value || p.as_rule() == Rule::value);

    let tag = extract_text(tag_pair);
    let value = if let Some(vp) = value_pair {
        CifValue::parse_value(vp.as_str())
    } else {
        CifValue::Unknown
    };

    Ok((tag, value))
}

/// Parse a save frame from the parse tree
pub(crate) fn parse_frame(pair: Pair<Rule>) -> Result<CifFrame, CifError> {
    let frame_location = extract_location(&pair);
    let inner: Vec<_> = pair.into_inner().collect();

    // Find framename (more precise error location)
    let framename_pair = inner
        .iter()
        .find(|p| p.as_rule() == Rule::framename)
        .ok_or_else(|| {
            CifError::invalid_structure("Save frame missing name")
                .at_location(frame_location.0, frame_location.1)
        })?;

    let mut frame = CifFrame::new(extract_text(framename_pair));

    // Process remaining elements
    for inner_pair in inner {
        match inner_pair.as_rule() {
            Rule::framename => {
                // Already processed
            }
            Rule::dataitem => {
                let (tag, value) = parse_dataitem(inner_pair)?;
                frame.items.insert(tag, value);
            }
            Rule::loop_block => {
                let loop_ = parse_loop(inner_pair)?;
                frame.loops.push(loop_);
            }
            _rule => {
                // Unknown rule - safely ignored
            }
        }
    }

    Ok(frame)
}

/// Extract block name from a data block heading with case-insensitive parsing.
///
/// # CIF Block Naming Rules
///
/// - `data_name` → `"name"`
/// - `DATA_NAME` → `"NAME"` (preserves original case of the name part)
/// - `global_` → `""` (empty string, as global blocks have no name)
///
/// # Case Sensitivity
///
/// The keywords (`data_`, `global_`) are case-insensitive per CIF specification,
/// but the name part preserves its original casing. This means:
/// - `DATA_MyProtein` → `"MyProtein"`
/// - `data_MyProtein` → `"MyProtein"`
fn extract_block_name(heading_str: &str) -> String {
    let lower = heading_str.to_lowercase();
    if lower.starts_with("data_") {
        heading_str[5..].to_string()
    } else if lower == "global_" {
        String::new() // Global block has no name
    } else {
        heading_str.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_block_name() {
        assert_eq!(extract_block_name("data_test"), "test");
        assert_eq!(extract_block_name("DATA_TEST"), "TEST");
        assert_eq!(extract_block_name("data_MyProtein"), "MyProtein");
        assert_eq!(extract_block_name("global_"), "");
        assert_eq!(extract_block_name("GLOBAL_"), "");
    }
}
