// Unit tests for CIF grammar productions (PEST rules)
// Tests that individual grammar rules parse correctly (version-agnostic)
// Reference: https://www.iucr.org/resources/cif/spec/version1.1/cifsyntax

use cif_parser::{CIFParser, Rule};
use pest::Parser;

// ===== HELPER MACRO =====
// Macro to test if a string matches a specific rule
macro_rules! assert_parse {
    ($rule:expr, $input:expr) => {
        let result = CIFParser::parse($rule, $input);
        assert!(
            result.is_ok(),
            "Failed to parse '{}' with rule {:?}: {:?}",
            $input,
            $rule,
            result.err()
        );
    };
}

macro_rules! assert_parse_fails {
    ($rule:expr, $input:expr) => {
        let result = CIFParser::parse($rule, $input);
        assert!(
            result.is_err(),
            "Should have failed to parse '{}' with rule {:?}",
            $input,
            $rule
        );
    };
}

// ===== PART 1: CHARACTER SETS (Paragraphs 40-42) =====

#[test]
fn test_ordinary_char() {
    // Paragraph 42: OrdinaryChar
    // Valid ordinary characters
    assert_parse!(Rule::ordinary_char, "!");
    assert_parse!(Rule::ordinary_char, "%");
    assert_parse!(Rule::ordinary_char, "A");
    assert_parse!(Rule::ordinary_char, "z");
    assert_parse!(Rule::ordinary_char, "9");
    assert_parse!(Rule::ordinary_char, "~");

    // Invalid ordinary characters
    assert_parse_fails!(Rule::ordinary_char, "'");
    assert_parse_fails!(Rule::ordinary_char, "\"");
    assert_parse_fails!(Rule::ordinary_char, "#");
    assert_parse_fails!(Rule::ordinary_char, "_");
    assert_parse_fails!(Rule::ordinary_char, " ");
}

#[test]
fn test_nonblank_char() {
    // Paragraph 41: NonBlankChar
    assert_parse!(Rule::nonblank_ch, "!");
    assert_parse!(Rule::nonblank_ch, "~");
    assert_parse!(Rule::nonblank_ch, "#");
    assert_parse!(Rule::nonblank_ch, "_");

    // Space and control characters should fail
    assert_parse_fails!(Rule::nonblank_ch, " ");
    assert_parse_fails!(Rule::nonblank_ch, "\t");
    assert_parse_fails!(Rule::nonblank_ch, "\n");
}

// ===== PART 2: COMMENTS AND WHITESPACE (Paragraphs 43, 45) =====

#[test]
fn test_comment() {
    // Paragraph 45: Comments
    assert_parse!(Rule::comment, "# This is a comment");
    assert_parse!(Rule::comment, "#");
    assert_parse!(Rule::comment, "# Comment with special chars !@#$%");
    assert_parse!(Rule::comment, "#\tComment with tab");
}

#[test]
fn test_whitespace() {
    // Paragraph 43: WhiteSpace
    assert_parse!(Rule::whitespace, " ");
    assert_parse!(Rule::whitespace, "\t");
    assert_parse!(Rule::whitespace, "\n");
    assert_parse!(Rule::whitespace, "\r\n");
    assert_parse!(Rule::whitespace, "   \t\n");
    assert_parse!(Rule::whitespace, " # comment\n");
    assert_parse!(Rule::whitespace, "\n# comment\n\t");
}

// ===== PART 3: RESERVED WORDS (Paragraph 19, 27-30) =====

#[test]
fn test_data_keyword() {
    // Paragraph 27: Data block header
    assert_parse!(Rule::str_data, "data_");
    assert_parse!(Rule::str_data, "DATA_");
    assert_parse!(Rule::str_data, "DaTa_");
}

#[test]
fn test_loop_keyword() {
    // Paragraph 19: loop_ keyword
    let input = "loop_ ";
    assert_parse!(Rule::str_loop, input);
    let input = "LOOP_\n";
    assert_parse!(Rule::str_loop, input);
    let input = "Loop_\t";
    assert_parse!(Rule::str_loop, input);
}

#[test]
fn test_global_keyword() {
    // Paragraph 29: Global block
    let input = "global_ ";
    assert_parse!(Rule::str_global, input);
    let input = "GLOBAL_\n";
    assert_parse!(Rule::str_global, input);
}

#[test]
fn test_save_keyword() {
    // Paragraph 30: Save frame
    assert_parse!(Rule::str_save, "save_");
    assert_parse!(Rule::str_save, "SAVE_");
}

#[test]
fn test_stop_keyword() {
    // Paragraph 19: stop_ keyword (deprecated)
    let input = "stop_ ";
    assert_parse!(Rule::str_stop, input);
    let input = "STOP_\n";
    assert_parse!(Rule::str_stop, input);
}

// ===== PART 4: STRINGS (Paragraphs 20-21) =====

#[test]
fn test_single_quoted_string() {
    // Paragraph 20: Single-quoted strings
    assert_parse!(Rule::singlequoted, "'simple' ");
    assert_parse!(Rule::singlequoted, "'with spaces inside' ");
    assert_parse!(Rule::singlequoted, "'with \"double\" quotes'\t");
    assert_parse!(Rule::singlequoted, "'123.456'#");
    assert_parse!(Rule::singlequoted, "''#empty");

    // Should fail without terminating quote
    assert_parse_fails!(Rule::singlequoted, "'unterminated");
    // Should fail with newline inside
    assert_parse_fails!(Rule::singlequoted, "'line\nbreak' ");
}

#[test]
fn test_double_quoted_string() {
    // Paragraph 20: Double-quoted strings
    assert_parse!(Rule::doublequoted, "\"simple\" ");
    assert_parse!(Rule::doublequoted, "\"with spaces inside\" ");
    assert_parse!(Rule::doublequoted, "\"with 'single' quotes\"\n");
    assert_parse!(Rule::doublequoted, "\"123.456\"#comment");
    assert_parse!(Rule::doublequoted, "\"\"#empty");

    // Should fail without terminating quote
    assert_parse_fails!(Rule::doublequoted, "\"unterminated");
    // Should fail with newline inside
    assert_parse_fails!(Rule::doublequoted, "\"line\nbreak\" ");
}

#[test]
fn test_doubled_quote_escaping() {
    // CIF 1.1: Doubled quotes for escaping (quotes must be followed by whitespace/comment/EOI)
    // Single quotes with doubled-quote escaping
    assert_parse!(Rule::singlequoted, "'O''Brien' ");
    assert_parse!(Rule::singlequoted, "'can''t'\n");
    assert_parse!(Rule::singlequoted, "'it''s'#comment");
    assert_parse!(Rule::singlequoted, "''''\t"); // Four single quotes = escaped quote

    // Double quotes with doubled-quote escaping
    assert_parse!(Rule::doublequoted, "\"He said \"\"Hi\"\"\" ");
    assert_parse!(Rule::doublequoted, "\"with \"\"quotes\"\"\" ");
    assert_parse!(Rule::doublequoted, "\"\"\"\"#empty"); // Four double quotes = escaped quote

    // General quoted_string rule
    assert_parse!(Rule::quoted_string, "'O''Brien' ");
    assert_parse!(Rule::quoted_string, "\"She said \"\"Hi\"\"\" ");
}

#[test]
fn test_text_field() {
    // Paragraph 21: Text fields
    // Note: text_delim = { line_term ~ ";" } - semicolon must be at START of line
    let text1 = "\n;Single line text\n;";
    assert_parse!(Rule::textfield, text1);

    let text2 = "\n;Multi\nline\ntext\n;";
    assert_parse!(Rule::textfield, text2);

    let text3 = "\n;Text with special chars !@#$%\n;";
    assert_parse!(Rule::textfield, text3);
}

#[test]
fn test_unquoted_string() {
    // Paragraph 20: Unquoted strings
    assert_parse!(Rule::unquoted, "simple");
    assert_parse!(Rule::unquoted, "C12H22O11");
    assert_parse!(Rule::unquoted, "123.456");
    assert_parse!(Rule::unquoted, "value_with_underscores");

    // Should not parse if starts with underscore (that's a tag)
    assert_parse_fails!(Rule::unquoted, "_tag");
    // Should not parse if starts with # (that's a comment)
    assert_parse_fails!(Rule::unquoted, "#comment");
    // Should not parse reserved words
    assert_parse_fails!(Rule::unquoted, "data_");
    assert_parse_fails!(Rule::unquoted, "loop_");
}

// ===== PART 5: TAGS (Paragraph 18) =====

#[test]
fn test_tag() {
    // Paragraph 18: Data names (tags)
    assert_parse!(Rule::tag, "_simple_tag");
    assert_parse!(Rule::tag, "_atom_site_label");
    assert_parse!(Rule::tag, "_cell.length_a");
    assert_parse!(Rule::tag, "_A");
    assert_parse!(Rule::tag, "_123");

    // Should fail without underscore
    assert_parse_fails!(Rule::tag, "not_a_tag");
    // Should fail with just underscore
    assert_parse_fails!(Rule::tag, "_");
}

// ===== PART 6: VALUES (Paragraph 15) =====

#[test]
fn test_value() {
    // All types of values
    assert_parse!(Rule::value, "simple");
    assert_parse!(Rule::value, "'quoted value'");
    assert_parse!(Rule::value, "\"double quoted\"");
    assert_parse!(Rule::value, "123.456");
    assert_parse!(Rule::value, "?"); // Unknown value
    assert_parse!(Rule::value, "."); // Not applicable value

    // Text field as value
    let text_value = ";text field\n;";
    assert_parse!(Rule::value, text_value);
}

// ===== PART 7: DATA STRUCTURES =====

#[test]
fn test_datablock_heading() {
    // Paragraph 27: Data block heading
    assert_parse!(Rule::datablockheading, "data_myblock");
    // Empty names allowed at grammar level for CIF 1.1 compatibility
    // Semantic validation rejects empty names in CIF 2.0 (enforced in parser, not grammar)
    assert_parse!(Rule::datablockheading, "data_");
    assert_parse!(Rule::datablockheading, "global_");
    assert_parse!(Rule::datablockheading, "DATA_BLOCK123");
}

#[test]
fn test_data_item() {
    // Paragraph 16: Data items
    let item1 = "_tag value ";
    assert_parse!(Rule::dataitem, item1);

    let item2 = "_cell.length_a   10.523\n";
    assert_parse!(Rule::dataitem, item2);

    let item3 = "_name 'quoted value' ";
    assert_parse!(Rule::dataitem, item3);

    let item4 = "_description\n;Text field\n;\n";
    assert_parse!(Rule::dataitem, item4);
}

#[test]
fn test_loop_structure() {
    // Paragraph 19: Loop structure
    let loop1 = "loop_
_tag1
_tag2
value1 value2
value3 value4
";
    assert_parse!(Rule::loop_block, loop1);

    let loop2 = "loop_
_atom_site.id
_atom_site.type_symbol
_atom_site.x
1 C 0.123
2 N 0.456
";
    assert_parse!(Rule::loop_block, loop2);

    // Loop with quoted values
    let loop3 = "loop_
_tag
'value 1'
'value 2'
";
    assert_parse!(Rule::loop_block, loop3);
}

#[test]
fn test_save_frame() {
    // Paragraph 30: Save frames
    let frame1 = "save_myframe
_item1 value1
_item2 value2
save_
";
    assert_parse!(Rule::frame, frame1);

    let frame2 = "save_frame_with_loop
loop_
_tag1
_tag2
val1 val2
save_
";
    assert_parse!(Rule::frame, frame2);
}

#[test]
fn test_datablock() {
    // Paragraph 27: Data block
    let block1 = "data_test
_item1 value1
_item2 value2
";
    assert_parse!(Rule::datablock, block1);

    let block2 = "data_block_with_loop
loop_
_tag1
_tag2
v1 v2
v3 v4
";
    assert_parse!(Rule::datablock, block2);

    let block3 = "global_
_global_item value
";
    assert_parse!(Rule::datablock, block3);
}

// ===== PART 8: COMPLETE CIF FILES =====

#[test]
fn test_complete_cif() {
    // Paragraph 11: Complete CIF
    let cif1 = "
# Simple CIF file
data_test
_item value
";
    assert_parse!(Rule::file, cif1);

    let cif2 = "
data_block1
_item1 value1

data_block2
_item2 value2
";
    assert_parse!(Rule::file, cif2);

    // CIF with all features (text fields tested separately in integration tests)
    let cif3 = "
# CIF with all features
data_complex
_simple_item  simple_value
_quoted_item  'quoted value'
_double_quoted \"double quoted\"

loop_
_loop.tag1
_loop.tag2
value1 value2
value3 value4

save_myframe
_frame_item frame_value
save_

data_second_block
_another_item another_value
";
    assert_parse!(Rule::file, cif3);
}

// ===== EDGE CASES AND ERROR CONDITIONS =====

#[test]
fn test_edge_cases() {
    // Empty loop (malformed but sometimes occurs)
    let empty_loop = "loop_
_tag1
_tag2
data_"; // Ends with keyword
    assert_parse!(Rule::loop_block, empty_loop);

    // Special values
    assert_parse!(Rule::value, "?"); // Unknown
    assert_parse!(Rule::value, "."); // Not applicable

    // Case insensitivity of keywords
    assert_parse!(Rule::str_data, "dAtA_");
    assert_parse!(Rule::str_loop, "LoOp_ ");

    // Numbers in various formats
    assert_parse!(Rule::value, "123");
    assert_parse!(Rule::value, "123.456");
    assert_parse!(Rule::value, "-123.456");
    assert_parse!(Rule::value, "1.23e-4");
    assert_parse!(Rule::value, "1.23E+4");
}

#[test]
fn test_whitespace_handling() {
    // Various whitespace combinations
    let item_with_spaces = "_tag    \t  value  \n";
    assert_parse!(Rule::dataitem, item_with_spaces);

    let item_with_comments = "_tag  # inline comment\n  value\n";
    assert_parse!(Rule::dataitem, item_with_comments);
}

#[test]
fn test_quoted_string_edge_cases() {
    // Empty quotes
    assert_parse!(Rule::singlequoted, "'' ");
    assert_parse!(Rule::doublequoted, "\"\" ");

    // Quotes with special characters
    assert_parse!(Rule::singlequoted, "'!@#$%^&*()' ");
    assert_parse!(Rule::doublequoted, "\"!@#$%^&*()\" ");

    // Mixed quotes
    assert_parse!(Rule::singlequoted, "'He said \"hello\"' ");
    assert_parse!(Rule::doublequoted, "\"It's fine\" ");
}
