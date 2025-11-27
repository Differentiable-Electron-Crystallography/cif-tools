//! Dictionary types for representing DDLm dictionaries.
//!
//! These types model the structure of DDLm dictionaries, which define
//! valid data names, types, and constraints for CIF files.

use cif_parser::Span;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A complete DDLm dictionary (potentially composed from multiple files)
#[derive(Debug, Clone, Default)]
pub struct Dictionary {
    /// Dictionary metadata
    pub metadata: DictionaryMetadata,
    /// Categories indexed by name (lowercase)
    pub categories: HashMap<String, Category>,
    /// All data items indexed by canonical name (lowercase)
    pub items: HashMap<String, DataItem>,
    /// Alias map: alias (lowercase) -> canonical name (lowercase)
    pub aliases: HashMap<String, String>,
}

impl Dictionary {
    /// Create a new empty dictionary
    pub fn new() -> Self {
        Self::default()
    }

    /// Resolve an alias or name to its canonical form (lowercase)
    ///
    /// Returns the canonical name if found, otherwise returns the input lowercased.
    pub fn resolve_name(&self, name: &str) -> String {
        let lower = name.to_lowercase();
        self.aliases.get(&lower).cloned().unwrap_or(lower)
    }

    /// Look up a data item by name (handles aliases, case-insensitive)
    pub fn get_item(&self, name: &str) -> Option<&DataItem> {
        let canonical = self.resolve_name(name);
        self.items.get(&canonical)
    }

    /// Check if an item exists (handles aliases, case-insensitive)
    pub fn has_item(&self, name: &str) -> bool {
        let canonical = self.resolve_name(name);
        self.items.contains_key(&canonical)
    }

    /// Look up a category by name (case-insensitive)
    pub fn get_category(&self, name: &str) -> Option<&Category> {
        self.categories.get(&name.to_lowercase())
    }

    /// Merge another dictionary into this one
    ///
    /// Later definitions override earlier ones (for domain-specific extensions).
    pub fn merge(&mut self, other: Dictionary) {
        // Merge metadata (other takes precedence for non-None fields)
        if other.metadata.title.is_some() {
            self.metadata.title = other.metadata.title;
        }
        if other.metadata.version.is_some() {
            self.metadata.version = other.metadata.version;
        }

        // Merge categories
        for (name, cat) in other.categories {
            self.categories.insert(name, cat);
        }

        // Merge items
        for (name, item) in other.items {
            // Register aliases from new item
            for alias in &item.aliases {
                self.aliases.insert(alias.to_lowercase(), name.clone());
            }
            self.items.insert(name, item);
        }

        // Merge aliases
        self.aliases.extend(other.aliases);
    }

    /// Get all item names
    pub fn item_names(&self) -> impl Iterator<Item = &str> {
        self.items.keys().map(|s| s.as_str())
    }

    /// Get all category names
    pub fn category_names(&self) -> impl Iterator<Item = &str> {
        self.categories.keys().map(|s| s.as_str())
    }
}

/// Dictionary-level metadata from _dictionary.* items
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DictionaryMetadata {
    /// Dictionary title (_dictionary.title)
    pub title: Option<String>,
    /// Version string (_dictionary.version)
    pub version: Option<String>,
    /// Publication/update date (_dictionary.date)
    pub date: Option<String>,
    /// URI where dictionary can be found (_dictionary.uri)
    pub uri: Option<String>,
    /// DDL conformance version (_dictionary.ddl_conformance)
    pub ddl_conformance: Option<String>,
    /// Namespace (_dictionary.namespace)
    pub namespace: Option<String>,
}

/// A category grouping related data items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    /// Category name (e.g., "atom_site")
    pub name: String,
    /// Full definition ID (e.g., "ATOM_SITE")
    pub definition_id: String,
    /// Description of the category
    pub description: Option<String>,
    /// Category class (Head, Loop, Set)
    pub class: CategoryClass,
    /// Parent category name (for hierarchy)
    pub parent: Option<String>,
    /// Category key items (items that uniquely identify a row)
    pub key_items: Vec<String>,
    /// Items in this category (populated during loading)
    pub item_names: Vec<String>,
    /// Location in dictionary file
    pub span: Span,
}

/// Category class indicating how items can appear
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CategoryClass {
    /// Head category (top of hierarchy)
    Head,
    /// Loop category (items can appear in loops)
    Loop,
    /// Set category (items cannot appear in loops, single values only)
    #[default]
    Set,
}

impl CategoryClass {
    /// Parse from string (case-insensitive)
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "head" => Self::Head,
            "loop" => Self::Loop,
            "set" => Self::Set,
            _ => Self::Set, // Default to Set for unknown
        }
    }
}

/// A single data item definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataItem {
    /// Canonical data name (e.g., "_atom_site.label")
    pub name: String,
    /// Category this item belongs to (e.g., "atom_site")
    pub category: String,
    /// Object name within category (e.g., "label")
    pub object: String,
    /// Legacy aliases (e.g., ["_atom_site_label"])
    pub aliases: Vec<String>,
    /// Type information
    pub type_info: TypeInfo,
    /// Value constraints
    pub constraints: ValueConstraints,
    /// Relationship to other items
    pub links: ItemLinks,
    /// Description text
    pub description: Option<String>,
    /// Default value
    pub default: Option<String>,
    /// dREL method source (for dictionary validation)
    pub drel_method: Option<String>,
    /// Location in dictionary file
    pub span: Span,
}

impl DataItem {
    /// Check if this item is mandatory
    pub fn is_mandatory(&self) -> bool {
        self.constraints.mandatory
    }

    /// Get the full data name including underscore prefix
    pub fn full_name(&self) -> String {
        if self.name.starts_with('_') {
            self.name.clone()
        } else {
            format!("_{}", self.name)
        }
    }
}

/// DDLm type information from _type.* items
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TypeInfo {
    /// Content type (_type.contents): Real, Integer, Text, etc.
    pub contents: ContentType,
    /// Container type (_type.container): Single, List, Matrix, etc.
    pub container: ContainerType,
    /// Purpose (_type.purpose): Measurand, Describe, Link, etc.
    pub purpose: Purpose,
    /// Source (_type.source): Recorded, Assigned, Derived, etc.
    pub source: Source,
    /// Units code (_units.code)
    pub units: Option<String>,
    /// Dimensions for matrix/list types
    pub dimensions: Option<Vec<usize>>,
}

/// DDLm _type.contents values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ContentType {
    /// Real number (floating point)
    Real,
    /// Integer
    Integer,
    /// Non-negative integer
    Count,
    /// Positive integer (1, 2, 3, ...)
    Index,
    /// Arbitrary text
    #[default]
    Text,
    /// Single word (no whitespace)
    Word,
    /// Enumerated code
    Code,
    /// CIF data name
    Name,
    /// CIF tag
    Tag,
    /// URI/URL
    Uri,
    /// Date (YYYY-MM-DD)
    Date,
    /// Date and time
    DateTime,
    /// Version string
    Version,
    /// Dimension specification
    Dimension,
    /// Range specification
    Range,
    /// Complex number
    Complex,
    /// Binary data (base64)
    Binary,
    /// By reference (pointer)
    ByReference,
    /// Implied value
    Implied,
}

impl ContentType {
    /// Parse from string (case-insensitive)
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "real" => Self::Real,
            "integer" => Self::Integer,
            "count" => Self::Count,
            "index" => Self::Index,
            "text" => Self::Text,
            "word" => Self::Word,
            "code" => Self::Code,
            "name" => Self::Name,
            "tag" => Self::Tag,
            "uri" => Self::Uri,
            "date" => Self::Date,
            "datetime" => Self::DateTime,
            "version" => Self::Version,
            "dimension" => Self::Dimension,
            "range" => Self::Range,
            "complex" => Self::Complex,
            "binary" => Self::Binary,
            "byreference" => Self::ByReference,
            "implied" => Self::Implied,
            _ => Self::Text, // Default to Text for unknown
        }
    }

    /// Check if this type is numeric
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Self::Real | Self::Integer | Self::Count | Self::Index | Self::Complex
        )
    }
}

/// DDLm _type.container values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ContainerType {
    /// Single value
    #[default]
    Single,
    /// List of values
    List,
    /// Array (legacy, treat as List)
    Array,
    /// 2D matrix
    Matrix,
    /// Key-value table
    Table,
}

impl ContainerType {
    /// Parse from string (case-insensitive)
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "single" => Self::Single,
            "list" => Self::List,
            "array" => Self::Array,
            "matrix" => Self::Matrix,
            "table" => Self::Table,
            _ => Self::Single, // Default to Single for unknown
        }
    }
}

/// DDLm _type.purpose values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Purpose {
    /// Measured quantity (may have standard uncertainty)
    Measurand,
    /// Pure number
    Number,
    /// Count of items
    Count,
    /// Index into a sequence
    Index,
    /// Text description
    #[default]
    Describe,
    /// Encoded value
    Encode,
    /// State indicator
    State,
    /// Foreign key link to another item
    Link,
    /// Primary key component
    Key,
    /// Composite/derived value
    Composite,
    /// Audit/provenance tracking
    Audit,
}

impl Purpose {
    /// Parse from string (case-insensitive)
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "measurand" => Self::Measurand,
            "number" => Self::Number,
            "count" => Self::Count,
            "index" => Self::Index,
            "describe" => Self::Describe,
            "encode" => Self::Encode,
            "state" => Self::State,
            "link" => Self::Link,
            "key" => Self::Key,
            "composite" => Self::Composite,
            "audit" => Self::Audit,
            _ => Self::Describe, // Default
        }
    }
}

/// DDLm _type.source values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Source {
    /// Recorded from experiment/observation
    #[default]
    Recorded,
    /// Assigned by user/software
    Assigned,
    /// Derived from other values
    Derived,
}

impl Source {
    /// Parse from string (case-insensitive)
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "recorded" => Self::Recorded,
            "assigned" => Self::Assigned,
            "derived" => Self::Derived,
            _ => Self::Recorded,
        }
    }
}

/// Value constraints from _enumeration.* items
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValueConstraints {
    /// Enumerated allowed values (_enumeration.set)
    pub enumeration: Option<EnumerationConstraint>,
    /// Numeric range constraint (_enumeration.range)
    pub range: Option<RangeConstraint>,
    /// Whether the item is mandatory
    pub mandatory: bool,
}

/// Enumeration constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumerationConstraint {
    /// Allowed values
    pub values: Vec<String>,
    /// Whether comparison is case-sensitive (default: false for CIF)
    pub case_sensitive: bool,
}

impl EnumerationConstraint {
    /// Check if a value is in the allowed set
    pub fn contains(&self, value: &str) -> bool {
        if self.case_sensitive {
            self.values.iter().any(|v| v == value)
        } else {
            self.values.iter().any(|v| v.eq_ignore_ascii_case(value))
        }
    }
}

/// Range constraint for numeric values
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RangeConstraint {
    /// Minimum value (inclusive), None = unbounded
    pub min: Option<f64>,
    /// Maximum value (inclusive), None = unbounded
    pub max: Option<f64>,
}

impl RangeConstraint {
    /// Parse from DDLm range string (e.g., "0.0:", ":100", "0:1")
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return None;
        }

        let min = if parts[0].is_empty() {
            None
        } else {
            parts[0].parse().ok()
        };

        let max = if parts[1].is_empty() {
            None
        } else {
            parts[1].parse().ok()
        };

        // At least one bound should be specified
        if min.is_none() && max.is_none() {
            return None;
        }

        Some(Self { min, max })
    }

    /// Check if a value is within the range
    pub fn contains(&self, value: f64) -> bool {
        if let Some(min) = self.min {
            if value < min {
                return false;
            }
        }
        if let Some(max) = self.max {
            if value > max {
                return false;
            }
        }
        true
    }
}

/// Links to other items (foreign keys, etc.)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemLinks {
    /// Item this links to (_name.linked_item_id)
    pub linked_item: Option<String>,
}

/// Parse a data name into (category, object)
///
/// Examples:
/// - "_atom_site.label" -> ("atom_site", "label")
/// - "_cell_length_a" -> ("cell", "length_a") (legacy format)
pub fn parse_data_name(name: &str) -> Option<(String, String)> {
    // Remove leading underscore if present
    let name = name.strip_prefix('_').unwrap_or(name);

    // Modern DDLm format: category.object
    if let Some(pos) = name.find('.') {
        let category = &name[..pos];
        let object = &name[pos + 1..];
        return Some((category.to_lowercase(), object.to_lowercase()));
    }

    // Legacy format: category_object (find first underscore)
    // This is a heuristic - may not always be correct
    if let Some(pos) = name.find('_') {
        let category = &name[..pos];
        let object = &name[pos + 1..];
        return Some((category.to_lowercase(), object.to_lowercase()));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_data_name_modern() {
        let (cat, obj) = parse_data_name("_atom_site.label").unwrap();
        assert_eq!(cat, "atom_site");
        assert_eq!(obj, "label");
    }

    #[test]
    fn test_parse_data_name_legacy() {
        let (cat, obj) = parse_data_name("_cell_length_a").unwrap();
        assert_eq!(cat, "cell");
        assert_eq!(obj, "length_a");
    }

    #[test]
    fn test_range_constraint() {
        let range = RangeConstraint::parse("0.0:").unwrap();
        assert_eq!(range.min, Some(0.0));
        assert_eq!(range.max, None);
        assert!(range.contains(0.0));
        assert!(range.contains(100.0));
        assert!(!range.contains(-1.0));

        let range = RangeConstraint::parse(":100").unwrap();
        assert_eq!(range.min, None);
        assert_eq!(range.max, Some(100.0));

        let range = RangeConstraint::parse("0:1").unwrap();
        assert_eq!(range.min, Some(0.0));
        assert_eq!(range.max, Some(1.0));
    }

    #[test]
    fn test_content_type_is_numeric() {
        assert!(ContentType::Real.is_numeric());
        assert!(ContentType::Integer.is_numeric());
        assert!(!ContentType::Text.is_numeric());
        assert!(!ContentType::Word.is_numeric());
    }

    #[test]
    fn test_enumeration_constraint() {
        let constraint = EnumerationConstraint {
            values: vec!["yes".to_string(), "no".to_string()],
            case_sensitive: false,
        };
        assert!(constraint.contains("yes"));
        assert!(constraint.contains("YES"));
        assert!(constraint.contains("Yes"));
        assert!(!constraint.contains("maybe"));
    }

    #[test]
    fn test_dictionary_resolve_name() {
        let mut dict = Dictionary::new();
        dict.aliases.insert(
            "_diffrn_ambient_pressure".to_string(),
            "_diffrn.ambient_pressure".to_string(),
        );

        assert_eq!(
            dict.resolve_name("_diffrn_ambient_pressure"),
            "_diffrn.ambient_pressure"
        );
        assert_eq!(
            dict.resolve_name("_DIFFRN_AMBIENT_PRESSURE"),
            "_diffrn.ambient_pressure"
        );
        assert_eq!(dict.resolve_name("_unknown_item"), "_unknown_item");
    }
}
