//! Value parsing logic with CIF version awareness.
//!
//! This module handles parsing of CIF values from the parse tree, with support
//! for both CIF 1.1 and CIF 2.0 value types.
//!
//! # Version-Aware Parsing
//!
//! The parser uses explicit version guards (`if version == CifVersion::V2_0`) to
//! enable CIF 2.0 features only when the magic comment is present. This ensures:
//! - CIF 1.1 files parse without overhead of checking for CIF 2.0 features
//! - CIF 2.0 files get full support for lists, tables, and triple-quoted strings
//! - No ambiguity or dynamic feature detection needed

use crate::ast::{CifValue, CifVersion};
use crate::error::CifError;
use crate::Rule;
use pest::iterators::Pair;
use std::collections::HashMap;

/// Parse a CIF value from a parse tree node with version awareness.
///
/// # CIF Version Handling
///
/// - **CIF 1.1 mode** (`version == V1_1`):
///   - Parses: text, quoted strings, text fields, unquoted strings, numbers
///   - Characters `[{]}` in unquoted strings are treated as regular characters
///
/// - **CIF 2.0 mode** (`version == V2_0`):
///   - All CIF 1.1 features PLUS:
///   - Lists: `[value1 value2 value3]`
///   - Tables: `{key1:value1 key2:value2}`
///   - Triple-quoted strings: `"""..."""` and `'''...'''`
///
/// # Examples
/// ```ignore
/// use cif_parser::{CifVersion, parser::value::parse_value};
///
/// // CIF 1.1 parsing
/// let pair = /* ... */;
/// let value = parse_value(pair, CifVersion::V1_1)?;
///
/// // CIF 2.0 parsing (with list support)
/// let value = parse_value(pair, CifVersion::V2_0)?;
/// ```
pub fn parse_value(pair: Pair<Rule>, version: CifVersion) -> Result<CifValue, CifError> {
    match pair.as_rule() {
        Rule::item_value | Rule::loop_value | Rule::value | Rule::data_value => {
            // Recursively parse the actual value inside
            let inner = pair.into_inner().next();
            if let Some(inner_pair) = inner {
                parse_value(inner_pair, version)
            } else {
                // Empty value node - treat as text
                Ok(CifValue::Text(String::new()))
            }
        }

        // CIF 2.0 ONLY: Lists
        Rule::list => {
            // VERSION GUARD: Only parse lists in CIF 2.0 mode
            if version == CifVersion::V2_0 {
                parse_list(pair, version)
            } else {
                // In CIF 1.1, this shouldn't be matched by grammar, but be defensive
                Ok(CifValue::Text(pair.as_str().to_string()))
            }
        }

        // CIF 2.0 ONLY: Tables
        Rule::table => {
            // VERSION GUARD: Only parse tables in CIF 2.0 mode
            if version == CifVersion::V2_0 {
                parse_table(pair, version)
            } else {
                // In CIF 1.1, this shouldn't be matched by grammar, but be defensive
                Ok(CifValue::Text(pair.as_str().to_string()))
            }
        }

        // CIF 2.0 ONLY: Triple-quoted strings
        Rule::triple_quoted_string => {
            // VERSION GUARD: Only in CIF 2.0 mode
            if version == CifVersion::V2_0 {
                parse_triple_quoted(pair)
            } else {
                Ok(CifValue::Text(pair.as_str().to_string()))
            }
        }

        // CIF 1.1 and 2.0: Quoted strings
        Rule::quoted_string | Rule::singlequoted | Rule::doublequoted => {
            parse_quoted_string(pair, version)
        }

        // CIF 1.1 and 2.0: Text fields
        Rule::text_field | Rule::textfield => parse_text_field(pair),

        // CIF 1.1 and 2.0: Unquoted strings (whitespace-delimited)
        Rule::wsdelim_string | Rule::unquoted | Rule::simunq => {
            // In CIF 1.1 mode, unquoted strings can contain [{]}
            // In CIF 2.0 mode, these would have been parsed as list/table
            parse_unquoted(pair)
        }

        // Fallback: treat as text
        _ => Ok(CifValue::Text(pair.as_str().to_string())),
    }
}

/// Parse a list value (CIF 2.0 only): `[value1 value2 value3]`
///
/// Lists can contain any CIF value type, including nested lists and tables.
fn parse_list(pair: Pair<Rule>, version: CifVersion) -> Result<CifValue, CifError> {
    let mut values = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::data_value | Rule::value | Rule::item_value | Rule::loop_value => {
                let value = parse_value(inner_pair, version)?;
                values.push(value);
            }
            _ => {
                // Skip whitespace and delimiters
            }
        }
    }

    Ok(CifValue::List(values))
}

/// Parse a table value (CIF 2.0 only): `{key1:value1 key2:value2}`
///
/// Tables map string keys to CIF values. Keys must be quoted strings.
fn parse_table(pair: Pair<Rule>, version: CifVersion) -> Result<CifValue, CifError> {
    let mut table = HashMap::new();

    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::table_entry {
            let (key, value) = parse_table_entry(inner_pair, version)?;
            table.insert(key, value);
        }
    }

    Ok(CifValue::Table(table))
}

/// Parse a single table entry: `"key":value`
fn parse_table_entry(
    pair: Pair<Rule>,
    version: CifVersion,
) -> Result<(String, CifValue), CifError> {
    let mut key = String::new();
    let mut value = CifValue::Unknown;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            // CIF 2.0 ONLY: Triple-quoted string keys
            Rule::triple_quoted_string => {
                // VERSION GUARD: Only in CIF 2.0 mode
                if version == CifVersion::V2_0 {
                    key = extract_quoted_content(inner_pair.as_str());
                } else {
                    // Defensive: shouldn't happen in CIF 1.1
                    key = inner_pair.as_str().to_string();
                }
            }
            // CIF 1.1 and 2.0: Regular quoted strings
            Rule::quoted_string
            | Rule::singlequoted
            | Rule::doublequoted
            | Rule::table_key_quoted => {
                key = extract_quoted_content(inner_pair.as_str());
            }
            Rule::data_value
            | Rule::value
            | Rule::item_value
            | Rule::loop_value
            | Rule::list
            | Rule::table
            | Rule::wsdelim_string
            | Rule::unquoted => {
                value = parse_value(inner_pair, version)?;
            }
            _ => {
                // Skip other tokens (colons, whitespace)
            }
        }
    }

    Ok((key, value))
}

/// Parse a triple-quoted string (CIF 2.0 only): `"""..."""` or `'''...'''`
///
/// Triple-quoted strings can span multiple lines and contain raw text without escaping.
fn parse_triple_quoted(pair: Pair<Rule>) -> Result<CifValue, CifError> {
    let text = pair.as_str();
    let span = pair.as_span();

    // Validate minimum length (must have opening and closing triple quotes)
    if text.len() < 6 {
        return Err(CifError::InvalidStructure {
            message: format!(
                "Triple-quoted string too short: expected at least 6 characters, got {}",
                text.len()
            ),
            location: Some((span.start_pos().line_col().0, span.start_pos().line_col().1)),
        });
    }

    // Remove triple-quote delimiters
    let content = if (text.starts_with("\"\"\"") && text.ends_with("\"\"\""))
        || (text.starts_with("'''") && text.ends_with("'''"))
    {
        &text[3..text.len() - 3]
    } else {
        // Grammar should prevent this, but be defensive
        return Err(CifError::InvalidStructure {
            message: "Triple-quoted string missing opening or closing delimiters".to_string(),
            location: Some((span.start_pos().line_col().0, span.start_pos().line_col().1)),
        });
    };

    Ok(CifValue::Text(content.to_string()))
}

/// Parse a quoted string (CIF 1.1 and 2.0): `'...'` or `"..."`
///
/// # CIF Version Differences
///
/// - **CIF 1.1**: Supports doubled-quote escaping (`'O''Brien'` → `O'Brien`)
/// - **CIF 2.0**: Doubled quotes are invalid; use triple-quoted strings instead
fn parse_quoted_string(pair: Pair<Rule>, version: CifVersion) -> Result<CifValue, CifError> {
    let text = pair.as_str();
    let span = pair.as_span();
    let content = extract_quoted_content(text);

    // VERSION GUARD: CIF 2.0 does not support doubled-quote escaping
    // Doubled quotes ('' or "") in CIF 2.0 are invalid - use triple quotes instead
    if version == CifVersion::V2_0 && (content.contains("''") || content.contains("\"\"")) {
        return Err(CifError::InvalidStructure {
            message: "Doubled-quote escaping ('''' or \"\"\"\") is not allowed in CIF 2.0. Use triple-quoted strings instead: '''...''' or \"\"\"...\"\"\"".to_string(),
            location: Some((span.start_pos().line_col().0, span.start_pos().line_col().1)),
        });
    }

    // Try to parse as number first, fall back to text
    if let Ok(num) = content.parse::<f64>() {
        Ok(CifValue::Numeric(num))
    } else {
        Ok(CifValue::Text(content))
    }
}

/// Parse a text field (CIF 1.1 and 2.0): `;...\n;`
///
/// Text fields are multi-line strings delimited by semicolons at line starts.
fn parse_text_field(pair: Pair<Rule>) -> Result<CifValue, CifError> {
    let text = pair.as_str();

    // Remove semicolon delimiters and surrounding whitespace
    let content = text.trim_start_matches(';').trim_end_matches(';').trim();

    Ok(CifValue::Text(content.to_string()))
}

/// Parse an unquoted string (CIF 1.1 and 2.0)
///
/// Handles special values (`?`, `.`) and numeric parsing.
fn parse_unquoted(pair: Pair<Rule>) -> Result<CifValue, CifError> {
    let text = pair.as_str().trim();

    // Check for special values
    match text {
        "?" => return Ok(CifValue::Unknown),
        "." => return Ok(CifValue::NotApplicable),
        _ => {}
    }

    // Try to parse as number
    if let Ok(num) = text.parse::<f64>() {
        Ok(CifValue::Numeric(num))
    } else {
        Ok(CifValue::Text(text.to_string()))
    }
}

/// Helper: Extract content from a quoted string (remove quotes)
///
/// Handles both CIF 1.1 quoted strings ('...' and "...") and CIF 2.0 triple-quoted
/// strings ('''...''' and """..."""). Triple-quotes are checked first to ensure
/// correct parsing.
fn extract_quoted_content(text: &str) -> String {
    let trimmed = text.trim();

    // Check for triple-quoted strings first (CIF 2.0)
    if (trimmed.starts_with("\"\"\"") && trimmed.ends_with("\"\"\"") && trimmed.len() >= 6)
        || (trimmed.starts_with("'''") && trimmed.ends_with("'''") && trimmed.len() >= 6)
    {
        trimmed[3..trimmed.len() - 3].to_string()
    }
    // Then check for single/double quotes (CIF 1.1 and 2.0)
    else if (trimmed.starts_with('\'') && trimmed.ends_with('\''))
        || (trimmed.starts_with('"') && trimmed.ends_with('"'))
    {
        trimmed[1..trimmed.len().saturating_sub(1)].to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    // Tests will be added as we implement the parser
}
