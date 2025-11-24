# justfile for cif-tools workspace
# A multi-language CIF parser and validator with Rust, Python, and JavaScript bindings
#
# Install just: cargo install just  (or: brew install just)
# List recipes: just --list
# Run recipe:   just <recipe-name>

# ============================================================================
# Configuration
# ============================================================================

# Variables
python_dir := "python"
js_dir := "javascript"
parser_crate := "crates/cif-parser"
validator_crate := "crates/cif-validator"

# ============================================================================
# Default & Help
# ============================================================================

# Show available recipes
default:
    @just --list

# ============================================================================
# Rust Recipes (Workspace)
# ============================================================================

# Format Rust code (entire workspace)
rust-fmt:
    cargo fmt --all

# Check Rust formatting (entire workspace)
rust-fmt-check:
    cargo fmt --all -- --check

# Lint Rust code with clippy (entire workspace)
rust-clippy:
    cargo clippy --workspace --all-features -- -D warnings

# Run Rust tests (entire workspace)
rust-test:
    cargo test --workspace --quiet

# Build Rust library (entire workspace)
rust-build:
    cargo build --workspace --release

# Build parser only
rust-build-parser:
    cargo build -p cif-parser --release

# Build validator only
rust-build-validator:
    cargo build -p cif-validator --release

# Test parser only
rust-test-parser:
    cargo test -p cif-parser --quiet

# Test validator only
rust-test-validator:
    cargo test -p cif-validator --quiet

# Check all Rust code (format, lint, test)
check-rust: rust-fmt-check rust-clippy rust-test
    @echo "✅ Rust workspace checks passed"

# ============================================================================
# Python Recipes
# ============================================================================

# Format Python code with black
python-fmt:
    cd {{python_dir}} && uv run black .

# Check Python formatting
python-fmt-check:
    cd {{python_dir}} && uv run black --check .

# Lint Python code with ruff
python-lint:
    cd {{python_dir}} && uv run ruff check .

# Fix Python linting issues
python-lint-fix:
    cd {{python_dir}} && uv run ruff check --fix .

# Type check Python code with mypy
python-typecheck:
    cd {{python_dir}} && uv run mypy .

# Install Python package in development mode (editable install)
python-develop:
    cd {{python_dir}} && uv sync --extra dev && uv run maturin develop

# Run Python tests (builds extension first)
python-test: python-develop
    cd {{python_dir}} && uv run pytest tests/ -q

# Clean Python build artifacts and compiled extensions
python-clean:
    python -c "import pathlib, sys; files = [p for pattern in ['*.so', '*.pyd', '*.dll'] for p in pathlib.Path('{{python_dir}}/src/cif_parser').glob(pattern)]; [print(f'Removing {p}') or p.unlink() for p in files] if files else print('No build artifacts to clean')"
    @if [ -d "target/maturin" ]; then rm -rf target/maturin && echo "Removed target/maturin directory"; fi
    @if [ -d "{{python_dir}}/dist" ]; then rm -rf {{python_dir}}/dist && echo "Removed {{python_dir}}/dist directory"; fi
    @if [ -d "{{python_dir}}/build" ]; then rm -rf {{python_dir}}/build && echo "Removed {{python_dir}}/build directory"; fi

# Build Python package with maturin
python-build: python-clean
    cd {{python_dir}} && uv run maturin build --release

# Check all Python code (format, lint, typecheck, test)
check-python: python-fmt-check python-lint python-typecheck python-test
    @echo "✅ Python checks passed"

# ============================================================================
# JavaScript/WASM Recipes
# ============================================================================

# Build WASM package for Node.js (required for JS tests)
wasm-build:
    wasm-pack build {{parser_crate}} --target nodejs --out-dir ../../javascript/pkg-node

# Build WASM package for web
wasm-build-web:
    wasm-pack build {{parser_crate}} --target web --out-dir ../../javascript/pkg

# Build WASM package for bundler
wasm-build-bundler:
    wasm-pack build {{parser_crate}} --target bundler --out-dir ../../javascript/pkg-bundler

# Build all WASM targets
wasm-build-all: wasm-build wasm-build-web wasm-build-bundler
    @echo "✅ All WASM builds complete"

# Format JavaScript code with Biome
js-fmt:
    cd {{js_dir}} && npx @biomejs/biome format --write .

# Check JavaScript formatting
js-fmt-check:
    cd {{js_dir}} && npx @biomejs/biome format .

# Lint JavaScript code with Biome
js-lint:
    cd {{js_dir}} && npx @biomejs/biome lint .

# Check and fix JavaScript code with Biome
js-check:
    cd {{js_dir}} && npx @biomejs/biome check --write .

# Check JavaScript code (Biome CI mode)
js-check-ci:
    cd {{js_dir}} && npx @biomejs/biome ci .

# Run JavaScript tests (builds WASM first)
js-test: wasm-build
    cd {{js_dir}} && npx mocha tests/basic.test.js --reporter min

# Check all JavaScript code (format, lint, test)
check-js: js-check-ci js-test
    @echo "✅ JavaScript checks passed"

# ============================================================================
# Aggregate Recipes
# ============================================================================

# Format all code
fmt: rust-fmt python-fmt js-fmt
    @echo "✅ All code formatted"

# Check formatting for all code
fmt-check: rust-fmt-check python-fmt-check js-fmt-check
    @echo "✅ All formatting checks passed"

# Lint all code
lint: rust-clippy python-lint js-lint
    @echo "✅ All linting passed"

# Type check all code
typecheck: python-typecheck
    @echo "✅ Type checking passed"

# Run all tests
test: rust-test python-test js-test
    @echo "✅ All tests passed"

# Run all CI checks (for pre-commit hook)
ci: rust-fmt-check rust-clippy python-fmt-check python-lint python-typecheck js-check-ci rust-test python-test js-test
    @echo "✅ All CI checks passed"

# Build all release artifacts
build-all: rust-build python-build wasm-build-all
    @echo "✅ All builds complete"

# ============================================================================
# Development & Utilities
# ============================================================================

# Install/setup all dependencies
setup:
    @echo "📦 Installing Rust dependencies..."
    cargo fetch
    @echo "📦 Installing Python dependencies..."
    cd {{python_dir}} && uv sync --extra dev
    @echo "📦 Installing JavaScript dependencies..."
    cd {{js_dir}} && npm install
    @echo "✅ Setup complete"

# Install git hooks
install-hooks:
    ./install-hooks.sh
