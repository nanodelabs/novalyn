# justfile for changelogen-rs development

tools := "cargo-nextest cargo-deny cargo-audit cargo-llvm-cov cargo-watch"

# Default recipe (shows help)
default:
    @just --list

# Show this help message
help:
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

# Run tests with nextest
test:
    cargo nextest run --all-features --locked
    cargo test --doc --locked

# Run tests without doc tests
test-fast:
    cargo nextest run --all-features --locked

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
    cargo deny check

# Run security audit
audit:
    cargo audit

# Install development tools using cargo-binstall
install-tools:
    @echo "Installing development tools..."
    cargo binstall -y {{tools}} || cargo install cargo-binstall && cargo binstall -y {{tools}}
    @echo "Tools installed!"

# Clean build artifacts
clean:
    cargo clean

# Watch for changes and run tests
watch:
    cargo watch -x 'clippy -- -D warnings' -x 'nextest run'

# Create a release build and run the binary
release:
    cargo build --release
    @echo "Release binary: target/release/changelogen"
