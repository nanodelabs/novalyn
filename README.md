# changelogen-rs

💅 **Beautiful Changelogs using Conventional Commits** - Rust port of [@unjs/changelogen](https://github.com/unjs/changelogen)

## Status

🚧 **Early Development** - Implementing core features from the JS version with Rust performance and safety.

See `tasks.md` for the detailed roadmap and current progress.

## Features

- ✅ **Conventional Commit Parsing** - Supports standard commit message formats  
- ✅ **Configurable Types** - Customize commit types, emojis, and semver impact
- ✅ **Multiple Providers** - GitHub, GitLab, Bitbucket repository detection
- ✅ **Parallel Processing** - Fast parsing of large commit histories
- ✅ **Author Attribution** - Automatic contributor detection and acknowledgment
- ✅ **Semantic Versioning** - Automatic version bumping based on changes
- ✅ **Idempotent Operation** - Safe to rerun without duplicating entries

## Quick Start

```bash
# Install from source (cargo publish pending)
git clone https://github.com/MuntasirSZN/changelogen-rs
cd changelogen-rs
cargo install --path .

# Basic usage
changelogen show          # Show next version
changelogen generate      # Generate changelog block  
changelogen release       # Full release pipeline
changelogen --help        # See all options
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
```

Or use `[package.metadata.changelogen]` in `Cargo.toml`.

## Differences from JS Version

- **Performance**: Rust implementation with optional parallel processing
- **Configuration**: TOML-based instead of JS/JSON configuration
- **Dependencies**: Minimal dependency footprint optimized for Rust ecosystem
- **Packaging**: Uses Cargo instead of npm for distribution

## Development

```bash
# Development setup
cargo build
cargo test
cargo clippy -- -D warnings

# Run benchmarks
cargo bench

# Environment variables
CHANGELOGEN_PARALLEL_THRESHOLD=50  # Parallel processing threshold
RUST_LOG=debug                     # Enable debug logging
```

## Goals

- **Deterministic Output**: Generate identical changelogs to the JavaScript version
- **Rust Performance**: Leverage Rust's speed and safety for large repositories
- **Minimal Dependencies**: Clean dependency tree without unnecessary bloat

## License

MIT - See [LICENSE](LICENSE) for details
