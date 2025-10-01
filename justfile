# justfile for changelogen-rs development

tools := "cargo-nextest cargo-deny cargo-audit cargo-llvm-cov cargo-watch cargo-codspeed"
clippy := ""

# Default recipe (shows help)
_default:
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

# Run clippy linter and autofix
lint-fix:
    cargo clippy --all-targets --all-features --fix -- -D warnings

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

# Run benchmarks (uses CodSpeed)
bench:
    cargo codspeed build
    cargo codspeed run

# Generate documentation
doc:
    cargo doc --all --no-deps --open

# Run all checks (format, lint, test)
check: fmt-check lint test

# Run pre-commit checks
pre-commit: fmt lint test deny
    @echo "{{ GREEN + BOLD }}✅ All pre-commit checks passed!{{ NORMAL }}"

# Run cargo-deny checks
deny:
    cargo deny check

# Run security audit
audit:
    cargo audit

# Install development tools using cargo-binstall
install-tools:
    @echo "{{ BLUE + BOLD }}Installing development tools...{{ NORMAL }}"
    cargo binstall -y {{tools}} || cargo install cargo-binstall && cargo binstall -y {{tools}}
    @echo "{{ GREEN + BOLD }}✅ Tools installed!{{ NORMAL }}"

# Install git pre-commit hook (run with just pre-commit)
install-hook:
    @echo -e "#!/bin/sh\njust pre-commit" > .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit
    @echo "{{ GREEN + BOLD }}✅ Pre-commit hook installed!{{ NORMAL }}"

# Clean build artifacts
clean:
    cargo clean

# Generate coverage with llvm-cov
coverage:
    cargo llvm-cov --all-features --no-report nextest
    cargo llvm-cov --all-features --no-report --doc
    cargo llvm-cov report --doctests

# Watch for changes and run tests
watch:
    cargo watch -x 'clippy -- -D warnings' -x 'nextest run'

# Create a release build and run the binary
release:
    cargo build --release
    @echo "{{ BLUE + BOLD }}Release binary:{{ NORMAL }} {{ UNDERLINE + CYAN }}target/release/changelogen{{ NORMAL }}"
