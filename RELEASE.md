# Release Process

This document describes how to release new versions of the CIF Parser library to various package managers.

## Prerequisites

### Required Secrets

Before releasing, ensure the following GitHub secrets are configured in the repository settings:

1. **NPM_TOKEN**: NPM authentication token for publishing packages
   - Get from: https://www.npmjs.com/settings/[username]/tokens
   - Create a "Classic Token" with "Automation" type
   - Add as repository secret: Settings → Secrets → Actions → New repository secret

2. **CARGO_REGISTRY_TOKEN**: Crates.io API token for Rust package publishing
   - Get from: https://crates.io/settings/tokens
   - Create a new token with publish permissions
   - Add as repository secret

### PyPI Setup (No Secret Required)

The Python package uses GitHub's trusted publishing feature:

1. Go to PyPI.org and create an account if needed
2. Navigate to your account settings → Publishing
3. Add a new trusted publisher:
   - Owner: `Differentiable-Electron-Crystallography`
   - Repository: `cif-parser`
   - Workflow: `publish-python.yml`
   - Environment: `pypi`

## Version Management

The version is managed in multiple places that must be kept in sync:

- `Cargo.toml` - Rust crate version
- `pyproject.toml` - Python package (reads from Cargo.toml)
- `pkg/package.json` - NPM web package (auto-generated)
- `pkg-node/package.json` - NPM Node.js package (auto-generated)

## Release Methods

### Method 1: Automated Release (Recommended)

1. **Update version** in `Cargo.toml`:
   ```toml
   version = "0.2.0"
   ```

2. **Commit and push** the version change:
   ```bash
   git add Cargo.toml
   git commit -m "Bump version to 0.2.0"
   git push origin main
   ```

3. **Create and push a version tag**:
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

This will automatically:
- Create a GitHub release
- Publish to crates.io
- Publish Python wheels to PyPI
- Publish WASM packages to NPM

### Method 2: Manual Workflow Trigger

1. Go to Actions → Release workflow
2. Click "Run workflow"
3. Enter the version number (e.g., `0.2.0`)
4. Click "Run workflow"

### Method 3: Individual Package Publishing

You can also trigger individual package publishing workflows:

- **Python/PyPI**: Actions → "Publish Python Package to PyPI" → Run workflow
- **NPM**: Actions → "Publish NPM Package" → Run workflow
- **Rust/Crates.io**: Actions → "Publish Rust Crate" → Run workflow

## Publishing Checklist

Before releasing:

- [ ] Update version in `Cargo.toml`
- [ ] Update CHANGELOG.md with release notes
- [ ] Run tests locally: `cargo test --all-features`
- [ ] Build Python package: `maturin build --features python`
- [ ] Build WASM: `wasm-pack build --target web`
- [ ] Commit all changes
- [ ] Create git tag: `git tag v0.x.x`

## Package Locations

Once published, packages will be available at:

- **Rust**: https://crates.io/crates/cif-parser
- **Python**: https://pypi.org/project/cif-parser/
- **NPM Web**: https://www.npmjs.com/package/@cif-parser/web
- **NPM Node**: https://www.npmjs.com/package/@cif-parser/node
- **NPM Bundler**: https://www.npmjs.com/package/@cif-parser/bundler

## Troubleshooting

### PyPI Publishing Issues

If PyPI publishing fails:
1. Check that the trusted publisher is configured correctly
2. Ensure the environment name is exactly `pypi`
3. Check that the workflow file name matches the configured trusted publisher

### NPM Publishing Issues

If NPM publishing fails:
1. Verify NPM_TOKEN is set correctly
2. Check if the package name already exists (may need scoping)
3. Ensure wasm-pack builds successfully

### Crates.io Publishing Issues

If crates.io publishing fails:
1. Verify CARGO_REGISTRY_TOKEN is valid
2. Check that the version hasn't been published already
3. Run `cargo publish --dry-run` locally to test

## Version Compatibility

- Python: Supports Python 3.8-3.12
- Node.js: Supports Node.js 14+
- Rust: MSRV (Minimum Supported Rust Version) is 1.70.0

## Rollback Procedure

If a release has issues:

1. **DO NOT** delete published packages (not possible on most registries)
2. Fix the issue in the code
3. Bump to a new patch version (e.g., 0.2.0 → 0.2.1)
4. Release the fixed version
5. Mark the problematic version as deprecated if the registry supports it