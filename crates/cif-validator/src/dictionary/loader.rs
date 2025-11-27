//! Dictionary loading from CIF 2.0 files.
//!
//! DDLm dictionaries are written in CIF 2.0 format. Each save frame
//! defines either a category or a data item.

use cif_parser::{CifBlock, CifDocument, CifFrame, CifValueKind};

use super::types::*;
use crate::error::DictionaryError;

/// Load a DDLm dictionary from a parsed CIF document.
///
/// # Arguments
/// * `doc` - A CIF document containing the dictionary
///
/// # Returns
/// * `Ok(Dictionary)` - The loaded dictionary
/// * `Err(Vec<DictionaryError>)` - Errors encountered during loading
///
/// # Example
/// ```ignore
/// use cif_parser::CifDocument;
/// use cif_validator::dictionary::load_dictionary;
///
/// let doc = CifDocument::from_file("cif_core.dic")?;
/// let dict = load_dictionary(&doc)?;
/// ```
pub fn load_dictionary(doc: &CifDocument) -> Result<Dictionary, Vec<DictionaryError>> {
    let mut dict = Dictionary::new();
    let mut errors = Vec::new();

    // Process first data block (dictionaries typically have one block)
    if let Some(block) = doc.first_block() {
        // Load metadata from block header
        load_metadata(&mut dict.metadata, block);

        // Process each save frame
        for frame in &block.frames {
            match load_frame(frame) {
                Ok(FrameContent::Category(cat)) => {
                    dict.categories.insert(cat.name.to_lowercase(), cat);
                }
                Ok(FrameContent::Item(item)) => {
                    let name_lower = item.name.to_lowercase();

                    // Register aliases
                    for alias in &item.aliases {
                        dict.aliases
                            .insert(alias.to_lowercase(), name_lower.clone());
                    }

                    dict.items.insert(name_lower, item);
                }
                Ok(FrameContent::Skip) => {
                    // Frame type not recognized, skip
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }
    }

    // Second pass: populate category.item_names
    populate_category_items(&mut dict);

    if errors.is_empty() {
        Ok(dict)
    } else {
        Err(errors)
    }
}

/// Result of loading a save frame
enum FrameContent {
    Category(Category),
    Item(DataItem),
    Skip, // Unknown frame type
}

/// Load dictionary metadata from block header items
fn load_metadata(metadata: &mut DictionaryMetadata, block: &CifBlock) {
    metadata.title = get_string_item(block, "_dictionary.title");
    metadata.version = get_string_item(block, "_dictionary.version");
    metadata.date = get_string_item(block, "_dictionary.date");
    metadata.uri = get_string_item(block, "_dictionary.uri");
    metadata.ddl_conformance = get_string_item(block, "_dictionary.ddl_conformance");
    metadata.namespace = get_string_item(block, "_dictionary.namespace");
}

/// Load a single save frame, returning Category or DataItem
fn load_frame(frame: &CifFrame) -> Result<FrameContent, DictionaryError> {
    // Determine if this is a category or item definition
    let scope = get_string_item_frame(frame, "_definition.scope");

    match scope.as_deref() {
        Some("Category") | Some("category") => load_category(frame).map(FrameContent::Category),
        _ => {
            // Check if this has type info (indicating it's a data item)
            if frame.get_item("_type.contents").is_some()
                || frame.get_item("_definition.id").is_some()
            {
                load_item(frame).map(FrameContent::Item)
            } else {
                // Unknown frame type, skip
                Ok(FrameContent::Skip)
            }
        }
    }
}

/// Load a category definition from a save frame
fn load_category(frame: &CifFrame) -> Result<Category, DictionaryError> {
    let definition_id = get_string_item_frame(frame, "_definition.id").ok_or_else(|| {
        DictionaryError::MissingField {
            item: frame.name.clone(),
            field: "_definition.id".to_string(),
            span: frame.span,
        }
    })?;

    let name =
        get_string_item_frame(frame, "_name.object_id").unwrap_or_else(|| definition_id.clone());

    let class_str = get_string_item_frame(frame, "_definition.class").unwrap_or_default();
    let class = CategoryClass::parse(&class_str);

    let parent = get_string_item_frame(frame, "_name.category_id");

    // Extract key items from _category_key.name (may be in a loop)
    let key_items = extract_category_keys(frame);

    Ok(Category {
        name: name.to_lowercase(),
        definition_id,
        description: get_string_item_frame(frame, "_description.text"),
        class,
        parent,
        key_items,
        item_names: Vec::new(), // Populated in second pass
        span: frame.span,
    })
}

/// Load a data item definition from a save frame
fn load_item(frame: &CifFrame) -> Result<DataItem, DictionaryError> {
    let name = get_string_item_frame(frame, "_definition.id").ok_or_else(|| {
        DictionaryError::MissingField {
            item: frame.name.clone(),
            field: "_definition.id".to_string(),
            span: frame.span,
        }
    })?;

    // Parse category and object from name
    let (category, object) = parse_data_name(&name).unwrap_or_else(|| {
        // Fall back to using frame values if parsing fails
        let cat = get_string_item_frame(frame, "_name.category_id").unwrap_or_default();
        let obj = get_string_item_frame(frame, "_name.object_id").unwrap_or_default();
        (cat.to_lowercase(), obj.to_lowercase())
    });

    // Extract aliases from _alias.definition_id (may be single value or loop)
    let aliases = extract_aliases(frame);

    // Extract type information
    let type_info = extract_type_info(frame);

    // Extract constraints
    let constraints = extract_constraints(frame);

    // Extract links
    let links = extract_links(frame);

    // Extract dREL method
    let drel_method = get_string_item_frame(frame, "_method.expression");

    Ok(DataItem {
        name,
        category,
        object,
        aliases,
        type_info,
        constraints,
        links,
        description: get_string_item_frame(frame, "_description.text"),
        default: get_string_item_frame(frame, "_enumeration.default"),
        drel_method,
        span: frame.span,
    })
}

/// Extract aliases from _alias.definition_id
fn extract_aliases(frame: &CifFrame) -> Vec<String> {
    let mut aliases = Vec::new();

    // Check for single value
    if let Some(value) = frame.get_item("_alias.definition_id") {
        if let Some(s) = value.as_string() {
            aliases.push(s.to_string());
        }
    }

    // Check for loop
    for loop_ in &frame.loops {
        if loop_
            .tags
            .iter()
            .any(|t| t.eq_ignore_ascii_case("_alias.definition_id"))
        {
            if let Some(col_idx) = loop_
                .tags
                .iter()
                .position(|t| t.eq_ignore_ascii_case("_alias.definition_id"))
            {
                for row in 0..loop_.len() {
                    if let Some(value) = loop_.get(row, col_idx) {
                        if let Some(s) = value.as_string() {
                            aliases.push(s.to_string());
                        }
                    }
                }
            }
        }
    }

    aliases
}

/// Extract category key items from _category_key.name
fn extract_category_keys(frame: &CifFrame) -> Vec<String> {
    let mut keys = Vec::new();

    // Check for single value
    if let Some(value) = frame.get_item("_category_key.name") {
        if let Some(s) = value.as_string() {
            keys.push(s.to_string());
        }
    }

    // Check for loop
    for loop_ in &frame.loops {
        if loop_
            .tags
            .iter()
            .any(|t| t.eq_ignore_ascii_case("_category_key.name"))
        {
            if let Some(col_idx) = loop_
                .tags
                .iter()
                .position(|t| t.eq_ignore_ascii_case("_category_key.name"))
            {
                for row in 0..loop_.len() {
                    if let Some(value) = loop_.get(row, col_idx) {
                        if let Some(s) = value.as_string() {
                            keys.push(s.to_string());
                        }
                    }
                }
            }
        }
    }

    keys
}

/// Extract type information from frame
fn extract_type_info(frame: &CifFrame) -> TypeInfo {
    let contents_str = get_string_item_frame(frame, "_type.contents").unwrap_or_default();
    let container_str = get_string_item_frame(frame, "_type.container").unwrap_or_default();
    let purpose_str = get_string_item_frame(frame, "_type.purpose").unwrap_or_default();
    let source_str = get_string_item_frame(frame, "_type.source").unwrap_or_default();

    TypeInfo {
        contents: ContentType::parse(&contents_str),
        container: ContainerType::parse(&container_str),
        purpose: Purpose::parse(&purpose_str),
        source: Source::parse(&source_str),
        units: get_string_item_frame(frame, "_units.code"),
        dimensions: None, // TODO: Parse from _type.dimension if present
    }
}

/// Extract value constraints from frame
fn extract_constraints(frame: &CifFrame) -> ValueConstraints {
    ValueConstraints {
        enumeration: extract_enumeration(frame),
        range: extract_range(frame),
        mandatory: is_mandatory(frame),
    }
}

/// Extract enumeration constraint from frame
fn extract_enumeration(frame: &CifFrame) -> Option<EnumerationConstraint> {
    let mut values = Vec::new();

    // Check for _enumeration.set (may be a list in CIF 2.0)
    if let Some(value) = frame.get_item("_enumeration.set") {
        match &value.kind {
            CifValueKind::List(items) => {
                for item in items {
                    if let Some(s) = item.as_string() {
                        values.push(s.to_string());
                    }
                }
            }
            CifValueKind::Text(s) => {
                values.push(s.clone());
            }
            _ => {}
        }
    }

    // Also check for loop with _enumeration_set.state or similar
    for loop_ in &frame.loops {
        if loop_.tags.iter().any(|t| {
            t.eq_ignore_ascii_case("_enumeration_set.state")
                || t.eq_ignore_ascii_case("_enumeration.set")
        }) {
            if let Some(col_idx) = loop_.tags.iter().position(|t| {
                t.eq_ignore_ascii_case("_enumeration_set.state")
                    || t.eq_ignore_ascii_case("_enumeration.set")
            }) {
                for row in 0..loop_.len() {
                    if let Some(value) = loop_.get(row, col_idx) {
                        if let Some(s) = value.as_string() {
                            values.push(s.to_string());
                        }
                    }
                }
            }
        }
    }

    if values.is_empty() {
        None
    } else {
        Some(EnumerationConstraint {
            values,
            case_sensitive: false, // CIF is case-insensitive by default
        })
    }
}

/// Extract range constraint from frame
fn extract_range(frame: &CifFrame) -> Option<RangeConstraint> {
    let range_str = get_string_item_frame(frame, "_enumeration.range")?;
    RangeConstraint::parse(&range_str)
}

/// Check if item is mandatory
fn is_mandatory(frame: &CifFrame) -> bool {
    get_string_item_frame(frame, "_definition.mandatory_code")
        .map(|s| s.eq_ignore_ascii_case("yes"))
        .unwrap_or(false)
}

/// Extract link information from frame
fn extract_links(frame: &CifFrame) -> ItemLinks {
    ItemLinks {
        linked_item: get_string_item_frame(frame, "_name.linked_item_id"),
    }
}

/// Populate category.item_names based on loaded items
fn populate_category_items(dict: &mut Dictionary) {
    // Collect items by category
    let mut category_items: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for (name, item) in &dict.items {
        category_items
            .entry(item.category.to_lowercase())
            .or_default()
            .push(name.clone());
    }

    // Update categories
    for (cat_name, item_names) in category_items {
        if let Some(category) = dict.categories.get_mut(&cat_name) {
            category.item_names = item_names;
        }
    }
}

// Helper functions

/// Get a string item from a block
fn get_string_item(block: &CifBlock, name: &str) -> Option<String> {
    block
        .get_item(name)
        .and_then(|v| v.as_string())
        .map(|s| s.to_string())
}

/// Get a string item from a frame
fn get_string_item_frame(frame: &CifFrame, name: &str) -> Option<String> {
    frame
        .get_item(name)
        .and_then(|v| v.as_string())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_simple_dictionary() {
        let cif_content = r#"
#\#CIF_2.0
data_TEST_DICT
    _dictionary.title             TEST_DICT
    _dictionary.version           1.0.0

save_TEST_CATEGORY
    _definition.id                TEST_CATEGORY
    _definition.scope             Category
    _definition.class             Set
    _name.category_id             TEST_DICT
    _name.object_id               TEST_CATEGORY
save_

save_test_category.item_a
    _definition.id                '_test_category.item_a'
    _name.category_id             test_category
    _name.object_id               item_a
    _type.purpose                 Describe
    _type.container               Single
    _type.contents                Text
    _description.text
;
    A test item.
;
save_

save_test_category.item_b
    _definition.id                '_test_category.item_b'
    _alias.definition_id          '_test_category_item_b'
    _name.category_id             test_category
    _name.object_id               item_b
    _type.purpose                 Number
    _type.container               Single
    _type.contents                Real
    _enumeration.range            0.0:100.0
save_
"#;

        let doc = CifDocument::parse(cif_content).expect("Failed to parse CIF");
        let dict = load_dictionary(&doc).expect("Failed to load dictionary");

        // Check metadata
        assert_eq!(dict.metadata.title, Some("TEST_DICT".to_string()));
        assert_eq!(dict.metadata.version, Some("1.0.0".to_string()));

        // Check category
        assert!(dict.categories.contains_key("test_category"));
        let cat = dict.categories.get("test_category").unwrap();
        assert_eq!(cat.class, CategoryClass::Set);

        // Check items
        assert!(dict.items.contains_key("_test_category.item_a"));
        assert!(dict.items.contains_key("_test_category.item_b"));

        let item_a = dict.items.get("_test_category.item_a").unwrap();
        assert_eq!(item_a.type_info.contents, ContentType::Text);

        let item_b = dict.items.get("_test_category.item_b").unwrap();
        assert_eq!(item_b.type_info.contents, ContentType::Real);
        assert!(item_b.constraints.range.is_some());

        // Check alias
        assert!(dict.aliases.contains_key("_test_category_item_b"));
        assert_eq!(
            dict.resolve_name("_test_category_item_b"),
            "_test_category.item_b"
        );
    }

    #[test]
    fn test_range_extraction() {
        // Test range parsing via RangeConstraint::parse
        let range = RangeConstraint::parse("0.0:100.0").unwrap();
        assert_eq!(range.min, Some(0.0));
        assert_eq!(range.max, Some(100.0));

        let range = RangeConstraint::parse("0.0:").unwrap();
        assert_eq!(range.min, Some(0.0));
        assert_eq!(range.max, None);
    }
}
