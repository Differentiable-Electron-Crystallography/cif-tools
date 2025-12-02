# Monorepo Architecture

cif-tools is a polyglot monorepo containing Rust libraries with bindings for Python and JavaScript/WebAssembly. The core parsing and validation logic is written in Rust, with language-specific bindings exposing the API to each target.

## Repository Structure

```
cif-tools/
├── Cargo.toml                 # Rust workspace root
├── justfile                   # Build orchestration
├── crates/                    # Rust crates
│   ├── cif-parser/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs         # Core Rust API
│   │       ├── python.rs      # PyO3 bindings
│   │       ├── wasm.rs        # WASM bindings
│   │       ├── cif.pest       # PEG grammar
│   │       ├── ast/           # AST types
│   │       ├── raw/           # Raw AST (Pass 1)
│   │       └── rules/         # Dialect rules (Pass 2)
│   ├── cif-validator/
│   │   └── src/
│   │       ├── lib.rs         # Validation logic
│   │       ├── python.rs      # PyO3 bindings
│   │       └── wasm.rs        # WASM bindings
│   └── drel-parser/
│       └── src/               # dREL expression parsing
├── python/                    # Python workspace (uv)
│   ├── pyproject.toml         # Workspace root
│   ├── uv.lock                # Shared lockfile
│   ├── cif-parser/            # cif-parser Python package
│   └── cif-validator/         # cif-validator Python package
├── javascript/                # JavaScript package
│   ├── package.json
│   └── pkg/                   # WASM build output (generated)
├── fixtures/                  # Shared test fixtures
└── docs/
```

---

## Rust Workspace

Defined in root `Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = [
    "crates/cif-parser",
    "crates/cif-validator",
    "crates/drel-parser",
]
```

### Crates

| Crate | Description |
|-------|-------------|
| `cif-parser` | Core CIF parsing (grammar, AST, dialect rules) |
| `cif-validator` | DDLm-based CIF validation |
| `drel-parser` | dREL expression parsing for dictionaries |

---

## Language Bindings

Each Rust crate can produce bindings for multiple targets via feature flags:

```toml
[features]
default = []
python = ["pyo3"]
wasm = ["wasm-bindgen"]
```

### Python (PyO3 + Maturin)

Architecture:
```
Python code
    ↓
PyO3 wrapper classes (src/python.rs)
    ↓
Core Rust library
```

**Wrapper pattern**: Each Rust type has a corresponding Python class:
- `CifDocument` → `Document`
- `CifBlock` → `Block`
- `CifLoop` → `Loop`
- `CifValue` → `Value`

**Python workspace** (uv-managed):
```
python/
├── pyproject.toml              # Workspace root
├── cif-parser/
│   ├── pyproject.toml          # Points to ../../crates/cif-parser
│   ├── src/cif_parser/
│   │   ├── __init__.py         # Re-exports from native module
│   │   ├── __init__.pyi        # Type stubs
│   │   └── py.typed            # PEP 561 marker
│   └── tests/
└── cif-validator/
    └── ...
```

### JavaScript/WASM (wasm-bindgen + wasm-pack)

Architecture:
```
JavaScript/TypeScript
    ↓
wasm-bindgen glue (generated)
    ↓
WASM module (compiled from src/wasm.rs)
    ↓
Core Rust library
```

**Wrapper pattern**: Each Rust type has a corresponding JS class:
- `CifDocument` → `JsCifDocument`
- `CifBlock` → `JsCifBlock`
- `CifLoop` → `JsCifLoop`
- `CifValue` → `JsCifValue`

**Build targets**:
- `web` - ES modules for browsers
- `nodejs` - CommonJS for Node.js
- `bundler` - For Webpack, Rollup, etc.

---

## Build System

All builds are orchestrated through `just` (install: `cargo install just`).

### Quick Reference

```bash
just --list              # Show all recipes
just setup               # Install dependencies
just ci                  # Run all CI checks
```

### Rust

```bash
just rust-fmt            # Format
just rust-clippy         # Lint
just rust-test           # Test workspace
```

### Python

```bash
# Workspace management
just python-sync                    # Install all packages (uv sync)

# Build native extensions
just python-develop cif-parser      # Build single package
just python-develop-all             # Build all packages

# Testing
just python-test cif-parser         # Test specific package
just python-test-all                # Test all packages

# Code quality
just python-fmt                     # Format with black
just python-lint                    # Lint with ruff
just python-typecheck               # Type check with mypy
```

### JavaScript/WASM

```bash
just wasm-build          # Build for Node.js
just wasm-build-web      # Build for web
just wasm-build-bundler  # Build for bundlers
just js-test             # Run tests
```

---

## API Design Principles

### Pythonic / JavaScript-idiomatic

Both bindings follow language conventions:

| Aspect | Python | JavaScript |
|--------|--------|------------|
| Naming | `snake_case` | `camelCase` |
| Properties | `loop.tags` | `loop.tags` |
| Length | `len(loop)` | `loop.numRows` |
| Iteration | `for row in loop:` | manual loop |
| Type check | `value.is_numeric` (property) | `value.is_numeric()` (method) |

### Consistent Patterns

Both languages share:
- Wrapper classes for all AST types
- `get_row_dict()` for loop row access
- `get_column()` for column extraction
- Type-checking methods/properties
- Error messages with source locations

### Example: Loop Access

**Python:**
```python
loop = block.find_loop("_atom_site_label")
for row in loop:
    print(row["_atom_site_label"])
```

**JavaScript:**
```javascript
const loop = block.find_loop("_atom_site_label");
for (let i = 0; i < loop.numRows; i++) {
    const row = loop.get_row_dict(i);
    console.log(row._atom_site_label);
}
```

---

## Development Workflow

### Initial Setup

```bash
git clone https://github.com/anthropics/cif-tools
cd cif-tools
just setup
```

### Making Changes

1. Edit Rust source in `crates/*/src/`
2. Run `just rust-test` to verify Rust
3. Run `just python-develop <package>` to rebuild Python extension
4. Run `just python-test <package>` to test Python
5. Run `just js-test` to test JavaScript

### Pre-commit

```bash
just ci
```

---

## Adding a New Crate

### 1. Create the Rust Crate

```bash
mkdir -p crates/<name>/src
```

Add to workspace in root `Cargo.toml`:
```toml
[workspace]
members = [
    "crates/cif-parser",
    "crates/cif-validator",
    "crates/<name>",
]
```

Create `crates/<name>/Cargo.toml`:
```toml
[package]
name = "<name>"
version.workspace = true
edition.workspace = true

[dependencies]
pyo3 = { workspace = true, optional = true }
wasm-bindgen = { workspace = true, optional = true }

[lib]
crate-type = ["cdylib", "rlib"]

[features]
default = []
python = ["pyo3"]
wasm = ["wasm-bindgen"]
```

### 2. Add Python Package

Create directory structure:
```
python/<name>/
├── pyproject.toml
├── src/<name_underscored>/
│   ├── __init__.py
│   ├── __init__.pyi
│   └── py.typed
└── tests/
```

Create `python/<name>/pyproject.toml`:
```toml
[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name = "<name>"
requires-python = ">=3.8"
dynamic = ["version"]

[tool.maturin]
features = ["pyo3/extension-module", "python"]
python-source = "src"
module-name = "<name_underscored>._<name_underscored>"
manifest-path = "../../crates/<name>/Cargo.toml"
```

### 3. Update Workspace

Add to `python/pyproject.toml`:
```toml
[tool.uv.workspace]
members = ["cif-parser", "cif-validator", "<name>"]
```

### 4. Build and Test

```bash
just python-sync
just python-develop <name>
just python-test <name>
```

---

## Package Outputs

| Package | Language | Registry | Source |
|---------|----------|----------|--------|
| `cif-parser` | Rust | crates.io | `crates/cif-parser` |
| `cif-parser` | Python | PyPI | `python/cif-parser/` |
| `@cif-tools/parser` | JavaScript | npm | `javascript/` |
| `cif-validator` | Rust | crates.io | `crates/cif-validator` |
| `cif-validator` | Python | PyPI | `python/cif-validator/` |

---

## CI/CD

All CI workflows use `just` recipes:

- **test.yml** - `just rust-test`, `just python-test-all`, `just js-test`
- **lint.yml** - `just ci`
- **publish-python.yml** - `just python-build-all`
- **publish-npm.yml** - `just wasm-build-all`

---

## Technology Choices

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Core language | Rust | Performance, safety, WASM target |
| Python bindings | PyO3 + Maturin | Industry standard, ergonomic API |
| WASM bindings | wasm-bindgen + wasm-pack | De facto standard for Rust→WASM |
| Python package manager | uv | Fast, modern, workspace support |
| Build orchestration | just | Simple, cross-platform task runner |
| Grammar parser | PEST | PEG grammars map well to CIF spec |

---

## References

- [PyO3 User Guide](https://pyo3.rs/)
- [Maturin User Guide](https://www.maturin.rs/)
- [wasm-bindgen Documentation](https://rustwasm.github.io/wasm-bindgen/)
- [wasm-pack Documentation](https://rustwasm.github.io/wasm-pack/)
- [uv Documentation](https://docs.astral.sh/uv/)
