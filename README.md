# changelogen-rs

💅 **Beautiful Changelogs using Conventional Commits** - Rust port of [@unjs/changelogen](https://github.com/unjs/changelogen)

[![CI](https://github.com/MuntasirSZN/changelogen-rs/workflows/CI/badge.svg)](https://github.com/MuntasirSZN/changelogen-rs/actions)

## Status

✨ **MVP Complete** - Core features implemented with parity to the JavaScript version.

**Parity Achieved**: This Rust implementation aims for output parity with [@unjs/changelogen](https://github.com/unjs/changelogen). Commit classification, version inference, and markdown output should match the JavaScript version exactly. See [PARITY_SPEC.md](PARITY_SPEC.md) for detailed requirements.

**Distribution**: Currently available via Cargo. npm package distribution via NAPI-RS is planned for a future release.

See [tasks.md](tasks.md) for detailed roadmap and implementation status.

## Features

- ✅ **Conventional Commit Parsing** - Supports standard commit message formats
- ✅ **Configurable Types** - Customize commit types, emojis, and semver impact
- ✅ **Multiple Providers** - GitHub, GitLab, Bitbucket repository detection
- ✅ **Parallel Processing** - Fast parsing of large commit histories
- ✅ **Author Attribution** - Automatic contributor detection and acknowledgment
- ✅ **Semantic Versioning** - Automatic version bumping based on changes
- ✅ **Idempotent Operation** - Safe to rerun without duplicating entries
- ✅ **Clean Code Quality** - No unwrap() outside tests, clippy clean, comprehensive test coverage

## Quick Start

```bash
# Install from source (cargo publish pending)
git clone https://github.com/MuntasirSZN/changelogen-rs
cd changelogen-rs
cargo install --path .

# Basic usage
changelogen show                    # Show next version
changelogen generate                # Generate changelog block  
changelogen generate --write        # Update CHANGELOG.md
changelogen release                 # Full release pipeline (tag + changelog)
changelogen --help                  # See all options
```

## Configuration

Create `changelogen.toml` in your project root:

```toml
# Customize commit types
[types.feat]
title = "✨ Features"
semver = "minor"

[types.fix] 
title = "🐛 Bug Fixes"
semver = "patch"

# Scope mapping
[scopeMap]
"ui" = "frontend"
"api" = "backend"

# GitHub token for release syncing
[tokens]
github = "${GITHUB_TOKEN}"
```

Or use `[package.metadata.changelogen]` in `Cargo.toml`.

## Differences from JS Version

### Intentional Differences

| Feature                  | JavaScript Version               | Rust Version                       | Notes                                    |
| ------------------------ | -------------------------------- | ---------------------------------- | ---------------------------------------- |
| **Configuration**        | JSON/JS files                    | TOML files                         | Rust ecosystem standard                  |
| **Config location**      | `package.json` or `.changelogrc` | `changelogen.toml` or `Cargo.toml` | Cargo integration                        |
| **Parallel processing**  | Single-threaded                  | Optional multi-threaded (rayon)    | Performance optimization for large repos |
| **Package distribution** | npm                              | Cargo (npm via NAPI-RS planned)    | Native Rust tooling                      |
| **Binary size**          | Node.js required (~50MB+)        | Static binary (~5MB)               | No runtime dependency                    |

### Parity Guarantees

These behaviors match the JavaScript version **exactly**:

- ✅ **Commit classification**: Type detection, scope parsing, breaking change identification
- ✅ **Version inference**: Semver rules including pre-1.0 adjustments
- ✅ **Markdown output**: Format, section ordering, reference linking
- ✅ **Filtering rules**: Disabled types, `chore(deps)` handling
- ✅ **Contributors**: Deduplication, co-author detection, ordering
- ✅ **Idempotence**: Safe to rerun without duplication

See [PARITY_SPEC.md](PARITY_SPEC.md) for comprehensive parity documentation and verification strategy.

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed contribution guidelines.

### Quick Commands

```bash
# Using just (recommended)
just check          # Run all checks (format, lint, test)
just test           # Run tests
just lint           # Run clippy
just fmt            # Format code
just coverage       # Generate coverage report (text summary)
just coverage-html  # Generate HTML coverage report and open in browser
just coverage-lcov  # Generate lcov.info for Codecov

# Manual commands
cargo build
cargo test
cargo clippy -- -D warnings
cargo fmt --all

# Coverage with cargo-llvm-cov
cargo install cargo-llvm-cov
cargo llvm-cov --all-features --workspace
cargo llvm-cov --all-features --workspace --html --open

# Run benchmarks
cargo bench
```

### Environment Variables

```bash
CHANGELOGEN_PARALLEL_THRESHOLD=50  # Parallel processing threshold (default: 50)
RUST_LOG=debug                     # Enable debug logging
GITHUB_TOKEN=xxx                   # GitHub API token for release sync
```

## Goals

- Parity with JavaScript Version
- Support more than just npm (cargo, go and others, contributions needed!)
- Performance
- Security
- else?

## License

MIT - See [LICENSE](LICENSE) for details

## Acknowledgments

This project is a Rust port of [@unjs/changelogen](https://github.com/unjs/changelogen) by the UnJS team.
