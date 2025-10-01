# justfile for changelogen-rs development
# Requires: cargo, cargo-nextest (optional)

# Show this help message
help:
    @echo "Available recipes:"
    @just --list

# Format code with rustfmt
fmt:
    cargo fmt --all

# Check code formatting
fmt-check:
    cargo fmt --all -- --check

# Run clippy linter
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Build the project
build:
    cargo build --all --locked

# Build release version
build-release:
    cargo build --all --locked --release

# Run tests (uses nextest if available, falls back to cargo test)
test:
    #!/usr/bin/env bash
    if command -v cargo-nextest >/dev/null 2>&1; then
        cargo nextest run --all-features --all-targets --locked
        cargo test --doc --locked
    else
        cargo test --all --locked
    fi

# Run tests without doc tests
test-fast:
    #!/usr/bin/env bash
    if command -v cargo-nextest >/dev/null 2>&1; then
        cargo nextest run --all-features --all-targets --locked
    else
        cargo test --lib --tests --locked
    fi

# Run benchmarks
bench:
    cargo bench

# Generate documentation
doc:
    cargo doc --all --no-deps --open

# Run all checks (format, lint, test)
check: fmt-check lint test

# Run pre-commit checks
pre-commit: fmt lint test
    @echo "All pre-commit checks passed!"

# Run cargo-deny checks
deny:
    #!/usr/bin/env bash
    if command -v cargo-deny >/dev/null 2>&1; then
        cargo deny check
    else
        echo "cargo-deny not installed. Install with: cargo install cargo-deny"
        exit 1
    fi

# Run security audit
audit:
    cargo audit

# Install development tools
install-tools:
    @echo "Installing development tools..."
    cargo install cargo-nextest --locked
    cargo install cargo-deny --locked
    cargo install cargo-audit --locked
    cargo install cargo-tarpaulin --locked
    @echo "Tools installed!"

# Clean build artifacts
clean:
    cargo clean

# Watch for changes and run tests (requires cargo-watch)
watch:
    #!/usr/bin/env bash
    if command -v cargo-watch >/dev/null 2>&1; then
        cargo watch -x 'clippy -- -D warnings' -x 'nextest run'
    else
        echo "cargo-watch not installed. Install with: cargo install cargo-watch"
        exit 1
    fi

# Create a release build and run the binary
release:
    cargo build --release
    @echo "Release binary: target/release/changelogen"
