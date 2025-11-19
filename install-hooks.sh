#!/bin/bash
# Install git hooks for the cif-parser project

echo "Installing git hooks..."

# Configure git to use the .githooks directory
git config core.hooksPath .githooks

if [ $? -eq 0 ]; then
    echo "✅ Git hooks installed successfully!"
    echo ""
    echo "The pre-commit hook will now run automatically before each commit."
    echo "It will check:"
    echo "  - Rust: formatting (cargo fmt) and linting (clippy)"
    echo "  - Python: formatting (black), linting (ruff), and type checking (mypy)"
    echo "  - JavaScript: formatting and linting (biome)"
    echo ""
    echo "To bypass the hooks when needed, use: git commit --no-verify"
else
    echo "❌ Failed to install git hooks"
    exit 1
fi
