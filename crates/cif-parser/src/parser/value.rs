//! Value parsing logic - produces RawValue types for version-agnostic parsing.
//!
//! This module handles parsing of CIF values from the parse tree to lossless
//! RawValue types. Version-specific interpretation happens in the rules module.

use crate::error::CifError;
use crate::parser::helpers::extract_span;
use crate::raw::{
    RawListSyntax, RawQuotedString, RawTableEntry, RawTableKey, RawTableSyntax, RawTextField,
    RawTripleQuoted, RawUnquoted, RawValue,
};
use crate::Rule;
use pest::iterators::Pair;

/// Parse a CIF value from a parse tree node to a RawValue.
///
/// This is version-agnostic - it produces lossless intermediate representations
/// that preserve all syntactic information for later version-specific resolution.
pub fn parse_value_raw(pair: Pair<Rule>) -> Result<RawValue, CifError> {
    let span = extract_span(&pair);

    match pair.as_rule() {
        Rule::item_value | Rule::loop_value | Rule::value | Rule::data_value => {
            // Recursively parse the actual value inside
            let inner = pair.into_inner().next();
            if let Some(inner_pair) = inner {
                parse_value_raw(inner_pair)
            } else {
                // Empty value node - treat as unquoted empty string
                Ok(RawValue::Unquoted(RawUnquoted {
                    text: String::new(),
                    span,
                }))
            }
        }

        // List syntax: [value1 value2]
        Rule::list => parse_list_syntax_raw(pair),

        // Table syntax: {key:value}
        Rule::table => parse_table_syntax_raw(pair),

        // Triple-quoted strings: '''...''' or """..."""
        Rule::triple_quoted_string => parse_triple_quoted_raw(pair),

        // Regular quoted strings: '...' or "..."
        Rule::quoted_string | Rule::singlequoted | Rule::doublequoted => {
            parse_quoted_string_raw(pair)
        }

        // Text fields: ;...;
        Rule::text_field | Rule::textfield => parse_text_field_raw(pair),

        // Unquoted strings (whitespace-delimited)
        Rule::wsdelim_string | Rule::unquoted | Rule::simunq => parse_unquoted_raw(pair),

        // Fallback: treat as unquoted
        _ => Ok(RawValue::Unquoted(RawUnquoted {
            text: pair.as_str().to_string(),
            span,
        })),
    }
}

/// Parse a list syntax node to RawListSyntax.
fn parse_list_syntax_raw(pair: Pair<Rule>) -> Result<RawValue, CifError> {
    let span = extract_span(&pair);
    let raw_text = pair.as_str().to_string();
    let mut elements = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::data_value | Rule::value | Rule::item_value | Rule::loop_value => {
                let value = parse_value_raw(inner_pair)?;
                elements.push(value);
            }
            _ => {
                // Skip whitespace and delimiters
            }
        }
    }

    Ok(RawValue::ListSyntax(RawListSyntax {
        raw_text,
        elements,
        span,
    }))
}

/// Parse a table syntax node to RawTableSyntax.
fn parse_table_syntax_raw(pair: Pair<Rule>) -> Result<RawValue, CifError> {
    let span = extract_span(&pair);
    let raw_text = pair.as_str().to_string();
    let mut entries = Vec::new();

    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::table_entry {
            let entry = parse_table_entry_raw(inner_pair)?;
            entries.push(entry);
        }
    }

    Ok(RawValue::TableSyntax(RawTableSyntax {
        raw_text,
        entries,
        span,
    }))
}

/// Parse a single table entry (key:value pair).
fn parse_table_entry_raw(pair: Pair<Rule>) -> Result<RawTableEntry, CifError> {
    let mut key: Option<RawTableKey> = None;
    let mut value: Option<RawValue> = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            // Triple-quoted string key
            Rule::triple_quoted_string => {
                let raw = parse_triple_quoted_raw(inner_pair)?;
                if let RawValue::TripleQuotedString(t) = raw {
                    key = Some(RawTableKey::TripleQuoted(t));
                }
            }
            // Regular quoted string key
            Rule::quoted_string
            | Rule::singlequoted
            | Rule::doublequoted
            | Rule::table_key_quoted => {
                let raw = parse_quoted_string_raw(inner_pair)?;
                if let RawValue::QuotedString(q) = raw {
                    key = Some(RawTableKey::Quoted(q));
                }
            }
            // Value
            Rule::data_value
            | Rule::value
            | Rule::item_value
            | Rule::loop_value
            | Rule::list
            | Rule::table
            | Rule::wsdelim_string
            | Rule::unquoted => {
                value = Some(parse_value_raw(inner_pair)?);
            }
            _ => {
                // Skip other tokens (colons, whitespace)
            }
        }
    }

    // Use defaults if not found
    let key = key.unwrap_or_else(|| {
        RawTableKey::Quoted(RawQuotedString {
            raw_content: String::new(),
            quote_char: '"',
            has_doubled_quotes: false,
            span: crate::ast::Span::default(),
        })
    });

    let value = value.unwrap_or_else(|| {
        RawValue::Unquoted(RawUnquoted {
            text: String::new(),
            span: crate::ast::Span::default(),
        })
    });

    Ok(RawTableEntry { key, value })
}

/// Parse a triple-quoted string to RawTripleQuoted.
fn parse_triple_quoted_raw(pair: Pair<Rule>) -> Result<RawValue, CifError> {
    let span = extract_span(&pair);
    let raw_content = pair.as_str().to_string();

    // Determine quote character
    let quote_char = if raw_content.starts_with("'''") {
        '\''
    } else {
        '"'
    };

    Ok(RawValue::TripleQuotedString(RawTripleQuoted {
        raw_content,
        quote_char,
        span,
    }))
}

/// Parse a quoted string to RawQuotedString.
fn parse_quoted_string_raw(pair: Pair<Rule>) -> Result<RawValue, CifError> {
    let span = extract_span(&pair);
    let raw_content = pair.as_str().to_string();

    // Determine quote character
    let quote_char = raw_content.chars().next().unwrap_or('\'');

    // Check for doubled quotes in the content (after removing outer quotes)
    let content_without_quotes = if raw_content.len() >= 2 {
        &raw_content[1..raw_content.len() - 1]
    } else {
        ""
    };

    let has_doubled_quotes = if quote_char == '\'' {
        content_without_quotes.contains("''")
    } else {
        content_without_quotes.contains("\"\"")
    };

    Ok(RawValue::QuotedString(RawQuotedString {
        raw_content,
        quote_char,
        has_doubled_quotes,
        span,
    }))
}

/// Parse a text field to RawTextField.
fn parse_text_field_raw(pair: Pair<Rule>) -> Result<RawValue, CifError> {
    let span = extract_span(&pair);
    let text = pair.as_str();

    // Remove semicolon delimiters and trim
    let content = text.trim_start_matches(';').trim_end_matches(';').trim();

    Ok(RawValue::TextField(RawTextField {
        content: content.to_string(),
        span,
    }))
}

/// Parse an unquoted string to RawUnquoted.
fn parse_unquoted_raw(pair: Pair<Rule>) -> Result<RawValue, CifError> {
    let span = extract_span(&pair);
    let text = pair.as_str().trim().to_string();

    Ok(RawValue::Unquoted(RawUnquoted { text, span }))
}

#[cfg(test)]
mod tests {
    // Tests will be added as we implement the parser
}
