//! Loop structure parsing logic - produces RawLoop.

use crate::error::CifError;
use crate::parser::helpers::{extract_span, extract_text};
use crate::parser::value::parse_value_raw;
use crate::raw::{RawLoop, RawLoopTag, RawValue};
use crate::Rule;
use pest::iterators::Pair;

/// Parse a loop structure from the parse tree to RawLoop.
///
/// This is version-agnostic - validation of loop contents happens
/// during resolution via VersionRules.
pub(crate) fn parse_loop_raw(pair: Pair<Rule>) -> Result<RawLoop, CifError> {
    let loop_span = extract_span(&pair);
    let inner: Vec<_> = pair.into_inner().collect();

    // Collect all tag pairs with spans
    let tags: Vec<RawLoopTag> = inner
        .iter()
        .filter(|p| p.as_rule() == Rule::loop_tag || p.as_rule() == Rule::tag)
        .map(|p| RawLoopTag {
            name: extract_text(p),
            span: extract_span(p),
        })
        .collect();

    // Collect values
    let mut values = Vec::new();
    for inner_pair in inner {
        match inner_pair.as_rule() {
            Rule::loop_tag | Rule::tag => {
                // Already processed
            }
            Rule::loop_values => {
                collect_loop_values_raw(inner_pair, &mut values)?;
            }
            Rule::loop_value | Rule::value => {
                let value = parse_value_raw(inner_pair.clone())?;
                values.push(value);
            }
            _rule => {
                // Unknown rule - safely ignored
            }
        }
    }

    Ok(RawLoop {
        tags,
        values,
        span: loop_span,
    })
}

/// Helper to collect values from loop_values rule.
fn collect_loop_values_raw(pair: Pair<Rule>, values: &mut Vec<RawValue>) -> Result<(), CifError> {
    for value_pair in pair.into_inner() {
        match value_pair.as_rule() {
            Rule::loop_value | Rule::value => {
                let value = parse_value_raw(value_pair)?;
                values.push(value);
            }
            _rule => {
                // Unknown rule - safely ignored
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    // Tests will be added as we implement the parser
}
