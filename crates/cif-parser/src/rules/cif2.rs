//! CIF 2.0 version rules implementation.
//!
//! CIF 2.0 is strict - methods perform validation AND transformation,
//! returning violations for invalid constructs.

use std::collections::HashMap;

use crate::ast::{CifBlock, CifDocument, CifFrame, CifLoop, CifValue, CifVersion, Span};
use crate::raw::{
    RawBlock, RawDataItem, RawDocument, RawFrame, RawListSyntax, RawLoop, RawQuotedString,
    RawTableKey, RawTableSyntax, RawTextField, RawTripleQuoted, RawUnquoted, RawValue,
};
use crate::rules::helpers::{
    extract_quoted_content, extract_triple_quoted_content, parse_unquoted_value,
};
use crate::rules::{rule_ids, VersionRules, VersionViolation};

/// CIF 2.0 version rules.
///
/// CIF 2.0 is strict:
/// - Empty block/frame names are NOT allowed
/// - Doubled-quote escaping (`''` and `""`) is NOT allowed
/// - Lists, tables, and triple-quoted strings are fully supported
pub struct Cif2Rules;

impl VersionRules for Cif2Rules {
    fn resolve(&self, raw: &RawDocument) -> Result<CifDocument, VersionViolation> {
        // CIF 2.0: VALIDATION - magic header is required
        if !raw.has_cif2_magic {
            return Err(VersionViolation::new(
                raw.span,
                "CIF 2.0 files must start with the #\\#CIF_2.0 magic header",
                rule_ids::CIF2_MISSING_MAGIC_HEADER,
            )
            .with_suggestion("Add '#\\#CIF_2.0' as the first line of the file"));
        }

        let mut doc = CifDocument::new_with_version(CifVersion::V2_0);
        doc.span = raw.span;

        for raw_block in &raw.blocks {
            let block = self.resolve_block(raw_block)?;
            doc.blocks.push(block);
        }

        Ok(doc)
    }

    fn resolve_value(&self, raw: &RawValue) -> Result<CifValue, VersionViolation> {
        match raw {
            RawValue::QuotedString(q) => self.resolve_quoted(q),
            RawValue::TripleQuotedString(t) => self.resolve_triple_quoted(t),
            RawValue::TextField(t) => self.resolve_text_field(t),
            RawValue::Unquoted(u) => self.resolve_unquoted(u),
            RawValue::ListSyntax(l) => self.resolve_list(l),
            RawValue::TableSyntax(t) => self.resolve_table(t),
        }
    }

    fn resolve_quoted(&self, raw: &RawQuotedString) -> Result<CifValue, VersionViolation> {
        // CIF 2.0: VALIDATION - reject doubled-quote escaping
        if raw.has_doubled_quotes {
            return Err(VersionViolation::new(
                raw.span,
                "Doubled-quote escaping not allowed in CIF 2.0",
                rule_ids::CIF2_NO_DOUBLED_QUOTES,
            )
            .with_suggestion("Use triple-quoted strings: '''...''' or \"\"\"...\"\"\""));
        }

        // CIF 2.0: TRANSFORMATION - extract content
        let content = extract_quoted_content(&raw.raw_content);
        Ok(CifValue::text(content, raw.span))
    }

    fn resolve_triple_quoted(&self, raw: &RawTripleQuoted) -> Result<CifValue, VersionViolation> {
        // CIF 2.0: TRANSFORMATION - extract content from triple quotes
        let content = extract_triple_quoted_content(&raw.raw_content);
        Ok(CifValue::text(content, raw.span))
    }

    fn resolve_text_field(&self, raw: &RawTextField) -> Result<CifValue, VersionViolation> {
        // Text fields are the same in both versions
        Ok(CifValue::text(raw.content.clone(), raw.span))
    }

    fn resolve_unquoted(&self, raw: &RawUnquoted) -> Result<CifValue, VersionViolation> {
        // Unquoted values are the same in both versions
        Ok(parse_unquoted_value(&raw.text, raw.span))
    }

    fn resolve_list(&self, raw: &RawListSyntax) -> Result<CifValue, VersionViolation> {
        // CIF 2.0: TRANSFORMATION - parse as actual list
        let mut values = Vec::new();
        for element in &raw.elements {
            values.push(self.resolve_value(element)?);
        }
        Ok(CifValue::list(values, raw.span))
    }

    fn resolve_table(&self, raw: &RawTableSyntax) -> Result<CifValue, VersionViolation> {
        // CIF 2.0: TRANSFORMATION - parse as actual table
        let mut table = HashMap::new();
        for entry in &raw.entries {
            let key = match &entry.key {
                RawTableKey::Quoted(q) => {
                    // Validate the key doesn't have doubled quotes
                    if q.has_doubled_quotes {
                        return Err(VersionViolation::new(
                            q.span,
                            "Doubled-quote escaping not allowed in CIF 2.0 table keys",
                            rule_ids::CIF2_NO_DOUBLED_QUOTES,
                        )
                        .with_suggestion("Use triple-quoted strings for keys with quotes"));
                    }
                    extract_quoted_content(&q.raw_content)
                }
                RawTableKey::TripleQuoted(t) => extract_triple_quoted_content(&t.raw_content),
            };
            let value = self.resolve_value(&entry.value)?;
            table.insert(key, value);
        }
        Ok(CifValue::table(table, raw.span))
    }

    fn validate_block_name(&self, name: &str, span: Span) -> Result<(), VersionViolation> {
        // CIF 2.0: VALIDATION - empty block names NOT allowed
        if name.is_empty() {
            return Err(VersionViolation::new(
                span,
                "Empty data block name not allowed in CIF 2.0",
                rule_ids::CIF2_NO_EMPTY_BLOCK_NAME,
            )
            .with_suggestion("Add a name after 'data_'"));
        }
        Ok(())
    }

    fn validate_frame_name(&self, name: &str, span: Span) -> Result<(), VersionViolation> {
        // CIF 2.0: VALIDATION - empty frame names NOT allowed
        if name.is_empty() {
            return Err(VersionViolation::new(
                span,
                "Empty save frame name not allowed in CIF 2.0",
                rule_ids::CIF2_NO_EMPTY_FRAME_NAME,
            )
            .with_suggestion("Add a name after 'save_'"));
        }
        Ok(())
    }

    fn resolve_block(&self, raw: &RawBlock) -> Result<CifBlock, VersionViolation> {
        // global_ blocks are allowed even with empty names
        if !raw.is_global {
            self.validate_block_name(&raw.name, raw.name_span)?;
        }

        let mut items = HashMap::new();
        for item in &raw.items {
            let value = self.resolve_value(&item.value)?;
            items.insert(item.tag.clone(), value);
        }

        let mut loops = Vec::new();
        for raw_loop in &raw.loops {
            let loop_ = self.resolve_loop(raw_loop)?;
            loops.push(loop_);
        }

        let mut frames = Vec::new();
        for raw_frame in &raw.frames {
            let frame = self.resolve_frame(raw_frame)?;
            frames.push(frame);
        }

        Ok(CifBlock {
            name: raw.name.clone(),
            items,
            loops,
            frames,
            span: raw.span,
        })
    }

    fn resolve_frame(&self, raw: &RawFrame) -> Result<CifFrame, VersionViolation> {
        self.validate_frame_name(&raw.name, raw.name_span)?;

        let mut items = HashMap::new();
        for item in &raw.items {
            let value = self.resolve_value(&item.value)?;
            items.insert(item.tag.clone(), value);
        }

        let mut loops = Vec::new();
        for raw_loop in &raw.loops {
            let loop_ = self.resolve_loop(raw_loop)?;
            loops.push(loop_);
        }

        Ok(CifFrame {
            name: raw.name.clone(),
            items,
            loops,
            span: raw.span,
        })
    }

    fn resolve_loop(&self, raw: &RawLoop) -> Result<CifLoop, VersionViolation> {
        let tags: Vec<String> = raw.tags.iter().map(|t| t.name.clone()).collect();
        let num_tags = tags.len();

        // Validate: loops must have at least one tag
        if tags.is_empty() {
            return Err(VersionViolation::new(
                raw.span,
                "Loop block has no tags",
                "loop-no-tags",
            ));
        }

        // Resolve all values
        let mut resolved_values = Vec::new();
        for v in &raw.values {
            resolved_values.push(self.resolve_value(v)?);
        }

        // Validate: values must align with tags
        if !resolved_values.is_empty() && resolved_values.len() % num_tags != 0 {
            return Err(VersionViolation::new(
                raw.span,
                format!(
                    "Loop has {} tags but {} values (not divisible)",
                    num_tags,
                    resolved_values.len()
                ),
                "loop-values-misaligned",
            ));
        }

        // Organize into rows
        let mut values: Vec<Vec<CifValue>> = Vec::new();
        if num_tags > 0 {
            for chunk in resolved_values.chunks(num_tags) {
                values.push(chunk.to_vec());
            }
        }

        Ok(CifLoop {
            tags,
            values,
            span: raw.span,
        })
    }

    fn collect_violations(&self, raw: &RawDocument) -> Vec<VersionViolation> {
        let mut violations = Vec::new();

        // Check for missing magic header
        if !raw.has_cif2_magic {
            violations.push(
                VersionViolation::new(
                    raw.span,
                    "CIF 2.0 files must start with the #\\#CIF_2.0 magic header",
                    rule_ids::CIF2_MISSING_MAGIC_HEADER,
                )
                .with_suggestion("Add '#\\#CIF_2.0' as the first line of the file"),
            );
        }

        for block in &raw.blocks {
            // Check block name (skip for global_ blocks)
            if !block.is_global {
                if let Err(v) = self.validate_block_name(&block.name, block.name_span) {
                    violations.push(v);
                }
            }

            // Check items
            collect_item_violations(&block.items, &mut violations);

            // Check loops
            for loop_ in &block.loops {
                collect_loop_violations(loop_, &mut violations);
            }

            // Check frames
            for frame in &block.frames {
                if let Err(v) = self.validate_frame_name(&frame.name, frame.name_span) {
                    violations.push(v);
                }
                collect_item_violations(&frame.items, &mut violations);
                for loop_ in &frame.loops {
                    collect_loop_violations(loop_, &mut violations);
                }
            }
        }

        violations
    }
}

/// Collect violations from data items.
fn collect_item_violations(items: &[RawDataItem], violations: &mut Vec<VersionViolation>) {
    for item in items {
        collect_value_violations(&item.value, violations);
    }
}

/// Collect violations from loop values.
fn collect_loop_violations(loop_: &RawLoop, violations: &mut Vec<VersionViolation>) {
    for value in &loop_.values {
        collect_value_violations(value, violations);
    }
}

/// Recursively collect violations from a value.
fn collect_value_violations(value: &RawValue, violations: &mut Vec<VersionViolation>) {
    match value {
        RawValue::QuotedString(qs) => {
            if qs.has_doubled_quotes {
                violations.push(
                    VersionViolation::new(
                        qs.span,
                        "Doubled-quote escaping not allowed in CIF 2.0",
                        rule_ids::CIF2_NO_DOUBLED_QUOTES,
                    )
                    .with_suggestion("Use triple-quoted strings: '''...'''"),
                );
            }
        }
        RawValue::ListSyntax(list) => {
            for element in &list.elements {
                collect_value_violations(element, violations);
            }
        }
        RawValue::TableSyntax(table) => {
            for entry in &table.entries {
                // Check key
                if let RawTableKey::Quoted(q) = &entry.key {
                    if q.has_doubled_quotes {
                        violations.push(
                            VersionViolation::new(
                                q.span,
                                "Doubled-quote escaping not allowed in CIF 2.0 table keys",
                                rule_ids::CIF2_NO_DOUBLED_QUOTES,
                            )
                            .with_suggestion("Use triple-quoted strings for keys with quotes"),
                        );
                    }
                }
                // Check value
                collect_value_violations(&entry.value, violations);
            }
        }
        // Other value types don't have version-specific violations
        _ => {}
    }
}
