//! Dictionary validation using dREL reference checking.
//!
//! This module validates that a dictionary is internally consistent by
//! checking that all dREL methods only reference items that exist in
//! the dictionary.

use cif_parser::Span;
use drel_parser::{extract_references, parse, ReferenceKind};

use super::types::Dictionary;
use crate::error::DictionaryError;

/// Convert a drel_parser::Span to cif_parser::Span
fn convert_span(drel_span: drel_parser::Span) -> Span {
    Span::new(
        drel_span.start_line,
        drel_span.start_col,
        drel_span.end_line,
        drel_span.end_col,
    )
}

/// Validate a dictionary's internal consistency.
///
/// This function checks:
/// - All dREL methods parse successfully
/// - All items referenced in dREL methods exist in the dictionary
///
/// # Arguments
/// * `dict` - The dictionary to validate
///
/// # Returns
/// A vector of errors found. Empty if the dictionary is valid.
///
/// # Example
/// ```ignore
/// use cif_validator::dictionary::{load_dictionary, validate_dictionary};
///
/// let dict = load_dictionary(&doc)?;
/// let errors = validate_dictionary(&dict);
/// if !errors.is_empty() {
///     for error in errors {
///         eprintln!("{}", error);
///     }
/// }
/// ```
pub fn validate_dictionary(dict: &Dictionary) -> Vec<DictionaryError> {
    let mut errors = Vec::new();

    for item in dict.items.values() {
        if let Some(drel_source) = &item.drel_method {
            // Try to parse the dREL method
            match parse(drel_source) {
                Ok(stmts) => {
                    // Extract all references from the parsed dREL
                    let refs = extract_references(&stmts);

                    for ref_ in refs {
                        // Only check data name references (not local variables)
                        if ref_.kind == ReferenceKind::DataName {
                            let ref_name = ref_.full_name();

                            // Check if the referenced item exists in the dictionary
                            if !dict.has_item(&ref_name) {
                                errors.push(DictionaryError::MissingDrelReference {
                                    item: item.name.clone(),
                                    referenced: ref_name,
                                    span: convert_span(ref_.span),
                                });
                            }
                        }

                        // Check category references (from Loop statements)
                        if ref_.kind == ReferenceKind::Category {
                            let cat_name = &ref_.category;
                            if !dict.categories.contains_key(&cat_name.to_lowercase()) {
                                // Category might be referenced indirectly, check if any
                                // items have this category
                                let category_exists = dict
                                    .items
                                    .values()
                                    .any(|i| i.category.eq_ignore_ascii_case(cat_name));

                                if !category_exists {
                                    errors.push(DictionaryError::MissingDrelReference {
                                        item: item.name.clone(),
                                        referenced: format!("category '{}'", cat_name),
                                        span: convert_span(ref_.span),
                                    });
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    errors.push(DictionaryError::InvalidDrel {
                        item: item.name.clone(),
                        message: e.to_string(),
                        span: item.span,
                    });
                }
            }
        }
    }

    errors
}

/// Check if a dictionary has any dREL methods
#[allow(dead_code)]
pub fn has_drel_methods(dict: &Dictionary) -> bool {
    dict.items.values().any(|item| item.drel_method.is_some())
}

/// Get all items with dREL methods
#[allow(dead_code)]
pub fn items_with_drel(dict: &Dictionary) -> Vec<&str> {
    dict.items
        .values()
        .filter(|item| item.drel_method.is_some())
        .map(|item| item.name.as_str())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::load_dictionary;
    use cif_parser::CifDocument;

    #[test]
    fn test_validate_valid_dictionary() {
        // A dictionary with a dREL method referencing existing items
        let cif_content = r#"
#\#CIF_2.0
data_TEST_DICT
    _dictionary.title             TEST_DICT

save_cell
    _definition.id                CELL
    _definition.scope             Category
    _definition.class             Set
save_

save_cell.length_a
    _definition.id                '_cell.length_a'
    _name.category_id             cell
    _name.object_id               length_a
    _type.contents                Real
save_

save_cell.length_b
    _definition.id                '_cell.length_b'
    _name.category_id             cell
    _name.object_id               length_b
    _type.contents                Real
save_

save_cell.area_ab
    _definition.id                '_cell.area_ab'
    _name.category_id             cell
    _name.object_id               area_ab
    _type.contents                Real
    _method.expression
;
    _cell.area_ab = _cell.length_a * _cell.length_b
;
save_
"#;

        let doc = CifDocument::parse(cif_content).expect("Failed to parse CIF");
        let dict = load_dictionary(&doc).expect("Failed to load dictionary");

        let errors = validate_dictionary(&dict);
        assert!(errors.is_empty(), "Expected no errors, got: {:?}", errors);
    }

    #[test]
    fn test_validate_missing_reference() {
        // A dictionary with a dREL method referencing a non-existent item
        let cif_content = r#"
#\#CIF_2.0
data_TEST_DICT
    _dictionary.title             TEST_DICT

save_cell
    _definition.id                CELL
    _definition.scope             Category
    _definition.class             Set
save_

save_cell.length_a
    _definition.id                '_cell.length_a'
    _name.category_id             cell
    _name.object_id               length_a
    _type.contents                Real
save_

save_cell.area_ab
    _definition.id                '_cell.area_ab'
    _name.category_id             cell
    _name.object_id               area_ab
    _type.contents                Real
    _method.expression
;
    _cell.area_ab = _cell.length_a * _cell.length_b
;
save_
"#;

        let doc = CifDocument::parse(cif_content).expect("Failed to parse CIF");
        let dict = load_dictionary(&doc).expect("Failed to load dictionary");

        let errors = validate_dictionary(&dict);
        assert_eq!(errors.len(), 1, "Expected one error, got: {:?}", errors);

        match &errors[0] {
            DictionaryError::MissingDrelReference {
                item, referenced, ..
            } => {
                assert_eq!(item, "_cell.area_ab");
                assert_eq!(referenced, "_cell.length_b");
            }
            _ => panic!("Expected MissingDrelReference error"),
        }
    }

    #[test]
    fn test_has_drel_methods() {
        let cif_content = r#"
#\#CIF_2.0
data_TEST_DICT

save_cell.length_a
    _definition.id                '_cell.length_a'
    _type.contents                Real
save_

save_cell.area
    _definition.id                '_cell.area'
    _type.contents                Real
    _method.expression            '_cell.area = 1.0'
save_
"#;

        let doc = CifDocument::parse(cif_content).expect("Failed to parse CIF");
        let dict = load_dictionary(&doc).expect("Failed to load dictionary");

        assert!(has_drel_methods(&dict));

        let items = items_with_drel(&dict);
        assert_eq!(items.len(), 1);
        assert!(items.contains(&"_cell.area"));
    }
}
