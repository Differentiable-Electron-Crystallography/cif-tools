//! CIF document (root container) structures.

use super::CifBlock;
use crate::error::CifError;
use std::fs;
use std::path::Path;

/// CIF specification version detected or specified for a document.
///
/// CIF 2.0 is backward compatible with CIF 1.1, but adds new features:
/// - Lists: `[value1 value2 value3]`
/// - Tables: `{key1:value1 key2:value2}`
/// - Triple-quoted strings: `"""..."""` and `'''...'''`
/// - Full Unicode support (UTF-8)
/// - Newlines allowed in quoted strings
///
/// # Version Detection
///
/// Version is automatically detected when parsing:
/// - Files starting with `#\#CIF_2.0` are CIF 2.0
/// - Files without the magic comment default to CIF 1.1
/// - Files can be forced to a specific version using `parse_with_version()`
///
/// # Examples
///
/// ```
/// use cif_parser::{CifVersion, Document};
///
/// // CIF 2.0 file (with magic comment)
/// let cif2 = "#\\#CIF_2.0\ndata_test\n_item [1 2 3]\n";
/// let doc = Document::parse(cif2).unwrap();
/// assert_eq!(doc.version, CifVersion::V2_0);
///
/// // CIF 1.1 file (no magic comment)
/// let cif1 = "data_test\n_item value\n";
/// let doc = Document::parse(cif1).unwrap();
/// assert_eq!(doc.version, CifVersion::V1_1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CifVersion {
    /// CIF 1.1 specification
    ///
    /// https://www.iucr.org/resources/cif/spec/version1.1/cifsyntax
    V1_1,

    /// CIF 2.0 specification
    ///
    /// https://www.iucr.org/__data/assets/text_file/0009/112131/CIF2-ENBF.txt
    V2_0,
}

impl Default for CifVersion {
    /// Default to CIF 1.1 for backward compatibility
    fn default() -> Self {
        CifVersion::V1_1
    }
}

impl std::fmt::Display for CifVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CifVersion::V1_1 => write!(f, "CIF 1.1"),
            CifVersion::V2_0 => write!(f, "CIF 2.0"),
        }
    }
}

/// Represents a complete CIF document containing one or more data blocks.
///
/// This is the root container for all parsed CIF data. A CIF file can contain
/// multiple data blocks, each with its own data, loops, and save frames.
///
/// # Structure
///
/// ```text
/// # Comments and whitespace (ignored)
///
/// data_first             # First data block
/// _item1 value1
///
/// global_                # Global block (applies to subsequent blocks)
/// _global_setting value
///
/// data_second            # Second data block
/// _item2 value2
/// ```
///
/// # Usage Patterns
///
/// ```
/// use cif_parser::{Document, CifError};
///
/// let cif_content = "data_structure\n_item value\n";
///
/// // Parse from string
/// let doc = Document::parse(cif_content)?;
///
/// // Access blocks
/// let first = doc.first_block().unwrap();           // First block
/// let named = doc.get_block("structure").unwrap();  // Block by name
/// let all_blocks = &doc.blocks;                     // All blocks
///
/// # Ok::<(), CifError>(())
/// ```
///
/// # Multi-block Files
///
/// Many CIF files contain multiple structures or datasets:
/// ```text
/// data_structure1
/// _cell_length_a 10.0
///
/// data_structure2
/// _cell_length_a 15.0
/// ```
///
/// Each structure gets its own [`CifBlock`] with independent data.
#[derive(Debug, Clone)]
pub struct CifDocument {
    /// All data blocks in this document
    pub blocks: Vec<CifBlock>,

    /// CIF specification version (auto-detected or explicitly set)
    ///
    /// Defaults to CIF 1.1 for backward compatibility.
    /// Set to CIF 2.0 if the file contains the magic comment `#\#CIF_2.0`.
    pub version: CifVersion,
}

impl Default for CifDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl CifDocument {
    /// Create a new empty document (defaults to CIF 1.1)
    pub fn new() -> Self {
        CifDocument {
            blocks: Vec::new(),
            version: CifVersion::default(),
        }
    }

    /// Create a new empty document with a specific CIF version
    pub fn new_with_version(version: CifVersion) -> Self {
        CifDocument {
            blocks: Vec::new(),
            version,
        }
    }

    /// Parse a CIF document from a string (auto-detects version)
    ///
    /// This is the main entry point for parsing CIF content.
    /// Version is auto-detected:
    /// - Files with `#\#CIF_2.0` magic comment → CIF 2.0
    /// - Files without magic comment → CIF 1.1
    ///
    /// The actual parsing logic is in the `parser` module.
    ///
    /// # Examples
    /// ```
    /// use cif_parser::Document;
    ///
    /// let cif = "data_test\n_item value\n";
    /// let doc = Document::parse(cif).unwrap();
    /// assert_eq!(doc.blocks.len(), 1);
    /// ```
    pub fn parse(input: &str) -> Result<Self, CifError> {
        crate::parser::document::parse_file(input)
    }

    /// Parse a CIF document from a file
    ///
    /// # Examples
    /// ```no_run
    /// use cif_parser::Document;
    ///
    /// let doc = Document::from_file("structure.cif").unwrap();
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, CifError> {
        let content = fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Get a block by name
    ///
    /// # Examples
    /// ```
    /// # use cif_parser::Document;
    /// # let cif = "data_test\n_item value\n";
    /// # let doc = Document::parse(cif).unwrap();
    /// let block = doc.get_block("test");
    /// assert!(block.is_some());
    /// ```
    pub fn get_block(&self, name: &str) -> Option<&CifBlock> {
        self.blocks.iter().find(|b| b.name == name)
    }

    /// Get the first block (common for single-block CIF files)
    ///
    /// # Examples
    /// ```
    /// # use cif_parser::Document;
    /// # let cif = "data_test\n_item value\n";
    /// # let doc = Document::parse(cif).unwrap();
    /// let block = doc.first_block().unwrap();
    /// assert_eq!(block.name, "test");
    /// ```
    pub fn first_block(&self) -> Option<&CifBlock> {
        self.blocks.first()
    }

    /// Iterate over all blocks
    pub fn blocks_iter(&self) -> impl Iterator<Item = &CifBlock> {
        self.blocks.iter()
    }

    /// Iterate over all tags across all blocks
    pub fn all_tags(&self) -> impl Iterator<Item = &str> {
        self.blocks.iter().flat_map(|b| b.all_tags())
    }
}
