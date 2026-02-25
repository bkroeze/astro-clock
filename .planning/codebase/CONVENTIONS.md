# Coding Conventions

**Analysis Date:** 2026-02-24

## Naming Patterns

**Files:**
- Use `snake_case.rs` for all source files (e.g., `swiss_eph_impl.rs`, `output_handler.rs`)
- Module directories use lowercase (e.g., `cli/`, `ephemeris/`, `database/`, `server/`)
- Main entry point: `src/bin/main.rs`
- Library root: `src/lib.rs`

**Structs/Enums:**
- Use `PascalCase` for types (e.g., `ChartCalculator`, `HouseSystem`, `PlanetPosition`)
- Error types use `Error` suffix (e.g., `ChartError`, `ConfigError`)
- Configuration types use `Config` suffix (e.g., `ChartConfig`, `AppConfig`)

**Functions/Methods:**
- Use `snake_case` for functions (e.g., `calculate_planets()`, `from_file_or_default()`)
- Constructor functions use `new()` or `with_*` prefix (e.g., `with_path()`)
- Validation methods use `validate()` or `is_*` prefix (e.g., `is_moon_void_of_course()`)
- Getter methods use field name directly (no `get_` prefix)

**Variables:**
- Use `snake_case` for variables (e.g., `geo_pos`, `julian_day`, `aspect_radius`)
- Constants use `SCREAMING_SNAKE_CASE` (e.g., `DEG_TO_RAD`, `SUN`, `MOON`)
- Module-level constants grouped in `constants` module

**Type Aliases:**
- Result type alias: `pub type Result<T> = std::result::Result<T, Error>;`

## Code Style

**Formatting:**
- Standard Rust formatting via `rustfmt`
- No custom `.rustfmt.toml` detected - using defaults

**Linting:**
- No custom clippy configuration detected
- Uses standard Rust compiler warnings

**Line Length:**
- No strict limit observed; lines occasionally exceed 100 characters

## Import Organization

**Order:**
1. Standard library imports (`std::*`)
2. External crate imports
3. Internal crate imports (`crate::*`)

**Example from `src/renderer.rs`:**
```rust
use std::path::Path;
use tiny_skia::{FillRule, Paint, Pixmap, Transform};

use crate::aspects::{find_aspects, AspectConfig, AspectType};
```

**Path Aliases:**
- No custom path aliases defined in `Cargo.toml`
- Uses standard module paths

## Error Handling

**Pattern:** Uses `thiserror` for error enums with structured error types

**Example from `src/errors.rs`:**
```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Swiss Ephemeris error: {0}")]
    SwissEphemeris(String),
    // ...
}

pub type Result<T> = std::result::Result<T, Error>;
```

**Error Conversion:**
- Implement `From` traits for automatic error conversion
- Map domain-specific errors to application errors in `From` implementations

**Example:**
```rust
impl From<ron::de::Error> for Error {
    fn from(err: ron::de::Error) -> Self {
        Error::Config(err.to_string())
    }
}
```

## Logging

**Framework:** `tracing` crate with `tracing-subscriber`

**Patterns:**
- Use `tracing::info!()` for operational messages
- Use `tracing::debug!()` for detailed debugging
- Use `tracing::error!()` for errors

**Example from `src/cli/app.rs`:**
```rust
tracing::info!("Starting Astro Clock");
tracing::debug!("CLI arguments: {:#?}", self.cli);
tracing::error!("Failed to create chart calculator: {}", e);
```

**Initialization:**
```rust
pub fn init_logging(level: tracing::Level) {
    let filter = EnvFilter::from_default_env().add_directive(level.into());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_writer(std::io::stderr)
        .try_init()
        .ok();
}
```

## Comments

**When to Comment:**
- Document public APIs with doc comments (`///`)
- Explain complex algorithms (e.g., aspect calculations)
- Mark sections that need attention with `//`

**Doc Comments:**
```rust
/// The type of astrological aspect
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AspectType {
    /// Planets at the same longitude (0°)
    Conjunction,
    /// ...
}
```

**Code Comments:**
- Use `//` for inline explanations
- Keep comments current with code changes

## Function Design

**Size:**
- Functions typically 10-50 lines
- Complex rendering broken into smaller methods (`draw_*`)

**Parameters:**
- Prefer struct parameters for related data
- Use builder pattern for complex construction

**Return Values:**
- Return `Result<T, Error>` for fallible operations
- Return `Option<T>` for optional values
- Use `Self` in trait implementations

**Example:**
```rust
pub fn new(house_system: HouseSystem, geo_pos: GeoPos, julian_day: f64) -> Self {
    Self {
        house_system,
        geo_pos,
        julian_day,
    }
}
```

## Module Design

**Structure:**
- Each module in its own file or directory
- `mod.rs` or module file contains public exports
- Private modules expose only intended API

**Example from `src/cli/mod.rs`:**
```rust
mod app;
pub use app::App;
```

**Exports:**
- Use `pub` for public API
- Use `pub(crate)` for crate-internal visibility
- Re-export key types at crate root (`src/lib.rs`)

**Example from `src/lib.rs`:**
```rust
pub use chart::{
    ChartCalculator, ChartData, Error as ChartError, GeoPos, HouseCusps, HouseSystem,
    PlanetPosition, Position,
};
pub use renderer::{Color, Point, Rect, Renderer, Size};
pub use swiss_eph_impl::SwissEphChartCalculator;
```

## Trait Implementation

**Pattern:** Define traits for core abstractions

**Example from `src/chart.rs`:**
```rust
pub trait ChartCalculator {
    fn new(config: ChartConfig) -> Result<Self, Error>
    where
        Self: Sized;

    fn calculate_planets(&self) -> Result<Vec<PlanetPosition>, Error>;
    fn calculate_houses(&self) -> Result<HouseCusps, Error>;
    fn calculate_sidereal_time(&self) -> Result<f64, Error>;
    fn calculate_chart(&self) -> Result<ChartData, Error>;
}
```

## Constants

**Pattern:** Group related constants in modules

**Example from `src/chart.rs`:**
```rust
pub mod planet {
    pub const SUN: &str = "Sun";
    pub const MOON: &str = "Moon";
    // ...
}

pub mod zodiac {
    pub const ARIEST: &str = "♈";
    pub const TAURUS: &str = "♉";
    // ...
}

pub mod constants {
    pub const DEG_TO_RAD: f64 = std::f64::consts::PI / 180.0;
    pub const RAD_TO_DEG: f64 = 180.0 / std::f64::consts::PI;
    pub const AU_TO_KM: f64 = 149_597_870.7;
}
```

## Configuration

**Pattern:** Use RON format with serde

**Example from `src/config.rs`:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    #[serde(default)]
    pub chart: ChartConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
}
```

**Default Values:**
- Implement `Default` trait
- Use `serde(default)` attribute
- Use `default_*` functions for complex defaults

## CLI Design

**Pattern:** Use `clap` derive macros

**Example from `src/cli/app.rs`:**
```rust
#[derive(Parser, Debug)]
#[command(name = "astro-clock")]
#[command(about = "An astrological chart rendering application", long_about = None)]
#[command(version)]
pub struct Cli {
    #[arg(short, long, global = true, value_name = "FILE")]
    pub config: Option<PathBuf>,
    // ...
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate an astrological chart
    Chart { /* ... */ },
    /// Serve charts over HTTP
    Serve { /* ... */ },
}
```

---

*Convention analysis: 2026-02-24*
