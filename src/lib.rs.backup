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
//! The parser uses a two-stage approach:
//! 1. **Grammar parsing** with Pest PEG parser (defined in `cif.pest`)
//! 2. **AST construction** that builds typed data structures from parse trees
//!
//! Key components:
//! - [`CifDocument`] - Root container for all data blocks
//! - [`CifBlock`] - Individual data blocks containing items, loops, and frames
//! - [`CifLoop`] - Tabular data structures
//! - [`CifValue`] - Individual values with type information
//! - `BlockBuilder` - Internal helper for managing parsing state
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

use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;

// WASM bindings module (conditionally compiled)
#[cfg(target_arch = "wasm32")]
pub mod wasm;

// Python bindings module (conditionally compiled)
#[cfg(feature = "python")]
pub mod python;

#[derive(Parser)]
#[grammar = "cif.pest"]
pub struct CIFParser;

/// Custom error type for CIF parsing
#[derive(Debug)]
pub enum CifError {
    ParseError(String),
    IoError(std::io::Error),
    InvalidStructure(String),
}

impl fmt::Display for CifError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CifError::ParseError(msg) => write!(f, "Parse error: {msg}"),
            CifError::IoError(err) => write!(f, "IO error: {err}"),
            CifError::InvalidStructure(msg) => write!(f, "Invalid CIF structure: {msg}"),
        }
    }
}

impl Error for CifError {}

impl From<std::io::Error> for CifError {
    fn from(err: std::io::Error) -> Self {
        CifError::IoError(err)
    }
}

impl From<pest::error::Error<Rule>> for CifError {
    fn from(err: pest::error::Error<Rule>) -> Self {
        CifError::ParseError(format!("{err}"))
    }
}

/// Represents a single value in a CIF file with automatic type detection.
///
/// CIF values come in many forms and require careful parsing to handle quotes,
/// special characters, and type detection. This enum represents the parsed and
/// typed result.
///
/// # Value Types
///
/// - **Text**: String values, including quoted strings and text fields
/// - **Numeric**: Floating-point numbers (integers are stored as f64)
/// - **Unknown**: The special value `?` indicating missing/unknown data
/// - **NotApplicable**: The special value `.` indicating not applicable
///
/// # Parsing Strategy
///
/// Values are parsed in this order:
/// 1. Check for special values (`?`, `.`)
/// 2. Remove quotes or extract text field content
/// 3. Try to parse as a number
/// 4. Fall back to text
///
/// # Examples
///
/// ```
/// use cif_parser::CifValue;
///
/// // Different ways values are parsed
/// assert_eq!(CifValue::parse_value("123.45"), CifValue::Numeric(123.45));
/// assert_eq!(CifValue::parse_value("'hello'"), CifValue::Text("hello".to_string()));
/// assert_eq!(CifValue::parse_value("?"), CifValue::Unknown);
/// assert_eq!(CifValue::parse_value("."), CifValue::NotApplicable);
/// ```
///
/// # Text Fields
///
/// Text fields are multi-line values delimited by semicolons at the start of lines:
/// ```text
/// ;This is a text field
/// that can span multiple lines
/// and contain special characters !@#$%
/// ;
/// ```
///
/// These are automatically detected and the semicolon delimiters are removed.
#[derive(Debug, Clone, PartialEq)]
pub enum CifValue {
    /// String value (from quoted strings, unquoted strings, or text fields)
    Text(String),
    /// Numeric value (both integers and floats are stored as f64)
    Numeric(f64),
    /// Unknown value (represented as `?` in CIF files)
    Unknown,
    /// Not applicable value (represented as `.` in CIF files)
    NotApplicable,
}

impl CifValue {
    /// Parse a CIF value from a raw string.
    ///
    /// This is the main entry point for value parsing. It handles all CIF value
    /// types including quoted strings, text fields, numbers, and special values.
    ///
    /// # Examples
    /// ```
    /// use cif_parser::CifValue;
    ///
    /// assert_eq!(CifValue::parse_value("42"), CifValue::Numeric(42.0));
    /// assert_eq!(CifValue::parse_value("'text'"), CifValue::Text("text".to_string()));
    /// assert_eq!(CifValue::parse_value("?"), CifValue::Unknown);
    /// ```
    pub fn parse_value(s: &str) -> Self {
        let trimmed = s.trim();

        // Check for special values first
        if let Some(special) = Self::parse_special_value(trimmed) {
            return special;
        }

        // Remove quotes and extract content
        let content = Self::extract_content(trimmed);

        // Try to parse as number, otherwise treat as text
        Self::parse_numeric_or_text(content)
    }
}

// Implement standard FromStr trait
impl std::str::FromStr for CifValue {
    type Err = std::convert::Infallible; // This method never fails

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse_value(s))
    }
}

impl CifValue {
    /// Check for CIF special values (`?` for unknown, `.` for not applicable).
    ///
    /// These values have special meaning in CIF and must be detected before
    /// other parsing logic.
    fn parse_special_value(s: &str) -> Option<Self> {
        match s {
            "?" => Some(CifValue::Unknown),
            "." => Some(CifValue::NotApplicable),
            _ => None,
        }
    }

    /// Extract content from quoted strings or text fields.
    ///
    /// Handles three cases:
    /// 1. Single/double quoted strings: removes the quotes
    /// 2. Text fields (start with `;`): removes semicolon delimiters
    /// 3. Unquoted strings: returns as-is
    ///
    /// # Text Field Handling
    /// Text fields in CIF are delimited by semicolons at the start of lines:
    /// ```text
    /// ;This is a text field
    /// with multiple lines
    /// ;
    /// ```
    /// The semicolons and surrounding whitespace are removed.
    fn extract_content(s: &str) -> &str {
        // Handle quoted strings
        if (s.starts_with('\'') && s.ends_with('\'')) || (s.starts_with('"') && s.ends_with('"')) {
            &s[1..s.len() - 1]
        }
        // Handle text fields (semicolon-delimited)
        else if s.starts_with(';') {
            s.trim_start_matches(';').trim_end_matches(';').trim()
        }
        // Unquoted string
        else {
            s
        }
    }

    /// Attempt to parse as a number, falling back to text.
    ///
    /// Uses Rust's built-in f64 parsing which handles:
    /// - Integers: `123`
    /// - Floats: `123.45`
    /// - Scientific notation: `1.23e-4`
    /// - Signs: `-123.45`
    ///
    /// If parsing fails, the string is stored as [`CifValue::Text`].
    fn parse_numeric_or_text(s: &str) -> Self {
        if let Ok(num) = s.parse::<f64>() {
            CifValue::Numeric(num)
        } else {
            CifValue::Text(s.to_string())
        }
    }

    /// Get the value as a string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            CifValue::Text(s) => Some(s),
            _ => None,
        }
    }

    /// Get the value as a number
    pub fn as_numeric(&self) -> Option<f64> {
        match self {
            CifValue::Numeric(n) => Some(*n),
            _ => None,
        }
    }
}

/// Represents a loop structure in a CIF file (tabular data).
///
/// Loops are one of the most important structures in CIF files, representing
/// tabular data with named columns (tags) and multiple rows of values.
///
/// # Structure
///
/// ```text
/// loop_
/// _atom_site_label          # Column 1
/// _atom_site_type_symbol    # Column 2  
/// _atom_site_fract_x        # Column 3
/// C1  C  0.1234            # Row 1
/// N1  N  0.5678            # Row 2
/// O1  O  0.9012            # Row 3
/// ```
///
/// # Data Organization
///
/// - **Tags**: Column headers (always start with `_`)
/// - **Values**: Organized as a vector of rows, each row containing values for all columns
/// - **Type safety**: Each value is parsed into a [`CifValue`] with appropriate type
///
/// # Access Patterns
///
/// ```
/// use cif_parser::Document;
///
/// # let cif = "data_test\nloop_\n_col1\n_col2\nval1 val2\nval3 val4\n";
/// # let doc = Document::parse(cif).unwrap();
/// # let loop_ = &doc.blocks[0].loops[0];
/// // By position
/// let value = loop_.get(0, 1);  // Row 0, Column 1
///
/// // By tag name  
/// let value = loop_.get_by_tag(0, "_col2");  // Row 0, "_col2" column
///
/// // Get entire column
/// let column = loop_.get_column("_col1");
/// ```
///
/// # Validation
///
/// The parser ensures that:
/// - Number of values is divisible by number of tags
/// - Each row has exactly the right number of values
/// - Empty loops (tags but no values) are valid
#[derive(Debug, Clone)]
pub struct CifLoop {
    /// Column names/headers (CIF tags starting with `_`)
    pub tags: Vec<String>,
    /// Data organized as rows, each containing one value per tag
    pub values: Vec<Vec<CifValue>>,
}

impl Default for CifLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl CifLoop {
    /// Create a new empty loop
    pub fn new() -> Self {
        CifLoop {
            tags: Vec::new(),
            values: Vec::new(),
        }
    }

    /// Get the number of rows in the loop
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if the loop is empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Get a specific value by row and column index
    pub fn get(&self, row: usize, col: usize) -> Option<&CifValue> {
        self.values.get(row)?.get(col)
    }

    /// Get a specific value by row index and tag name
    pub fn get_by_tag(&self, row: usize, tag: &str) -> Option<&CifValue> {
        let col = self.tags.iter().position(|t| t == tag)?;
        self.get(row, col)
    }

    /// Get all values for a specific tag
    pub fn get_column(&self, tag: &str) -> Option<Vec<&CifValue>> {
        let col = self.tags.iter().position(|t| t == tag)?;
        Some(self.values.iter().map(|row| &row[col]).collect())
    }
}

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
    /// Create a new empty block
    pub fn new(name: String) -> Self {
        CifBlock {
            name,
            items: HashMap::new(),
            loops: Vec::new(),
            frames: Vec::new(),
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

    /// Get all loop tags in this block
    pub fn get_loop_tags(&self) -> Vec<&String> {
        self.loops.iter().flat_map(|l| &l.tags).collect()
    }
}

/// Internal helper for building CIF blocks while managing pending loop state.
///
/// # The "Pending Loop State" Problem
///
/// In CIF files, loops can be interrupted by other elements, creating a complex
/// state management challenge. Consider this example:
///
/// ```text
/// data_test
/// loop_
/// _atom.id
/// _atom.type
/// _some_other_item  other_value    # This interrupts the loop!
/// 1 C
/// 2 N
/// ```
///
/// The parser encounters elements in this sequence:
/// 1. `loop_` keyword - signals start of a loop
/// 2. Loop tags (`_atom.id`, `_atom.type`) - defines the columns
/// 3. **Data item** (`_some_other_item other_value`) - interrupts!
/// 4. Loop values (`1 C`, `2 N`) - completes the original loop
///
/// # State Management Challenge
///
/// When the parser sees `_some_other_item`, it must:
/// 1. Finalize the incomplete loop (with tags but no values yet)
/// 2. Add it to the block
/// 3. Process the data item
/// 4. Continue parsing
///
/// Without proper state management, the parser would either:
/// - Lose the incomplete loop
/// - Mix the data item with the loop
/// - Crash on malformed structure
///
/// # Solution: BlockBuilder
///
/// The `BlockBuilder` encapsulates this state management:
///
/// ```rust,ignore
/// let mut builder = BlockBuilder::new("test".to_string());
///
/// // Start a loop
/// builder.start_loop(incomplete_loop);
///
/// // Add a data item - this automatically finalizes the pending loop
/// builder.add_item("_other_item".to_string(), value);
///
/// // The pending loop has been safely added to the block
/// let block = builder.finish();
/// ```
///
/// # Key Benefits
///
/// - **Automatic finalization**: Pending loops are automatically added when needed
/// - **No manual state tracking**: Eliminates repetitive `if let Some(loop_) = ...` code
/// - **Error prevention**: Impossible to forget to finalize a pending loop
/// - **Clean code**: Parse methods focus on their core logic, not state management
///
/// # Usage Pattern
///
/// The builder follows a clear pattern for each type of element:
/// - **Data items**: `add_item()` - finalizes pending loop, then adds item
/// - **New loops**: `start_loop()` - finalizes pending loop, then starts new one
/// - **Save frames**: `add_frame()` - finalizes pending loop, then adds frame
/// - **Block completion**: `finish()` - finalizes any remaining pending loop
struct BlockBuilder {
    /// The block being constructed
    block: CifBlock,
    /// Current incomplete loop waiting for values or finalization
    pending_loop: Option<CifLoop>,
}

impl BlockBuilder {
    fn new(name: String) -> Self {
        Self {
            block: CifBlock::new(name),
            pending_loop: None,
        }
    }

    /// Finalize any pending loop and add a data item
    fn add_item(&mut self, tag: String, value: CifValue) {
        self.finalize_pending_loop();
        self.block.items.insert(tag, value);
    }

    /// Finalize any pending loop and start a new one
    fn start_loop(&mut self, loop_: CifLoop) {
        self.finalize_pending_loop();
        self.pending_loop = Some(loop_);
    }

    /// Finalize any pending loop and add a frame
    fn add_frame(&mut self, frame: CifFrame) {
        self.finalize_pending_loop();
        self.block.frames.push(frame);
    }

    /// Finalize any pending loop by adding it to the block
    fn finalize_pending_loop(&mut self) {
        if let Some(loop_) = self.pending_loop.take() {
            self.block.loops.push(loop_);
        }
    }

    /// Consume the builder and return the completed block
    fn finish(mut self) -> CifBlock {
        self.finalize_pending_loop();
        self.block
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
}

impl Default for CifDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl CifDocument {
    /// Create a new empty document
    pub fn new() -> Self {
        CifDocument { blocks: Vec::new() }
    }

    /// Parse a CIF document from a string
    pub fn parse(input: &str) -> Result<Self, CifError> {
        let pairs = CIFParser::parse(Rule::file, input)?;
        let mut doc = CifDocument::new();

        for pair in pairs {
            if pair.as_rule() == Rule::file {
                doc = Self::parse_file(pair)?;
            }
        }

        Ok(doc)
    }

    /// Parse a CIF document from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, CifError> {
        let content = fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Get a block by name
    pub fn get_block(&self, name: &str) -> Option<&CifBlock> {
        self.blocks.iter().find(|b| b.name == name)
    }

    /// Get the first block (common for single-block CIF files)
    pub fn first_block(&self) -> Option<&CifBlock> {
        self.blocks.first()
    }

    // Internal parsing methods

    /// Extract block name from a data block heading with case-insensitive parsing.
    ///
    /// # CIF Block Naming Rules
    ///
    /// - `data_name` → `"name"`
    /// - `DATA_NAME` → `"NAME"` (preserves original case of the name part)
    /// - `global_` → `""` (empty string, as global blocks have no name)
    ///
    /// # Case Sensitivity
    ///
    /// The keywords (`data_`, `global_`) are case-insensitive per CIF specification,
    /// but the name part preserves its original casing. This means:
    /// - `DATA_MyProtein` → `"MyProtein"`
    /// - `data_MyProtein` → `"MyProtein"`
    ///
    /// # Examples
    /// ```ignore
    /// // This method is private, shown for documentation only
    /// assert_eq!(extract_block_name("data_test"), "test");
    /// assert_eq!(extract_block_name("DATA_TEST"), "TEST");
    /// assert_eq!(extract_block_name("global_"), "");
    /// ```
    fn extract_block_name(heading_str: &str) -> String {
        let lower = heading_str.to_lowercase();
        if lower.starts_with("data_") {
            heading_str[5..].to_string()
        } else if lower == "global_" {
            String::new() // Global block has no name
        } else {
            heading_str.to_string()
        }
    }

    fn parse_file(pair: Pair<Rule>) -> Result<Self, CifError> {
        let mut doc = CifDocument::new();

        for inner_pair in pair.into_inner() {
            if inner_pair.as_rule() == Rule::content {
                for content_pair in inner_pair.into_inner() {
                    if content_pair.as_rule() == Rule::datablock {
                        let block = Self::parse_datablock(content_pair)?;
                        doc.blocks.push(block);
                    }
                }
            }
        }

        Ok(doc)
    }

    fn parse_datablock(pair: Pair<Rule>) -> Result<CifBlock, CifError> {
        let mut builder = BlockBuilder::new(String::new());

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::datablockheading => {
                    let name = Self::extract_block_name(inner_pair.as_str());
                    builder.block.name = name;
                }
                Rule::dataitem => {
                    let (tag, value) = Self::parse_dataitem(inner_pair)?;
                    builder.add_item(tag, value);
                }
                Rule::loop_block => {
                    let loop_ = Self::parse_loop(inner_pair)?;
                    builder.start_loop(loop_);
                }
                Rule::frame => {
                    let frame = Self::parse_frame(inner_pair)?;
                    builder.add_frame(frame);
                }
                _ => {
                    // Ignore unknown rules for now
                }
            }
        }

        Ok(builder.finish())
    }

    fn parse_dataitem(pair: Pair<Rule>) -> Result<(String, CifValue), CifError> {
        let mut tag = String::new();
        let mut value = CifValue::Unknown;

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::item_tag | Rule::tag => {
                    tag = inner_pair.as_str().to_string();
                }
                Rule::item_value | Rule::value => {
                    value = CifValue::parse_value(inner_pair.as_str());
                }
                _ => {
                    // Ignore other rules
                }
            }
        }

        if tag.is_empty() {
            return Err(CifError::InvalidStructure(
                "Data item missing tag".to_string(),
            ));
        }

        Ok((tag, value))
    }

    /// Parse a loop structure from the parse tree.
    ///
    /// # Loop Structure Validation
    ///
    /// Loops must have:
    /// 1. At least one tag (column header)
    /// 2. Values count divisible by tag count (complete rows)
    /// 3. Each value must be parseable as a [`CifValue`]
    ///
    /// # Error Conditions
    ///
    /// - [`CifError::InvalidStructure`]: No tags found
    /// - [`CifError::InvalidStructure`]: Values don't align with tags (wrong count)
    ///
    /// # Empty Loops
    ///
    /// Loops with tags but no values are valid (represents an empty table).
    fn parse_loop(pair: Pair<Rule>) -> Result<CifLoop, CifError> {
        let mut loop_ = CifLoop::new();
        let mut values = Vec::new();

        // Collect tags and values
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::loop_tag | Rule::tag => {
                    loop_.tags.push(inner_pair.as_str().to_string());
                }
                Rule::loop_values => {
                    Self::collect_loop_values(inner_pair, &mut values);
                }
                Rule::loop_value | Rule::value => {
                    values.push(CifValue::parse_value(inner_pair.as_str()));
                }
                _ => {
                    // Ignore other rules
                }
            }
        }

        // Validate and organize into rows
        if loop_.tags.is_empty() {
            return Err(CifError::InvalidStructure(
                "Loop block has no tags".to_string(),
            ));
        }

        Self::organize_loop_values(&mut loop_, values)?;
        Ok(loop_)
    }

    /// Helper to collect values from loop_values rule
    fn collect_loop_values(pair: Pair<Rule>, values: &mut Vec<CifValue>) {
        for value_pair in pair.into_inner() {
            match value_pair.as_rule() {
                Rule::loop_value | Rule::value => {
                    values.push(CifValue::parse_value(value_pair.as_str()));
                }
                _ => {
                    // Ignore other rules
                }
            }
        }
    }

    /// Organize values into rows based on tag count.
    ///
    /// # Algorithm
    ///
    /// Values in CIF loops are stored sequentially and must be organized into
    /// rows based on the number of tags (columns):
    ///
    /// ```text
    /// Tags: [_col1, _col2, _col3]     # 3 columns
    /// Values: [v1, v2, v3, v4, v5, v6] # 6 values
    ///
    /// Result:
    /// Row 0: [v1, v2, v3]
    /// Row 1: [v4, v5, v6]
    /// ```
    ///
    /// # Validation
    ///
    /// - Total values must be divisible by tag count
    /// - Empty loops (0 values) are valid
    /// - Partial rows are rejected with [`CifError::InvalidStructure`]
    fn organize_loop_values(loop_: &mut CifLoop, values: Vec<CifValue>) -> Result<(), CifError> {
        if values.is_empty() {
            return Ok(()); // Empty loop is valid
        }

        let tag_count = loop_.tags.len();
        if values.len() % tag_count != 0 {
            return Err(CifError::InvalidStructure(format!(
                "Loop has {} tags but {} values (not divisible)",
                tag_count,
                values.len()
            )));
        }

        for row_values in values.chunks(tag_count) {
            loop_.values.push(row_values.to_vec());
        }

        Ok(())
    }

    fn parse_frame(pair: Pair<Rule>) -> Result<CifFrame, CifError> {
        let mut frame = CifFrame {
            name: String::new(),
            items: HashMap::new(),
            loops: Vec::new(),
        };

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::framename => {
                    frame.name = inner_pair.as_str().to_string();
                }
                Rule::dataitem => {
                    let (tag, value) = Self::parse_dataitem(inner_pair)?;
                    frame.items.insert(tag, value);
                }
                Rule::loop_block => {
                    let loop_ = Self::parse_loop(inner_pair)?;
                    frame.loops.push(loop_);
                }
                _ => {
                    // Ignore other rules
                }
            }
        }

        if frame.name.is_empty() {
            return Err(CifError::InvalidStructure(
                "Save frame missing name".to_string(),
            ));
        }

        Ok(frame)
    }
}

// Public convenience functions

/// Parse a CIF file from a path
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<CifDocument, CifError> {
    CifDocument::from_file(path)
}

/// Parse a CIF string
pub fn parse_string(input: &str) -> Result<CifDocument, CifError> {
    CifDocument::parse(input)
}

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

// Re-export commonly used types
pub use CifBlock as Block;
pub use CifDocument as Document;
pub use CifFrame as Frame;
pub use CifLoop as Loop;
pub use CifValue as Value;
