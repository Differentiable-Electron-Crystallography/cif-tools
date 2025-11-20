//! Python bindings for the CIF parser using PyO3.
//!
//! This module provides Python-native wrappers around the core CIF parsing
//! functionality, following Python naming conventions and idioms.

use crate::{CifBlock, CifDocument, CifError, CifFrame, CifLoop, CifValue, CifVersion};
use pyo3::exceptions::{PyIOError, PyIndexError, PyKeyError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyString;
use std::collections::HashMap;

/// Convert a Rust CifError to a Python exception
fn cif_error_to_py_err(err: CifError) -> PyErr {
    match err {
        CifError::ParseError(msg) => PyValueError::new_err(format!("Parse error: {msg}")),
        CifError::IoError(err) => PyIOError::new_err(format!("IO error: {err}")),
        CifError::InvalidStructure { message, location } => {
            if let Some((line, col)) = location {
                PyValueError::new_err(format!(
                    "Invalid structure at line {}, col {}: {}",
                    line, col, message
                ))
            } else {
                PyValueError::new_err(format!("Invalid CIF structure: {message}"))
            }
        }
    }
}

/// Python wrapper for CifVersion enum
#[pyclass(name = "Version", eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PyVersion {
    /// CIF 1.1 specification
    V1_1 = 0,
    /// CIF 2.0 specification
    V2_0 = 1,
}

#[pymethods]
impl PyVersion {
    /// String representation
    fn __str__(&self) -> &'static str {
        match self {
            PyVersion::V1_1 => "CIF 1.1",
            PyVersion::V2_0 => "CIF 2.0",
        }
    }

    /// Debug representation
    fn __repr__(&self) -> String {
        format!(
            "Version.{}",
            match self {
                PyVersion::V1_1 => "V1_1",
                PyVersion::V2_0 => "V2_0",
            }
        )
    }

    /// Check if this is CIF 2.0
    #[getter]
    fn is_cif2(&self) -> bool {
        matches!(self, PyVersion::V2_0)
    }

    /// Check if this is CIF 1.1
    #[getter]
    fn is_cif1(&self) -> bool {
        matches!(self, PyVersion::V1_1)
    }
}

impl From<CifVersion> for PyVersion {
    fn from(version: CifVersion) -> Self {
        match version {
            CifVersion::V1_1 => PyVersion::V1_1,
            CifVersion::V2_0 => PyVersion::V2_0,
        }
    }
}

/// Python wrapper for CifValue with Pythonic interface
#[pyclass(name = "Value")]
#[derive(Clone)]
pub struct PyValue {
    inner: CifValue,
}

#[pymethods]
impl PyValue {
    /// Check if this is a text value
    #[getter]
    fn is_text(&self) -> bool {
        matches!(self.inner, CifValue::Text(_))
    }

    /// Check if this is a numeric value
    #[getter]
    fn is_numeric(&self) -> bool {
        matches!(self.inner, CifValue::Numeric(_))
    }

    /// Check if this is an unknown value (?)
    #[getter]
    fn is_unknown(&self) -> bool {
        matches!(self.inner, CifValue::Unknown)
    }

    /// Check if this is a not-applicable value (.)
    #[getter]
    fn is_not_applicable(&self) -> bool {
        matches!(self.inner, CifValue::NotApplicable)
    }

    /// Check if this is a list value (CIF 2.0 only)
    #[getter]
    fn is_list(&self) -> bool {
        matches!(self.inner, CifValue::List(_))
    }

    /// Check if this is a table value (CIF 2.0 only)
    #[getter]
    fn is_table(&self) -> bool {
        matches!(self.inner, CifValue::Table(_))
    }

    /// Get the value as text (returns None if not a text value)
    #[getter]
    fn text(&self) -> Option<String> {
        self.inner.as_string().map(|s| s.to_string())
    }

    /// Get the value as a number (returns None if not numeric)
    #[getter]
    fn numeric(&self) -> Option<f64> {
        self.inner.as_numeric()
    }

    /// Get the value type as a string
    #[getter]
    fn value_type(&self) -> String {
        match self.inner {
            CifValue::Text(_) => "text".to_string(),
            CifValue::Numeric(_) => "numeric".to_string(),
            CifValue::Unknown => "unknown".to_string(),
            CifValue::NotApplicable => "not_applicable".to_string(),
            CifValue::List(_) => "list".to_string(),
            CifValue::Table(_) => "table".to_string(),
        }
    }

    /// Convert to Python native type
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        match &self.inner {
            CifValue::Text(s) => Ok(PyString::new(py, s).into_any().unbind()),
            CifValue::Numeric(n) => Ok(n.into_pyobject(py)?.into_any().unbind()),
            CifValue::Unknown => Ok(py.None()),
            CifValue::NotApplicable => Ok(py.None()),
            CifValue::List(values) => {
                // Convert Vec<CifValue> to Python list
                let py_list: Vec<Py<PyAny>> = values
                    .iter()
                    .map(|v| PyValue::from(v.clone()).to_python(py))
                    .collect::<PyResult<Vec<_>>>()?;
                Ok(py_list.into_pyobject(py)?.into_any().unbind())
            }
            CifValue::Table(map) => {
                // Convert HashMap to Python dict
                let py_dict: HashMap<String, Py<PyAny>> = map
                    .iter()
                    .map(|(k, v)| Ok((k.clone(), PyValue::from(v.clone()).to_python(py)?)))
                    .collect::<PyResult<HashMap<_, _>>>()?;
                Ok(py_dict.into_pyobject(py)?.into_any().unbind())
            }
        }
    }

    /// String representation
    fn __str__(&self) -> String {
        match &self.inner {
            CifValue::Text(s) => format!("'{s}'"),
            CifValue::Numeric(n) => n.to_string(),
            CifValue::Unknown => "?".to_string(),
            CifValue::NotApplicable => ".".to_string(),
            CifValue::List(values) => {
                let items: Vec<String> = values
                    .iter()
                    .map(|v| PyValue::from(v.clone()).__str__())
                    .collect();
                format!("[{}]", items.join(" "))
            }
            CifValue::Table(map) => {
                let items: Vec<String> = map
                    .iter()
                    .map(|(k, v)| format!("{}:{}", k, PyValue::from(v.clone()).__str__()))
                    .collect();
                format!("{{{}}}", items.join(" "))
            }
        }
    }

    /// Debug representation
    fn __repr__(&self) -> String {
        format!("Value({})", self.__str__())
    }

    /// Python equality
    fn __eq__(&self, other: &PyValue) -> bool {
        self.inner == other.inner
    }
}

impl From<CifValue> for PyValue {
    fn from(value: CifValue) -> Self {
        PyValue { inner: value }
    }
}

/// Python wrapper for CifLoop with Pythonic interface
#[pyclass(name = "Loop")]
#[derive(Clone)]
pub struct PyLoop {
    inner: CifLoop,
}

#[pymethods]
impl PyLoop {
    /// Get the column tags (headers)
    #[getter]
    fn tags(&self) -> Vec<String> {
        self.inner.tags.clone()
    }

    /// Get the number of rows
    fn __len__(&self) -> usize {
        self.inner.len()
    }

    /// Get the number of columns
    #[getter]
    fn num_columns(&self) -> usize {
        self.inner.tags.len()
    }

    /// Check if the loop is empty
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get a value by row and column index
    fn get(&self, row: usize, col: usize) -> Option<PyValue> {
        self.inner.get(row, col).map(|v| v.clone().into())
    }

    /// Get a value by row index and tag name
    fn get_by_tag(&self, row: usize, tag: &str) -> Option<PyValue> {
        self.inner.get_by_tag(row, tag).map(|v| v.clone().into())
    }

    /// Get all values for a specific tag as a list
    fn get_column(&self, tag: &str) -> Option<Vec<PyValue>> {
        self.inner
            .get_column(tag)
            .map(|values| values.iter().map(|v| (*v).clone().into()).collect())
    }

    /// Iterate over rows
    fn rows(&self) -> Vec<Vec<PyValue>> {
        self.inner
            .values
            .iter()
            .map(|row| row.iter().map(|v| v.clone().into()).collect())
            .collect()
    }

    /// Get a row as a dictionary mapping tags to values
    fn get_row_dict(&self, row: usize) -> Option<HashMap<String, PyValue>> {
        if row >= self.inner.len() {
            return None;
        }

        let mut result = HashMap::new();
        for (col, tag) in self.inner.tags.iter().enumerate() {
            if let Some(value) = self.inner.get(row, col) {
                result.insert(tag.clone(), value.clone().into());
            }
        }
        Some(result)
    }

    /// Python iterator protocol
    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<PyLoopIterator>> {
        let py = slf.py();
        let loop_clone = slf.clone();
        Py::new(
            py,
            PyLoopIterator {
                loop_: Py::new(py, loop_clone)?,
                index: 0,
            },
        )
    }

    /// String representation
    fn __str__(&self) -> String {
        format!(
            "Loop({} columns, {} rows)",
            self.inner.tags.len(),
            self.inner.len()
        )
    }

    /// Debug representation
    fn __repr__(&self) -> String {
        format!(
            "Loop(tags={:?}, rows={})",
            self.inner.tags,
            self.inner.len()
        )
    }
}

impl From<CifLoop> for PyLoop {
    fn from(loop_: CifLoop) -> Self {
        PyLoop { inner: loop_ }
    }
}

/// Iterator for PyLoop that yields row dictionaries
#[pyclass]
struct PyLoopIterator {
    loop_: Py<PyLoop>,
    index: usize,
}

#[pymethods]
impl PyLoopIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<HashMap<String, PyValue>> {
        let py = slf.py();
        let current_index = slf.index;

        // Get the row dict in a scope so the borrow is dropped before we mutate slf.index
        let result = {
            let loop_ = slf.loop_.borrow(py);
            if current_index < loop_.inner.len() {
                loop_.get_row_dict(current_index)
            } else {
                None
            }
        };

        // Now we can safely increment index after the borrow is dropped
        if result.is_some() {
            slf.index += 1;
        }

        result
    }
}

/// Python wrapper for CifFrame
#[pyclass(name = "Frame")]
#[derive(Clone)]
pub struct PyFrame {
    inner: CifFrame,
}

#[pymethods]
impl PyFrame {
    /// Get the frame name
    #[getter]
    fn name(&self) -> String {
        self.inner.name.clone()
    }

    /// Get all item keys
    #[getter]
    fn item_keys(&self) -> Vec<String> {
        self.inner.items.keys().cloned().collect()
    }

    /// Get an item by key
    fn get_item(&self, key: &str) -> Option<PyValue> {
        self.inner.items.get(key).map(|v| v.clone().into())
    }

    /// Get all items as a dictionary
    fn items(&self) -> HashMap<String, PyValue> {
        self.inner
            .items
            .iter()
            .map(|(k, v)| (k.clone(), v.clone().into()))
            .collect()
    }

    /// Get the number of loops
    #[getter]
    fn num_loops(&self) -> usize {
        self.inner.loops.len()
    }

    /// Get a loop by index
    fn get_loop(&self, index: usize) -> Option<PyLoop> {
        self.inner.loops.get(index).map(|l| l.clone().into())
    }

    /// Get all loops
    #[getter]
    fn loops(&self) -> Vec<PyLoop> {
        self.inner.loops.iter().map(|l| l.clone().into()).collect()
    }

    /// String representation
    fn __str__(&self) -> String {
        format!(
            "Frame('{}', {} items, {} loops)",
            self.inner.name,
            self.inner.items.len(),
            self.inner.loops.len()
        )
    }

    /// Debug representation
    fn __repr__(&self) -> String {
        format!(
            "Frame(name='{}', items={}, loops={})",
            self.inner.name,
            self.inner.items.len(),
            self.inner.loops.len()
        )
    }
}

impl From<CifFrame> for PyFrame {
    fn from(frame: CifFrame) -> Self {
        PyFrame { inner: frame }
    }
}

/// Python wrapper for CifBlock with Pythonic interface
#[pyclass(name = "Block")]
#[derive(Clone)]
pub struct PyBlock {
    inner: CifBlock,
}

#[pymethods]
impl PyBlock {
    /// Get the block name
    #[getter]
    fn name(&self) -> String {
        self.inner.name.clone()
    }

    /// Get all item keys
    #[getter]
    fn item_keys(&self) -> Vec<String> {
        self.inner.items.keys().cloned().collect()
    }

    /// Get an item by key
    fn get_item(&self, key: &str) -> Option<PyValue> {
        self.inner.items.get(key).map(|v| v.clone().into())
    }

    /// Get all items as a dictionary
    fn items(&self) -> HashMap<String, PyValue> {
        self.inner
            .items
            .iter()
            .map(|(k, v)| (k.clone(), v.clone().into()))
            .collect()
    }

    /// Get the number of loops
    #[getter]
    fn num_loops(&self) -> usize {
        self.inner.loops.len()
    }

    /// Get a loop by index
    fn get_loop(&self, index: usize) -> Option<PyLoop> {
        self.inner.loops.get(index).map(|l| l.clone().into())
    }

    /// Find a loop containing a specific tag
    fn find_loop(&self, tag: &str) -> Option<PyLoop> {
        self.inner.find_loop(tag).map(|l| l.clone().into())
    }

    /// Get all loops
    #[getter]
    fn loops(&self) -> Vec<PyLoop> {
        self.inner.loops.iter().map(|l| l.clone().into()).collect()
    }

    /// Get all loop tags
    fn get_loop_tags(&self) -> Vec<String> {
        self.inner.get_loop_tags().into_iter().cloned().collect()
    }

    /// Get the number of frames
    #[getter]
    fn num_frames(&self) -> usize {
        self.inner.frames.len()
    }

    /// Get a frame by index
    fn get_frame(&self, index: usize) -> Option<PyFrame> {
        self.inner.frames.get(index).map(|f| f.clone().into())
    }

    /// Get all frames
    #[getter]
    fn frames(&self) -> Vec<PyFrame> {
        self.inner.frames.iter().map(|f| f.clone().into()).collect()
    }

    /// String representation
    fn __str__(&self) -> String {
        format!(
            "Block('{}', {} items, {} loops, {} frames)",
            self.inner.name,
            self.inner.items.len(),
            self.inner.loops.len(),
            self.inner.frames.len()
        )
    }

    /// Debug representation
    fn __repr__(&self) -> String {
        format!(
            "Block(name='{}', items={}, loops={}, frames={})",
            self.inner.name,
            self.inner.items.len(),
            self.inner.loops.len(),
            self.inner.frames.len()
        )
    }
}

impl From<CifBlock> for PyBlock {
    fn from(block: CifBlock) -> Self {
        PyBlock { inner: block }
    }
}

/// Python wrapper for CifDocument with Pythonic interface
#[pyclass(name = "Document")]
#[derive(Clone)]
pub struct PyDocument {
    inner: CifDocument,
}

#[pymethods]
impl PyDocument {
    /// Parse a CIF string
    #[staticmethod]
    fn parse(content: &str) -> PyResult<PyDocument> {
        CifDocument::parse(content)
            .map(|doc| PyDocument { inner: doc })
            .map_err(cif_error_to_py_err)
    }

    /// Parse a CIF file
    #[staticmethod]
    fn from_file(path: &str) -> PyResult<PyDocument> {
        CifDocument::from_file(path)
            .map(|doc| PyDocument { inner: doc })
            .map_err(cif_error_to_py_err)
    }

    /// Get the CIF version of this document
    ///
    /// Returns the detected or explicitly set CIF version.
    /// CIF 2.0 is indicated by the `#\#CIF_2.0` magic header.
    /// Documents without this header default to CIF 1.1.
    #[getter]
    fn version(&self) -> PyVersion {
        self.inner.version.into()
    }

    /// Check if this document is CIF 2.0
    ///
    /// CIF 2.0 adds support for lists, tables, and other advanced features.
    fn is_cif2(&self) -> bool {
        matches!(self.inner.version, CifVersion::V2_0)
    }

    /// Check if this document is CIF 1.1
    fn is_cif1(&self) -> bool {
        matches!(self.inner.version, CifVersion::V1_1)
    }

    /// Get the number of blocks
    fn __len__(&self) -> usize {
        self.inner.blocks.len()
    }

    /// Get a block by index
    fn get_block(&self, index: usize) -> Option<PyBlock> {
        self.inner.blocks.get(index).map(|b| b.clone().into())
    }

    /// Get a block by name
    fn get_block_by_name(&self, name: &str) -> Option<PyBlock> {
        self.inner.get_block(name).map(|b| b.clone().into())
    }

    /// Get the first block
    fn first_block(&self) -> Option<PyBlock> {
        self.inner.first_block().map(|b| b.clone().into())
    }

    /// Get all blocks
    #[getter]
    fn blocks(&self) -> Vec<PyBlock> {
        self.inner.blocks.iter().map(|b| b.clone().into()).collect()
    }

    /// Get all block names
    #[getter]
    fn block_names(&self) -> Vec<String> {
        self.inner.blocks.iter().map(|b| b.name.clone()).collect()
    }

    /// Python iterator protocol
    fn __iter__(slf: PyRef<'_, Self>) -> PyDocumentIterator {
        PyDocumentIterator {
            doc: slf.clone(),
            index: 0,
        }
    }

    /// Python getitem protocol (allows doc[0], doc["name"])
    fn __getitem__(&self, key: &Bound<'_, PyAny>) -> PyResult<PyBlock> {
        // Try to extract as signed integer first to handle negative indices
        if let Ok(index) = key.extract::<isize>() {
            let len = self.inner.blocks.len() as isize;
            let actual_index = if index < 0 {
                // Python-style negative indexing
                let positive_index = len + index;
                if positive_index < 0 {
                    return Err(PyIndexError::new_err("Block index out of range"));
                }
                positive_index as usize
            } else {
                index as usize
            };

            self.inner
                .blocks
                .get(actual_index)
                .map(|b| b.clone().into())
                .ok_or_else(|| PyIndexError::new_err("Block index out of range"))
        } else if let Ok(name) = key.extract::<String>() {
            self.inner
                .get_block(&name)
                .map(|b| b.clone().into())
                .ok_or_else(|| PyKeyError::new_err(format!("Block '{name}' not found")))
        } else {
            Err(PyTypeError::new_err("Block key must be int or str"))
        }
    }

    /// String representation
    fn __str__(&self) -> String {
        format!("Document({} blocks)", self.inner.blocks.len())
    }

    /// Debug representation
    fn __repr__(&self) -> String {
        let names: Vec<&str> = self.inner.blocks.iter().map(|b| b.name.as_str()).collect();
        format!("Document(blocks={names:?})")
    }
}

/// Iterator for PyDocument
#[pyclass]
pub struct PyDocumentIterator {
    doc: PyDocument,
    index: usize,
}

#[pymethods]
impl PyDocumentIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self) -> Option<PyBlock> {
        if self.index < self.doc.inner.blocks.len() {
            let block = self.doc.inner.blocks[self.index].clone().into();
            self.index += 1;
            Some(block)
        } else {
            None
        }
    }
}

/// Module initialization function
#[pymodule]
fn _cif_parser(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyVersion>()?;
    m.add_class::<PyDocument>()?;
    m.add_class::<PyDocumentIterator>()?;
    m.add_class::<PyBlock>()?;
    m.add_class::<PyLoop>()?;
    m.add_class::<PyLoopIterator>()?;
    m.add_class::<PyFrame>()?;
    m.add_class::<PyValue>()?;

    // Convenience functions
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(parse_file, m)?)?;

    // Module metadata
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__author__", "Iain Maitland")?;
    m.add("__doc__", "CIF (Crystallographic Information File) parser")?;

    Ok(())
}

/// Convenience function for parsing CIF content
#[pyfunction]
fn parse(content: &str) -> PyResult<PyDocument> {
    PyDocument::parse(content)
}

/// Convenience function for parsing CIF files
#[pyfunction]
fn parse_file(path: &str) -> PyResult<PyDocument> {
    PyDocument::from_file(path)
}
