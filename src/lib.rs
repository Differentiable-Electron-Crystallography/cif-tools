// lib.rs - A general-purpose CIF parser library

use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;

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
            CifError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            CifError::IoError(err) => write!(f, "IO error: {}", err),
            CifError::InvalidStructure(msg) => write!(f, "Invalid CIF structure: {}", msg),
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
        CifError::ParseError(format!("{}", err))
    }
}

/// Represents a single value in a CIF file
#[derive(Debug, Clone, PartialEq)]
pub enum CifValue {
    Text(String),
    Numeric(f64),
    Unknown,       // For '?' values
    NotApplicable, // For '.' values
}

impl CifValue {
    /// Parse a CIF value from a string
    pub fn from_str(s: &str) -> Self {
        let trimmed = s.trim();

        // Check for special values
        if trimmed == "?" {
            return CifValue::Unknown;
        }
        if trimmed == "." {
            return CifValue::NotApplicable;
        }

        // Remove quotes if present
        let unquoted = if (trimmed.starts_with('\'') && trimmed.ends_with('\''))
            || (trimmed.starts_with('"') && trimmed.ends_with('"'))
        {
            &trimmed[1..trimmed.len() - 1]
        } else if trimmed.starts_with(';') {
            // Text field - remove semicolons and newlines
            trimmed.trim_start_matches(';').trim_end_matches(';').trim()
        } else {
            trimmed
        };

        // Try to parse as number
        if let Ok(num) = unquoted.parse::<f64>() {
            CifValue::Numeric(num)
        } else {
            CifValue::Text(unquoted.to_string())
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

/// Represents a loop in a CIF file
#[derive(Debug, Clone)]
pub struct CifLoop {
    pub tags: Vec<String>,
    pub values: Vec<Vec<CifValue>>, // Each inner Vec is one row
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

/// Represents a save frame in a CIF file
#[derive(Debug, Clone)]
pub struct CifFrame {
    pub name: String,
    pub items: HashMap<String, CifValue>,
    pub loops: Vec<CifLoop>,
}

/// Represents a data block in a CIF file
#[derive(Debug, Clone)]
pub struct CifBlock {
    pub name: String,
    pub items: HashMap<String, CifValue>,
    pub loops: Vec<CifLoop>,
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

/// Represents a complete CIF document
#[derive(Debug, Clone)]
pub struct CifDocument {
    pub blocks: Vec<CifBlock>,
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
            match pair.as_rule() {
                Rule::file => {
                    doc = Self::parse_file(pair)?;
                }
                _ => {}
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

    fn parse_file(pair: Pair<Rule>) -> Result<Self, CifError> {
        let mut doc = CifDocument::new();

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::content => {
                    for content_pair in inner_pair.into_inner() {
                        if content_pair.as_rule() == Rule::datablock {
                            let block = Self::parse_datablock(content_pair)?;
                            doc.blocks.push(block);
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(doc)
    }

    fn parse_datablock(pair: Pair<Rule>) -> Result<CifBlock, CifError> {
        let mut block = CifBlock::new(String::new());
        let mut current_loop: Option<CifLoop> = None;

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::datablockheading => {
                    let heading_str = inner_pair.as_str();
                    block.name = if heading_str.to_lowercase().starts_with("data_") {
                        heading_str[5..].to_string()
                    } else if heading_str.to_lowercase() == "global_" {
                        String::new() // Global block has no name
                    } else {
                        heading_str.to_string()
                    };
                }
                Rule::dataitem => {
                    // Save any pending loop
                    if let Some(loop_) = current_loop.take() {
                        block.loops.push(loop_);
                    }

                    let (tag, value) = Self::parse_dataitem(inner_pair)?;
                    block.items.insert(tag, value);
                }
                Rule::loop_block => {
                    // Save any pending loop
                    if let Some(loop_) = current_loop.take() {
                        block.loops.push(loop_);
                    }

                    current_loop = Some(Self::parse_loop(inner_pair)?);
                }
                Rule::frame => {
                    // Save any pending loop
                    if let Some(loop_) = current_loop.take() {
                        block.loops.push(loop_);
                    }

                    let frame = Self::parse_frame(inner_pair)?;
                    block.frames.push(frame);
                }
                _ => {}
            }
        }

        // Save any final pending loop
        if let Some(loop_) = current_loop {
            block.loops.push(loop_);
        }

        Ok(block)
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
                    value = CifValue::from_str(inner_pair.as_str());
                }
                _ => {}
            }
        }

        Ok((tag, value))
    }

    fn parse_loop(pair: Pair<Rule>) -> Result<CifLoop, CifError> {
        let mut loop_ = CifLoop::new();
        let mut current_values = Vec::new();

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::loop_tag | Rule::tag => {
                    loop_.tags.push(inner_pair.as_str().to_string());
                }
                Rule::loop_values => {
                    for value_pair in inner_pair.into_inner() {
                        if value_pair.as_rule() == Rule::loop_value
                            || value_pair.as_rule() == Rule::value
                        {
                            current_values.push(CifValue::from_str(value_pair.as_str()));
                        }
                    }
                }
                Rule::loop_value | Rule::value => {
                    current_values.push(CifValue::from_str(inner_pair.as_str()));
                }
                _ => {}
            }
        }

        // Organize values into rows
        if !loop_.tags.is_empty() && !current_values.is_empty() {
            let cols = loop_.tags.len();
            if current_values.len() % cols != 0 {
                return Err(CifError::InvalidStructure(format!(
                    "Loop values count {} is not divisible by tag count {}",
                    current_values.len(),
                    cols
                )));
            }

            for row_values in current_values.chunks(cols) {
                loop_.values.push(row_values.to_vec());
            }
        }

        Ok(loop_)
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
                _ => {}
            }
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
