# Copilot Instructions for changelogen-rs

## Project Overview

This is changelogen-rs, a Rust implementation of the `@unjs/changelogen` JavaScript library. The goal is to create a **parity port** that generates identical changelog output while leveraging Rust's performance and safety benefits.

### Key Characteristics
- **Parity-focused**: Output must match the JS version exactly (format, ordering, inference)
- **Changelog generation**: Parses conventional commits and generates beautiful markdown changelogs
- **CLI tool**: Built with clap for command-line usage
- **Git integration**: Uses git2 for repository operations
- **Configuration-driven**: Supports `changelogen.toml` and `Cargo.toml` metadata

## Development Workflow

### Required Tools
- Rust 1.85+ (MSRV enforced via CI)
- cargo-nextest for testing
- cargo-deny for dependency auditing

### Pre-commit Checklist
Always run these commands before committing:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run
cargo test --doc
```

Make sure it follows the conventional commits specification at https://www.conventionalcommits.org/.

### Quality Gates
- **No `unwrap()`** outside of tests (use proper error handling)
- **Clippy clean**: All warnings must be addressed
- **Formatted code**: Use `cargo fmt --all`
- **Tests pass**: Both unit tests and doc tests
- **Deterministic output**: Ensure repeated runs produce identical results

## Codebase Structure

### Core Modules
- `config.rs`: Configuration loading and merging (TOML, CLI overrides)
- `git.rs`: Git repository operations and commit parsing
- `parse.rs`: Conventional commit parsing and classification
- `pipeline.rs`: Main orchestration logic for changelog generation
- `render.rs`: Markdown rendering and formatting
- `authors.rs`: Contributor aggregation and normalization
- `changelog.rs`: File operations for CHANGELOG.md management

### Key Design Patterns
- **Error handling**: Use `anyhow::Result` for most operations, `thiserror` for specific error types
- **Configuration**: Layered approach (defaults < file < CLI overrides)
- **Tracing**: Use structured logging with `tracing` crate
- **Testing**: Snapshot tests with `insta`, property tests with `proptest`

## Coding Standards

### Error Handling
- Prefer `anyhow::Context` for adding context to errors
- Use `?` operator liberally for error propagation
- Avoid `panic!`, `unwrap()`, and `expect()` in library code
- Return meaningful error messages with context

### Dependencies
- Keep dependencies minimal and justify additions
- Use feature flags to make heavy dependencies optional
- Prefer `std` over external crates when performance is similar

### Testing
- Write comprehensive unit tests for all public APIs
- Use snapshot testing (`insta`) for output validation
- Property-based tests (`proptest`) for edge cases
- Integration tests in `tests/` directory

### Performance
- Use `rayon` for parallelizable operations
- Consider `dashmap` for concurrent hash maps (evaluate after benchmarks)
- Profile memory usage and avoid unnecessary allocations

## Configuration

### Supported Formats
1. `changelogen.toml` in project root
2. `[package.metadata.changelogen]` in `Cargo.toml`
3. CLI arguments (highest precedence)

### Environment Variables
Token precedence: `CHANGELOGEN_TOKENS_GITHUB` > `GITHUB_TOKEN` > `GH_TOKEN`

## Testing Strategy

### Test Types
- **Unit tests**: Individual module functionality
- **Integration tests**: End-to-end CLI behavior
- **Snapshot tests**: Changelog output format validation
- **Property tests**: Edge cases and invariants
- **Determinism tests**: Ensure reproducible output

### Running Tests
```bash
# Fast feedback loop during development
cargo watch -x 'clippy -- -D warnings' -x 'nextest run'

# Full test suite
cargo nextest run --all --locked
cargo test --doc
```

## Common Tasks

### Adding New Commit Types
1. Update `default_types()` in `config.rs`
2. Add corresponding emoji and semver impact
3. Update tests and snapshots
4. Verify parity with JS version

### Modifying Output Format
1. Check `render.rs` for template logic
2. Update corresponding tests
3. Validate against JS version output
4. Update snapshots if needed

### Git Operations
- Use `git2` crate consistently
- Handle edge cases (empty repos, no commits, etc.)
- Test with various Git configurations

## CI/CD Pipeline

### Automated Checks
- Formatting (`cargo fmt --check`)
- Linting (`cargo clippy -- -D warnings`)
- Testing (`cargo nextest run`)
- License compliance (`cargo deny check`)

### Quality Gates
- All tests must pass
- No clippy warnings
- Properly formatted code
- No vulnerable dependencies

## Parity Requirements

### Critical Invariants
- Commit classification must match JS version exactly
- Semver inference follows identical rules
- Markdown output format preserved
- Contributor aggregation logic identical
- Configuration precedence matches

### Verification
- Compare output against JS version using same git history
- Property tests ensure edge cases work consistently
- Benchmark performance vs JS version (target: no >10% regression)

## Performance Considerations

### Optimization Areas
- Git operations (use libgit2 efficiently)
- Parallel processing (rayon for independent operations)
- Memory usage (avoid cloning large data structures)
- I/O operations (batch file operations)

### Profiling
- Use `criterion` for micro-benchmarks
- Profile with realistic repository sizes
- Memory profiling for large changelog generation

## Documentation

### Code Documentation
- Public APIs must have doc comments
- Include examples in doc tests
- Document invariants and assumptions
- Explain complex algorithms

### User Documentation
- Keep README focused and clear
- Document CLI usage with examples
- Explain configuration options
- Migration guide from JS version

## Common Pitfalls

### Git Handling
- Always handle empty repositories gracefully
- Consider different Git configurations
- Test with various commit message formats
- Handle merge commits appropriately

### Configuration
- Validate all configuration values
- Provide helpful error messages for invalid config
- Test configuration precedence thoroughly
- Handle missing/malformed TOML gracefully

### Output Generation
- Ensure deterministic ordering (stable sorts)
- Handle Unicode normalization consistently
- Test with various commit message encodings
- Validate markdown output structure

## Getting Help

### Resources
- Review `tasks.md` for current implementation status
- Check `PARITY_SPEC.md` for specific requirements
- Examine existing tests for usage patterns
- Compare with JS version when in doubt

### Code Review Focus
- Parity with JS version output
- Error handling robustness
- Test coverage completeness
- Performance implications
- Code clarity and maintainability
