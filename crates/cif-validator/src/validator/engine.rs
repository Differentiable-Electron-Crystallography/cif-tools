//! Core validation engine implementation.

use std::collections::HashSet;

use cif_parser::{CifBlock, CifDocument, CifLoop, CifValue, CifValueKind};

use crate::dictionary::{
    ContainerType, ContentType, DataItem, Dictionary, EnumerationConstraint, RangeConstraint,
};
use crate::error::{ValidationError, ValidationResult, ValidationWarning, WarningCategory};

/// Validation mode controlling strictness
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ValidationMode {
    /// Strict: All errors are fatal, unknown data names are errors
    #[default]
    Strict,
    /// Lenient: Unknown data names are warnings, some type coercions allowed
    Lenient,
    /// Pedantic: Include stylistic warnings
    Pedantic,
}

/// Main validation engine
pub struct ValidationEngine<'dict> {
    dictionary: &'dict Dictionary,
    mode: ValidationMode,
    result: ValidationResult,
}

impl<'dict> ValidationEngine<'dict> {
    /// Create a new validation engine
    pub fn new(dictionary: &'dict Dictionary, mode: ValidationMode) -> Self {
        Self {
            dictionary,
            mode,
            result: ValidationResult::new(),
        }
    }

    /// Validate a CIF document
    pub fn validate(mut self, doc: &CifDocument) -> ValidationResult {
        for block in &doc.blocks {
            self.validate_block(block);
        }
        self.result
    }

    /// Validate a single data block
    fn validate_block(&mut self, block: &CifBlock) {
        // Validate individual items
        for (name, value) in &block.items {
            self.validate_item(name, value);
        }

        // Validate loops
        for loop_ in &block.loops {
            self.validate_loop(loop_);
        }

        // Validate save frames
        for frame in &block.frames {
            for (name, value) in &frame.items {
                self.validate_item(name, value);
            }
            for loop_ in &frame.loops {
                self.validate_loop(loop_);
            }
        }

        // Check mandatory items
        self.check_mandatory_items(block);
    }

    /// Validate a single item
    fn validate_item(&mut self, name: &str, value: &CifValue) {
        // Look up definition
        let Some(def) = self.dictionary.get_item(name) else {
            // Unknown data name
            match self.mode {
                ValidationMode::Strict => {
                    self.result
                        .add_error(ValidationError::unknown_data_name(name, value.span));
                }
                ValidationMode::Lenient | ValidationMode::Pedantic => {
                    self.result.add_warning(ValidationWarning::new(
                        WarningCategory::UnknownItem,
                        format!("Unknown data name '{}'", name),
                        value.span,
                    ));
                }
            }
            return;
        };

        // Skip special values for type checking
        if value.is_unknown() || value.is_not_applicable() {
            return;
        }

        // Type validation
        self.validate_type(name, value, def);

        // Container validation
        self.validate_container(name, value, def);

        // Constraint validation
        self.validate_constraints(name, value, def);
    }

    /// Validate value type matches definition
    fn validate_type(&mut self, name: &str, value: &CifValue, def: &DataItem) {
        match def.type_info.contents {
            ContentType::Integer | ContentType::Index | ContentType::Count => {
                self.validate_integer(name, value, def);
            }
            ContentType::Real => {
                self.validate_real(name, value, def);
            }
            ContentType::Word | ContentType::Code => {
                self.validate_word(name, value);
            }
            ContentType::Date => {
                self.validate_date(name, value);
            }
            ContentType::DateTime => {
                self.validate_datetime(name, value);
            }
            // Text, Name, Tag, Uri, etc. accept any string
            _ => {}
        }
    }

    /// Validate integer type
    fn validate_integer(&mut self, name: &str, value: &CifValue, def: &DataItem) {
        match &value.kind {
            CifValueKind::Numeric(n) => {
                // Check if it's actually an integer
                if n.fract() != 0.0 {
                    self.result.add_error(
                        ValidationError::type_error(
                            name,
                            "integer",
                            format!("float {}", n),
                            value.span,
                        )
                        .with_definition_span(def.span),
                    );
                }

                // Check Index (must be positive) and Count (must be non-negative)
                match def.type_info.contents {
                    ContentType::Index => {
                        if *n < 1.0 {
                            self.result.add_error(ValidationError::range_error(
                                name,
                                *n,
                                Some(1.0),
                                None,
                                value.span,
                            ));
                        }
                    }
                    ContentType::Count => {
                        if *n < 0.0 {
                            self.result.add_error(ValidationError::range_error(
                                name,
                                *n,
                                Some(0.0),
                                None,
                                value.span,
                            ));
                        }
                    }
                    _ => {}
                }
            }
            CifValueKind::NumericWithUncertainty { value: n, .. } => {
                // Integers shouldn't have uncertainty in strict mode
                if self.mode == ValidationMode::Strict && n.fract() != 0.0 {
                    self.result.add_error(ValidationError::type_error(
                        name,
                        "integer",
                        format!("float {}", n),
                        value.span,
                    ));
                }
            }
            CifValueKind::Text(s) => {
                // Try to parse as integer
                if s.parse::<i64>().is_err() {
                    self.result.add_error(ValidationError::type_error(
                        name,
                        "integer",
                        format!("text '{}'", s),
                        value.span,
                    ));
                }
            }
            _ => {
                self.result.add_error(ValidationError::type_error(
                    name,
                    "integer",
                    "non-numeric value",
                    value.span,
                ));
            }
        }
    }

    /// Validate real number type
    fn validate_real(&mut self, name: &str, value: &CifValue, _def: &DataItem) {
        match &value.kind {
            CifValueKind::Numeric(_) | CifValueKind::NumericWithUncertainty { .. } => {
                // Valid
            }
            CifValueKind::Text(s) => {
                // Try to parse as number
                if s.parse::<f64>().is_err() {
                    self.result.add_error(ValidationError::type_error(
                        name,
                        "real number",
                        format!("text '{}'", s),
                        value.span,
                    ));
                }
            }
            _ => {
                self.result.add_error(ValidationError::type_error(
                    name,
                    "real number",
                    "non-numeric value",
                    value.span,
                ));
            }
        }
    }

    /// Validate word type (single word, no whitespace)
    fn validate_word(&mut self, name: &str, value: &CifValue) {
        if let Some(s) = value.as_string() {
            if s.contains(char::is_whitespace) {
                self.result.add_error(ValidationError::type_error(
                    name,
                    "single word (no whitespace)",
                    format!("text with whitespace '{}'", s),
                    value.span,
                ));
            }
        }
    }

    /// Validate date format (YYYY-MM-DD)
    fn validate_date(&mut self, name: &str, value: &CifValue) {
        if let Some(s) = value.as_string() {
            // Simple date format check: YYYY-MM-DD
            let parts: Vec<&str> = s.split('-').collect();
            if parts.len() != 3 || parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2
            {
                self.result.add_error(ValidationError::type_error(
                    name,
                    "date (YYYY-MM-DD)",
                    format!("'{}'", s),
                    value.span,
                ));
            }
        }
    }

    /// Validate datetime format
    fn validate_datetime(&mut self, name: &str, value: &CifValue) {
        if let Some(s) = value.as_string() {
            // Basic datetime check - should contain date and time parts
            if !s.contains('T') && !s.contains(' ') {
                // Might just be a date
                self.validate_date(name, value);
            }
            // More detailed validation could be added
        }
    }

    /// Validate container type
    fn validate_container(&mut self, name: &str, value: &CifValue, def: &DataItem) {
        match def.type_info.container {
            ContainerType::List | ContainerType::Array => {
                if !value.is_list() {
                    // In lenient mode, single values can be auto-promoted to lists
                    if self.mode == ValidationMode::Strict {
                        self.result.add_error(ValidationError::type_error(
                            name,
                            "list",
                            "single value",
                            value.span,
                        ));
                    }
                }
            }
            ContainerType::Matrix => {
                // Matrix should be a list of lists
                if let Some(outer) = value.as_list() {
                    let is_matrix = outer.iter().all(|inner| inner.is_list());
                    if !is_matrix {
                        self.result.add_error(ValidationError::type_error(
                            name,
                            "matrix (list of lists)",
                            "non-matrix structure",
                            value.span,
                        ));
                    }
                } else {
                    self.result.add_error(ValidationError::type_error(
                        name,
                        "matrix",
                        "non-list value",
                        value.span,
                    ));
                }
            }
            ContainerType::Table => {
                if !value.is_table() {
                    self.result.add_error(ValidationError::type_error(
                        name,
                        "table",
                        "non-table value",
                        value.span,
                    ));
                }
            }
            ContainerType::Single => {
                // Single is the default, no special validation needed
            }
        }
    }

    /// Validate value constraints (enumeration, range)
    fn validate_constraints(&mut self, name: &str, value: &CifValue, def: &DataItem) {
        // Enumeration check
        if let Some(enum_constraint) = &def.constraints.enumeration {
            self.validate_enumeration(name, value, enum_constraint);
        }

        // Range check
        if let Some(range) = &def.constraints.range {
            self.validate_range(name, value, range);
        }
    }

    /// Validate enumeration constraint
    fn validate_enumeration(
        &mut self,
        name: &str,
        value: &CifValue,
        constraint: &EnumerationConstraint,
    ) {
        let value_str = match &value.kind {
            CifValueKind::Text(s) => s.as_str(),
            _ => return, // Non-text values don't match enumeration
        };

        if !constraint.contains(value_str) {
            let mut error =
                ValidationError::enumeration_error(name, value_str, &constraint.values, value.span);

            // Add suggestions for similar values
            let suggestions = suggest_similar(&value_str.to_lowercase(), &constraint.values);
            if !suggestions.is_empty() {
                error = error.with_suggestions(suggestions);
            }

            self.result.add_error(error);
        }
    }

    /// Validate range constraint
    fn validate_range(&mut self, name: &str, value: &CifValue, range: &RangeConstraint) {
        let num = match value.as_numeric() {
            Some(n) => n,
            None => return, // Non-numeric values don't match range
        };

        if !range.contains(num) {
            self.result.add_error(ValidationError::range_error(
                name, num, range.min, range.max, value.span,
            ));
        }
    }

    /// Validate a loop structure
    fn validate_loop(&mut self, loop_: &CifLoop) {
        // Collect categories for each tag
        let mut categories: Vec<Option<String>> = Vec::new();
        let mut unknown_tags = Vec::new();

        for tag in &loop_.tags {
            if let Some(def) = self.dictionary.get_item(tag) {
                categories.push(Some(def.category.clone()));
            } else {
                categories.push(None);
                unknown_tags.push(tag.clone());
            }
        }

        // Report unknown tags
        for tag in &unknown_tags {
            if self.mode == ValidationMode::Strict {
                self.result
                    .add_error(ValidationError::unknown_data_name(tag, loop_.span));
            }
        }

        // Check if all known tags are from the same category
        let known_categories: Vec<&str> = categories.iter().filter_map(|c| c.as_deref()).collect();

        if !known_categories.is_empty() {
            let first_cat = known_categories[0];
            let mixed = known_categories.iter().any(|c| *c != first_cat);

            if mixed
                && (self.mode == ValidationMode::Pedantic || self.mode == ValidationMode::Strict)
            {
                let unique_cats: HashSet<&str> = known_categories.into_iter().collect();
                self.result.add_warning(ValidationWarning::mixed_categories(
                    &unique_cats
                        .into_iter()
                        .map(String::from)
                        .collect::<Vec<_>>(),
                    loop_.span,
                ));
            }
        }

        // Validate each value in the loop
        for (col, tag) in loop_.tags.iter().enumerate() {
            for row in 0..loop_.len() {
                if let Some(value) = loop_.get(row, col) {
                    self.validate_item(tag, value);
                }
            }
        }
    }

    /// Check mandatory items for present categories
    fn check_mandatory_items(&mut self, block: &CifBlock) {
        // Find all categories present in the block
        let mut present_categories: HashSet<String> = HashSet::new();
        let mut present_items: HashSet<String> = HashSet::new();

        // Check individual items
        for name in block.items.keys() {
            present_items.insert(self.dictionary.resolve_name(name));
            if let Some(def) = self.dictionary.get_item(name) {
                present_categories.insert(def.category.clone());
            }
        }

        // Check loop items
        for loop_ in &block.loops {
            for tag in &loop_.tags {
                present_items.insert(self.dictionary.resolve_name(tag));
                if let Some(def) = self.dictionary.get_item(tag) {
                    present_categories.insert(def.category.clone());
                }
            }
        }

        // For each present category, check mandatory items
        for cat_name in &present_categories {
            if let Some(category) = self.dictionary.get_category(cat_name) {
                for item_name in &category.item_names {
                    if let Some(item) = self.dictionary.items.get(item_name) {
                        if item.is_mandatory() {
                            // Check if item is present (including aliases)
                            let is_present = present_items.contains(&item.name.to_lowercase())
                                || item
                                    .aliases
                                    .iter()
                                    .any(|a| present_items.contains(&a.to_lowercase()));

                            if !is_present {
                                self.result.add_error(ValidationError::missing_mandatory(
                                    &item.name, block.span,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Suggest similar strings using simple substring matching
fn suggest_similar(input: &str, candidates: &[String]) -> Vec<String> {
    candidates
        .iter()
        .filter(|c| {
            let c_lower = c.to_lowercase();
            c_lower.contains(input) || input.contains(&c_lower)
        })
        .take(3)
        .map(|c| format!("Did you mean '{}'?", c))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::load_dictionary;
    use crate::error::ErrorCategory;
    use cif_parser::CifDocument;

    fn create_test_dict() -> Dictionary {
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
    _enumeration.range            0.0:
save_

save_cell.setting
    _definition.id                '_cell.setting'
    _name.category_id             cell
    _name.object_id               setting
    _type.contents                Code

    loop_
      _enumeration_set.state
        triclinic
        monoclinic
        orthorhombic
save_
"#;
        let doc = CifDocument::parse(cif_content).unwrap();
        load_dictionary(&doc).unwrap()
    }

    #[test]
    fn test_valid_cif() {
        let dict = create_test_dict();
        let cif = CifDocument::parse(
            r#"
data_test
_cell.length_a 10.5
_cell.setting monoclinic
"#,
        )
        .unwrap();

        let engine = ValidationEngine::new(&dict, ValidationMode::Strict);
        let result = engine.validate(&cif);

        assert!(
            result.is_valid,
            "Expected valid, got errors: {:?}",
            result.errors
        );
    }

    #[test]
    fn test_range_error() {
        let dict = create_test_dict();
        let cif = CifDocument::parse(
            r#"
data_test
_cell.length_a -5.0
"#,
        )
        .unwrap();

        let engine = ValidationEngine::new(&dict, ValidationMode::Strict);
        let result = engine.validate(&cif);

        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].category, ErrorCategory::RangeError);
    }

    #[test]
    fn test_enumeration_error() {
        let dict = create_test_dict();
        let cif = CifDocument::parse(
            r#"
data_test
_cell.setting hexagonal
"#,
        )
        .unwrap();

        let engine = ValidationEngine::new(&dict, ValidationMode::Strict);
        let result = engine.validate(&cif);

        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].category, ErrorCategory::EnumerationError);
    }

    #[test]
    fn test_unknown_item_strict() {
        let dict = create_test_dict();
        let cif = CifDocument::parse(
            r#"
data_test
_unknown.item value
"#,
        )
        .unwrap();

        let engine = ValidationEngine::new(&dict, ValidationMode::Strict);
        let result = engine.validate(&cif);

        assert!(!result.is_valid);
        assert_eq!(result.errors[0].category, ErrorCategory::UnknownDataName);
    }

    #[test]
    fn test_unknown_item_lenient() {
        let dict = create_test_dict();
        let cif = CifDocument::parse(
            r#"
data_test
_unknown.item value
"#,
        )
        .unwrap();

        let engine = ValidationEngine::new(&dict, ValidationMode::Lenient);
        let result = engine.validate(&cif);

        assert!(result.is_valid); // Lenient mode: unknown items are warnings
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_type_error() {
        let dict = create_test_dict();
        let cif = CifDocument::parse(
            r#"
data_test
_cell.length_a not_a_number
"#,
        )
        .unwrap();

        let engine = ValidationEngine::new(&dict, ValidationMode::Strict);
        let result = engine.validate(&cif);

        assert!(!result.is_valid);
        assert_eq!(result.errors[0].category, ErrorCategory::TypeError);
    }
}
