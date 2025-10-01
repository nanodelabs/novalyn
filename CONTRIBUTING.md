# Contributing to changelogen-rs

Thank you for your interest in contributing to changelogen-rs! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Development Setup](#development-setup)
- [Development Workflow](#development-workflow)
- [Code Quality Standards](#code-quality-standards)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Project Goals](#project-goals)

## Development Setup

### Prerequisites

- **Rust**: 1.85.0 or later
- **Git**: For version control
- **just**: Command runner - `cargo install just` or see [installation options](https://github.com/casey/just#installation)
- **cargo-nextest**: Test runner - `cargo install cargo-nextest` (or use `just install-tools`)
- **cargo-binstall** (optional but recommended): Faster binary installation - `cargo install cargo-binstall`

### Clone and Build

```bash
git clone https://github.com/MuntasirSZN/changelogen-rs
cd changelogen-rs
cargo build --all --locked
```

### Verify Setup

Run the test suite to ensure everything works:

```bash
just test
# or
cargo test
```

## Development Workflow

### Quick Start with just

We provide a justfile with common development tasks:

```bash
just help           # Show all available commands
just check          # Run all checks (format, lint, test)
just pre-commit     # Run pre-commit checks
just install-hook   # Install pre-commit hook
just watch          # Watch for changes and run tests
```

### Pre-commit Checklist

Before committing, always run:

```bash
just pre-commit # Or run just install-hook and git will do it
```

This runs:

1. **Format check**: `cargo fmt --all -- --check`
1. **Linting**: `cargo clippy --all-targets --all-features -- -D warnings`
1. **Tests**: `cargo nextest run` (or `cargo test`)
1. **Doc tests**: `cargo test --doc`

Alternatively, run each step manually:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run
cargo test --doc
```

### Fast Feedback Loop

For iterative development with automatic rebuilds:

```bash
cargo watch -x 'clippy -- -D warnings' -x 'nextest run'
```

## Code Quality Standards

### Formatting

- Use `rustfmt` for all code
- Configuration is in `rustfmt.toml` (uses default settings)
- Run: `cargo fmt --all`

### Linting

- Code must pass `clippy` with `-D warnings`
- No `unwrap()` calls outside of tests (use proper error handling)
- Use meaningful variable names
- Add documentation comments for public APIs

### Error Handling

- Use `anyhow::Result` for most operations
- Use `anyhow::Context` to add context to errors
- Avoid panics outside of unrecoverable invariants
- Never use `unwrap()` or `expect()` in library code

### Testing

- Write comprehensive unit tests for all public APIs
- Use snapshot testing (`insta`) for output validation
- Property-based tests (`proptest`) for edge cases
- Integration tests in `tests/` directory
- Ensure tests are deterministic

## Testing

### Running Tests

```bash
# All tests
just test
# or
cargo nextest run
cargo test --doc

# Fast tests (skip doc tests)
just test-fast

# Specific test file
cargo test --test determinism

# Specific test function
cargo test repeated_parse_identical

# With output
cargo test -- --nocapture
```

### Writing Tests

- Tests go in `tests/` for integration tests or in module with `#[cfg(test)]`
- Use descriptive test names: `test_feature_behavior_expected_result`
- Test both happy paths and error cases
- Keep tests focused and independent

### Benchmarks

The project uses [CodSpeed](https://codspeed.io/) for continuous benchmarking. Install the CLI tool first:

```bash
# Install cargo-codspeed (first time only)
cargo install cargo-codspeed
```

Then run benchmarks:

```bash
# Build benchmarks
cargo codspeed build

# Run all benchmarks
cargo codspeed run

# Run specific benchmark
cargo codspeed run parse_sequential

# Run with specific arguments
cargo codspeed run -- --bench 100
```

Benchmarks are in `benches/` and use the `codspeed-divan-compat` framework (CodSpeed-instrumented divan API). See [PERF.md](PERF.md) for detailed performance documentation.

**Available benchmarks**:

- `parse_sequential`: Baseline single-threaded parsing (10, 50, 100, 500 commits)
- `parse_parallel`: Multi-threaded parsing with rayon (50, 100, 500 commits)
- `version_inference`: Semver bump calculation (10, 50, 100, 500 commits)
- `render_block`: Markdown changelog generation (10, 50, 100, 500 commits)

**CI Integration**: Benchmarks run automatically on every PR via `.github/workflows/benches.yml` and results are tracked in the CodSpeed dashboard.

## Pull Request Process

1. **Fork and branch**: Create a feature branch from `main`
1. **Make changes**: Follow code quality standards
1. **Test**: Ensure all tests pass
1. **Commit**: Write clear, descriptive commit messages
1. **Push**: Push to your fork
1. **PR**: Open a pull request with a clear description

### PR Requirements

- [ ] Code is formatted (`just fmt`)
- [ ] Clippy passes with no warnings (`just lint`)
- [ ] All tests pass (`just test`)
- [ ] New features have tests
- [ ] Public APIs have documentation
- [ ] Changes are described in PR description

### Commit Messages

Follow conventional commit format:

```
feat: add new feature
fix: resolve bug
docs: update documentation
test: add missing tests
refactor: improve code structure
```

## Project Goals

### Parity with JavaScript Version

This project aims for **output parity** with [`@unjs/changelogen`](https://github.com/unjs/changelogen):

- Commit classification must match exactly
- Markdown output format preserved
- Configuration precedence identical
- Semver inference follows same rules

### Code Quality Principles

1. **Correctness**: Code must be correct first
1. **Safety**: Avoid unsafe code and panics
1. **Performance**: Leverage Rust's performance where beneficial
1. **Clarity**: Code should be readable and well-documented

### What to Contribute

**Good PRs:**

- Bug fixes with tests
- Performance improvements with benchmarks
- Documentation improvements
- Test coverage improvements
- Code quality improvements (following standards)
- Support for other languages

**Discuss First:**

- New features (open an issue first)
- Breaking changes
- Major refactors
- Significant dependency additions
- Language extras (like new parser etc)

## Need Help?

- Check [tasks.md](tasks.md) for current work and roadmap
- Review [PARITY_SPEC.md](PARITY_SPEC.md) for parity requirements
- Review [PERF.md](PERF.md) for performance guidelines
- Look at existing tests for examples
- Open an issue for questions

______________________________________________________________________

## Release Process

> [!NOTE]
> This section is for maintainers only.

### Pre-release Checklist

Before cutting a release:

1. **All tests pass**: `just check`
1. **Benchmarks run**: `cargo codspeed build && cargo codspeed run` (document any regressions)
1. **Documentation updated**: README, CHANGELOG, version numbers
1. **MSRV validated**: CI passes on minimum Rust version
1. **No clippy warnings**: `cargo clippy -- -D warnings`
1. **Cargo.lock committed**: Ensure lock file is up to date

### Version Bumping

The tool can bump its own version! Use the `release` command:

```bash
# Dry run first to preview
changelogen release --dry-run

# Create release with automatic version bump
changelogen release

# Or specify explicit version
changelogen release --new-version 1.2.3

# With signed tag
changelogen release --sign
```

This will:

1. Analyze commits since last tag
1. Infer semantic version bump (major/minor/patch)
1. Update `Cargo.toml` version
1. Generate changelog entry in `CHANGELOG.md`
1. Create git tag

### Manual Release Steps

If you need to release manually:

```bash
# 1. Update version in Cargo.toml
vim Cargo.toml  # Bump version field

# 2. Generate changelog
changelogen generate --write

# 3. Commit changes
git add Cargo.toml CHANGELOG.md
git commit -m "chore: release v1.2.3"

# 4. Create tag
git tag -a v1.2.3 -m "Release v1.2.3"

# 5. Push
git push origin main --tags

# 6. Publish to crates.io
cargo publish
```

### Publishing to crates.io

```bash
# Test the package first
cargo package --list
cargo package --allow-dirty  # If you have uncommitted docs

# Publish
cargo publish

# If you need to specify token
cargo publish --token $CARGO_REGISTRY_TOKEN
```

### GitHub Release

After tagging, create a GitHub release:

```bash
# Using the tool (requires GITHUB_TOKEN)
changelogen github v1.2.3

# Or manually via GitHub UI
# 1. Go to https://github.com/MuntasirSZN/changelogen-rs/releases/new
# 2. Select tag v1.2.3
# 3. Copy content from CHANGELOG.md for release notes
# 4. Publish
```

### npm Package (Future)

Once NAPI-RS integration is complete:

```bash
# Prerequisites
npm install -g @napi-rs/cli

# Build native modules for all platforms
npm run build

# Or build for specific platform
napi build --platform --release --features napi

# Run tests (once implemented)
npm test

# Publish to npm (requires NPM_TOKEN)
npm publish
```

#### Cross-platform Builds

For cross-platform releases, use GitHub Actions with the NAPI-RS workflow:

```yaml
# .github/workflows/release-npm.yml
name: Release NPM Package
on:
  push:
    tags:
      - 'v*'
jobs:
  build:
    strategy:
      matrix:
        settings:
          - host: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - host: macos-latest
            target: x86_64-apple-darwin
          - host: windows-latest
            target: x86_64-pc-windows-msvc
    runs-on: ${{ matrix.settings.host }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: npm install -g @napi-rs/cli
      - run: napi build --platform --release --features napi
      - run: napi artifacts
```

See [NAPI-RS Documentation](https://napi.rs/) for detailed setup instructions.

**Current Status**: Core bindings implemented. Tasks remaining:
- Set up CI/CD pipeline for cross-platform builds (task 80.5)
- Add integration tests for npm package
- Test npm package installation and usage

See task 80 (Section 12.5) in [tasks.md](tasks.md) for detailed NAPI-RS integration status.

### Post-release

1. **Verify installation**: `cargo install changelogen --version 1.2.3`
1. **Test published crate**: In a new directory, `cargo install changelogen && changelogen --version`
1. **Update documentation**: Ensure README reflects new version capabilities
1. **Announce**: Create announcement issue/discussion if significant changes

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (MIT License).
