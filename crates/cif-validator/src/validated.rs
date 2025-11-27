//! ValidatedCIF type for span-to-definition mapping.
//!
//! This module provides the `ValidatedCif` type that pairs a parsed CIF document
//! with dictionary metadata, enabling:
//! - Definition lookup at any source position (for IDE hover)
//! - Typed accessors based on dictionary type information
//! - Rich error context with dictionary definitions

use std::sync::Arc;

use cif_parser::{CifBlock, CifDocument, CifLoop, CifValue, Span};

use crate::dictionary::{DataItem, Dictionary};

/// A CIF document that has been validated against a dictionary.
///
/// This type provides:
/// - Access to the original parsed document
/// - Dictionary metadata for each data item
/// - Span-to-definition lookup for IDE features
///
/// # Example
/// ```ignore
/// let validated = ValidatedCif::new(doc, dictionary);
///
/// // Look up definition at cursor position (for hover info)
/// if let Some(def) = validated.definition_at(5, 10) {
///     println!("Item: {}", def.name);
///     println!("Description: {}", def.description.unwrap_or_default());
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ValidatedCif {
    /// The underlying CIF document
    document: CifDocument,
    /// The dictionary used for validation
    dictionary: Arc<Dictionary>,
    /// Precomputed index for span-to-definition lookup
    span_index: SpanIndex,
}

impl ValidatedCif {
    /// Create a ValidatedCif from a document and dictionary.
    pub fn new(document: CifDocument, dictionary: Arc<Dictionary>) -> Self {
        let span_index = SpanIndex::build(&document, &dictionary);
        Self {
            document,
            dictionary,
            span_index,
        }
    }

    /// Get the underlying CIF document.
    pub fn document(&self) -> &CifDocument {
        &self.document
    }

    /// Get the dictionary.
    pub fn dictionary(&self) -> &Dictionary {
        &self.dictionary
    }

    /// Look up the definition for a source position (for IDE hover).
    ///
    /// Returns the DataItem definition if the position is within a data value
    /// that has a known dictionary definition.
    pub fn definition_at(&self, line: usize, col: usize) -> Option<&DataItem> {
        self.span_index
            .find(line, col)
            .and_then(|item_name| self.dictionary.items.get(&item_name.to_lowercase()))
    }

    /// Get a typed value with its definition.
    pub fn get_typed<T: FromCifValue>(
        &self,
        block_name: &str,
        item_name: &str,
    ) -> Option<TypedValue<T>> {
        let block = self.document.get_block(block_name)?;
        let value = block.get_item(item_name)?;
        let canonical = self.dictionary.resolve_name(item_name);
        let definition = self.dictionary.items.get(&canonical)?.clone();

        T::from_cif_value(value).map(|typed| TypedValue {
            value: typed,
            raw: value.clone(),
            definition,
        })
    }

    /// Get a validated block wrapper.
    pub fn block(&self, name: &str) -> Option<ValidatedBlock<'_>> {
        self.document.get_block(name).map(|block| ValidatedBlock {
            block,
            dictionary: &self.dictionary,
        })
    }

    /// Get the first validated block.
    pub fn first_block(&self) -> Option<ValidatedBlock<'_>> {
        self.document.first_block().map(|block| ValidatedBlock {
            block,
            dictionary: &self.dictionary,
        })
    }

    /// Iterate over all validated blocks.
    pub fn blocks(&self) -> impl Iterator<Item = ValidatedBlock<'_>> {
        self.document
            .blocks
            .iter()
            .map(move |block| ValidatedBlock {
                block,
                dictionary: &self.dictionary,
            })
    }
}

/// Index for quick span-to-definition lookup.
#[derive(Debug, Clone, Default)]
struct SpanIndex {
    /// Entries mapping spans to item names
    entries: Vec<SpanIndexEntry>,
}

#[derive(Debug, Clone)]
struct SpanIndexEntry {
    span: Span,
    item_name: String, // Canonical name (lowercase)
}

impl SpanIndex {
    /// Build a span index from a document and dictionary.
    fn build(doc: &CifDocument, dict: &Dictionary) -> Self {
        let mut entries = Vec::new();

        for block in &doc.blocks {
            Self::index_block(block, dict, &mut entries);
        }

        SpanIndex { entries }
    }

    fn index_block(block: &CifBlock, dict: &Dictionary, entries: &mut Vec<SpanIndexEntry>) {
        // Index individual items
        for (name, value) in &block.items {
            let canonical = dict.resolve_name(name);
            entries.push(SpanIndexEntry {
                span: value.span,
                item_name: canonical,
            });
        }

        // Index loop values
        for loop_ in &block.loops {
            for (col, tag) in loop_.tags.iter().enumerate() {
                let canonical = dict.resolve_name(tag);
                for row in 0..loop_.len() {
                    if let Some(value) = loop_.get(row, col) {
                        entries.push(SpanIndexEntry {
                            span: value.span,
                            item_name: canonical.clone(),
                        });
                    }
                }
            }
        }

        // Index frames
        for frame in &block.frames {
            for (name, value) in &frame.items {
                let canonical = dict.resolve_name(name);
                entries.push(SpanIndexEntry {
                    span: value.span,
                    item_name: canonical,
                });
            }
            for loop_ in &frame.loops {
                for (col, tag) in loop_.tags.iter().enumerate() {
                    let canonical = dict.resolve_name(tag);
                    for row in 0..loop_.len() {
                        if let Some(value) = loop_.get(row, col) {
                            entries.push(SpanIndexEntry {
                                span: value.span,
                                item_name: canonical.clone(),
                            });
                        }
                    }
                }
            }
        }
    }

    /// Find the item name at a given position.
    fn find(&self, line: usize, col: usize) -> Option<&str> {
        self.entries
            .iter()
            .find(|e| e.span.contains(line, col))
            .map(|e| e.item_name.as_str())
    }
}

/// A typed value with its dictionary definition.
#[derive(Debug, Clone)]
pub struct TypedValue<T> {
    /// The typed Rust value
    pub value: T,
    /// The original CIF value (with span)
    pub raw: CifValue,
    /// The dictionary definition
    pub definition: DataItem,
}

impl<T> TypedValue<T> {
    /// Get the span where this value appears.
    pub fn span(&self) -> Span {
        self.raw.span
    }

    /// Get the definition.
    pub fn definition(&self) -> &DataItem {
        &self.definition
    }

    /// Get the description from the definition.
    pub fn description(&self) -> Option<&str> {
        self.definition.description.as_deref()
    }
}

/// A validated block wrapper providing typed access.
#[derive(Debug, Clone)]
pub struct ValidatedBlock<'a> {
    block: &'a CifBlock,
    dictionary: &'a Dictionary,
}

impl<'a> ValidatedBlock<'a> {
    /// Get the block name.
    pub fn name(&self) -> &str {
        &self.block.name
    }

    /// Get a value with its definition.
    pub fn get_with_def(&self, name: &str) -> Option<(&CifValue, Option<&DataItem>)> {
        let value = self.block.get_item(name)?;
        let canonical = self.dictionary.resolve_name(name);
        let def = self.dictionary.items.get(&canonical);
        Some((value, def))
    }

    /// Get a typed value.
    pub fn get_typed<T: FromCifValue>(&self, name: &str) -> Option<TypedValue<T>> {
        let (value, def) = self.get_with_def(name)?;
        let definition = def?.clone();
        T::from_cif_value(value).map(|typed| TypedValue {
            value: typed,
            raw: value.clone(),
            definition,
        })
    }

    /// Get a typed loop accessor.
    pub fn find_loop(&self, tag: &str) -> Option<ValidatedLoop<'a>> {
        self.block.find_loop(tag).map(|loop_| ValidatedLoop {
            loop_,
            dictionary: self.dictionary,
        })
    }

    /// Get all item names in this block.
    pub fn item_names(&self) -> impl Iterator<Item = &str> {
        self.block.items.keys().map(|s| s.as_str())
    }
}

/// A validated loop wrapper.
#[derive(Debug, Clone)]
pub struct ValidatedLoop<'a> {
    loop_: &'a CifLoop,
    dictionary: &'a Dictionary,
}

impl<'a> ValidatedLoop<'a> {
    /// Get the number of rows.
    pub fn len(&self) -> usize {
        self.loop_.len()
    }

    /// Check if the loop is empty.
    pub fn is_empty(&self) -> bool {
        self.loop_.len() == 0
    }

    /// Get the tags (column names).
    pub fn tags(&self) -> &[String] {
        &self.loop_.tags
    }

    /// Get definitions for all columns.
    pub fn column_definitions(&self) -> Vec<Option<&DataItem>> {
        self.loop_
            .tags
            .iter()
            .map(|tag| {
                let canonical = self.dictionary.resolve_name(tag);
                self.dictionary.items.get(&canonical)
            })
            .collect()
    }

    /// Get a value at (row, col) with its definition.
    pub fn get_with_def(&self, row: usize, col: usize) -> Option<(&CifValue, Option<&DataItem>)> {
        let value = self.loop_.get(row, col)?;
        let tag = self.loop_.tags.get(col)?;
        let canonical = self.dictionary.resolve_name(tag);
        let def = self.dictionary.items.get(&canonical);
        Some((value, def))
    }

    /// Get a typed column.
    pub fn get_column_typed<T: FromCifValue>(&self, tag: &str) -> Option<Vec<Option<T>>> {
        let col_idx = self
            .loop_
            .tags
            .iter()
            .position(|t| t.eq_ignore_ascii_case(tag))?;
        Some(
            (0..self.loop_.len())
                .map(|row| {
                    self.loop_
                        .get(row, col_idx)
                        .and_then(|v| T::from_cif_value(v))
                })
                .collect(),
        )
    }

    /// Iterate over rows with definitions.
    pub fn rows(&self) -> impl Iterator<Item = ValidatedRow<'a>> + 'a {
        let loop_ = self.loop_;
        let dictionary = self.dictionary;
        (0..loop_.len()).map(move |row| ValidatedRow {
            loop_,
            row,
            dictionary,
        })
    }
}

/// A single row in a validated loop.
#[derive(Debug)]
pub struct ValidatedRow<'a> {
    loop_: &'a CifLoop,
    row: usize,
    dictionary: &'a Dictionary,
}

impl<'a> ValidatedRow<'a> {
    /// Get the row index.
    pub fn index(&self) -> usize {
        self.row
    }

    /// Get a value by tag name with its definition.
    pub fn get(&self, tag: &str) -> Option<(&CifValue, Option<&DataItem>)> {
        let col = self
            .loop_
            .tags
            .iter()
            .position(|t| t.eq_ignore_ascii_case(tag))?;
        let value = self.loop_.get(self.row, col)?;
        let canonical = self.dictionary.resolve_name(tag);
        let def = self.dictionary.items.get(&canonical);
        Some((value, def))
    }

    /// Get a typed value by tag name.
    pub fn get_typed<T: FromCifValue>(&self, tag: &str) -> Option<TypedValue<T>> {
        let (value, def) = self.get(tag)?;
        let definition = def?.clone();
        T::from_cif_value(value).map(|typed| TypedValue {
            value: typed,
            raw: value.clone(),
            definition,
        })
    }
}

/// Trait for converting CIF values to typed Rust values.
pub trait FromCifValue: Sized {
    /// Try to convert a CIF value to this type.
    fn from_cif_value(value: &CifValue) -> Option<Self>;
}

impl FromCifValue for f64 {
    fn from_cif_value(value: &CifValue) -> Option<Self> {
        value.as_numeric()
    }
}

impl FromCifValue for i64 {
    fn from_cif_value(value: &CifValue) -> Option<Self> {
        value.as_numeric().map(|n| n as i64)
    }
}

impl FromCifValue for String {
    fn from_cif_value(value: &CifValue) -> Option<Self> {
        value.as_string().map(|s| s.to_string())
    }
}

impl FromCifValue for bool {
    fn from_cif_value(value: &CifValue) -> Option<Self> {
        value
            .as_string()
            .map(|s| matches!(s.to_lowercase().as_str(), "yes" | "y" | "true" | "1"))
    }
}

/// A value with standard uncertainty.
#[derive(Debug, Clone, Copy)]
pub struct Measurand {
    /// The numeric value
    pub value: f64,
    /// The standard uncertainty (if present)
    pub uncertainty: Option<f64>,
}

impl FromCifValue for Measurand {
    fn from_cif_value(value: &CifValue) -> Option<Self> {
        match value.as_numeric_with_uncertainty() {
            Some((v, u)) => Some(Measurand {
                value: v,
                uncertainty: Some(u),
            }),
            None => value.as_numeric().map(|v| Measurand {
                value: v,
                uncertainty: None,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::load_dictionary;

    #[test]
    fn test_validated_cif() {
        let dict_content = r#"
#\#CIF_2.0
data_TEST_DICT

save_cell.length_a
    _definition.id                '_cell.length_a'
    _type.contents                Real
    _description.text             'Unit cell length a'
save_
"#;
        let dict_doc = CifDocument::parse(dict_content).unwrap();
        let dict = Arc::new(load_dictionary(&dict_doc).unwrap());

        let cif_content = r#"
data_test
_cell.length_a 10.5
"#;
        let cif_doc = CifDocument::parse(cif_content).unwrap();

        let validated = ValidatedCif::new(cif_doc, dict);

        // Test definition lookup
        let block = validated.first_block().unwrap();
        let (value, def) = block.get_with_def("_cell.length_a").unwrap();
        assert!(def.is_some());
        assert_eq!(
            def.unwrap().description,
            Some("Unit cell length a".to_string())
        );
        assert!(value.is_numeric());
    }

    #[test]
    fn test_typed_value() {
        let dict_content = r#"
#\#CIF_2.0
data_TEST_DICT

save_cell.length_a
    _definition.id                '_cell.length_a'
    _type.contents                Real
save_
"#;
        let dict_doc = CifDocument::parse(dict_content).unwrap();
        let dict = Arc::new(load_dictionary(&dict_doc).unwrap());

        let cif_content = r#"
data_test
_cell.length_a 10.5
"#;
        let cif_doc = CifDocument::parse(cif_content).unwrap();

        let validated = ValidatedCif::new(cif_doc, dict);
        let typed: TypedValue<f64> = validated
            .get_typed("test", "_cell.length_a")
            .expect("Should get typed value");

        assert!((typed.value - 10.5).abs() < 1e-10);
    }

    #[test]
    fn test_measurand() {
        let cif_content = r#"
data_test
_cell.length_a 7.470(6)
"#;
        let cif_doc = CifDocument::parse(cif_content).unwrap();
        let value = cif_doc
            .first_block()
            .unwrap()
            .get_item("_cell.length_a")
            .unwrap();

        let measurand = Measurand::from_cif_value(value).unwrap();
        assert!((measurand.value - 7.470).abs() < 1e-10);
        assert!(measurand.uncertainty.is_some());
        assert!((measurand.uncertainty.unwrap() - 0.006).abs() < 1e-10);
    }
}
