# Architecture

**Analysis Date:** 2026-02-24

## Pattern Overview

**Overall:** Layered Architecture with Command Pattern

**Key Characteristics:**
- Clear separation between CLI interface, business logic, and external integrations
- Trait-based abstraction for chart calculation (allows swapping implementations)
- Feature-gated optional database support
- Async HTTP server using Axum framework
- Error propagation using `thiserror` for structured error handling

## Layers

**CLI Layer:**
- Purpose: Command-line interface parsing and application orchestration
- Location: `src/cli/`
- Contains: CLI argument definitions, subcommand dispatch, main app logic
- Depends on: All other layers
- Used by: Binary entry point (`src/bin/main.rs`)

**Domain Layer (Chart):**
- Purpose: Core astrological domain models and calculation interface
- Location: `src/chart.rs`
- Contains: Data structures (`ChartData`, `PlanetPosition`, `HouseCusps`), trait definitions (`ChartCalculator`), constants
- Depends on: None (pure domain)
- Used by: All other layers

**Ephemeris Layer:**
- Purpose: Astronomical calculation abstraction and Swiss Ephemeris integration
- Location: `src/ephemeris/`, `src/swiss_eph_impl.rs`
- Contains: Julian day calculations, ephemeris initialization, Swiss Ephemeris wrapper
- Depends on: `chart` module
- Used by: CLI, Server

**Aspects Layer:**
- Purpose: Astrological aspect detection and analysis
- Location: `src/aspects.rs`
- Contains: Aspect types, aspect detection algorithms, grand trine detection, moon void-of-course calculation
- Depends on: `chart` module
- Used by: Renderer, OutputHandler, CLI

**Rendering Layer:**
- Purpose: 2D chart visualization generation
- Location: `src/renderer.rs`
- Contains: `Renderer` struct, drawing primitives, PNG/WebP export
- Depends on: `chart`, `aspects` modules, `tiny-skia`, `fontdue`
- Used by: CLI, Server

**Output Layer:**
- Purpose: Multi-format output handling
- Location: `src/output_handler.rs`
- Contains: `OutputHandler`, format dispatch, Markdown table generation
- Depends on: `chart`, `aspects`, `renderer` modules
- Used by: CLI

**Server Layer:**
- Purpose: HTTP API for chart generation
- Location: `src/server/mod.rs`
- Contains: Axum routes, request handlers
- Depends on: `chart`, `renderer`, `swiss_eph_impl` modules
- Used by: CLI (via `serve` subcommand)

**Configuration Layer:**
- Purpose: Application configuration management
- Location: `src/config.rs`
- Contains: Config structs, RON parsing, validation
- Depends on: None
- Used by: CLI

**Error Layer:**
- Purpose: Centralized error types and conversions
- Location: `src/errors.rs`
- Contains: `Error` enum, `From` implementations
- Depends on: `chart` module
- Used by: All layers

**Database Layer (Optional):**
- Purpose: PostgreSQL persistence for chart data
- Location: `src/database/`
- Contains: Connection pooling, schema definitions
- Depends on: `sqlx` (feature-gated)
- Used by: Not currently integrated (feature flag `db`)

## Data Flow

**Chart Generation Flow:**

1. CLI parses arguments (`src/cli/app.rs`)
2. Load configuration from RON file or defaults (`src/config.rs`)
3. Parse time input or use current time, convert to Julian Day (`src/ephemeris/mod.rs`)
4. Create `ChartConfig` with location, house system, and Julian Day
5. Initialize `SwissEphChartCalculator` (wrapper around Swiss Ephemeris library)
6. Calculate planet positions using `swiss-eph` crate (`src/swiss_eph_impl.rs`)
7. Calculate house cusps using selected house system
8. Calculate sidereal time
9. Assemble `ChartData` struct
10. If rendering: Create `Renderer`, render chart wheel with zodiac, houses, planets, aspects
11. If Markdown: Generate tables via `OutputHandler`
12. Save to file system

**HTTP Server Flow:**

1. Start Axum server (`src/server/mod.rs`)
2. Receive GET request to `/chart` with query params (lat, lon, time)
3. Create `ChartConfig` and `SwissEphChartCalculator`
4. Calculate chart data
5. Render to PNG via `Renderer`
6. Return PNG bytes in HTTP response

**Aspect Analysis Flow:**

1. CLI invokes `aspects` subcommand
2. Calculate chart data (same flow as above)
3. Call `analyze_aspects()` from `src/aspects.rs`
4. Detect conjunctions, oppositions, squares, trines, sextiles
5. Find grand trines (three planets in mutual trine)
6. Check for moon void-of-course
7. Print formatted results to stdout

## State Management

**No Persistent Application State:**
- Each command invocation is stateless
- Configuration loaded fresh each run
- No shared mutable state between requests (server creates new calculator per request)

**Ephemeris State:**
- Swiss Ephemeris uses global state via `SE_EPHE_PATH` environment variable
- `Ephemeris::ensure_initialized()` sets path to `data/` directory
- Path must be set before any calculations

## Key Abstractions

**ChartCalculator Trait:**
- Purpose: Abstract interface for chart calculation backends
- Location: `src/chart.rs` (lines 136-145)
- Pattern: Strategy pattern - allows swapping calculation implementations
- Current implementation: `SwissEphChartCalculator` in `src/swiss_eph_impl.rs`
- Methods: `new()`, `calculate_planets()`, `calculate_houses()`, `calculate_sidereal_time()`, `calculate_chart()`

**OutputFormat Enum:**
- Purpose: Unified interface for different output formats
- Location: `src/output_handler.rs` (lines 7-29)
- Pattern: Type-safe dispatch
- Variants: `Png`, `Webp`, `Markdown`

**HouseSystem Enum:**
- Purpose: Type-safe house system selection
- Location: `src/chart.rs` (lines 61-73)
- Supports: Placidus, Koch, Equal, Whole, Porphyry, Regiomontanus, Campanus, Morinus, Alcabitus, Topocentric, Vehlow

## Entry Points

**Binary Entry Point:**
- Location: `src/bin/main.rs`
- Triggers: Direct execution of compiled binary
- Responsibilities: Instantiate `App`, run it, handle errors

**Library Entry Point:**
- Location: `src/lib.rs`
- Triggers: External crate usage
- Responsibilities: Module exports, public API surface

**HTTP Server:**
- Location: `src/server/mod.rs`
- Triggers: `astro-clock serve` CLI command
- Responsibilities: Bind to TCP port, route requests, generate chart images

## Error Handling

**Strategy:** Structured error types with `thiserror`

**Patterns:**
- Domain errors in `src/chart.rs` (`ChartError`)
- Application errors in `src/errors.rs` (`Error`)
- Automatic conversion via `From` implementations
- `?` operator used throughout for propagation
- Tracing for error logging in server context

**Error Types:**
- `Config`: Invalid configuration values
- `Io`: File system operations
- `SwissEphemeris`: Ephemeris calculation failures
- `Rendering`: Image generation errors
- `HttpServer`: Server startup/runtime errors
- `Database`: Database operations (feature-gated)
- `Chart`: Chart calculation failures

## Cross-Cutting Concerns

**Logging:**
- Framework: `tracing` with `tracing-subscriber`
- Initialization: `src/logging.rs`
- Levels: ERROR (quiet), INFO (default), DEBUG (verbose)
- Output: stderr

**Configuration:**
- Format: RON (Rusty Object Notation)
- Default file: `config.ron`
- Validation: Range checking for coordinates, dimensions, orb values
- CLI overrides: All config values can be overridden via CLI flags

**Validation:**
- Location: Config structs have `validate()` methods
- Checks: Latitude (-90 to 90), Longitude (-180 to 180), Dimensions (100-4096), Orb (0-15°), House system validity

---

*Architecture analysis: 2026-02-24*
