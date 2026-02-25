# Technology Stack

**Analysis Date:** 2026-02-24

## Languages

**Primary:**
- Rust - Edition 2024
  - Used for: All application logic, CLI, HTTP server, chart calculations, rendering
  - File locations: `src/**/*.rs`, `src/bin/main.rs`

## Runtime

**Environment:**
- Rust native (no WASM target detected)
- Tokio async runtime (version 1.0 with full features)

**Package Manager:**
- Cargo (standard Rust package manager)
- Lockfile: `Cargo.lock` present

## Frameworks

**Core:**
- `axum` 0.7 - HTTP web framework for REST API server (`src/server/mod.rs`)
- `tokio` 1.0 - Async runtime with full features
- `clap` 4.0 - CLI argument parsing with derive macros (`src/cli/app.rs`)

**Testing:**
- Built-in `cargo test` with standard Rust testing
- `tower` 0.5 - HTTP testing utilities (`src/server/mod.rs` tests)
- `http-body-util` 0.1 - HTTP body testing utilities
- `tempfile` 3 - Temporary file creation for tests

**Build/Dev:**
- `just` - Command runner (Justfile with 30+ recipes)
- Standard `cargo` toolchain

## Key Dependencies

**Critical:**
- `swiss-eph` 0.2.1 - Swiss Ephemeris library for astronomical calculations (`src/swiss_eph_impl.rs`)
- `tiny-skia` 0.8 - 2D raster graphics rendering (`src/renderer.rs`)
- `fontdue` 0.8 - Font loading and rasterization
- `chrono` 0.4 - Date/time handling

**Infrastructure:**
- `sqlx` 0.8 - Async PostgreSQL ORM (optional `db` feature, `src/database/`)
- `serde` 1.0 + `serde_json` 1.0 - Serialization/deserialization
- `ron` 0.8 - RON (Rusty Object Notation) configuration format (`config.ron`)
- `tracing` 0.1 + `tracing-subscriber` 0.3 - Structured logging
- `anyhow` 1.0 - Error handling
- `thiserror` 1.0 - Custom error types
- `rust_decimal` 1.30 - Decimal arithmetic for precise calculations
- `image-webp` 0.2.4 - WebP image encoding
- `png` 0.17 - PNG image encoding

## Configuration

**Environment:**
- `.env` file exists (contains environment configuration)
- `RUST_LOG` environment variable controls tracing level
- Configuration via `config.ron` file (RON format)

**Build:**
- `Cargo.toml` - Package manifest with optional `db` feature
- `Justfile` - Task runner with 30+ commands for build, test, lint, etc.

**Features:**
- `default` - Basic CLI and chart generation
- `db` - PostgreSQL database support (includes `sqlx` and `chrono/serde`)

## Platform Requirements

**Development:**
- Rust toolchain (Edition 2024)
- PostgreSQL (optional, for `db` feature)
- `just` command runner (optional but recommended)
- `sqlx-cli` (optional, for database migrations)

**Production:**
- Binary deployment target
- Ephemeris data files in `data/` directory:
  - `sepl_18.se1` - Planetary ephemeris
  - `semo_18.se1` - Lunar ephemeris
  - `seas_18.se1` - Asteroid ephemeris
- Optional: PostgreSQL database (if using `db` feature)

**External Data Dependencies:**
- Swiss Ephemeris data files downloaded from GitHub (`aloistr/swisseph`)
- Font file: `Roboto-Regular.ttf` (for chart rendering)

---

*Stack analysis: 2026-02-24*
