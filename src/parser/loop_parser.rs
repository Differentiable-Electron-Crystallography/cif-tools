//! Loop structure parsing logic.

use crate::ast::{CifLoop, CifValue};
use crate::error::CifError;
use crate::parser::helpers::{extract_location, extract_text};
use crate::Rule;
use pest::iterators::Pair;

/// Parse a loop structure from the parse tree.
///
/// # Loop Structure Validation
///
/// Loops must have:
/// 1. At least one tag (column header)
/// 2. Values count divisible by tag count (complete rows)
/// 3. Each value must be parseable as a [`CifValue`]
///
/// # Error Conditions
///
/// - [`CifError::InvalidStructure`]: No tags found
/// - [`CifError::InvalidStructure`]: Values don't align with tags (wrong count)
///
/// # Empty Loops
///
/// Loops with tags but no values are valid (represents an empty table).
pub(crate) fn parse_loop(pair: Pair<Rule>) -> Result<CifLoop, CifError> {
    let loop_location = extract_location(&pair);
    let inner: Vec<_> = pair.into_inner().collect();

    // Collect all tag pairs (preserves individual tag locations)
    let tag_pairs: Vec<_> = inner
        .iter()
        .filter(|p| p.as_rule() == Rule::loop_tag || p.as_rule() == Rule::tag)
        .collect();

    // Validate tags exist
    if tag_pairs.is_empty() {
        return Err(CifError::invalid_structure("Loop block has no tags")
            .at_location(loop_location.0, loop_location.1));
    }

    // Extract tag strings
    let mut loop_ = CifLoop::new();
    loop_.tags = tag_pairs.iter().map(|p| extract_text(p)).collect();

    // Collect values
    let mut values = Vec::new();
    for inner_pair in inner {
        match inner_pair.as_rule() {
            Rule::loop_tag | Rule::tag => {
                // Already processed
            }
            Rule::loop_values => {
                collect_loop_values(inner_pair, &mut values);
            }
            Rule::loop_value | Rule::value => {
                values.push(CifValue::parse_value(inner_pair.as_str()));
            }
            _rule => {
                // Unknown rule - safely ignored
            }
        }
    }

    organize_loop_values(&mut loop_, values, loop_location)?;
    Ok(loop_)
}

/// Helper to collect values from loop_values rule
fn collect_loop_values(pair: Pair<Rule>, values: &mut Vec<CifValue>) {
    for value_pair in pair.into_inner() {
        match value_pair.as_rule() {
            Rule::loop_value | Rule::value => {
                values.push(CifValue::parse_value(value_pair.as_str()));
            }
            _rule => {
                // Unknown rule - safely ignored
            }
        }
    }
}

/// Organize values into rows based on tag count.
///
/// # Algorithm
///
/// Values in CIF loops are stored sequentially and must be organized into
/// rows based on the number of tags (columns):
///
/// ```text
/// Tags: [_col1, _col2, _col3]     # 3 columns
/// Values: [v1, v2, v3, v4, v5, v6] # 6 values
///
/// Result:
/// Row 0: [v1, v2, v3]
/// Row 1: [v4, v5, v6]
/// ```
///
/// # Validation
///
/// - Total values must be divisible by tag count
/// - Empty loops (0 values) are valid
/// - Partial rows are rejected with [`CifError::InvalidStructure`]
fn organize_loop_values(
    loop_: &mut CifLoop,
    values: Vec<CifValue>,
    location: (usize, usize),
) -> Result<(), CifError> {
    if values.is_empty() {
        return Ok(()); // Empty loop is valid
    }

    let tag_count = loop_.tags.len();
    if !values.len().is_multiple_of(tag_count) {
        return Err(CifError::invalid_structure(format!(
            "Loop has {} tags but {} values (not divisible)",
            tag_count,
            values.len()
        ))
        .at_location(location.0, location.1));
    }

    for row_values in values.chunks(tag_count) {
        loop_.values.push(row_values.to_vec());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organize_loop_values_valid() {
        let mut loop_ = CifLoop::new();
        loop_.tags = vec!["_col1".to_string(), "_col2".to_string()];

        let values = vec![
            CifValue::Text("v1".to_string()),
            CifValue::Text("v2".to_string()),
            CifValue::Text("v3".to_string()),
            CifValue::Text("v4".to_string()),
        ];

        organize_loop_values(&mut loop_, values, (1, 1)).unwrap();
        assert_eq!(loop_.len(), 2);
        assert_eq!(loop_.get(0, 0).unwrap().as_string().unwrap(), "v1");
        assert_eq!(loop_.get(1, 1).unwrap().as_string().unwrap(), "v4");
    }

    #[test]
    fn test_organize_loop_values_empty() {
        let mut loop_ = CifLoop::new();
        loop_.tags = vec!["_col1".to_string()];

        organize_loop_values(&mut loop_, vec![], (1, 1)).unwrap();
        assert_eq!(loop_.len(), 0);
    }

    #[test]
    fn test_organize_loop_values_misaligned() {
        let mut loop_ = CifLoop::new();
        loop_.tags = vec!["_col1".to_string(), "_col2".to_string()];

        let values = vec![CifValue::Text("v1".to_string())]; // Only 1 value for 2 columns

        let result = organize_loop_values(&mut loop_, values, (42, 5));
        assert!(result.is_err());
        // Verify error includes location
        if let Err(CifError::InvalidStructure { location, .. }) = result {
            assert_eq!(location, Some((42, 5)));
        } else {
            panic!("Expected InvalidStructure error with location");
        }
    }
}
