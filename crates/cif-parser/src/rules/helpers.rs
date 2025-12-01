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
