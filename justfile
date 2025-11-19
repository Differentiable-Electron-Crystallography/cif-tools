# justfile for cif-parser
# A multi-language CIF parser with Rust, Python, and JavaScript bindings
#
# Install just: cargo install just  (or: brew install just)
# List recipes: just --list
# Run recipe:   just <recipe-name>

# ============================================================================
# Configuration
# ============================================================================
set shell := ["bash", "-cu"]

# Variables
python_dir := "python"
js_dir := "javascript"

# ============================================================================
# Default & Help
# ============================================================================

# Show available recipes
default:
    @just --list

# ============================================================================
# Rust Recipes
# ============================================================================

# Format Rust code
rust-fmt:
    cargo fmt

# Check Rust formatting
rust-fmt-check:
    cargo fmt -- --check

# Lint Rust code with clippy
rust-clippy:
    cargo clippy --all-features -- -D warnings

# Run Rust tests
rust-test:
    cargo test --quiet

# Build Rust library
rust-build:
    cargo build --release

# Check all Rust code (format, lint, test)
check-rust: rust-fmt-check rust-clippy rust-test
    @echo "✅ Rust checks passed"

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

# Run Python tests
python-test:
    cd {{python_dir}} && uv run pytest tests/ -q

# Build Python package with maturin
python-build:
    cd {{python_dir}} && uv run maturin build --release

# Check all Python code (format, lint, typecheck, test)
check-python: python-fmt-check python-lint python-typecheck python-test
    @echo "✅ Python checks passed"

# ============================================================================
# JavaScript/WASM Recipes
# ============================================================================

# Build WASM package for Node.js (required for JS tests)
wasm-build:
    wasm-pack build --target nodejs --out-dir {{js_dir}}/pkg-node

# Build WASM package for web
wasm-build-web:
    wasm-pack build --target web --out-dir {{js_dir}}/pkg

# Build WASM package for bundler
wasm-build-bundler:
    wasm-pack build --target bundler --out-dir {{js_dir}}/pkg-bundler

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

# Clean all build artifacts
clean:
    cargo clean
    rm -rf {{python_dir}}/target
    rm -rf {{js_dir}}/pkg-node {{js_dir}}/pkg {{js_dir}}/pkg-bundler
    find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
    find . -type d -name ".pytest_cache" -exec rm -rf {} + 2>/dev/null || true
    find . -type d -name ".mypy_cache" -exec rm -rf {} + 2>/dev/null || true
    find . -type d -name ".ruff_cache" -exec rm -rf {} + 2>/dev/null || true
    @echo "✅ Cleaned all build artifacts"

# Install/setup all dependencies
setup:
    @echo "📦 Installing Rust dependencies..."
    cargo fetch
    @echo "📦 Installing Python dependencies..."
    cd {{python_dir}} && uv sync
    @echo "📦 Installing JavaScript dependencies..."
    cd {{js_dir}} && npm install
    @echo "✅ Setup complete"

# Install git hooks
install-hooks:
    ./install-hooks.sh
