//! WebAssembly bindings for the CIF parser.
//!
//! This module provides JavaScript-compatible wrappers around the core CIF parsing
//! functionality, using wasm-bindgen for seamless interop with JavaScript.

use crate::{CifBlock, CifDocument, CifFrame, CifLoop, CifValue};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// Console logging for debugging
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

/// JavaScript-compatible representation of a CIF value
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsCifValue {
    value_type: String,
    text_value: Option<String>,
    numeric_value: Option<f64>,
}

#[wasm_bindgen]
impl JsCifValue {
    /// Get the type of this value as a string
    #[wasm_bindgen(getter)]
    pub fn value_type(&self) -> String {
        self.value_type.clone()
    }

    /// Get the text value (if this is a text value)
    #[wasm_bindgen(getter)]
    pub fn text_value(&self) -> Option<String> {
        self.text_value.clone()
    }

    /// Get the numeric value (if this is a numeric value)
    #[wasm_bindgen(getter)]
    pub fn numeric_value(&self) -> Option<f64> {
        self.numeric_value
    }

    /// Check if this is a text value
    #[wasm_bindgen]
    pub fn is_text(&self) -> bool {
        self.value_type == "Text"
    }

    /// Check if this is a numeric value
    #[wasm_bindgen]
    pub fn is_numeric(&self) -> bool {
        self.value_type == "Numeric"
    }

    /// Check if this is an unknown value (?)
    #[wasm_bindgen]
    pub fn is_unknown(&self) -> bool {
        self.value_type == "Unknown"
    }

    /// Check if this is a not-applicable value (.)
    #[wasm_bindgen]
    pub fn is_not_applicable(&self) -> bool {
        self.value_type == "NotApplicable"
    }
}

impl From<&CifValue> for JsCifValue {
    fn from(value: &CifValue) -> Self {
        match value {
            CifValue::Text(s) => JsCifValue {
                value_type: "Text".to_string(),
                text_value: Some(s.clone()),
                numeric_value: None,
            },
            CifValue::Numeric(n) => JsCifValue {
                value_type: "Numeric".to_string(),
                text_value: None,
                numeric_value: Some(*n),
            },
            CifValue::Unknown => JsCifValue {
                value_type: "Unknown".to_string(),
                text_value: None,
                numeric_value: None,
            },
            CifValue::NotApplicable => JsCifValue {
                value_type: "NotApplicable".to_string(),
                text_value: None,
                numeric_value: None,
            },
        }
    }
}

/// JavaScript-compatible representation of a CIF loop
#[wasm_bindgen]
pub struct JsCifLoop {
    inner: CifLoop,
}

#[wasm_bindgen]
impl JsCifLoop {
    /// Get the tag names (column headers)
    #[wasm_bindgen(getter)]
    pub fn tags(&self) -> Vec<String> {
        self.inner.tags.clone()
    }

    /// Get the number of rows
    #[wasm_bindgen(getter = numRows)]
    pub fn num_rows(&self) -> usize {
        self.inner.len()
    }

    /// Get the number of columns
    #[wasm_bindgen(getter = numColumns)]
    pub fn num_columns(&self) -> usize {
        self.inner.tags.len()
    }

    /// Get the tag names (column headers) - method alias for compatibility
    #[wasm_bindgen]
    pub fn get_tags(&self) -> Vec<String> {
        self.tags()
    }

    /// Get the number of rows - method alias for compatibility
    #[wasm_bindgen]
    pub fn get_row_count(&self) -> usize {
        self.num_rows()
    }

    /// Get the number of columns - method alias for compatibility
    #[wasm_bindgen]
    pub fn get_column_count(&self) -> usize {
        self.num_columns()
    }

    /// Get a value by row and column index
    #[wasm_bindgen]
    pub fn get_value(&self, row: usize, col: usize) -> Option<JsCifValue> {
        self.inner.get(row, col).map(|v| v.into())
    }

    /// Get a value by row index and tag name
    #[wasm_bindgen]
    pub fn get_value_by_tag(&self, row: usize, tag: &str) -> Option<JsCifValue> {
        self.inner.get_by_tag(row, tag).map(|v| v.into())
    }

    /// Get all values for a specific tag as an array
    #[wasm_bindgen]
    pub fn get_column(&self, tag: &str) -> Option<Vec<JsCifValue>> {
        self.inner
            .get_column(tag)
            .map(|values| values.iter().map(|v| (*v).into()).collect())
    }

    /// Get a row as a JavaScript object (dictionary) mapping tags to values
    #[wasm_bindgen]
    pub fn get_row_dict(&self, row: usize) -> Result<JsValue, JsValue> {
        use js_sys::Object;
        use wasm_bindgen::JsValue;

        if row >= self.inner.len() {
            return Err(JsValue::from_str("Row index out of bounds"));
        }

        let obj = Object::new();
        for (col, tag) in self.inner.tags.iter().enumerate() {
            if let Some(value) = self.inner.get(row, col) {
                let js_value: JsCifValue = value.into();
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str(tag),
                    &serde_wasm_bindgen::to_value(&js_value).unwrap_or(JsValue::NULL),
                );
            }
        }
        Ok(obj.into())
    }

    /// Check if the loop is empty
    #[wasm_bindgen]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl From<CifLoop> for JsCifLoop {
    fn from(loop_: CifLoop) -> Self {
        JsCifLoop { inner: loop_ }
    }
}

/// JavaScript-compatible representation of a CIF frame
#[wasm_bindgen]
pub struct JsCifFrame {
    inner: CifFrame,
}

#[wasm_bindgen]
impl JsCifFrame {
    /// Get the frame name
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.inner.name.clone()
    }

    /// Get all item keys
    #[wasm_bindgen(getter = itemKeys)]
    pub fn item_keys(&self) -> Vec<String> {
        self.inner.items.keys().cloned().collect()
    }

    /// Get the number of loops in this frame
    #[wasm_bindgen(getter = numLoops)]
    pub fn num_loops(&self) -> usize {
        self.inner.loops.len()
    }

    /// Get all item keys - method alias for compatibility
    #[wasm_bindgen]
    pub fn get_item_keys(&self) -> Vec<String> {
        self.item_keys()
    }

    /// Get an item value by key
    #[wasm_bindgen]
    pub fn get_item(&self, key: &str) -> Option<JsCifValue> {
        self.inner.items.get(key).map(|v| v.into())
    }

    /// Get the number of loops in this frame - method alias for compatibility
    #[wasm_bindgen]
    pub fn get_loop_count(&self) -> usize {
        self.num_loops()
    }

    /// Get a loop by index
    #[wasm_bindgen]
    pub fn get_loop(&self, index: usize) -> Option<JsCifLoop> {
        self.inner.loops.get(index).cloned().map(|l| l.into())
    }
}

impl From<CifFrame> for JsCifFrame {
    fn from(frame: CifFrame) -> Self {
        JsCifFrame { inner: frame }
    }
}

/// JavaScript-compatible representation of a CIF block
#[wasm_bindgen]
pub struct JsCifBlock {
    inner: CifBlock,
}

#[wasm_bindgen]
impl JsCifBlock {
    /// Get the block name
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.inner.name.clone()
    }

    /// Get all item keys
    #[wasm_bindgen(getter = itemKeys)]
    pub fn item_keys(&self) -> Vec<String> {
        self.inner.items.keys().cloned().collect()
    }

    /// Get the number of loops in this block
    #[wasm_bindgen(getter = numLoops)]
    pub fn num_loops(&self) -> usize {
        self.inner.loops.len()
    }

    /// Get the number of frames in this block
    #[wasm_bindgen(getter = numFrames)]
    pub fn num_frames(&self) -> usize {
        self.inner.frames.len()
    }

    /// Get all item keys - method alias for compatibility
    #[wasm_bindgen]
    pub fn get_item_keys(&self) -> Vec<String> {
        self.item_keys()
    }

    /// Get an item value by key
    #[wasm_bindgen]
    pub fn get_item(&self, key: &str) -> Option<JsCifValue> {
        self.inner.items.get(key).map(|v| v.into())
    }

    /// Get the number of loops in this block - method alias for compatibility
    #[wasm_bindgen]
    pub fn get_loop_count(&self) -> usize {
        self.num_loops()
    }

    /// Get a loop by index
    #[wasm_bindgen]
    pub fn get_loop(&self, index: usize) -> Option<JsCifLoop> {
        self.inner.loops.get(index).cloned().map(|l| l.into())
    }

    /// Find a loop containing a specific tag
    #[wasm_bindgen]
    pub fn find_loop(&self, tag: &str) -> Option<JsCifLoop> {
        self.inner.find_loop(tag).cloned().map(|l| l.into())
    }

    /// Get all loop tags in this block
    #[wasm_bindgen]
    pub fn get_loop_tags(&self) -> Vec<String> {
        self.inner.get_loop_tags().into_iter().cloned().collect()
    }

    /// Get the number of frames in this block - method alias for compatibility
    #[wasm_bindgen]
    pub fn get_frame_count(&self) -> usize {
        self.num_frames()
    }

    /// Get a frame by index
    #[wasm_bindgen]
    pub fn get_frame(&self, index: usize) -> Option<JsCifFrame> {
        self.inner.frames.get(index).cloned().map(|f| f.into())
    }
}

impl From<CifBlock> for JsCifBlock {
    fn from(block: CifBlock) -> Self {
        JsCifBlock { inner: block }
    }
}

/// JavaScript-compatible representation of a CIF document
#[wasm_bindgen]
pub struct JsCifDocument {
    inner: CifDocument,
}

#[wasm_bindgen]
impl JsCifDocument {
    /// Parse a CIF string and return a document
    #[wasm_bindgen]
    pub fn parse(input: &str) -> Result<JsCifDocument, String> {
        console_log!("Parsing CIF content of length: {}", input.len());

        match CifDocument::parse(input) {
            Ok(doc) => {
                console_log!("Successfully parsed {} blocks", doc.blocks.len());
                Ok(JsCifDocument { inner: doc })
            }
            Err(e) => {
                // Format error message with location info if available
                let error_msg = match e {
                    crate::CifError::ParseError(msg) => {
                        format!("Parse error: {}", msg)
                    }
                    crate::CifError::IoError(err) => {
                        format!("IO error: {}", err)
                    }
                    crate::CifError::InvalidStructure { message, location } => {
                        if let Some((line, col)) = location {
                            format!(
                                "Invalid structure at line {}, col {}: {}",
                                line, col, message
                            )
                        } else {
                            format!("Invalid structure: {}", message)
                        }
                    }
                };
                console_log!("{}", error_msg);
                Err(error_msg)
            }
        }
    }

    /// Get the number of blocks
    #[wasm_bindgen(getter = blockCount)]
    pub fn block_count(&self) -> usize {
        self.inner.blocks.len()
    }

    /// Get all block names
    #[wasm_bindgen(getter = blockNames)]
    pub fn block_names(&self) -> Vec<String> {
        self.inner.blocks.iter().map(|b| b.name.clone()).collect()
    }

    /// Get the number of blocks - method alias for compatibility
    #[wasm_bindgen]
    pub fn get_block_count(&self) -> usize {
        self.block_count()
    }

    /// Get a block by index
    #[wasm_bindgen]
    pub fn get_block(&self, index: usize) -> Option<JsCifBlock> {
        self.inner.blocks.get(index).cloned().map(|b| b.into())
    }

    /// Get a block by name
    #[wasm_bindgen]
    pub fn get_block_by_name(&self, name: &str) -> Option<JsCifBlock> {
        self.inner.get_block(name).cloned().map(|b| b.into())
    }

    /// Get the first block (common for single-block CIF files)
    #[wasm_bindgen]
    pub fn first_block(&self) -> Option<JsCifBlock> {
        self.inner.first_block().cloned().map(|b| b.into())
    }

    /// Get the first block - method alias for compatibility
    #[wasm_bindgen]
    pub fn get_first_block(&self) -> Option<JsCifBlock> {
        self.first_block()
    }

    /// Get all block names - method alias for compatibility
    #[wasm_bindgen]
    pub fn get_block_names(&self) -> Vec<String> {
        self.block_names()
    }
}

/// Initialize the WASM module (optional, for any setup needed)
#[wasm_bindgen(start)]
pub fn main() {
    console_log!("CIF Parser WASM module initialized");
}

/// Parse a CIF string into a document (convenience function)
#[wasm_bindgen]
pub fn parse(content: &str) -> Result<JsCifDocument, String> {
    JsCifDocument::parse(content)
}

/// Get the version of the CIF parser
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Get the version of the CIF parser - method alias for compatibility
#[wasm_bindgen]
pub fn get_version() -> String {
    version()
}

/// Get the author of the CIF parser
#[wasm_bindgen]
pub fn author() -> String {
    "Iain Maitland".to_string()
}

/// Simple test function to verify WASM is working
#[wasm_bindgen]
pub fn test_wasm() -> String {
    console_log!("WASM test function called");
    "CIF Parser WASM module is working!".to_string()
}
