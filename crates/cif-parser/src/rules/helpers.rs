//! Helper functions for version rule implementations.

use crate::ast::{CifValue, Span};

/// Extract content from a quoted string (remove surrounding quotes).
///
/// Handles both single and double quotes.
pub fn extract_quoted_content(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.len() < 2 {
        return trimmed.to_string();
    }

    let first = trimmed.chars().next().unwrap();
    let last = trimmed.chars().last().unwrap();

    if (first == '\'' && last == '\'') || (first == '"' && last == '"') {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

/// Extract content from a triple-quoted string (remove `'''` or `"""`).
pub fn extract_triple_quoted_content(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.len() < 6 {
        return trimmed.to_string();
    }

    if (trimmed.starts_with("'''") && trimmed.ends_with("'''"))
        || (trimmed.starts_with("\"\"\"") && trimmed.ends_with("\"\"\""))
    {
        trimmed[3..trimmed.len() - 3].to_string()
    } else {
        trimmed.to_string()
    }
}

/// Unescape doubled quotes in a CIF 1.1 quoted string.
///
/// CIF 1.1 allows `''` inside single-quoted strings and `""` inside
/// double-quoted strings as escape sequences.
#[allow(dead_code)]
pub fn unescape_doubled_quotes(content: &str, quote_char: char) -> String {
    let doubled = format!("{}{}", quote_char, quote_char);
    content.replace(&doubled, &quote_char.to_string())
}

/// Check if a quoted string contains doubled-quote escaping.
#[allow(dead_code)]
pub fn has_doubled_quotes(raw: &str, quote_char: char) -> bool {
    // First extract the content (without outer quotes)
    let content = extract_quoted_content(raw);
    let doubled = format!("{}{}", quote_char, quote_char);
    content.contains(&doubled)
}

/// Parse an unquoted value to a CifValue.
///
/// Handles:
/// - Special values: `?` (unknown) and `.` (not applicable)
/// - Numbers (with optional uncertainty notation)
/// - Plain text
pub fn parse_unquoted_value(text: &str, span: Span) -> CifValue {
    let trimmed = text.trim();

    // Special values
    if trimmed == "?" {
        return CifValue::unknown(span);
    }
    if trimmed == "." {
        return CifValue::not_applicable(span);
    }

    // Try parsing as number
    if let Ok(num) = trimmed.parse::<f64>() {
        return CifValue::numeric(num, span);
    }

    // Try uncertainty notation like "1.234(5)"
    if let Some((value, uncertainty)) = CifValue::parse_with_uncertainty(trimmed) {
        return CifValue::numeric_with_uncertainty(value, uncertainty, span);
    }

    // Fall back to text
    CifValue::text(trimmed.to_string(), span)
}
