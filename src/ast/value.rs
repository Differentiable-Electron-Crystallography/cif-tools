//! CIF value types with automatic type detection.

/// Represents a single value in a CIF file with automatic type detection.
///
/// CIF values come in many forms and require careful parsing to handle quotes,
/// special characters, and type detection. This enum represents the parsed and
/// typed result.
///
/// # Value Types
///
/// ## CIF 1.1 and 2.0:
/// - **Text**: String values, including quoted strings and text fields
/// - **Numeric**: Floating-point numbers (integers are stored as f64)
/// - **Unknown**: The special value `?` indicating missing/unknown data
/// - **NotApplicable**: The special value `.` indicating not applicable
///
/// ## CIF 2.0 only:
/// - **List**: Ordered collection of values `[value1 value2 value3]`
/// - **Table**: Key-value pairs `{key1:value1 key2:value2}`
///
/// # Parsing Strategy
///
/// Values are parsed in this order:
/// 1. Check for special values (`?`, `.`)
/// 2. Check for composite structures (lists `[...]`, tables `{...}`)
/// 3. Check for triple-quoted strings (`"""..."""` or `'''...'''`)
/// 4. Remove quotes or extract text field content
/// 5. Try to parse as a number
/// 6. Fall back to text
///
/// # Examples
///
/// ```
/// use cif_parser::CifValue;
///
/// // CIF 1.1 values
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
    // ===== CIF 1.1 Value Types =====
    /// String value (from quoted strings, unquoted strings, or text fields)
    Text(String),
    /// Numeric value (both integers and floats are stored as f64)
    Numeric(f64),
    /// Unknown value (represented as `?` in CIF files)
    Unknown,
    /// Not applicable value (represented as `.` in CIF files)
    NotApplicable,

    // ===== CIF 2.0 Value Types =====
    /// List of values (CIF 2.0 only)
    /// Example: `[value1 value2 value3]`
    /// Lists can contain any CIF value type, including nested lists and tables
    List(Vec<CifValue>),

    /// Table/dictionary of key-value pairs (CIF 2.0 only)
    /// Example: `{key1:value1 key2:value2}`
    /// Keys must be quoted strings, values can be any CIF value type
    Table(std::collections::HashMap<String, CifValue>),
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

    /// Get the value as a list, if it's a List variant (CIF 2.0 only).
    ///
    /// # Examples
    /// ```
    /// use cif_parser::CifValue;
    ///
    /// let list = CifValue::List(vec![
    ///     CifValue::Text("a".to_string()),
    ///     CifValue::Numeric(1.0),
    /// ]);
    /// assert_eq!(list.as_list().unwrap().len(), 2);
    ///
    /// let text = CifValue::Text("hello".to_string());
    /// assert_eq!(text.as_list(), None);
    /// ```
    pub fn as_list(&self) -> Option<&Vec<CifValue>> {
        match self {
            CifValue::List(list) => Some(list),
            _ => None,
        }
    }

    /// Get the value as a mutable list, if it's a List variant (CIF 2.0 only).
    pub fn as_list_mut(&mut self) -> Option<&mut Vec<CifValue>> {
        match self {
            CifValue::List(list) => Some(list),
            _ => None,
        }
    }

    /// Get the value as a table, if it's a Table variant (CIF 2.0 only).
    ///
    /// # Examples
    /// ```
    /// use cif_parser::CifValue;
    /// use std::collections::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert("key".to_string(), CifValue::Text("value".to_string()));
    /// let table = CifValue::Table(map);
    ///
    /// assert_eq!(table.as_table().unwrap().len(), 1);
    ///
    /// let text = CifValue::Text("hello".to_string());
    /// assert_eq!(text.as_table(), None);
    /// ```
    pub fn as_table(&self) -> Option<&std::collections::HashMap<String, CifValue>> {
        match self {
            CifValue::Table(table) => Some(table),
            _ => None,
        }
    }

    /// Get the value as a mutable table, if it's a Table variant (CIF 2.0 only).
    pub fn as_table_mut(&mut self) -> Option<&mut std::collections::HashMap<String, CifValue>> {
        match self {
            CifValue::Table(table) => Some(table),
            _ => None,
        }
    }

    /// Check if this value is a CIF 2.0-only type (List or Table).
    ///
    /// Returns `true` for List and Table variants, `false` for CIF 1.1 types.
    ///
    /// # Examples
    /// ```
    /// use cif_parser::CifValue;
    ///
    /// let list = CifValue::List(vec![]);
    /// assert!(list.is_cif2_only());
    ///
    /// let text = CifValue::Text("hello".to_string());
    /// assert!(!text.is_cif2_only());
    /// ```
    pub fn is_cif2_only(&self) -> bool {
        matches!(self, CifValue::List(_) | CifValue::Table(_))
    }

    /// Get the length of a list without borrowing.
    ///
    /// Returns `Some(length)` if this is a List, `None` otherwise.
    ///
    /// # Examples
    /// ```
    /// use cif_parser::CifValue;
    ///
    /// let list = CifValue::List(vec![
    ///     CifValue::Numeric(1.0),
    ///     CifValue::Numeric(2.0),
    ///     CifValue::Numeric(3.0),
    /// ]);
    /// assert_eq!(list.as_list_len(), Some(3));
    ///
    /// let text = CifValue::Text("hello".to_string());
    /// assert_eq!(text.as_list_len(), None);
    /// ```
    pub fn as_list_len(&self) -> Option<usize> {
        match self {
            CifValue::List(list) => Some(list.len()),
            _ => None,
        }
    }

    /// Get an iterator over table keys.
    ///
    /// Returns `Some(iterator)` if this is a Table, `None` otherwise.
    ///
    /// # Examples
    /// ```
    /// use cif_parser::CifValue;
    /// use std::collections::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert("key1".to_string(), CifValue::Numeric(1.0));
    /// map.insert("key2".to_string(), CifValue::Numeric(2.0));
    /// let table = CifValue::Table(map);
    ///
    /// let keys: Vec<&str> = table.as_table_keys().unwrap().collect();
    /// assert_eq!(keys.len(), 2);
    /// ```
    pub fn as_table_keys(&self) -> Option<impl Iterator<Item = &str>> {
        match self {
            CifValue::Table(table) => Some(table.keys().map(|s| s.as_str())),
            _ => None,
        }
    }

    /// Get a value from a table by key.
    ///
    /// Returns `Some(&value)` if this is a Table and the key exists, `None` otherwise.
    ///
    /// # Examples
    /// ```
    /// use cif_parser::CifValue;
    /// use std::collections::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert("x".to_string(), CifValue::Numeric(1.0));
    /// let table = CifValue::Table(map);
    ///
    /// assert!(table.as_table_get("x").is_some());
    /// assert!(table.as_table_get("y").is_none());
    ///
    /// let text = CifValue::Text("hello".to_string());
    /// assert!(text.as_table_get("x").is_none());
    /// ```
    pub fn as_table_get(&self, key: &str) -> Option<&CifValue> {
        match self {
            CifValue::Table(table) => table.get(key),
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
