//! CIF 1.1 version rules implementation.
//!
//! CIF 1.1 is permissive - methods perform transformations, never return violations.

use std::collections::HashMap;

use crate::ast::{CifBlock, CifDocument, CifFrame, CifLoop, CifValue, CifVersion, Span};
use crate::raw::{
    RawBlock, RawDocument, RawFrame, RawListSyntax, RawLoop, RawQuotedString, RawTableSyntax,
    RawTextField, RawTripleQuoted, RawUnquoted, RawValue,
};
use crate::rules::helpers::{extract_quoted_content, parse_unquoted_value};
use crate::rules::{VersionRules, VersionViolation};

/// CIF 1.1 version rules.
///
/// CIF 1.1 is permissive:
/// - Empty block/frame names are allowed
/// - Doubled-quote escaping (`''` and `""`) is supported
/// - CIF 2.0 features (lists, tables, triple-quotes) degrade to literal text
pub struct Cif1Rules;

impl VersionRules for Cif1Rules {
    fn resolve(&self, raw: &RawDocument) -> Result<CifDocument, VersionViolation> {
        let mut doc = CifDocument::new_with_version(CifVersion::V1_1);
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
        // CIF 1.1: Extract content but PRESERVE doubled quotes as-is
        // (Grammar captures doubled quotes in content, and we preserve them)
        let content = extract_quoted_content(&raw.raw_content);
        Ok(CifValue::text(content, raw.span))
    }

    fn resolve_triple_quoted(&self, raw: &RawTripleQuoted) -> Result<CifValue, VersionViolation> {
        // CIF 1.1: TRANSFORMATION - treat as literal text (silent degradation)
        // Triple-quoted strings aren't a CIF 1.1 feature, so return the raw content
        Ok(CifValue::text(raw.raw_content.clone(), raw.span))
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
        // CIF 1.1: TRANSFORMATION - treat as literal text (silent degradation)
        Ok(CifValue::text(raw.raw_text.clone(), raw.span))
    }

    fn resolve_table(&self, raw: &RawTableSyntax) -> Result<CifValue, VersionViolation> {
        // CIF 1.1: TRANSFORMATION - treat as literal text (silent degradation)
        Ok(CifValue::text(raw.raw_text.clone(), raw.span))
    }

    fn validate_block_name(&self, _name: &str, _span: Span) -> Result<(), VersionViolation> {
        // CIF 1.1: Empty block names allowed - always succeeds
        Ok(())
    }

    fn validate_frame_name(&self, _name: &str, _span: Span) -> Result<(), VersionViolation> {
        // CIF 1.1: Empty frame names allowed - always succeeds
        Ok(())
    }

    fn resolve_block(&self, raw: &RawBlock) -> Result<CifBlock, VersionViolation> {
        self.validate_block_name(&raw.name, raw.name_span)?;

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

    fn collect_violations(&self, _raw: &RawDocument) -> Vec<VersionViolation> {
        // CIF 1.1 is permissive - no violations to collect
        vec![]
    }
}
