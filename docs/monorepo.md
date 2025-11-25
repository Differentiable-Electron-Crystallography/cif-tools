# Monorepo Architecture

cif-tools is a polyglot monorepo containing Rust libraries with bindings for Python and JavaScript/WebAssembly. The core parsing and validation logic is written in Rust, with language-specific bindings exposing the API to Python and JavaScript.

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
│   │       └── parser/        # Parser implementation
│   └── cif-validator/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs         # Validation logic
│           └── python.rs      # PyO3 bindings
├── python/                    # Python workspace (uv)
│   ├── pyproject.toml         # Workspace root
│   ├── uv.lock                # Shared lockfile
│   ├── .venv/                 # Shared virtual environment
│   ├── cif-parser/            # cif-parser Python package
│   │   ├── pyproject.toml
│   │   ├── src/cif_parser/
│   │   └── tests/
│   └── cif-validator/         # cif-validator Python package
│       ├── pyproject.toml
│       ├── src/cif_validator/
│       └── tests/
├── javascript/                # JavaScript package
│   ├── package.json
│   ├── tests/
│   └── pkg-node/              # WASM build output (generated)
├── fixtures/                  # Shared test fixtures
└── docs/
```

## Rust Workspace

The workspace is defined in the root `Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = [
    "crates/cif-parser",
    "crates/cif-validator",
]
```

### Crates

| Crate | Description | Status |
|-------|-------------|--------|
| `cif-parser` | Core CIF parsing (syntax, AST, semantic rules) | Implemented |
| `cif-validator` | DDLm-based CIF validation | In Development |

### Shared Configuration

The workspace defines shared settings in `[workspace.package]` and common dependencies in `[workspace.dependencies]`. Crates inherit these:

```toml
[package]
version.workspace = true
edition.workspace = true
```

## Language Bindings

Each Rust crate can produce bindings for multiple targets:

### Python (via PyO3 + Maturin + uv Workspace)

The Python bindings use a **uv workspace** to manage multiple Python packages that each wrap a Rust crate.

**Architecture:**
```
python/
├── pyproject.toml              # Workspace root (defines members)
├── uv.lock                     # Single lockfile for all packages
├── .venv/                      # Shared virtual environment
├── cif-parser/
│   ├── pyproject.toml          # Points to ../../crates/cif-parser
│   ├── src/cif_parser/
│   │   ├── __init__.py         # Re-exports from native module
│   │   ├── __init__.pyi        # Type stubs
│   │   ├── _cif_parser.pyi     # Native module type stubs
│   │   └── py.typed            # PEP 561 marker
│   └── tests/
└── cif-validator/
    ├── pyproject.toml          # Points to ../../crates/cif-validator
    ├── src/cif_validator/
    │   ├── __init__.py
    │   ├── __init__.pyi
    │   ├── _cif_validator.pyi
    │   └── py.typed
    └── tests/
```

**Key Configuration:**

Each Python package's `pyproject.toml` points to its Rust crate:
```toml
[tool.maturin]
features = ["pyo3/extension-module", "python"]
python-source = "src"
module-name = "cif_parser._cif_parser"
manifest-path = "../../crates/cif-parser/Cargo.toml"
```

The workspace root defines members and allows packages to depend on each other:
```toml
[tool.uv.workspace]
members = ["cif-parser", "cif-validator"]

[tool.uv.sources]
cif-parser = { workspace = true }
cif-validator = { workspace = true }
```

**Benefits of this setup:**
- Single `uv sync` installs all packages in editable mode
- Shared lockfile ensures consistent dependencies across packages
- Packages can depend on each other (e.g., `cif-validator` depends on `cif-parser`)
- Dev tools (pytest, mypy, black, ruff) installed once at workspace level
- Each package builds independently with maturin

### JavaScript/WASM (via wasm-bindgen + wasm-pack)

- Bindings defined in `src/wasm.rs` within each crate
- Built with `wasm-pack` to `javascript/pkg-node/`
- Supports multiple targets: nodejs, web, bundler

## Build System

All builds are orchestrated through `just` (install: `cargo install just` or `brew install just`).

### Quick Reference

```bash
just --list              # Show all recipes
just setup               # Install dependencies
just ci                  # Run all CI checks
just build-all           # Build all artifacts
```

### Rust

```bash
just rust-fmt            # Format
just rust-clippy         # Lint
just rust-test           # Test workspace
just rust-test-parser    # Test parser only
just check-rust          # All Rust checks
```

### Python

```bash
# Workspace management
just python-sync                    # Install all packages (uv sync)

# Build native extensions
just python-develop cif-parser      # Build cif-parser extension
just python-develop cif-validator   # Build cif-validator extension
just python-develop-all             # Build all extensions

# Testing
just python-test cif-parser         # Test specific package
just python-test-all                # Test all packages

# Build wheels
just python-build cif-parser        # Build wheel for specific package
just python-build-all               # Build wheels for all packages

# Code quality
just python-fmt                     # Format with black
just python-lint                    # Lint with ruff
just python-typecheck               # Type check with mypy
just check-python                   # All Python checks
```

### JavaScript

```bash
just wasm-build          # Build for Node.js
just wasm-build-web      # Build for web
just wasm-build-bundler  # Build for bundlers
just js-test             # Run tests
just check-js            # All JS checks
```

## Package Outputs

| Package | Language | Registry | Source |
|---------|----------|----------|--------|
| `cif-parser` | Rust | crates.io | `crates/cif-parser` |
| `cif-parser` | Python | PyPI | `python/cif-parser/` |
| `@cif-parser/node` | JavaScript | npm | `javascript/` |
| `cif-validator` | Rust | crates.io | `crates/cif-validator` |
| `cif-validator` | Python | PyPI | `python/cif-validator/` |
| `cif-validator` | JavaScript | npm | (planned) |

## CI/CD

All CI workflows use `just` recipes to ensure parity between local and CI builds:

- **test.yml** - Runs `just rust-test`, `just python-test`, `just js-test`
- **lint-and-format.yml** - Runs `just ci`
- **publish-python.yml** - Runs `just python-build`, `just python-sdist`
- **publish-npm.yml** - Runs `just wasm-build-all`

## Development Workflow

### Initial Setup

```bash
git clone https://github.com/Differentiable-Electron-Crystallography/cif-tools
cd cif-tools
just setup
```

### Making Changes

1. Edit Rust source in `crates/*/src/`
2. Run `just check-rust` to verify
3. Run `just python-develop <package>` to rebuild Python extension (e.g., `just python-develop cif-parser`)
4. Run `just python-test <package>` to test Python bindings (e.g., `just python-test cif-parser`)
5. Run `just js-test` to test JavaScript bindings

### Pre-commit

```bash
just ci
```

## Adding a New Crate

### 1. Create the Rust Crate

```bash
mkdir -p crates/<name>/src
```

Add to workspace members in root `Cargo.toml`:
```toml
[workspace]
members = [
    "crates/cif-parser",
    "crates/cif-validator",
    "crates/<name>",  # Add new crate
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

[lib]
crate-type = ["cdylib", "rlib"]

[features]
default = []
python = ["pyo3"]
```

### 2. Add Python Bindings (Optional)

Create `crates/<name>/src/python.rs` with PyO3 bindings.

Add to `crates/<name>/src/lib.rs`:
```rust
#[cfg(feature = "python")]
pub mod python;
```

### 3. Create Python Package

Create directory structure:
```
python/<name>/
├── pyproject.toml
├── src/<name_underscored>/
│   ├── __init__.py
│   ├── __init__.pyi
│   ├── _<name_underscored>.pyi
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
requires-python = ">=3.8,<3.13"
dynamic = ["version"]

[tool.maturin]
features = ["pyo3/extension-module", "python"]
python-source = "src"
module-name = "<name_underscored>._<name_underscored>"
manifest-path = "../../crates/<name>/Cargo.toml"
```

### 4. Update Workspace Configuration

Add to `python/pyproject.toml`:
```toml
[tool.uv.workspace]
members = ["cif-parser", "cif-validator", "<name>"]

[tool.uv.sources]
<name> = { workspace = true }
```

Add to `justfile`:
```just
python_packages := "cif-parser cif-validator <name>"
```

### 5. Build and Test

```bash
cd python && uv sync --extra dev
just python-develop <name>
just python-test <name>
```

### 6. Update CI Workflows

Update CI workflows to include the new package as needed.
