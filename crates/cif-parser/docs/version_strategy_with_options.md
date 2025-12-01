/# CIF Parser Version Rules Refactoring Plan

## Overview

Refactor cif-parser to use a principled two-pass architecture with a strategy pattern for CIF 1.1 / CIF 2.0 dialect handling.

## Architecture

```
Input String
    │
    ▼
Pass 1: parse_raw() → RawDocument (version-agnostic, lossless)
    │
    ▼
detect_version() → CifVersion
    │
    ▼
Pass 2: VersionRules.resolve(RawDocument) → CifDocument
    │
    ├─ Cif1Rules.resolve() → CifDocument (CIF 1.1 semantics)
    └─ Cif2Rules.resolve() → CifDocument (CIF 2.0 semantics)

Optional: VersionRules.collect_violations(RawDocument) → Vec<Violation>
```

## Phase 1: Define Raw AST Types

Create `src/raw/` module with lossless intermediate types:

### `src/raw/mod.rs`
```rust
mod value;
mod document;
mod block;

pub use value::*;
pub use document::*;
pub use block::*;
```

### `src/raw/value.rs`
```rust
/// Lossless representation of a CIF value before version resolution
pub enum RawValue {
    /// Quoted string: 'content' or "content"
    QuotedString(RawQuotedString),

    /// Triple-quoted string: '''content''' or """content"""
    TripleQuotedString(RawTripleQuoted),

    /// Text field: ;content;
    TextField(RawTextField),

    /// Unquoted string (could be number, special value, or text)
    Unquoted(RawUnquoted),

    /// List syntax: [value1 value2]
    /// In CIF 1.1, this will resolve to text; in CIF 2.0, to a list
    ListSyntax(RawListSyntax),

    /// Table syntax: {key:value}
    /// In CIF 1.1, this will resolve to text; in CIF 2.0, to a table
    TableSyntax(RawTableSyntax),
}

pub struct RawQuotedString {
    pub raw_content: String,       // Full string including quotes
    pub quote_char: char,          // ' or "
    pub has_doubled_quotes: bool,  // Contains '' or ""
    pub span: Span,
}

pub struct RawTripleQuoted {
    pub raw_content: String,
    pub quote_char: char,  // ' or "
    pub span: Span,
}

pub struct RawTextField {
    pub content: String,
    pub span: Span,
}

pub struct RawUnquoted {
    pub text: String,
    pub span: Span,
}

pub struct RawListSyntax {
    pub raw_text: String,          // Original "[...]" text for CIF 1.1 fallback
    pub elements: Vec<RawValue>,   // Parsed elements for CIF 2.0
    pub span: Span,
}

pub struct RawTableSyntax {
    pub raw_text: String,              // Original "{...}" text for CIF 1.1 fallback
    pub entries: Vec<RawTableEntry>,   // Parsed entries for CIF 2.0
    pub span: Span,
}

pub struct RawTableEntry {
    pub key: RawQuotedString,  // or RawTripleQuoted
    pub value: RawValue,
}
```

### `src/raw/block.rs`
```rust
pub struct RawBlock {
    pub name: String,           // May be empty (valid in CIF 1.1, not CIF 2.0)
    pub name_span: Span,        // For error reporting
    pub items: Vec<RawDataItem>,
    pub loops: Vec<RawLoop>,
    pub frames: Vec<RawFrame>,
    pub span: Span,
}

pub struct RawDataItem {
    pub tag: String,
    pub value: RawValue,
    pub span: Span,
}

pub struct RawLoop {
    pub tags: Vec<String>,
    pub values: Vec<RawValue>,
    pub span: Span,
}

pub struct RawFrame {
    pub name: String,
    pub name_span: Span,
    pub items: Vec<RawDataItem>,
    pub loops: Vec<RawLoop>,
    pub span: Span,
}
```

### `src/raw/document.rs`
```rust
pub struct RawDocument {
    pub blocks: Vec<RawBlock>,
    pub has_cif2_magic: bool,  // Whether #\#CIF_2.0 was present
    pub span: Span,
}
```

## Phase 2: Define VersionRules Trait

The trait methods encapsulate the **full decision** - both validation AND transformation.
They return `Result<T, VersionViolation>` where violations have structured metadata.

Create `src/rules/mod.rs`:

```rust
mod cif1;
mod cif2;
mod violation;

pub use cif1::Cif1Rules;
pub use cif2::Cif2Rules;
pub use violation::VersionViolation;

use crate::ast::{CifDocument, CifValue, CifBlock, CifFrame};
use crate::raw::*;

/// Strategy trait for version-specific parsing rules.
///
/// Each method encapsulates the FULL decision for that construct:
/// - Validation (reject invalid constructs)
/// - Transformation (e.g., unescape doubled quotes in CIF 1.1)
pub trait VersionRules {
    /// Resolve a raw document to a typed CifDocument
    fn resolve(&self, raw: &RawDocument) -> Result<CifDocument, VersionViolation>;

    /// Resolve a raw value to a typed CifValue (dispatches to specific methods)
    fn resolve_value(&self, raw: &RawValue) -> Result<CifValue, VersionViolation>;

    /// Resolve a quoted string.
    /// - CIF 1.1: Unescapes doubled quotes (transformation)
    /// - CIF 2.0: Rejects doubled quotes (validation)
    fn resolve_quoted(&self, raw: &RawQuotedString) -> Result<CifValue, VersionViolation>;

    /// Resolve a triple-quoted string.
    /// - CIF 1.1: Treats as literal text (transformation)
    /// - CIF 2.0: Extracts content (transformation)
    fn resolve_triple_quoted(&self, raw: &RawTripleQuoted) -> Result<CifValue, VersionViolation>;

    /// Resolve list syntax.
    /// - CIF 1.1: Returns as literal text (transformation)
    /// - CIF 2.0: Returns as List value (transformation)
    fn resolve_list(&self, raw: &RawListSyntax) -> Result<CifValue, VersionViolation>;

    /// Resolve table syntax.
    /// - CIF 1.1: Returns as literal text (transformation)
    /// - CIF 2.0: Returns as Table value (transformation)
    fn resolve_table(&self, raw: &RawTableSyntax) -> Result<CifValue, VersionViolation>;

    /// Validate a block name.
    /// - CIF 1.1: Allows empty names
    /// - CIF 2.0: Rejects empty names
    fn validate_block_name(&self, name: &str, span: Span) -> Result<(), VersionViolation>;

    /// Validate a frame name.
    /// - CIF 1.1: Allows empty names
    /// - CIF 2.0: Rejects empty names
    fn validate_frame_name(&self, name: &str, span: Span) -> Result<(), VersionViolation>;

    /// Resolve a complete block (validates name, then resolves contents)
    fn resolve_block(&self, raw: &RawBlock) -> Result<CifBlock, VersionViolation>;

    /// Resolve a complete frame (validates name, then resolves contents)
    fn resolve_frame(&self, raw: &RawFrame) -> Result<CifFrame, VersionViolation>;

    /// Collect all violations without failing (for upgrade guidance).
    /// Walks the entire raw AST and returns all rule violations found.
    fn collect_violations(&self, raw: &RawDocument) -> Vec<VersionViolation>;
}
```

### `src/rules/violation.rs`
```rust
use crate::ast::Span;

/// A violation of version-specific rules.
/// Contains structured metadata for error reporting and upgrade guidance.
#[derive(Debug, Clone)]
pub struct VersionViolation {
    /// Source location of the violation
    pub span: Span,
    /// Human-readable error message
    pub message: String,
    /// Suggested fix (for upgrade guidance)
    pub suggestion: Option<String>,
    /// Machine-readable rule identifier (e.g., "cif2-no-doubled-quotes")
    pub rule_id: &'static str,
}

impl VersionViolation {
    pub fn new(span: Span, message: impl Into<String>, rule_id: &'static str) -> Self {
        Self {
            span,
            message: message.into(),
            suggestion: None,
            rule_id,
        }
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

// Rule IDs for machine-readable identification
pub mod rule_ids {
    pub const CIF2_NO_DOUBLED_QUOTES: &str = "cif2-no-doubled-quotes";
    pub const CIF2_NO_EMPTY_BLOCK_NAME: &str = "cif2-no-empty-block-name";
    pub const CIF2_NO_EMPTY_FRAME_NAME: &str = "cif2-no-empty-frame-name";
    // Note: Lists/tables/triple-quotes in CIF 1.1 are transformations, not violations
}
```

## Phase 3: Implement Cif1Rules

CIF 1.1 is permissive - methods perform transformations, never return violations.

### `src/rules/cif1.rs`
```rust
pub struct Cif1Rules;

impl VersionRules for Cif1Rules {
    fn resolve_quoted(&self, raw: &RawQuotedString) -> Result<CifValue, VersionViolation> {
        // CIF 1.1: TRANSFORMATION - unescape doubled quotes
        let content = if raw.has_doubled_quotes {
            unescape_doubled_quotes(&raw.raw_content, raw.quote_char)
        } else {
            extract_quoted_content(&raw.raw_content)
        };
        Ok(CifValue::text(content, raw.span))
    }

    fn resolve_list(&self, raw: &RawListSyntax) -> Result<CifValue, VersionViolation> {
        // CIF 1.1: TRANSFORMATION - treat as literal text (silent degradation)
        Ok(CifValue::text(raw.raw_text.clone(), raw.span))
    }

    fn resolve_table(&self, raw: &RawTableSyntax) -> Result<CifValue, VersionViolation> {
        // CIF 1.1: TRANSFORMATION - treat as literal text (silent degradation)
        Ok(CifValue::text(raw.raw_text.clone(), raw.span))
    }

    fn resolve_triple_quoted(&self, raw: &RawTripleQuoted) -> Result<CifValue, VersionViolation> {
        // CIF 1.1: TRANSFORMATION - treat as literal text (silent degradation)
        Ok(CifValue::text(raw.raw_content.clone(), raw.span))
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
        // ... resolve items, loops, frames using self.resolve_value()
    }

    fn collect_violations(&self, _raw: &RawDocument) -> Vec<VersionViolation> {
        // CIF 1.1 is permissive - no violations to collect
        vec![]
    }
}
```

## Phase 4: Implement Cif2Rules

CIF 2.0 is strict - methods perform validation AND transformation, returning violations for invalid constructs.

### `src/rules/cif2.rs`
```rust
use super::violation::{VersionViolation, rule_ids};

pub struct Cif2Rules;

impl VersionRules for Cif2Rules {
    fn resolve_quoted(&self, raw: &RawQuotedString) -> Result<CifValue, VersionViolation> {
        // CIF 2.0: VALIDATION - reject doubled-quote escaping
        if raw.has_doubled_quotes {
            return Err(VersionViolation::new(
                raw.span,
                "Doubled-quote escaping not allowed in CIF 2.0",
                rule_ids::CIF2_NO_DOUBLED_QUOTES,
            ).with_suggestion("Use triple-quoted strings: '''...''' or \"\"\"...\"\"\""));
        }
        // CIF 2.0: TRANSFORMATION - extract content
        let content = extract_quoted_content(&raw.raw_content);
        Ok(CifValue::text(content, raw.span))
    }

    fn resolve_list(&self, raw: &RawListSyntax) -> Result<CifValue, VersionViolation> {
        // CIF 2.0: TRANSFORMATION - parse as actual list
        let values: Result<Vec<_>, _> = raw.elements
            .iter()
            .map(|v| self.resolve_value(v))
            .collect();
        Ok(CifValue::list(values?, raw.span))
    }

    fn resolve_table(&self, raw: &RawTableSyntax) -> Result<CifValue, VersionViolation> {
        // CIF 2.0: TRANSFORMATION - parse as actual table
        let mut table = HashMap::new();
        for entry in &raw.entries {
            let key = extract_quoted_content(&entry.key.raw_content);
            let value = self.resolve_value(&entry.value)?;
            table.insert(key, value);
        }
        Ok(CifValue::table(table, raw.span))
    }

    fn resolve_triple_quoted(&self, raw: &RawTripleQuoted) -> Result<CifValue, VersionViolation> {
        // CIF 2.0: TRANSFORMATION - extract content from triple quotes
        let content = &raw.raw_content[3..raw.raw_content.len()-3];
        Ok(CifValue::text(content.to_string(), raw.span))
    }

    fn validate_block_name(&self, name: &str, span: Span) -> Result<(), VersionViolation> {
        // CIF 2.0: VALIDATION - empty block names NOT allowed
        if name.is_empty() {
            return Err(VersionViolation::new(
                span,
                "Empty data block name not allowed in CIF 2.0",
                rule_ids::CIF2_NO_EMPTY_BLOCK_NAME,
            ).with_suggestion("Add a name after 'data_'"));
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
            ).with_suggestion("Add a name after 'save_'"));
        }
        Ok(())
    }

    fn resolve_block(&self, raw: &RawBlock) -> Result<CifBlock, VersionViolation> {
        self.validate_block_name(&raw.name, raw.name_span)?;
        // ... resolve items, loops, frames using self.resolve_value()
    }

    fn collect_violations(&self, raw: &RawDocument) -> Vec<VersionViolation> {
        // Walk entire raw AST, collecting all violations without short-circuiting
        let mut violations = vec![];

        for block in &raw.blocks {
            // Check block name
            if let Err(v) = self.validate_block_name(&block.name, block.name_span) {
                violations.push(v);
            }

            // Check frame names
            for frame in &block.frames {
                if let Err(v) = self.validate_frame_name(&frame.name, frame.name_span) {
                    violations.push(v);
                }
            }

            // Recursively check all values for doubled-quote violations
            collect_value_violations(&block.items, &block.loops, &mut violations);
        }

        violations
    }
}

fn collect_value_violations(
    items: &[RawDataItem],
    loops: &[RawLoop],
    violations: &mut Vec<VersionViolation>,
) {
    for item in items {
        if let RawValue::QuotedString(qs) = &item.value {
            if qs.has_doubled_quotes {
                violations.push(VersionViolation::new(
                    qs.span,
                    "Doubled-quote escaping not allowed in CIF 2.0",
                    rule_ids::CIF2_NO_DOUBLED_QUOTES,
                ).with_suggestion("Use triple-quoted strings: '''...'''"));
            }
        }
        // ... recurse into lists/tables if needed
    }
    // ... similar for loops
}
```

## Phase 5: Update Entry Point with Options Pattern

### `src/parser/options.rs` (NEW)
```rust
use crate::ast::CifDocument;
use crate::rules::VersionViolation;

/// Options for parsing CIF documents
#[derive(Debug, Clone, Default)]
pub struct ParseOptions {
    /// Collect upgrade guidance (what would make CIF 1.1 valid CIF 2.0)
    pub upgrade_guidance: bool,
}

impl ParseOptions {
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable upgrade guidance collection
    pub fn upgrade_guidance(mut self, enabled: bool) -> Self {
        self.upgrade_guidance = enabled;
        self
    }
}

/// Result of parsing with options
pub struct ParseResult {
    pub document: CifDocument,
    /// Upgrade issues (empty unless upgrade_guidance was enabled AND file is CIF 1.1)
    pub upgrade_issues: Vec<VersionViolation>,
}
```

### `src/ast/document.rs` (Updated API)
```rust
impl CifDocument {
    /// Parse a CIF document from a string (auto-detects version)
    /// Simple case stays simple - no options needed
    pub fn parse(input: &str) -> Result<Self, CifError> {
        crate::parser::document::parse_file(input)
    }

    /// Parse with options (builder pattern)
    /// Example:
    /// ```
    /// let result = Document::parse_with_options(input,
    ///     ParseOptions::default().upgrade_guidance(true)
    /// )?;
    /// // result.document: CifDocument
    /// // result.upgrade_issues: Vec<Violation>
    /// ```
    pub fn parse_with_options(input: &str, options: ParseOptions) -> Result<ParseResult, CifError> {
        crate::parser::document::parse_file_with_options(input, options)
    }
}
```

### `src/parser/document.rs` (Internal Implementation)
```rust
pub fn parse_file(input: &str) -> Result<CifDocument, CifError> {
    let result = parse_file_with_options(input, ParseOptions::default())?;
    Ok(result.document)
}

pub fn parse_file_with_options(input: &str, options: ParseOptions) -> Result<ParseResult, CifError> {
    // Pass 1: Parse to raw AST (version-agnostic)
    let raw_doc = parse_raw(input)?;

    // Detect version from magic comment
    let version = if raw_doc.has_cif2_magic {
        CifVersion::V2_0
    } else {
        CifVersion::V1_1
    };

    // Pass 2: Resolve with version rules
    let rules: &dyn VersionRules = match version {
        CifVersion::V1_1 => &Cif1Rules,
        CifVersion::V2_0 => &Cif2Rules,
    };

    let mut document = rules.resolve(&raw_doc)?;
    document.version = version;

    // Collect upgrade issues if requested AND file is CIF 1.1
    let upgrade_issues = if options.upgrade_guidance && version == CifVersion::V1_1 {
        Cif2Rules.collect_violations(&raw_doc)
    } else {
        vec![]
    };

    Ok(ParseResult { document, upgrade_issues })
}
```

## Phase 6: Refactor Raw Parsing

Update `src/parser/value.rs` to produce `RawValue` instead of `CifValue`:

```rust
pub fn parse_value_raw(pair: Pair<Rule>) -> Result<RawValue, CifError> {
    match pair.as_rule() {
        Rule::quoted_string | Rule::singlequoted | Rule::doublequoted => {
            parse_quoted_string_raw(pair)
        }
        Rule::triple_quoted_string => {
            parse_triple_quoted_raw(pair)
        }
        Rule::list => {
            parse_list_syntax_raw(pair)
        }
        Rule::table => {
            parse_table_syntax_raw(pair)
        }
        // ... etc
    }
}

fn parse_quoted_string_raw(pair: Pair<Rule>) -> Result<RawValue, CifError> {
    let raw = pair.as_str();
    let quote_char = raw.chars().next().unwrap_or('\'');
    let has_doubled = raw.contains("''") || raw.contains("\"\"");

    Ok(RawValue::QuotedString(RawQuotedString {
        raw_content: raw.to_string(),
        quote_char,
        has_doubled_quotes: has_doubled,
        span: extract_span(&pair),
    }))
}
```

## Files to Modify

| File | Action |
|------|--------|
| `src/lib.rs` | Add `mod raw;` and `mod rules;`, export new types |
| `src/raw/mod.rs` | **NEW** - Raw AST module |
| `src/raw/value.rs` | **NEW** - Raw value types |
| `src/raw/block.rs` | **NEW** - Raw block/frame types |
| `src/raw/document.rs` | **NEW** - Raw document type |
| `src/rules/mod.rs` | **NEW** - VersionRules trait |
| `src/rules/cif1.rs` | **NEW** - CIF 1.1 implementation |
| `src/rules/cif2.rs` | **NEW** - CIF 2.0 implementation |
| `src/rules/violation.rs` | **NEW** - Violation types |
| `src/parser/mod.rs` | Add `mod options;` |
| `src/parser/options.rs` | **NEW** - ParseOptions and ParseResult |
| `src/parser/value.rs` | Refactor to produce RawValue |
| `src/parser/block.rs` | Refactor to produce RawBlock |
| `src/parser/document.rs` | Add two-pass entry point with options |
| `src/parser/loop_parser.rs` | Refactor to produce RawLoop |
| `src/ast/document.rs` | Add `parse_with_options()` method |

## Implementation Order

1. Create `raw/` module with all raw types (`RawValue`, `RawBlock`, `RawDocument`, etc.)
2. Create `rules/` module with `VersionRules` trait and `Violation` types
3. Create `parser/options.rs` with `ParseOptions` and `ParseResult`
4. Implement `Cif1Rules` (permissive: allows empty names, doubled quotes, degrades CIF 2.0 features to text)
5. Implement `Cif2Rules` (strict: rejects empty names, doubled quotes; parses lists/tables)
6. Refactor `parser/value.rs` to produce `RawValue` (version-agnostic)
7. Refactor `parser/block.rs` to produce `RawBlock` (version-agnostic)
8. Refactor `parser/loop_parser.rs` to produce `RawLoop` (version-agnostic)
9. Update `parser/document.rs` with two-pass entry point and options support
10. Add `parse_with_options()` to `CifDocument` in `ast/document.rs`
11. Update public exports in `lib.rs` (`ParseOptions`, `ParseResult`, `Violation`)
12. Update/add tests for both CIF versions and upgrade guidance

## Version-Specific Rules Summary

| Rule | CIF 1.1 | CIF 2.0 |
|------|---------|---------|
| Empty block name | ✅ Allowed | ❌ Error |
| Empty frame name | ✅ Allowed | ❌ Error |
| Lists `[...]` | → Text | → List |
| Tables `{...}` | → Text | → Table |
| Triple quotes | → Text | → Text (parsed) |
| Doubled quotes `''` | ✅ Unescape | ❌ Error |
