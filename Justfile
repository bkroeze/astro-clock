# Astro Clock - Justfile
# A command runner for common development tasks

# Load environment from .env file
set dotenv-load := true

# Export environment variables from shell to recipes
set export := true

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

# Run the CLI application with default settings (e.g., just run chart --lat 0 --lon 0)
run *ARGS:
    cargo run --bin astro-clock {{ARGS}}

# Run the HTTP server
run-server:
    cargo run --bin main --features db serve

# Run with database feature
run-db *ARGS:
    cargo run --features db {{ARGS}}

# Check code without building
check:
    cargo check

# Check all features
check-all:
    cargo check --all-features

# Start the api server
serve:
    cargo run --features db --bin astro-clock serve --port 8086

# Run the explore notebook
notebook:
    marimo edit notebooks/explore.py --mcp

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

# Reset and set up test database with seed data
# Requires TEST_PG_URL env var (e.g. postgresql://postgres:password@localhost:5432/astrology_test)
test-db-setup:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "${TEST_PG_URL:-}" ]; then
        echo "Error: TEST_PG_URL environment variable is not set"
        echo "Set it with: export TEST_PG_URL=postgresql://user:password@host:port/astrology_test"
        exit 1
    fi
    scripts/reset-db.sh "${TEST_PG_URL}" --seed

# Reset and set up the primary database (no seed data)
# Requires PG_URL env var (e.g. postgresql://postgres:password@localhost:5432/astrology)
reset-db:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "${PG_URL:-}" ]; then
        echo "Error: PG_URL environment variable is not set"
        echo "Set it with: export PG_URL=postgresql://user:password@host:port/database"
        exit 1
    fi
    scripts/reset-db.sh "${PG_URL}"

# Run integration tests (requires TEST_PG_URL and test database set up)
test-integration:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ -z "${TEST_PG_URL:-}" ]; then
        echo "Error: TEST_PG_URL environment variable is not set"
        echo "Set it with: export TEST_PG_URL=postgresql://user:password@host:port/astrology_test"
        exit 1
    fi

    cargo test --features db -- --ignored --test-threads=1

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

# Run the application with hot reload (requires cargo-watch) (e.g., just watch-run chart --lat 0 --lon 0)
watch-run *ARGS:
    cargo watch -x "run {{ARGS}}"

# Build and run with tracing output (e.g., just run-trace chart --lat 0 --lon 0)
run-trace *ARGS:
    RUST_LOG=trace cargo run {{ARGS}}

# Run with debug logging (e.g., just run-debug chart --lat 0 --lon 0)
run-debug *ARGS:
    RUST_LOG=debug cargo run {{ARGS}}

# ============================================================================
# Database (requires db feature)
# ============================================================================

# Run database migrations (requires sqlx-cli and PG_URL env var)
migrate:
    @if [ -z "${PG_URL:-}" ]; then \
        echo "Error: PG_URL environment variable is not set"; \
        echo "Set it with: export PG_URL=postgresql://user:password@host:port/database"; \
        exit 1; \
    fi
    sqlx migrate run --database-url "${PG_URL}"

# Create a new migration (requires sqlx-cli)
migrate-create NAME:
    sqlx migrate add {{NAME}}

# Revert the last migration (requires sqlx-cli and PG_URL env var)
migrate-revert:
    @if [ -z "${PG_URL:-}" ]; then \
        echo "Error: PG_URL environment variable is not set"; \
        echo "Set it with: export PG_URL=postgresql://user:password@host:port/database"; \
        exit 1; \
    fi
    sqlx migrate revert --database-url "${PG_URL}"

# Show migration status (requires sqlx-cli and PG_URL env var)
migrate-info:
    @if [ -z "${PG_URL:-}" ]; then \
        echo "Error: PG_URL environment variable is not set"; \
        echo "Set it with: export PG_URL=postgresql://user:password@host:port/database"; \
        exit 1; \
    fi
    sqlx migrate info --database-url "${PG_URL}"

# Set up database: verify PG_URL, run migrations, and check connection
db-setup:
    #!/usr/bin/env bash
    set -euo pipefail
    
    echo "=== Database Setup ==="
    echo ""
    
    # Check PG_URL is set
    if [ -z "${PG_URL:-}" ]; then
        echo "Error: PG_URL environment variable is not set"
        echo ""
        echo "To set it, run:"
        echo "  export PG_URL=postgresql://user:password@host:port/database"
        echo ""
        echo "Example for local development:"
        echo "  export PG_URL=postgresql://postgres:password@localhost:5432/astrology"
        exit 1
    fi
    
    echo "✓ PG_URL is set"
    echo "  Database: $(echo "${PG_URL}" | sed 's/.*@//; s/:.*//')"
    echo ""
    
    # Run migrations
    echo "Running migrations..."
    sqlx migrate run --database-url "${PG_URL}"
    echo ""
    
    # Verify connection by checking migration status
    echo "Migration status:"
    sqlx migrate info --database-url "${PG_URL}"
    echo ""
    
    echo "=== Database setup complete ==="

# Check database connection
db-check:
    cargo run --features db db-check 2>/dev/null || echo "Database feature not configured"

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

# Full verification gate: build with db feature + test all features
verify-full: build-db test-all

# ============================================================================
# Ephemeris Data
# ============================================================================

# Download Swiss Ephemeris data files from GitHub (if not already present)
download-ephemeris:
    #!/usr/bin/env bash
    set -euo pipefail
    DATA_DIR="{{justfile_directory()}}/data"
    mkdir -p "$DATA_DIR"
    
    BASE_URL="https://raw.githubusercontent.com/aloistr/swisseph/master/ephe"
    FILES="sepl_18.se1 semo_18.se1 seas_18.se1"
    
    for file in $FILES; do
        if [ -f "$DATA_DIR/$file" ]; then
            echo "✓ $file already exists"
        else
            echo "↓ Downloading $file..."
            curl -sL "$BASE_URL/$file" -o "$DATA_DIR/$file"
            if [ -f "$DATA_DIR/$file" ] && [ -s "$DATA_DIR/$file" ]; then
                echo "✓ Downloaded $file ($(stat -c%s "$DATA_DIR/$file" | numfmt --to=iec))"
            else
                echo "✗ Failed to download $file"
                rm -f "$DATA_DIR/$file"
                exit 1
            fi
        fi
    done
    echo ""
    echo "Ephemeris data ready in $DATA_DIR"
