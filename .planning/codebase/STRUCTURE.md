# Codebase Structure

**Analysis Date:** 2026-02-24

## Directory Layout

```
[project-root]/
├── src/                    # Source code
│   ├── bin/               # Binary entry points
│   │   └── main.rs        # CLI application entry
│   ├── cli/               # Command-line interface
│   │   ├── mod.rs         # Module exports
│   │   └── app.rs         # CLI parsing and app logic
│   ├── database/          # Database layer (optional feature)
│   │   ├── mod.rs         # Module exports
│   │   ├── pool.rs        # Connection pooling
│   │   └── schema.rs      # Database schema
│   ├── ephemeris/         # Astronomical calculations
│   │   └── mod.rs         # Julian day, ephemeris init
│   ├── aspects.rs         # Aspect detection algorithms
│   ├── chart.rs           # Core domain types and traits
│   ├── config.rs          # Configuration management
│   ├── errors.rs          # Error types
│   ├── lib.rs             # Library exports
│   ├── logging.rs         # Logging initialization
│   ├── output_handler.rs  # Multi-format output
│   ├── renderer.rs        # Chart visualization
│   ├── server/            # HTTP server
│   │   └── mod.rs         # Axum routes and handlers
│   └── swiss_eph_impl.rs  # Swiss Ephemeris integration
├── assets/                # Static assets
│   └── Roboto-Regular.ttf # Font for chart rendering
├── data/                  # Ephemeris data files
│   ├── seas_18.se1        # Asteroid ephemeris
│   ├── semo_18.se1        # Moon ephemeris
│   └── sepl_18.se1        # Planet ephemeris
├── docs/                  # Documentation
│   ├── RUST_CLI_TOOLS_BEST_PRACTICES.md
│   └── swiss-eph.md       # Swiss Ephemeris docs
├── Cargo.toml             # Package manifest
├── Cargo.lock             # Dependency lockfile
├── config.ron             # Default configuration
├── Justfile               # Task runner recipes
└── .planning/             # Planning documents (this directory)
```

## Directory Purposes

**src/bin/:**
- Purpose: Binary entry points
- Contains: `main.rs` - thin wrapper that instantiates and runs App
- Pattern: Standard Rust binary structure

**src/cli/:**
- Purpose: CLI argument parsing and command dispatch
- Contains: `mod.rs` (exports), `app.rs` (clap definitions, App struct)
- Key types: `Cli`, `Commands`, `App`

**src/database/:**
- Purpose: PostgreSQL persistence (optional `db` feature)
- Contains: `mod.rs`, `pool.rs`, `schema.rs`
- Key types: `DatabasePool`, `ChartRecord`, `ChartSchema`
- Status: Schema defined but not integrated into main flow

**src/ephemeris/:**
- Purpose: Date/time conversions and ephemeris initialization
- Contains: `mod.rs`
- Key types: `Ephemeris`, `DateTime`
- Functions: Julian day calculations, Swiss Ephemeris path management

**src/server/:**
- Purpose: HTTP API server
- Contains: `mod.rs`
- Key types: `Server`
- Routes: `/health`, `/chart`

**assets/:**
- Purpose: Static files for runtime
- Contains: `Roboto-Regular.ttf` (embedded font for rendering)
- Access: Loaded via `include_bytes!` macro

**data/:**
- Purpose: Swiss Ephemeris binary data files
- Contains: `.se1` files (asteroid, moon, planet ephemerides)
- Required: At runtime for accurate calculations
- Path: Set via `SE_EPHE_PATH` env var or defaults to `data/`

## Key File Locations

**Entry Points:**
- `src/bin/main.rs`: CLI binary entry point
- `src/lib.rs`: Library entry point with module exports

**Configuration:**
- `config.ron`: Default runtime configuration (RON format)
- `Cargo.toml`: Package dependencies and features
- `.env`: Environment variables (not committed)

**Core Logic:**
- `src/chart.rs`: Domain models (`ChartData`, `PlanetPosition`, `HouseCusps`)
- `src/swiss_eph_impl.rs`: Swiss Ephemeris calculator implementation
- `src/aspects.rs`: Aspect detection and analysis

**Rendering:**
- `src/renderer.rs`: 2D chart rendering with tiny-skia
- `src/output_handler.rs`: Format dispatch (PNG/WebP/Markdown)

**Testing:**
- Tests are co-located in source files under `#[cfg(test)]` modules
- Example: `src/chart.rs` has tests at end of file
- Dev dependencies in `Cargo.toml`: `tempfile`, `tower`, `http-body-util`, `http`

## Naming Conventions

**Files:**
- Module files: `snake_case.rs` (e.g., `output_handler.rs`)
- Directory modules: `mod.rs` inside directory
- Binary files: `main.rs` in `bin/` subdirectory

**Directories:**
- Source directories: `snake_case` (e.g., `ephemeris/`)
- Test directories: N/A (tests are co-located)

**Structs/Enums:**
- PascalCase: `ChartCalculator`, `HouseSystem`, `PlanetPosition`

**Traits:**
- PascalCase with descriptive names: `ChartCalculator`

**Functions:**
- snake_case: `calculate_planets()`, `julian_day_from_chrono()`

**Constants:**
- SCREAMING_SNAKE_CASE in modules: `planet::SUN`, `zodiac::ARIES`

## Where to Add New Code

**New CLI Subcommand:**
- Add variant to `Commands` enum in `src/cli/app.rs`
- Add handler logic in `App::run()` match arm
- Add tests in `src/cli/app.rs` test module

**New House System:**
- Add variant to `HouseSystem` enum in `src/chart.rs`
- Add display string in `Display` impl
- Add mapping in `src/swiss_eph_impl.rs` `get_house_system()`
- Add to validation list in `src/config.rs`
- Add to CLI parser in `src/cli/app.rs` `parse_house_system()`

**New Output Format:**
- Add variant to `OutputFormat` enum in `src/output_handler.rs`
- Add extension in `extension()` method
- Add case in `save_chart()` method
- Add rendering logic as new method

**New Planet:**
- Add constant to `planet` module in `src/chart.rs`
- Add to planet list in `SwissEphChartCalculator::calculate_planets()`
- Add symbol mapping in `src/renderer.rs` and `src/output_handler.rs`

**New Aspect Type:**
- Add variant to `AspectType` enum in `src/aspects.rs`
- Add angle in `angle()` method
- Add name and symbol
- Add to `find_aspects()` aspect types array
- Add color mapping in `src/renderer.rs` `draw_aspects()`

**New HTTP Endpoint:**
- Add route in `Server::run()` in `src/server/mod.rs`
- Add handler function in `src/server/mod.rs`
- Add test in server test module

**Database Integration:**
- Enable `db` feature in `Cargo.toml`
- Use `DatabasePool` from `src/database/pool.rs`
- Use schema from `src/database/schema.rs`

## Special Directories

**.planning/:**
- Purpose: Architecture and planning documents
- Generated: No (manually maintained)
- Committed: Yes

**target/:**
- Purpose: Cargo build artifacts
- Generated: Yes (by `cargo build`)
- Committed: No (in `.gitignore`)

**.tickets/:**
- Purpose: CLI ticket system data
- Generated: Yes (by `tk` command)
- Committed: Yes

**.iteratr/:**
- Purpose: Iteratr event streaming data
- Generated: Yes (by iteratr system)
- Committed: No (should be in `.gitignore`)

**.history/:**
- Purpose: File history/backup
- Generated: Yes (by editor/IDE)
- Committed: No (should be in `.gitignore`)

---

*Structure analysis: 2026-02-24*
