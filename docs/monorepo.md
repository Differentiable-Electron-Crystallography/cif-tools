# Monorepo Architecture

cif-tools is a polyglot monorepo containing Rust libraries with bindings for Python and JavaScript/WebAssembly. The core parsing and validation logic is written in Rust, with language-specific bindings exposing the API to Python and JavaScript.

## Repository Structure

```
cif-tools/
├── Cargo.toml                 # Workspace root
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
│           └── lib.rs
├── python/                    # Python package
│   ├── pyproject.toml
│   ├── README.md
│   └── src/cif_parser/
├── javascript/                # JavaScript package
│   ├── package.json
│   ├── tests/
│   └── pkg-node/              # WASM build output (generated)
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
| `cif-validator` | DDLm-based CIF validation | Planned |

### Shared Configuration

The workspace defines shared settings in `[workspace.package]` and common dependencies in `[workspace.dependencies]`. Crates inherit these:

```toml
[package]
version.workspace = true
edition.workspace = true
```

## Language Bindings

Each Rust crate can produce bindings for multiple targets:

### Python (via PyO3 + Maturin)

- Bindings defined in `src/python.rs` within each crate
- Built with `maturin` from the `python/` directory
- Uses src layout: source in `python/src/cif_parser/`
- Output: `python/dist/*.whl`

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
just python-develop      # Editable install
just python-test         # Run pytest
just python-build        # Build wheel to python/dist/
just python-sdist        # Build source distribution
just check-python        # All Python checks
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
| `cif-parser` | Python | PyPI | `python/` |
| `@cif-parser/node` | JavaScript | npm | `javascript/` |
| `cif-validator` | Rust | crates.io | `crates/cif-validator` |
| `cif-validator` | Python | PyPI | (planned) |
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
3. Run `just python-develop` to rebuild Python extension
4. Run `just python-test` to test Python bindings
5. Run `just js-test` to test JavaScript bindings

### Pre-commit

```bash
just ci
```

## Adding a New Crate

1. Create crate in `crates/<name>/`
2. Add to workspace members in root `Cargo.toml`
3. Define Python bindings in `src/python.rs` (optional feature)
4. Define WASM bindings in `src/wasm.rs` (optional feature)
5. Add justfile recipes for the new crate
6. Update CI workflows as needed
