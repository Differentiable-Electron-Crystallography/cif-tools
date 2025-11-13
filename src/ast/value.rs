//! CIF value types with automatic type detection.

/// Represents a single value in a CIF file with automatic type detection.
///
/// CIF values come in many forms and require careful parsing to handle quotes,
/// special characters, and type detection. This enum represents the parsed and
/// typed result.
///
/// # Value Types
///
/// - **Text**: String values, including quoted strings and text fields
/// - **Numeric**: Floating-point numbers (integers are stored as f64)
/// - **Unknown**: The special value `?` indicating missing/unknown data
/// - **NotApplicable**: The special value `.` indicating not applicable
///
/// # Parsing Strategy
///
/// Values are parsed in this order:
/// 1. Check for special values (`?`, `.`)
/// 2. Remove quotes or extract text field content
/// 3. Try to parse as a number
/// 4. Fall back to text
///
/// # Examples
///
/// ```
/// use cif_parser::CifValue;
///
/// // Different ways values are parsed
/// assert_eq!(CifValue::parse_value("123.45"), CifValue::Numeric(123.45));
/// assert_eq!(CifValue::parse_value("'hello'"), CifValue::Text("hello".to_string()));
/// assert_eq!(CifValue::parse_value("?"), CifValue::Unknown);
/// assert_eq!(CifValue::parse_value("."), CifValue::NotApplicable);
/// ```
///
/// # Text Fields
///
/// Text fields are multi-line values delimited by semicolons at the start of lines:
/// ```text
/// ;This is a text field
/// that can span multiple lines
/// and contain special characters !@#$%
/// ;
/// ```
///
/// These are automatically detected and the semicolon delimiters are removed.
#[derive(Debug, Clone, PartialEq)]
pub enum CifValue {
    /// String value (from quoted strings, unquoted strings, or text fields)
    Text(String),
    /// Numeric value (both integers and floats are stored as f64)
    Numeric(f64),
    /// Unknown value (represented as `?` in CIF files)
    Unknown,
    /// Not applicable value (represented as `.` in CIF files)
    NotApplicable,
}

impl CifValue {
    /// Parse a CIF value from a raw string.
    ///
    /// This is the main entry point for value parsing. It handles all CIF value
    /// types including quoted strings, text fields, numbers, and special values.
    ///
    /// # Examples
    /// ```
    /// use cif_parser::CifValue;
    ///
    /// assert_eq!(CifValue::parse_value("42"), CifValue::Numeric(42.0));
    /// assert_eq!(CifValue::parse_value("'text'"), CifValue::Text("text".to_string()));
    /// assert_eq!(CifValue::parse_value("?"), CifValue::Unknown);
    /// ```
    pub fn parse_value(s: &str) -> Self {
        let trimmed = s.trim();

        // Check for special values first
        if let Some(special) = Self::parse_special_value(trimmed) {
            return special;
        }

        // Remove quotes and extract content
        let content = Self::extract_content(trimmed);

        // Try to parse as number, otherwise treat as text
        Self::parse_numeric_or_text(content)
    }

    /// Check for CIF special values (`?` for unknown, `.` for not applicable).
    ///
    /// These values have special meaning in CIF and must be detected before
    /// other parsing logic.
    fn parse_special_value(s: &str) -> Option<Self> {
        match s {
            "?" => Some(CifValue::Unknown),
            "." => Some(CifValue::NotApplicable),
            _ => None,
        }
    }

    /// Extract content from quoted strings or text fields.
    ///
    /// Handles three cases:
    /// 1. Single/double quoted strings: removes the quotes
    /// 2. Text fields (start with `;`): removes semicolon delimiters
    /// 3. Unquoted strings: returns as-is
    ///
    /// # Text Field Handling
    /// Text fields in CIF are delimited by semicolons at the start of lines:
    /// ```text
    /// ;This is a text field
    /// with multiple lines
    /// ;
    /// ```
    /// The semicolons and surrounding whitespace are removed.
    fn extract_content(s: &str) -> &str {
        // Handle quoted strings
        if (s.starts_with('\'') && s.ends_with('\'')) || (s.starts_with('"') && s.ends_with('"')) {
            &s[1..s.len() - 1]
        }
        // Handle text fields (semicolon-delimited)
        else if s.starts_with(';') {
            s.trim_start_matches(';').trim_end_matches(';').trim()
        }
        // Unquoted string
        else {
            s
        }
    }

    /// Attempt to parse as a number, falling back to text.
    ///
    /// Uses Rust's built-in f64 parsing which handles:
    /// - Integers: `123`
    /// - Floats: `123.45`
    /// - Scientific notation: `1.23e-4`
    /// - Signs: `-123.45`
    ///
    /// If parsing fails, the string is stored as [`CifValue::Text`].
    fn parse_numeric_or_text(s: &str) -> Self {
        if let Ok(num) = s.parse::<f64>() {
            CifValue::Numeric(num)
        } else {
            CifValue::Text(s.to_string())
        }
    }

    /// Get the value as a string reference, if it's a Text variant.
    ///
    /// # Examples
    /// ```
    /// use cif_parser::CifValue;
    ///
    /// let val = CifValue::Text("hello".to_string());
    /// assert_eq!(val.as_string(), Some("hello"));
    ///
    /// let num = CifValue::Numeric(42.0);
    /// assert_eq!(num.as_string(), None);
    /// ```
    pub fn as_string(&self) -> Option<&str> {
        match self {
            CifValue::Text(s) => Some(s),
            _ => None,
        }
    }

    /// Get the value as a number, if it's a Numeric variant.
    ///
    /// # Examples
    /// ```
    /// use cif_parser::CifValue;
    ///
    /// let val = CifValue::Numeric(42.0);
    /// assert_eq!(val.as_numeric(), Some(42.0));
    ///
    /// let text = CifValue::Text("hello".to_string());
    /// assert_eq!(text.as_numeric(), None);
    /// ```
    pub fn as_numeric(&self) -> Option<f64> {
        match self {
            CifValue::Numeric(n) => Some(*n),
            _ => None,
        }
    }
}

// Implement standard FromStr trait
impl std::str::FromStr for CifValue {
    type Err = std::convert::Infallible; // This method never fails

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse_value(s))
    }
}
