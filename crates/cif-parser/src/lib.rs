//! # CIF Parser Library
//!
//! A comprehensive parser for Crystallographic Information Framework (CIF) files,
//! implementing the CIF 1.1 specification.
//!
//! ## What is CIF?
//!
//! CIF is a standard file format used in crystallography and chemistry to store
//! structured data about crystal structures, molecular information, and related
//! metadata. Files contain data blocks with key-value pairs, loops (tables), and
//! text fields.
//!
//! ## Key Parsing Challenges
//!
//! CIF parsing presents several unique challenges that this library addresses:
//!
//! ### 1. Case-Insensitive Keywords
//! Keywords like `data_`, `loop_`, and `global_` are case-insensitive, so
//! `DATA_BLOCK`, `data_block`, and `DaTa_BlOcK` are all valid.
//!
//! ### 2. Complex Value Types
//! Values can be:
//! - Unquoted strings: `value`
//! - Quoted strings: `'value'` or `"value"`
//! - Multi-line text fields: `;text\nfield;`
//! - Special values: `?` (unknown) and `.` (not applicable)
//! - Numbers: `123.45`, `-1.5e-3`
//!
//! ### 3. Loop State Management
//! Loops can be interrupted by other elements, requiring careful state management:
//! ```text
//! loop_
//! _atom.id _atom.type
//! _other_item other_value    # Interrupts the loop!
//! 1 C
//! 2 N
//! ```
//!
//! ### 4. Text Field Parsing
//! Text fields use semicolons at the beginning of lines as delimiters, requiring
//! special handling in the grammar and parsing logic.
//!
//! ## Architecture
//!
//! The parser uses a **two-stage approach**:
//! 1. **Grammar parsing** with PEST PEG parser (defined in `cif.pest`)
//! 2. **AST construction** that builds typed data structures from parse trees
//!
//! This architecture is **decoupled**: AST types don't know how to parse themselves,
//! and parsing logic is separate from data representation. This follows Rust best
//! practices and makes the code easier to test and maintain.
//!
//! ## Why PEST (Not Parser Combinators)?
//!
//! CIF uses PEST rather than parser combinators (like `nom`) because:
//! - CIF has a formal specification that maps naturally to PEG grammars
//! - The grammar file (`cif.pest`) serves as living documentation
//! - Declarative grammars are easier to review and verify against the spec
//! - Grammar changes don't require recompiling Rust code
//!
//! Parser combinators would be appropriate for binary formats or simpler grammars,
//! but PEST is the right tool for CIF's complexity.
//!
//! ## Why Not Pratt Parsing?
//!
//! Pratt parsers handle operator precedence in expression languages (like `a + b * c`).
//! CIF is a **data format**, not an expression language - it has no operators or
//! precedence rules. Pratt parsing doesn't apply to this domain.
//!
//! ## Module Organization
//!
//! - [`ast`] - Abstract Syntax Tree types (data structures)
//! - [`parser`] - Parsing logic (PEST → AST conversion)
//! - [`error`] - Error types
//! - `builder` - Internal state management helpers (not public)
//!
//! ## Examples
//!
//! ### Basic Usage
//! ```
//! use cif_parser::{Document, CifError};
//!
//! let cif_content = r#"
//! data_example
//! _cell_length_a  10.000
//! _cell_length_b  20.000
//! _title 'My Structure'
//! "#;
//!
//! let doc = Document::parse(cif_content)?;
//! let block = doc.first_block().unwrap();
//!
//! assert_eq!(block.name, "example");
//! assert_eq!(block.get_item("_cell_length_a").unwrap().as_numeric(), Some(10.0));
//! # Ok::<(), CifError>(())
//! ```
//!
//! ### Working with Loops
//! ```
//! use cif_parser::Document;
//!
//! let cif_content = r#"
//! data_atoms
//! loop_
//! _atom_site_label
//! _atom_site_type_symbol
//! _atom_site_fract_x
//! C1  C  0.1234
//! N1  N  0.5678
//! O1  O  0.9012
//! "#;
//!
//! let doc = Document::parse(cif_content).unwrap();
//! let block = doc.first_block().unwrap();
//! let loop_ = &block.loops[0];
//!
//! assert_eq!(loop_.len(), 3); // 3 rows
//! assert_eq!(loop_.tags.len(), 3); // 3 columns
//!
//! // Access by row and tag name
//! let atom_type = loop_.get_by_tag(0, "_atom_site_type_symbol").unwrap();
//! assert_eq!(atom_type.as_string().unwrap(), "C");
//! ```

use pest_derive::Parser;
use std::path::Path;

// ===== Core Modules =====

pub mod ast;
pub mod error;
pub mod parser;

mod builder; // Internal only

// ===== PEST Parser =====

#[derive(Parser)]
#[grammar = "cif.pest"]
pub struct CIFParser;

// ===== Re-exports =====

// AST types
pub use ast::{CifBlock, CifDocument, CifFrame, CifLoop, CifValue, CifVersion};

// Error types
pub use error::CifError;

// Convenient type aliases (matching old API)
pub use CifBlock as Block;
pub use CifDocument as Document;
pub use CifFrame as Frame;
pub use CifLoop as Loop;
pub use CifValue as Value;
pub use CifVersion as Version;

// ===== Public Convenience Functions =====

/// Parse a CIF file from a path
///
/// # Examples
/// ```no_run
/// use cif_parser::parse_file;
///
/// let doc = parse_file("structure.cif").unwrap();
/// ```
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<CifDocument, CifError> {
    CifDocument::from_file(path)
}

/// Parse a CIF string
///
/// # Examples
/// ```
/// use cif_parser::parse_string;
///
/// let doc = parse_string("data_test\n_item value\n").unwrap();
/// ```
pub fn parse_string(input: &str) -> Result<CifDocument, CifError> {
    CifDocument::parse(input)
}

// ===== Re-export for internal use =====
pub use pest::iterators::Pair;
// Note: Rule enum is automatically public via the #[derive(Parser)] macro

// ===== Conditional Compilation Modules =====

// WASM bindings module (conditionally compiled)
#[cfg(target_arch = "wasm32")]
pub mod wasm;

// Python bindings module (conditionally compiled)
#[cfg(feature = "python")]
pub mod python;

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_cif() {
        let cif_content = r#"
data_test
_tag1 value1
_tag2 'quoted value'
_tag3 123.45
"#;

        let doc = CifDocument::parse(cif_content).unwrap();
        assert_eq!(doc.blocks.len(), 1);

        let block = &doc.blocks[0];
        assert_eq!(block.name, "test");
        assert_eq!(block.items.len(), 3);

        assert_eq!(
            block.get_item("_tag1").unwrap().as_string().unwrap(),
            "value1"
        );
        assert_eq!(
            block.get_item("_tag2").unwrap().as_string().unwrap(),
            "quoted value"
        );
        assert_eq!(
            block.get_item("_tag3").unwrap().as_numeric().unwrap(),
            123.45
        );
    }

    #[test]
    fn test_parse_loop() {
        let cif_content = r#"
data_test
loop_
_atom.id
_atom.type
_atom.x
1 C 1.0
2 N 2.0
3 O 3.0
"#;

        let doc = CifDocument::parse(cif_content).unwrap();
        let block = &doc.blocks[0];
        assert_eq!(block.loops.len(), 1);

        let loop_ = &block.loops[0];
        assert_eq!(loop_.tags.len(), 3);
        assert_eq!(loop_.len(), 3);

        assert_eq!(
            loop_
                .get_by_tag(0, "_atom.type")
                .unwrap()
                .as_string()
                .unwrap(),
            "C"
        );
        assert_eq!(loop_.get(1, 2).unwrap().as_numeric().unwrap(), 2.0);
    }

    #[test]
    fn test_special_values() {
        let cif_content = r#"
data_test
_unknown ?
_not_applicable .
"#;

        let doc = CifDocument::parse(cif_content).unwrap();
        let block = &doc.blocks[0];

        assert_eq!(*block.get_item("_unknown").unwrap(), CifValue::Unknown);
        assert_eq!(
            *block.get_item("_not_applicable").unwrap(),
            CifValue::NotApplicable
        );
    }
}
