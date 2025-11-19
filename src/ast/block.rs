//! Data block structures in CIF files.

use super::{CifFrame, CifLoop, CifValue};
use std::collections::HashMap;

/// Represents a data block in a CIF file.
///
/// Data blocks are the primary organizational unit in CIF files. Each block
/// has a name and contains data items, loops, and save frames.
///
/// # Types of Blocks
///
/// - **Regular data blocks**: `data_name` - contain structure-specific data
/// - **Global blocks**: `global_` - contain data applying to subsequent blocks
///
/// # Structure
///
/// ```text
/// data_example           # Block name is "example"
/// _item1 value1         # Data items (key-value pairs)
/// _item2 'quoted value'
///
/// loop_                 # Tabular data
/// _col1 _col2
/// val1  val2
/// val3  val4
///
/// save_frame1           # Named sub-containers
/// _frame_item value
/// save_
/// ```
///
/// # Block Names
///
/// Block names are extracted from the header:
/// - `data_protein` → name is `"protein"`
/// - `DATA_STRUCTURE` → name is `"STRUCTURE"` (case-insensitive parsing)
/// - `global_` → name is `""` (empty string for global blocks)
///
/// # Access Methods
///
/// ```
/// use cif_parser::Document;
///
/// # let cif = "data_test\n_item value\n";
/// # let doc = Document::parse(cif).unwrap();
/// # let block = doc.first_block().unwrap();
/// // Get data items
/// let value = block.get_item("_item");
///
/// // Find loops containing specific tags
/// let loop_ = block.find_loop("_atom_site_label");
///
/// // Get all loop tags
/// let all_tags = block.get_loop_tags();
/// ```
#[derive(Debug, Clone)]
pub struct CifBlock {
    /// Block name (extracted from `data_name` header)
    pub name: String,
    /// Data items (key-value pairs) in this block
    pub items: HashMap<String, CifValue>,
    /// Loop structures (tabular data) in this block
    pub loops: Vec<CifLoop>,
    /// Save frames (named sub-containers) in this block
    pub frames: Vec<CifFrame>,
}

impl CifBlock {
    /// Create a new empty block with the given name
    pub fn new(name: String) -> Self {
        CifBlock {
            name,
            items: HashMap::new(),
            loops: Vec::new(),
            frames: Vec::new(),
        }
    }

    /// Get a data item value by tag name
    ///
    /// # Examples
    /// ```
    /// # use cif_parser::Document;
    /// # let cif = "data_test\n_item value\n";
    /// # let doc = Document::parse(cif).unwrap();
    /// # let block = doc.first_block().unwrap();
    /// let value = block.get_item("_item");
    /// ```
    pub fn get_item(&self, tag: &str) -> Option<&CifValue> {
        self.items.get(tag)
    }

    /// Find a loop containing a specific tag
    ///
    /// # Examples
    /// ```
    /// # use cif_parser::Document;
    /// # let cif = "data_test\nloop_\n_col1\n_col2\nval1 val2\n";
    /// # let doc = Document::parse(cif).unwrap();
    /// # let block = doc.first_block().unwrap();
    /// let loop_ = block.find_loop("_col1");
    /// assert!(loop_.is_some());
    /// ```
    pub fn find_loop(&self, tag: &str) -> Option<&CifLoop> {
        self.loops
            .iter()
            .find(|loop_| loop_.tags.contains(&tag.to_string()))
    }

    /// Get a frame by name
    ///
    /// # Examples
    /// ```
    /// # use cif_parser::Document;
    /// # let cif = "data_test\nsave_frame1\n_item val\nsave_\n";
    /// # let doc = Document::parse(cif).unwrap();
    /// # let block = doc.first_block().unwrap();
    /// let frame = block.get_frame("frame1");
    /// ```
    pub fn get_frame(&self, name: &str) -> Option<&CifFrame> {
        self.frames.iter().find(|f| f.name == name)
    }

    /// Get all loop tags in this block
    pub fn get_loop_tags(&self) -> Vec<&String> {
        self.loops.iter().flat_map(|l| &l.tags).collect()
    }

    /// Iterate over all tags in this block (from items, loops, and frames)
    pub fn all_tags(&self) -> impl Iterator<Item = &str> {
        self.items
            .keys()
            .map(|s| s.as_str())
            .chain(
                self.loops
                    .iter()
                    .flat_map(|l| l.tags.iter().map(|s| s.as_str())),
            )
            .chain(self.frames.iter().flat_map(|f| f.all_tags()))
    }

    /// Iterate over loops
    pub fn loops_iter(&self) -> impl Iterator<Item = &CifLoop> {
        self.loops.iter()
    }

    /// Iterate over frames
    pub fn frames_iter(&self) -> impl Iterator<Item = &CifFrame> {
        self.frames.iter()
    }

    /// Iterate over all items (key-value pairs)
    pub fn items_iter(&self) -> impl Iterator<Item = (&String, &CifValue)> {
        self.items.iter()
    }
}
