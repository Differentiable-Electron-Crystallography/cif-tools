//! Save frame structures in CIF files.

use super::{CifLoop, CifValue};
use std::collections::HashMap;

/// Represents a save frame in a CIF file.
///
/// Save frames are named containers within data blocks that group related
/// data items and loops. They're bounded by `save_name` and `save_` keywords.
///
/// # Structure
///
/// ```text
/// data_main
/// _main_item value
///
/// save_frame1
/// _frame_item1 value1
/// _frame_item2 value2
/// loop_
/// _col1 _col2
/// val1  val2
/// save_
/// ```
///
/// # Use Cases
///
/// Save frames are commonly used for:
/// - Grouping related molecular fragments
/// - Storing restraint definitions
/// - Organizing refinement parameters
/// - Creating reusable data templates
///
/// # Relationship to Data Blocks
///
/// Save frames are contained within data blocks and can contain the same
/// types of content (data items and loops) but cannot contain other save frames.
#[derive(Debug, Clone)]
pub struct CifFrame {
    /// Name of the save frame (from `save_name`)
    pub name: String,
    /// Data items (key-value pairs) within this frame
    pub items: HashMap<String, CifValue>,
    /// Loop structures within this frame
    pub loops: Vec<CifLoop>,
}

impl CifFrame {
    /// Create a new empty frame with the given name
    pub fn new(name: String) -> Self {
        CifFrame {
            name,
            items: HashMap::new(),
            loops: Vec::new(),
        }
    }

    /// Get a data item value by tag name
    pub fn get_item(&self, tag: &str) -> Option<&CifValue> {
        self.items.get(tag)
    }

    /// Find a loop containing a specific tag
    pub fn find_loop(&self, tag: &str) -> Option<&CifLoop> {
        self.loops
            .iter()
            .find(|loop_| loop_.tags.contains(&tag.to_string()))
    }

    /// Iterate over all tags in this frame (from both items and loops)
    pub fn all_tags(&self) -> impl Iterator<Item = &str> {
        self.items.keys().map(|s| s.as_str()).chain(
            self.loops
                .iter()
                .flat_map(|l| l.tags.iter().map(|s| s.as_str())),
        )
    }
}
