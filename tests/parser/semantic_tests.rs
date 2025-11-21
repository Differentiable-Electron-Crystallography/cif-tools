//! Semantic validation tests for CIF parser
//!
//! Tests version-dependent validation logic (CIF 1.1 vs CIF 2.0).
//! All semantic validation must be gated on `if version == CifVersion::V2_0` checks.
//!
//! Coverage:
//! - Version detection (magic header)
//! - Empty container name validation
//! - CIF 2.0 feature gating (lists, tables, triple-quoted strings)
//! - Reserved character handling

use cif_parser::{CifDocument, CifVersion};

// ========================================================================
// Version Detection Tests
// ========================================================================

#[test]
fn test_cif2_magic_header_detection() {
    let cif = "#\\#CIF_2.0\ndata_test\n_item value\n";
    let doc = CifDocument::parse(cif).unwrap();
    assert_eq!(doc.version, CifVersion::V2_0);
}

#[test]
fn test_cif1_no_magic_header() {
    let cif = "data_test\n_item value\n";
    let doc = CifDocument::parse(cif).unwrap();
    assert_eq!(doc.version, CifVersion::V1_1);
}

#[test]
fn test_cif1_with_comment_not_magic_header() {
    let cif = "# This is a comment\ndata_test\n_item value\n";
    let doc = CifDocument::parse(cif).unwrap();
    assert_eq!(doc.version, CifVersion::V1_1);
}

// ========================================================================
// Empty Container Name Validation
// ========================================================================

#[test]
fn test_empty_datablock_name_allowed_in_cif1() {
    // CIF 1.1: Empty names are allowed (backward compatibility for legacy bug)
    let cif = "data_\n_item value\n";
    let doc = CifDocument::parse(cif);
    assert!(
        doc.is_ok(),
        "CIF 1.1 should allow empty data block names: {:?}",
        doc.err()
    );
    let doc = doc.unwrap();
    assert_eq!(doc.version, CifVersion::V1_1);
    assert_eq!(doc.first_block().unwrap().name, "");
}

#[test]
fn test_empty_datablock_name_rejected_in_cif2() {
    // CIF 2.0: Empty names are rejected (stricter validation)
    let cif = "#\\#CIF_2.0\ndata_\n_item value\n";
    let doc = CifDocument::parse(cif);
    assert!(doc.is_err(), "CIF 2.0 should reject empty data block names");
    let err = doc.unwrap_err();
    let err_str = err.to_string();
    assert!(
        err_str.contains("Empty data block name"),
        "Error message should mention empty name: {}",
        err_str
    );
}

#[test]
fn test_global_block_allowed_in_both_versions() {
    // global_ is a special case - allowed in both versions
    let cif1 = "global_\n_item value\n";
    let doc1 = CifDocument::parse(cif1);
    assert!(doc1.is_ok(), "CIF 1.1 should allow global_ blocks");

    let cif2 = "#\\#CIF_2.0\nglobal_\n_item value\n";
    let doc2 = CifDocument::parse(cif2);
    assert!(doc2.is_ok(), "CIF 2.0 should allow global_ blocks");
}

// ========================================================================
// CIF 2.0 Feature Gating: Lists
// ========================================================================

#[test]
fn test_lists_only_in_cif2() {
    // CIF 2.0: Lists should parse correctly
    let cif2 = "#\\#CIF_2.0\ndata_test\n_coords [1.0 2.0 3.0]\n";
    let doc2 = CifDocument::parse(cif2).unwrap();
    assert_eq!(doc2.version, CifVersion::V2_0);
    let coords = doc2.first_block().unwrap().items.get("_coords").unwrap();
    assert!(
        coords.as_list().is_some(),
        "CIF 2.0 should parse lists correctly"
    );
    assert_eq!(coords.as_list().unwrap().len(), 3);
}

#[test]
fn test_lists_as_text_in_cif1() {
    // CIF 1.1: Brackets must be quoted (treated as text, not parsed as lists)
    // This tests that CIF 1.1 mode doesn't parse quoted brackets as lists
    let cif1 = "data_test\n_item '[1.0 2.0 3.0]'\n";
    let doc1 = CifDocument::parse(cif1).unwrap();
    assert_eq!(doc1.version, CifVersion::V1_1);
    let item = doc1.first_block().unwrap().items.get("_item").unwrap();
    // In CIF 1.1, quoted brackets should be treated as text
    assert_eq!(item.as_string(), Some("[1.0 2.0 3.0]"));
}

// ========================================================================
// CIF 2.0 Feature Gating: Tables
// ========================================================================

#[test]
fn test_tables_only_in_cif2() {
    // CIF 2.0: Tables should parse correctly
    let cif2 = "#\\#CIF_2.0\ndata_test\n_point {'x':1.0 'y':2.0}\n";
    let doc2 = CifDocument::parse(cif2).unwrap();
    assert_eq!(doc2.version, CifVersion::V2_0);
    let point = doc2.first_block().unwrap().items.get("_point").unwrap();
    assert!(
        point.as_table().is_some(),
        "CIF 2.0 should parse tables correctly"
    );
    assert_eq!(point.as_table().unwrap().len(), 2);
}

#[test]
fn test_tables_as_text_in_cif1() {
    // CIF 1.1: Braces must be quoted (treated as text, not parsed as tables)
    let cif1 = "data_test\n_item '{not_a_table}'\n";
    let doc1 = CifDocument::parse(cif1).unwrap();
    assert_eq!(doc1.version, CifVersion::V1_1);
    let item = doc1.first_block().unwrap().items.get("_item").unwrap();
    // In CIF 1.1, quoted braces should be treated as text
    assert_eq!(item.as_string(), Some("{not_a_table}"));
}

// ========================================================================
// CIF 2.0 Feature Gating: Triple-Quoted Strings
// ========================================================================

#[test]
fn test_triple_quoted_only_in_cif2() {
    // CIF 2.0: Triple-quoted strings should work
    let cif2 = "#\\#CIF_2.0\ndata_test\n_text \"\"\"multi\nline\"\"\"\n";
    let doc2 = CifDocument::parse(cif2).unwrap();
    assert_eq!(doc2.version, CifVersion::V2_0);
    let text = doc2.first_block().unwrap().items.get("_text").unwrap();
    assert_eq!(text.as_string(), Some("multi\nline"));
}

#[test]
fn test_triple_quoted_treated_as_text_in_cif1() {
    // CIF 1.1: Triple quotes parsed but processed as text (version guard in parser)
    // The grammar accepts triple-quoted strings, but parser returns them as text values
    let cif1 = "data_test\n_text \"\"\"text\"\"\"\n";
    let doc1 = CifDocument::parse(cif1).unwrap();
    assert_eq!(doc1.version, CifVersion::V1_1);
    let text = doc1.first_block().unwrap().items.get("_text").unwrap();
    // In CIF 1.1, the version guard causes triple-quoted strings to be stored as Text(..)
    // The content is extracted correctly, but it's not treated as a special CIF 2.0 feature
    assert!(text.as_string().is_some(), "Should have a text value");
    // The value should be the content (quotes stripped)
    let text_val = text.as_string().unwrap();
    assert!(
        text_val == "text" || text_val == "\"\"\"text\"\"\"",
        "Expected 'text' or literal triple quotes, got: {:?}",
        text_val
    );
}

// ========================================================================
// Quote Escaping Validation (CIF 1.1 vs CIF 2.0)
// ========================================================================

#[test]
fn test_cif1_doubled_quote_escaping_allowed() {
    // CIF 1.1: Doubled quotes for escaping are supported
    let cif1 = r#"data_test
_name 'O''Brien'
_phrase "He said ""Hi"""
"#;
    let doc = CifDocument::parse(cif1).unwrap();
    assert_eq!(doc.version, CifVersion::V1_1);

    let name = doc.first_block().unwrap().items.get("_name").unwrap();
    let phrase = doc.first_block().unwrap().items.get("_phrase").unwrap();

    // The grammar captures the doubled quotes in the content
    assert!(
        name.as_string().unwrap().contains("''"),
        "CIF 1.1 should preserve doubled quotes in content"
    );
    assert!(
        phrase.as_string().unwrap().contains("\"\""),
        "CIF 1.1 should preserve doubled quotes in content"
    );
}

#[test]
fn test_cif2_doubled_quote_escaping_rejected() {
    // CIF 2.0: Doubled-quote escaping is invalid; use triple quotes instead
    let cif2_single = r#"#\#CIF_2.0
data_test
_name 'O''Brien'
"#;
    let result = CifDocument::parse(cif2_single);
    assert!(
        result.is_err(),
        "CIF 2.0 should reject doubled single quotes: {:?}",
        result
    );
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(
        err_msg.contains("Doubled-quote") || err_msg.contains("not allowed"),
        "Error should mention doubled-quote escaping is not allowed in CIF 2.0"
    );

    let cif2_double = r#"#\#CIF_2.0
data_test
_phrase "He said ""Hi"""
"#;
    let result = CifDocument::parse(cif2_double);
    assert!(
        result.is_err(),
        "CIF 2.0 should reject doubled double quotes: {:?}",
        result
    );
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(
        err_msg.contains("Doubled-quote") || err_msg.contains("not allowed"),
        "Error should mention doubled-quote escaping is not allowed in CIF 2.0"
    );
}

#[test]
fn test_cif2_use_triple_quotes_instead() {
    // CIF 2.0: Use triple-quoted strings for quotes within text
    let cif2 = r####"#\#CIF_2.0
data_test
_name '''O'Brien says "hello"'''
_phrase """She said 'goodbye'"""
"####;
    let doc = CifDocument::parse(cif2).unwrap();
    assert_eq!(doc.version, CifVersion::V2_0);

    let name = doc.first_block().unwrap().items.get("_name").unwrap();
    let phrase = doc.first_block().unwrap().items.get("_phrase").unwrap();

    assert_eq!(name.as_string(), Some("O'Brien says \"hello\""));
    assert_eq!(phrase.as_string(), Some("She said 'goodbye'"));
}

// ========================================================================
// Reserved Characters (Cross-version behavior)
// ========================================================================

#[test]
fn test_reserved_characters_must_be_quoted() {
    // Brackets and braces are reserved in both CIF 1.1 and 2.0
    // They must be quoted to be used as literal text

    // Test that quoted brackets/braces work as text in both versions
    let cif1 = "data_test\n_brackets '[text]'\n_braces '{text}'\n";
    let doc1 = CifDocument::parse(cif1).unwrap();
    assert_eq!(
        doc1.first_block()
            .unwrap()
            .items
            .get("_brackets")
            .unwrap()
            .as_string(),
        Some("[text]")
    );
    assert_eq!(
        doc1.first_block()
            .unwrap()
            .items
            .get("_braces")
            .unwrap()
            .as_string(),
        Some("{text}")
    );

    let cif2 = "#\\#CIF_2.0\ndata_test\n_brackets '[text]'\n_braces '{text}'\n";
    let doc2 = CifDocument::parse(cif2).unwrap();
    // In CIF 2.0, quoted brackets/braces should still be text (not parsed as lists/tables)
    assert_eq!(
        doc2.first_block()
            .unwrap()
            .items
            .get("_brackets")
            .unwrap()
            .as_string(),
        Some("[text]")
    );
    assert_eq!(
        doc2.first_block()
            .unwrap()
            .items
            .get("_braces")
            .unwrap()
            .as_string(),
        Some("{text}")
    );
}
