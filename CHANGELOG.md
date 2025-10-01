# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.1.0 (Initial Release)

### 🎉 Initial Implementation

This is the initial release of changelogen-rs, a Rust port of [@unjs/changelogen](https://github.com/unjs/changelogen).

**Core Features:**
- ✅ Conventional commit parsing with breaking change detection
- ✅ Semantic version inference with pre-1.0 adjustments
- ✅ Configurable commit types with emoji and semver impact
- ✅ Multiple repository providers (GitHub, GitLab, Bitbucket)
- ✅ Parallel commit processing for large repositories
- ✅ Automatic contributor attribution with co-author support
- ✅ Idempotent changelog operations
- ✅ Clean code quality (no unwrap outside tests, clippy clean)

**Configuration:**
- TOML-based configuration via `changelogen.toml`
- Cargo.toml integration via `[package.metadata.changelogen]`
- Environment variable support for tokens and thresholds
- Scope mapping and type filtering

**CLI Commands:**
- `changelogen show` - Display next inferred version
- `changelogen generate` - Generate changelog block
- `changelogen release` - Full release pipeline
- `changelogen github` - Sync release to GitHub

**Quality:**
- Comprehensive test suite with 25+ integration tests
- Snapshot testing for output validation
- Determinism tests for reproducible results
- Property-based testing infrastructure
- Benchmark suite for performance tracking

**Documentation:**
- Detailed parity specification with JS version
- Contributing guidelines with dev workflow
- Performance documentation and benchmarking guide
- Comprehensive README with examples

**Distribution:**
- Available via Cargo: `cargo install changelogen`
- Static binary with no runtime dependencies
- npm package via NAPI-RS (planned for future release)

### 📦 Dependencies

Core dependencies:
- git2: Git repository operations
- semver: Semantic versioning
- clap: CLI argument parsing
- anyhow/thiserror: Error handling
- serde/toml_edit: Configuration management
- tracing: Structured logging
- rayon: Parallel processing
- jiff: Date/time handling
- reqwest: HTTP client for GitHub API
- git-conventional: Conventional commit parsing

### 🙏 Acknowledgments

This project is a Rust port of [@unjs/changelogen](https://github.com/unjs/changelogen) by the UnJS team.
Special thanks to the UnJS community for creating the original tool and specification.
