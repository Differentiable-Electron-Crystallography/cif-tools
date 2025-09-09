# Python Bindings with Maturin and PyO3

This document provides comprehensive documentation on how the Python bindings for the CIF Parser are created using Maturin and PyO3.

## Table of Contents
- [Overview](#overview)
- [Architecture](#architecture)
- [Technology Stack](#technology-stack)
- [Directory Structure](#directory-structure)
- [How Maturin Works](#how-maturin-works)
- [PyO3 Integration](#pyo3-integration)
- [Type Mappings](#type-mappings)
- [Building and Development](#building-and-development)
- [Testing](#testing)
- [Debugging](#debugging)
- [Performance Considerations](#performance-considerations)
- [Publishing to PyPI](#publishing-to-pypi)
- [Troubleshooting](#troubleshooting)

## Overview

The Python bindings for CIF Parser allow Python developers to use the high-performance Rust-based CIF parser with a native Python API. This is achieved through:

- **PyO3**: A Rust crate that provides bindings to Python's C API
- **Maturin**: A build tool that handles packaging Rust code as Python extensions
- **Type conversion**: Automatic conversion between Rust and Python types

## Architecture

```
┌─────────────────────────────────────────┐
│           Python Application            │
├─────────────────────────────────────────┤
│         Python API Layer                │
│    (python/cif_parser/__init__.py)      │
├─────────────────────────────────────────┤
│      PyO3 Bindings (src/python.rs)     │
│   - PyValue, PyLoop, PyBlock, etc.     │
├─────────────────────────────────────────┤
│     Core Rust Library (src/lib.rs)     │
│   - CifDocument, CifBlock, CifValue    │
└─────────────────────────────────────────┘
```

## Technology Stack

### PyO3 (v0.21)
PyO3 provides:
- Rust bindings to Python's C API
- Automatic reference counting
- Type conversion between Rust and Python
- Exception handling
- Python module creation

### Maturin (v1.0+)
Maturin provides:
- Build system for Rust-based Python extensions
- Wheel building for multiple platforms
- Development mode for quick iteration
- PyPI publishing support

## Directory Structure

```
cif-parser/
├── Cargo.toml                 # Rust package manifest
├── pyproject.toml             # Python package configuration
├── src/
│   ├── lib.rs                 # Core Rust library
│   └── python.rs              # PyO3 bindings (conditionally compiled)
├── python/
│   └── cif_parser/
│       ├── __init__.py        # Python package entry point
│       ├── __init__.pyi       # Type stubs for IDE support
│       └── py.typed           # PEP 561 marker for type hints
└── tests/
    └── test_python.py         # Python tests
```

## How Maturin Works

### 1. Build Process

Maturin orchestrates the build process:

```bash
# Development build (fast, with debug symbols)
maturin develop --features python

# Release build (optimized)
maturin build --release --features python

# Build wheels for distribution
maturin build --release --features python --out dist
```

### 2. What Happens During Build

1. **Cargo Build**: Maturin invokes Cargo to compile the Rust code
2. **Library Creation**: Creates a dynamic library (`.so` on Linux, `.dylib` on macOS, `.pyd` on Windows)
3. **Wheel Packaging**: Packages the library with Python metadata into a wheel
4. **Installation**: Installs the wheel into the Python environment

### 3. Configuration

**pyproject.toml**:
```toml
[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[tool.maturin]
features = ["pyo3/extension-module", "python"]
python-source = "python"
module-name = "cif_parser._cif_parser"
```

**Cargo.toml**:
```toml
[dependencies]
pyo3 = { version = "0.21", features = ["extension-module"], optional = true }

[features]
python = ["pyo3"]

[lib]
crate-type = ["cdylib", "rlib"]  # cdylib for Python, rlib for Rust
```

## PyO3 Integration

### 1. Module Definition

```rust
// src/python.rs
use pyo3::prelude::*;

/// The main Python module
#[pymodule]
fn _cif_parser(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyDocument>()?;
    m.add_class::<PyBlock>()?;
    m.add_class::<PyLoop>()?;
    m.add_class::<PyValue>()?;
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(parse_file, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
```

### 2. Class Wrapping

```rust
/// Python wrapper for CifDocument
#[pyclass(name = "Document")]
pub struct PyDocument {
    inner: CifDocument,
}

#[pymethods]
impl PyDocument {
    /// Parse CIF content from a string
    #[staticmethod]
    fn parse(content: &str) -> PyResult<Self> {
        CifDocument::parse(content)
            .map(|doc| PyDocument { inner: doc })
            .map_err(cif_error_to_py_err)
    }
    
    /// Get the first block
    fn first_block(&self) -> Option<PyBlock> {
        self.inner.first_block()
            .map(|block| PyBlock { inner: block.clone() })
    }
    
    /// Python special methods
    fn __len__(&self) -> usize {
        self.inner.blocks.len()
    }
    
    fn __getitem__(&self, index: usize) -> PyResult<PyBlock> {
        self.inner.blocks.get(index)
            .map(|block| PyBlock { inner: block.clone() })
            .ok_or_else(|| PyIndexError::new_err("Index out of range"))
    }
}
```

### 3. Property Getters

```rust
#[pymethods]
impl PyValue {
    /// Check if this is a text value (property)
    #[getter]
    fn is_text(&self) -> bool {
        matches!(self.inner, CifValue::Text(_))
    }
    
    /// Get numeric value (property)
    #[getter]
    fn numeric(&self) -> Option<f64> {
        self.inner.as_numeric()
    }
}
```

## Type Mappings

### Rust to Python

| Rust Type | Python Type | Notes |
|-----------|-------------|-------|
| `String` | `str` | Automatic conversion |
| `&str` | `str` | Automatic conversion |
| `f64` | `float` | Direct mapping |
| `i32`, `i64` | `int` | Direct mapping |
| `bool` | `bool` | Direct mapping |
| `Vec<T>` | `list` | Converted to Python list |
| `HashMap<K, V>` | `dict` | Converted to Python dict |
| `Option<T>` | `T or None` | None for None variant |
| `Result<T, E>` | `T or Exception` | Err becomes Python exception |

### Python to Rust

| Python Type | Rust Type | Notes |
|-------------|-----------|-------|
| `str` | `String` or `&str` | Based on function signature |
| `float` | `f64` | Direct mapping |
| `int` | `i32`, `i64`, etc. | Based on function signature |
| `bool` | `bool` | Direct mapping |
| `list` | `Vec<T>` | Elements must be convertible |
| `dict` | `HashMap<K, V>` | Keys/values must be convertible |
| `None` | `Option::None` | When expecting Option<T> |

## Building and Development

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install maturin
pip install maturin

# Or using uv (recommended)
uv tool install maturin
```

### Development Workflow

1. **Make changes** to Rust code in `src/python.rs`

2. **Build in development mode**:
```bash
maturin develop --features python
```

3. **Test interactively**:
```python
import cif_parser
doc = cif_parser.parse("data_test\n_item value")
print(doc.first_block().get_item("_item"))
```

4. **Run tests**:
```bash
pytest tests/
```

### Building Wheels

```bash
# Build wheel for current platform
maturin build --release --features python

# Build wheels for multiple Python versions
maturin build --release --features python --interpreter python3.8 python3.9 python3.10 python3.11 python3.12

# Build manylinux wheels (Linux)
docker run --rm -v $(pwd):/io ghcr.io/pyo3/maturin build --release --features python

# Output is in target/wheels/
```

## Testing

### Unit Tests (Rust)

```rust
// src/python.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_value_conversion() {
        let rust_value = CifValue::Numeric(42.0);
        let py_value = PyValue { inner: rust_value };
        assert!(py_value.is_numeric());
        assert_eq!(py_value.numeric(), Some(42.0));
    }
}
```

### Integration Tests (Python)

```python
# tests/test_python.py
import cif_parser
import pytest

def test_parse_simple():
    content = """
    data_test
    _cell_length_a 10.0
    """
    doc = cif_parser.parse(content)
    block = doc.first_block()
    assert block.name == "test"
    assert block.get_item("_cell_length_a").numeric == 10.0

def test_parse_file():
    doc = cif_parser.parse_file("test_data/simple.cif")
    assert len(doc) > 0
```

## Debugging

### Enable Debug Output

```rust
// src/python.rs
use pyo3::Python;

#[pyfunction]
fn debug_parse(content: &str) -> PyResult<()> {
    Python::with_gil(|py| {
        // Print to Python's stdout
        py.run("print('Debug: Starting parse')", None, None)?;
        
        // Or use println! (goes to stderr)
        eprintln!("Rust debug: content length = {}", content.len());
        
        Ok(())
    })
}
```

### Common Issues and Solutions

1. **ImportError: dynamic module does not define module export function**
   - Solution: Ensure `#[pymodule]` function name matches `module-name` in pyproject.toml

2. **Symbol not found errors**
   - Solution: Rebuild with `maturin develop --release`

3. **Type conversion errors**
   - Solution: Check that Rust types implement `IntoPy<PyObject>`

4. **Memory leaks**
   - Solution: Use `Py<T>` for Python-managed objects, avoid circular references

## Performance Considerations

### 1. Minimize Python/Rust Boundary Crossings

```rust
// Bad: Multiple crossings
#[pymethods]
impl PyLoop {
    fn get_value(&self, row: usize, col: usize) -> Option<PyValue> {
        // Crosses boundary for each value
    }
}

// Good: Batch operations
#[pymethods]
impl PyLoop {
    fn get_row(&self, row: usize) -> Vec<PyValue> {
        // Single crossing for entire row
    }
}
```

### 2. Use References When Possible

```rust
// Avoid cloning when possible
#[pymethods]
impl PyDocument {
    // Returns reference-counted Python object
    fn get_block(&self, index: usize) -> Option<Py<PyBlock>> {
        // ...
    }
}
```

### 3. Release GIL for CPU-Intensive Operations

```rust
#[pyfunction]
fn parse_large_file(path: &str) -> PyResult<PyDocument> {
    Python::with_gil(|py| {
        // Release GIL during parsing
        py.allow_threads(|| {
            // CPU-intensive parsing here
            CifDocument::from_file(path)
        })
        .map(|doc| PyDocument { inner: doc })
        .map_err(cif_error_to_py_err)
    })
}
```

## Publishing to PyPI

### 1. Build Distribution Files

```bash
# Build source distribution
maturin sdist

# Build wheels for current platform
maturin build --release --features python

# Build universal wheel (if pure Python)
maturin build --release --features python --universal2
```

### 2. GitHub Actions Workflow

See `.github/workflows/publish-python.yml` for automated publishing.

### 3. Manual Publishing

```bash
# Test on TestPyPI first
maturin publish --repository testpypi

# Publish to PyPI
maturin publish
```

### 4. Platform-Specific Wheels

- **Linux**: Use manylinux Docker images
- **macOS**: Build on multiple macOS versions
- **Windows**: Build on Windows runners
- **Cross-compilation**: Use `cross` or `cargo-zigbuild`

## Troubleshooting

### Build Issues

**Problem**: `error: Microsoft Visual C++ 14.0 is required` (Windows)
```bash
# Solution: Install Visual Studio Build Tools
# Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/
```

**Problem**: `OSError: Python.h: No such file or directory`
```bash
# Solution: Install Python development headers
# Ubuntu/Debian:
sudo apt-get install python3-dev
# macOS:
brew install python@3.x
# Or use pyenv for version management
```

### Runtime Issues

**Problem**: `ImportError: libpython3.x.so.1: cannot open shared object file`
```bash
# Solution: Set LD_LIBRARY_PATH
export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:$(python3 -c "import sysconfig; print(sysconfig.get_config_var('LIBDIR'))")
```

**Problem**: Segmentation fault when using the module
```python
# Debug with:
import faulthandler
faulthandler.enable()
import cif_parser  # Will show stack trace on segfault
```

### Performance Issues

**Problem**: Slow import time
```bash
# Profile import:
python -X importtime -c "import cif_parser"

# Solution: Reduce module initialization work
# Move heavy operations to lazy initialization
```

**Problem**: High memory usage
```python
# Monitor memory:
import tracemalloc
tracemalloc.start()
# ... use cif_parser ...
snapshot = tracemalloc.take_snapshot()
top_stats = snapshot.statistics('lineno')
```

## Advanced Topics

### Custom Converters

```rust
use pyo3::conversion::{FromPyObject, IntoPy};

struct CustomType;

impl IntoPy<PyObject> for CustomType {
    fn into_py(self, py: Python) -> PyObject {
        // Custom conversion logic
    }
}
```

### Async Support

```rust
#[pyfunction]
#[pyo3(signature = (path, /))]
async fn parse_async(path: String) -> PyResult<PyDocument> {
    // Async parsing implementation
}
```

### Submodules

```rust
#[pymodule]
fn submodule(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Submodule content
}

#[pymodule]
fn _cif_parser(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let submod = PyModule::new(py, "utils")?;
    submodule(&submod)?;
    m.add_submodule(&submod)?;
}
```

## Resources

- [PyO3 Documentation](https://pyo3.rs/)
- [Maturin Documentation](https://maturin.rs/)
- [Python C API Reference](https://docs.python.org/3/c-api/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Python Packaging Guide](https://packaging.python.org/)