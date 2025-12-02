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
python_packages := "cif-parser cif-validator"
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
# Python Recipes (uv workspace)
# ============================================================================

# Sync workspace dependencies (installs all packages in editable mode)
python-sync:
    cd {{python_dir}} && uv sync --extra dev

# Build native extensions for a specific package
python-develop pkg:
    cd {{python_dir}}/{{pkg}} && uv run maturin develop

# Build native extensions for all packages
python-develop-all: python-sync
    for pkg in {{python_packages}}; do just python-develop $pkg; done

# Run tests for a specific package
python-test pkg: (python-develop pkg)
    cd {{python_dir}}/{{pkg}} && uv run pytest tests/ -q

# Run all Python tests
python-test-all: python-develop-all
    cd {{python_dir}} && uv run pytest */tests/ -q

# Build wheel for a specific package
python-build pkg:
    cd {{python_dir}}/{{pkg}} && uv run maturin build --release --out dist

# Build wheels for all packages
python-build-all:
    for pkg in {{python_packages}}; do just python-build $pkg; done

# Format all Python code (runs from workspace root)
python-fmt:
    cd {{python_dir}} && uv run black .

# Check Python formatting
python-fmt-check:
    cd {{python_dir}} && uv run black --check .

# Lint all Python code
python-lint:
    cd {{python_dir}} && uv run ruff check .

# Fix Python linting issues
python-lint-fix:
    cd {{python_dir}} && uv run ruff check --fix .

# Type check all Python code
python-typecheck:
    cd {{python_dir}} && uv run mypy .

# Clean Python build artifacts
python-clean:
    for pkg in {{python_packages}}; do \
        find {{python_dir}}/$pkg/src -name "*.so" -delete 2>/dev/null || true; \
        rm -rf {{python_dir}}/$pkg/dist 2>/dev/null || true; \
    done
    rm -rf target/maturin 2>/dev/null || true

# Full Python check (format, lint, typecheck, test)
check-python: python-fmt-check python-lint python-typecheck python-test-all
    @echo "✅ Python checks passed"

# ============================================================================
# JavaScript/WASM Recipes (npm workspace)
# ============================================================================

# Build parser WASM for Node.js (required for JS tests)
wasm-build-parser:
    wasm-pack build {{parser_crate}} --target nodejs --out-dir ../../javascript/packages/cif-parser/pkg-node

# Build parser WASM for web
wasm-build-parser-web:
    wasm-pack build {{parser_crate}} --target web --out-dir ../../javascript/packages/cif-parser/pkg

# Build parser WASM for bundler
wasm-build-parser-bundler:
    wasm-pack build {{parser_crate}} --target bundler --out-dir ../../javascript/packages/cif-parser/pkg-bundler

# Build validator WASM for Node.js
wasm-build-validator:
    wasm-pack build {{validator_crate}} --target nodejs --out-dir ../../javascript/packages/cif-validator/pkg-node --features wasm

# Build validator WASM for web
wasm-build-validator-web:
    wasm-pack build {{validator_crate}} --target web --out-dir ../../javascript/packages/cif-validator/pkg --features wasm

# Build validator WASM for bundler
wasm-build-validator-bundler:
    wasm-pack build {{validator_crate}} --target bundler --out-dir ../../javascript/packages/cif-validator/pkg-bundler --features wasm

# Build all WASM for Node.js (alias for backwards compatibility)
wasm-build: wasm-build-parser wasm-build-validator
    @echo "✅ WASM Node.js builds complete"

# Build all WASM for web
wasm-build-web: wasm-build-parser-web wasm-build-validator-web
    @echo "✅ WASM web builds complete"

# Build all WASM for bundler
wasm-build-bundler: wasm-build-parser-bundler wasm-build-validator-bundler
    @echo "✅ WASM bundler builds complete"

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

# Run JavaScript tests (builds parser WASM first)
js-test: wasm-build-parser
    cd {{js_dir}} && npm test --workspaces --if-present

# Start Vite demo dev server
js-dev: wasm-build-parser-web
    cd {{js_dir}} && npm run dev

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
test: rust-test python-test-all js-test
    @echo "✅ All tests passed"

# Run all CI checks (for pre-commit hook)
ci: rust-fmt-check rust-clippy python-sync python-fmt-check python-lint python-typecheck js-check-ci rust-test python-test-all js-test
    @echo "✅ All CI checks passed"

# Build all release artifacts
build-all: rust-build python-build-all wasm-build-all
    @echo "✅ All builds complete"

# ============================================================================
# Benchmarking & Performance
# ============================================================================

# Run all benchmarks (uses criterion)
bench:
    cargo bench

# Run parser benchmarks only
bench-parser:
    cargo bench -p cif-parser

# Run validator benchmarks only
bench-validator:
    cargo bench -p cif-validator

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
