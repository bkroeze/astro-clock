# Testing Patterns

**Analysis Date:** 2026-02-24

## Test Framework

**Runner:** Built-in Rust test framework (`cargo test`)

**Assertion Library:** Standard `assert!`, `assert_eq!`, `assert_ne!` macros

**Additional Testing Dependencies:**
- `tempfile = "3"` - Temporary file creation for config tests
- `tower = { version = "0.5", features = ["util"] }` - HTTP testing utilities
- `http-body-util = "0.1"` - HTTP body handling in tests
- `http = "1.0"` - HTTP types for testing

**Run Commands:**
```bash
cargo test              # Run all tests
cargo test --lib        # Run library tests only
cargo test <module>     # Run specific module tests (e.g., `cargo test aspects`)
cargo test <test_name>  # Run specific test by name
```

## Test File Organization

**Location:** Tests are co-located with source code using `#[cfg(test)]` modules

**Pattern:** Each module file contains its own tests at the bottom:

```rust
// src/chart.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geo_pos_creation() {
        let pos = GeoPos::new(40.7128, -74.0060, 0.0);
        assert_eq!(pos.latitude, 40.7128);
        assert_eq!(pos.longitude, -74.0060);
        assert_eq!(pos.altitude, 0.0);
    }
}
```

**Test Module Structure:**
- Tests are in a `tests` submodule within each source file
- Use `use super::*;` to import parent module items
- Test functions use `#[test]` attribute

## Test Structure

**Naming Convention:**
- Test functions use `test_*` prefix
- Descriptive names indicating what's being tested
- Examples: `test_geo_pos_creation`, `test_find_conjunction`, `test_config_validation`

**Basic Pattern:**
```rust
#[test]
fn test_aspect_type_properties() {
    assert_eq!(AspectType::Conjunction.angle(), 0.0);
    assert_eq!(AspectType::Opposition.angle(), 180.0);
    assert_eq!(AspectType::Conjunction.name(), "Conjunction");
}
```

**Async Testing:**
```rust
#[tokio::test]
async fn test_health_endpoint() {
    let app = axum::Router::new().route("/health", get(health_handler));
    let response = tower::ServiceBuilder::new()
        .service(app)
        .oneshot(
            http::Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert!(response.unwrap().status().is_success());
}
```

## Mocking

**Approach:** No mocking framework used; tests use real implementations or simple stubs

**External Dependencies:**
- Tests use actual Swiss Ephemeris library
- Tests use real file system via `tempfile` crate

**Example - Config File Testing:**
```rust
#[test]
fn test_config_from_file() {
    let mut temp_file = NamedTempFile::new().unwrap();
    let config_content = r##"
        (
            chart: (
                width: 1024,
                height: 768,
            ),
        )
    "##;
    temp_file.write_all(config_content.as_bytes()).unwrap();

    let config = AppConfig::from_file(temp_file.path()).unwrap();
    assert_eq!(config.chart.width, 1024);
    assert_eq!(config.chart.height, 768);
}
```

## Fixtures and Factories

**Test Data Helpers:**
- Simple helper functions for creating test objects
- No formal fixture framework

**Example from `src/aspects.rs`:**
```rust
#[cfg(test)]
mod tests {
    fn create_planet(name: &str, longitude: f64) -> PlanetPosition {
        PlanetPosition::new(
            name,
            Position::new(longitude, 0.0, 1.0, 0.0, 0.0, 0.0),
            false,
        )
    }

    #[test]
    fn test_find_conjunction() {
        let planets = vec![
            create_planet(planet::SUN, 10.0),
            create_planet(planet::MOON, 12.0),
        ];
        // ...
    }
}
```

**Location:** Helper functions defined within `#[cfg(test)]` modules

## Coverage

**Current Status:** 51 tests passing, 0 failures

**Test Distribution:**
- `aspects.rs`: 14 tests (aspect calculations, grand trines, moon void)
- `chart.rs`: 3 tests (struct creation, Display trait)
- `cli/app.rs`: 7 tests (CLI parsing, extension handling)
- `config.rs`: 10 tests (validation, file parsing, defaults)
- `ephemeris/mod.rs`: 12 tests (Julian day calculations, ephemeris init)
- `lib.rs`: 1 test (integration test)
- `logging.rs`: 1 test (logging init)
- `output_handler.rs`: 2 tests (format parsing)
- `renderer.rs`: 3 tests (creation, text rendering, chart rendering)
- `server/mod.rs`: 1 test (health endpoint)
- `swiss_eph_impl.rs`: 3 tests (calculator creation, planet calc, houses)

**Coverage Gaps:**
- No integration tests in `tests/` directory
- Limited error path testing
- No property-based testing
- Database module (`db` feature) has no tests

## Test Types

**Unit Tests:**
- Co-located with source code
- Test individual functions and methods
- Fast execution (< 1 second total)

**Integration Tests:**
- Single integration test in `src/lib.rs`
- Tests Swiss Ephemeris integration end-to-end
- No separate `tests/` directory

**Example Integration Test:**
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_swiss_eph_integration() {
        let config = crate::chart::ChartConfig::new(
            crate::chart::HouseSystem::Placidus,
            crate::chart::GeoPos::new(40.7128, -74.0060, 0.0),
            2451545.0,
        );
        let calculator = crate::swiss_eph_impl::SwissEphChartCalculator::new(config);
        assert!(calculator.is_ok());
    }
}
```

## Common Patterns

**Float Comparison:**
```rust
#[test]
fn test_julian_day_from_datetime() {
    let jd = julian_day_from_datetime(2000, 1, 1, 12.0);
    assert!((jd - 2451545.0).abs() < 0.0001);
}
```

**Error Testing:**
```rust
#[test]
fn test_config_validation() {
    let mut config = AppConfig::default();
    config.chart.width = 50; // Invalid: too small
    assert!(config.validate().is_err());
}

#[test]
fn test_config_parse_error() {
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(b"invalid ron content here").unwrap();
    let result = AppConfig::from_file(temp_file.path());
    assert!(result.is_err());
}
```

**Option/Result Testing:**
```rust
#[test]
fn test_find_conjunction() {
    let aspects = find_aspects(&planets, config);
    let opposition = aspects
        .iter()
        .find(|a| a.aspect_type == AspectType::Opposition);
    assert!(opposition.is_some());
}
```

**Environment Variable Testing:**
```rust
#[test]
fn test_ephemeris_env_override() {
    unsafe {
        env::set_var("SE_EPHE_PATH", "/env/path");
    }
    let eph = Ephemeris::with_path(None);
    assert!(eph.is_ok());
    assert_eq!(eph.unwrap().path(), "/env/path");
    unsafe {
        env::remove_var("SE_EPHE_PATH");
    }
}
```

## Test Organization Best Practices

**Module-Level Tests:**
- Keep tests close to code being tested
- Use `mod tests` with `#[cfg(test)]` attribute
- Import parent scope with `use super::*;`

**Test Independence:**
- Each test is self-contained
- Tests don't depend on execution order
- Use `tempfile` for file operations to avoid side effects

**Cleanup:**
- Tests that create files clean up after themselves
- Example from `src/renderer.rs`:
```rust
#[test]
fn test_text_rendering() {
    // ... render text ...
    renderer.save("test_text.png").unwrap();
    let path = std::path::Path::new("test_text.png");
    assert!(path.exists());
    std::fs::remove_file(path).unwrap(); // Cleanup
}
```

## Running Tests

**All Tests:**
```bash
cargo test
```

**Specific Module:**
```bash
cargo test aspects
cargo test config
cargo test ephemeris
```

**Specific Test:**
```bash
cargo test test_find_conjunction
cargo test test_config_validation
```

**With Output:**
```bash
cargo test -- --nocapture
```

**Features:**
```bash
cargo test --features db  # Test with database feature enabled
```

---

*Testing analysis: 2026-02-24*
