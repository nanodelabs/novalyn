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

- **Rust**: 1.90.0 or later (MSRV enforced by CI)
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
just watch          # Watch for changes and run tests
```

### Pre-commit Checklist

Before committing, always run:

```bash
just pre-commit
```

This runs:
1. **Format check**: `cargo fmt --all -- --check`
2. **Linting**: `cargo clippy --all-targets --all-features -- -D warnings`
3. **Tests**: `cargo nextest run` (or `cargo test`)
4. **Doc tests**: `cargo test --doc`

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

```bash
cargo bench
```

Benchmarks are in `benches/` and use the `divan` framework.

## Pull Request Process

1. **Fork and branch**: Create a feature branch from `main`
2. **Make changes**: Follow code quality standards
3. **Test**: Ensure all tests pass
4. **Commit**: Write clear, descriptive commit messages
5. **Push**: Push to your fork
6. **PR**: Open a pull request with a clear description

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
2. **Safety**: Avoid unsafe code and panics
3. **Performance**: Leverage Rust's performance where beneficial
4. **Clarity**: Code should be readable and well-documented

### What to Contribute

**Good PRs:**
- Bug fixes with tests
- Performance improvements with benchmarks
- Documentation improvements
- Test coverage improvements
- Code quality improvements (following standards)

**Discuss First:**
- New features (open an issue first)
- Breaking changes
- Major refactors
- Significant dependency additions

## Need Help?

- Check `tasks.md` for current work and roadmap
- Review `PARITY_SPEC.md` for parity requirements
- Look at existing tests for examples
- Open an issue for questions

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (MIT License).
