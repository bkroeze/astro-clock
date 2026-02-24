# Astro Clock - Justfile
# A command runner for common development tasks

# Default recipe - show available commands
default:
    @just --list --unsorted

# ============================================================================
# Build Commands
# ============================================================================

# Build the project in debug mode
build:
    cargo build

# Build the project in release mode
build-release:
    cargo build --release

# Build with database feature enabled
build-db:
    cargo build --features db

# Clean build artifacts
clean:
    cargo clean

# ============================================================================
# Development Commands
# ============================================================================

# Run the CLI application with default settings
run *ARGS:
    cargo run --bin astro-clock -- {{ARGS}}

# Run the HTTP server
run-server:
    cargo run --bin astro-clock -- server

# Run with database feature
run-db *ARGS:
    cargo run --features db --bin astro-clock -- {{ARGS}}

# Check code without building
check:
    cargo check

# Check all features
check-all:
    cargo check --all-features

# ============================================================================
# Code Quality
# ============================================================================

# Run Clippy lints
lint:
    cargo clippy --all-features -- -D warnings

# Run Clippy with all features (pedantic)
lint-pedantic:
    cargo clippy --all-features -- -W clippy::pedantic

# Format code
fmt:
    cargo fmt

# Check formatting without modifying files
fmt-check:
    cargo fmt -- --check

# Run full code quality checks
quality: fmt-check lint check-all

# ============================================================================
# Testing
# ============================================================================

# Run all tests
test:
    cargo test

# Run tests with all features
test-all:
    cargo test --all-features

# Run tests with output visible
test-verbose:
    cargo test -- --nocapture

# Run a specific test by name
test-one TEST_NAME:
    cargo test {{TEST_NAME}} -- --nocapture

# Run tests continuously on file changes (requires cargo-watch)
watch-test:
    cargo watch -x test

# ============================================================================
# Documentation
# ============================================================================

# Generate and open documentation
doc:
    cargo doc --open

# Generate documentation for all features
doc-all:
    cargo doc --all-features --open

# Serve documentation locally (requires cargo-docserver or similar)
doc-serve:
    cargo doc && python3 -m http.server -d target/doc 8080

# ============================================================================
# Release & Distribution
# ============================================================================

# Create a release build and run tests
release-prep: test-all build-release
    @echo "Release build ready in target/release/"

# Get binary size info
size:
    cargo build --release && ls -lh target/release/astro-clock

# Check for outdated dependencies (requires cargo-outdated)
outdated:
    cargo outdated

# Update dependencies
update:
    cargo update

# Audit dependencies for security issues (requires cargo-audit)
audit:
    cargo audit

# ============================================================================
# Development Utilities
# ============================================================================

# Watch and rebuild on changes (requires cargo-watch)
watch:
    cargo watch -x check

# Run the application with hot reload (requires cargo-watch)
watch-run *ARGS:
    cargo watch -x "run -- {{ARGS}}"

# Build and run with tracing output
run-trace *ARGS:
    RUST_LOG=trace cargo run -- {{ARGS}}

# Run with debug logging
run-debug *ARGS:
    RUST_LOG=debug cargo run -- {{ARGS}}

# ============================================================================
# Database (requires db feature)
# ============================================================================

# Run database migrations (requires sqlx-cli)
migrate:
    sqlx migrate run

# Create a new migration (requires sqlx-cli)
migrate-new NAME:
    sqlx migrate add {{NAME}}

# Check database connection
db-check:
    cargo run --features db -- db-check 2>/dev/null || echo "Database feature not configured"

# ============================================================================
# Project Maintenance
# ============================================================================

# Show project info
info:
    @echo "Project: astro-clock"
    @echo "Version: $(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)"
    @echo "Rust: $(rustc --version)"
    @echo ""
    @echo "Features:"
    @echo "  - default: Basic CLI and chart generation"
    @echo "  - db: PostgreSQL database support"

# Count lines of code
loc:
    @echo "Source code lines:"
    find src -name '*.rs' | xargs wc -l | tail -1

# Show dependency tree
tree:
    cargo tree

# Verify project structure is valid
verify:
    cargo check && cargo test && cargo fmt -- --check && cargo clippy
