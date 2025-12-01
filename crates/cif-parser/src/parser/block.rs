//! Data block and save frame parsing logic - produces Raw types.

use crate::ast::Span;
use crate::error::CifError;
use crate::parser::helpers::{extract_span, extract_text};
use crate::parser::loop_parser::parse_loop_raw;
use crate::parser::value::parse_value_raw;
use crate::raw::{RawBlock, RawDataItem, RawFrame, RawValue};
use crate::Rule;
use pest::iterators::Pair;

/// Parse a data block from the parse tree to RawBlock.
pub(crate) fn parse_datablock_raw(pair: Pair<Rule>) -> Result<RawBlock, CifError> {
    let block_span = extract_span(&pair);
    let mut name = String::new();
    let mut is_global = false;
    let mut name_span = Span::default();
    let mut items = Vec::new();
    let mut loops = Vec::new();
    let mut frames = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::datablockheading => {
                name_span = extract_span(&inner_pair);
                let heading = inner_pair.as_str();
                is_global = heading.to_lowercase() == "global_";
                name = extract_block_name(heading);
            }
            Rule::dataitem => {
                let item = parse_dataitem_raw(inner_pair)?;
                items.push(item);
            }
            Rule::loop_block => {
                let loop_ = parse_loop_raw(inner_pair)?;
                loops.push(loop_);
            }
            Rule::frame => {
                let frame = parse_frame_raw(inner_pair)?;
                frames.push(frame);
            }
            _rule => {
                // Unknown rule - safely ignored
            }
        }
    }

    Ok(RawBlock {
        name,
        is_global,
        name_span,
        items,
        loops,
        frames,
        span: block_span,
    })
}

/// Parse a data item (tag-value pair) to RawDataItem.
pub(crate) fn parse_dataitem_raw(pair: Pair<Rule>) -> Result<RawDataItem, CifError> {
    let item_span = extract_span(&pair);
    let inner: Vec<_> = pair.into_inner().collect();

    // Find tag pair
    let tag_pair = inner
        .iter()
        .find(|p| p.as_rule() == Rule::item_tag || p.as_rule() == Rule::tag);

    let tag_span = tag_pair.map(|p| extract_span(p)).unwrap_or_default();
    let tag = tag_pair.map(|p| extract_text(p)).unwrap_or_default();

    // Find value pair
    let value_pair = inner
        .iter()
        .find(|p| p.as_rule() == Rule::item_value || p.as_rule() == Rule::value);

    let value = if let Some(vp) = value_pair {
        parse_value_raw(vp.clone())?
    } else {
        RawValue::Unquoted(crate::raw::RawUnquoted {
            text: String::new(),
            span: Span::default(),
        })
    };

    Ok(RawDataItem {
        tag,
        tag_span,
        value,
        span: item_span,
    })
}

/// Parse a save frame to RawFrame.
pub(crate) fn parse_frame_raw(pair: Pair<Rule>) -> Result<RawFrame, CifError> {
    let frame_span = extract_span(&pair);
    let inner: Vec<_> = pair.into_inner().collect();

    // Find save_heading, then extract framename from within it
    let save_heading_pair = inner.iter().find(|p| p.as_rule() == Rule::save_heading);

    let (name, name_span) = if let Some(heading) = save_heading_pair {
        let heading_span = extract_span(heading);
        let framename_pair = heading
            .clone()
            .into_inner()
            .find(|p| p.as_rule() == Rule::framename);
        if let Some(fname) = framename_pair {
            (extract_text(&fname), extract_span(&fname))
        } else {
            (String::new(), heading_span)
        }
    } else {
        (String::new(), Span::default())
    };

    let mut items = Vec::new();
    let mut loops = Vec::new();

    // Process remaining elements
    for inner_pair in inner {
        match inner_pair.as_rule() {
            Rule::save_heading => {
                // Already processed
            }
            Rule::dataitem => {
                let item = parse_dataitem_raw(inner_pair)?;
                items.push(item);
            }
            Rule::loop_block => {
                let loop_ = parse_loop_raw(inner_pair)?;
                loops.push(loop_);
            }
            _rule => {
                // Unknown rule - safely ignored
            }
        }
    }

    Ok(RawFrame {
        name,
        name_span,
        items,
        loops,
        span: frame_span,
    })
}

/// Extract block name from a data block heading with case-insensitive parsing.
///
/// # CIF Block Naming Rules
///
/// - `data_name` → `"name"`
/// - `DATA_NAME` → `"NAME"` (preserves original case of the name part)
/// - `global_` → `""` (empty string, as global blocks have no name)
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
