# Codebase Concerns

**Analysis Date:** 2026-02-24

## Tech Debt

### Error Handling Inconsistencies
- **Issue:** Mixed error handling patterns across the codebase
- **Files:** `src/errors.rs`, `src/chart.rs`, `src/config.rs`
- **Impact:** Multiple error types (`crate::errors::Error`, `crate::chart::Error`, `crate::config::ConfigError`) create confusion about which to use
- **Fix approach:** Consolidate error types into a single hierarchy or use `thiserror` derive macros consistently

### Hardcoded Values
- **Issue:** Magic numbers and hardcoded defaults scattered throughout
- **Files:** `src/renderer.rs` (lines 166, 276, 279, 360), `src/server/mod.rs` (line 69)
- **Impact:** Difficult to maintain consistency when defaults need to change
- **Fix approach:** Centralize defaults in `src/config.rs` and reference via constants

### Unwrap Usage in Non-Test Code
- **Issue:** Several `unwrap()` calls in production code paths
- **Files:**
  - `src/renderer.rs:150` - `Pixmap::new(...).unwrap()`
  - `src/renderer.rs:371` - `unwrap_or(&default_symbol)`
  - `src/renderer.rs:434` - `Rect::from_xywh(...).unwrap()`
  - `src/renderer.rs:446` - `path.finish().unwrap()`
  - `src/renderer.rs:470` - `path.finish().unwrap()`
  - `src/renderer.rs:496` - `path.finish().unwrap()`
  - `src/renderer.rs:524` - `symbol_font.as_ref().unwrap()`
  - `src/renderer.rs:566` - `PremultipliedColorU8::from_rgba(...).unwrap()`
  - `src/ephemeris/mod.rs:115` - `data_dir.to_str().unwrap()`
- **Impact:** Potential panics in production instead of graceful error handling
- **Fix approach:** Replace with proper error propagation using `?` operator

### Unsafe Code Blocks
- **Issue:** `unsafe` blocks used for environment variable manipulation in tests
- **Files:** `src/ephemeris/mod.rs:172-178`
- **Impact:** Tests modify global state unsafely, could cause flaky tests in parallel execution
- **Fix approach:** Use `std::sync::Mutex` for test isolation or use `temp_env` crate

### Server Hardcoded Julian Day
- **Issue:** HTTP server endpoint uses hardcoded Julian day (2451545.0) instead of parsing from query params
- **Files:** `src/server/mod.rs:69`
- **Impact:** Server always returns chart for fixed date (J2000.0 epoch)
- **Fix approach:** Implement proper time parsing from `query.time` parameter

## Known Bugs

### Time Parameter Ignored in Server
- **Symptoms:** Server `/chart` endpoint ignores the `time` query parameter
- **Files:** `src/server/mod.rs:64`
- **Trigger:** Any request to `/chart?time=...`
- **Workaround:** None - server always uses hardcoded Julian day

### Font Loading Failure Silent
- **Symptoms:** If symbol font fails to load, rendering silently degrades
- **Files:** `src/renderer.rs:17-41`
- **Trigger:** System without Noto Sans Symbols font installed
- **Workaround:** None - no warning or error is emitted

### House System String Parsing Duplication
- **Symptoms:** House system parsing logic duplicated in two places
- **Files:** `src/cli/app.rs:314-331`, `src/config.rs:115-135`
- **Trigger:** Adding new house system requires changes in multiple places
- **Workaround:** None - maintain both locations

## Security Considerations

### Path Traversal Risk in Output Handler
- **Risk:** Output path is passed directly to file operations without sanitization
- **Files:** `src/output_handler.rs:54`, `src/output_handler.rs:67`, `src/output_handler.rs:77`, `src/output_handler.rs:87`
- **Current mitigation:** Uses standard library path handling
- **Recommendations:** Validate output paths are within allowed directories

### Database Connection String in Config
- **Risk:** Default database URL contains placeholder credentials
- **Files:** `src/config.rs:263-265`
- **Current mitigation:** Database feature is optional and disabled by default
- **Recommendations:** Remove default credentials, require explicit configuration

### No Input Validation on Server Query Params
- **Risk:** Server accepts arbitrary lat/lon values without bounds checking
- **Files:** `src/server/mod.rs:61-63`
- **Current mitigation:** None
- **Recommendations:** Add validation matching CLI bounds (-90 to 90, -180 to 180)

## Performance Bottlenecks

### Renderer Font Rasterization
- **Problem:** Text rendering rasterizes characters individually without caching
- **Files:** `src/renderer.rs:514-573`
- **Cause:** `font.rasterize()` called for every character on every render
- **Improvement path:** Implement glyph cache using `HashMap<(char, f32), GlyphCache>`

### Aspect Calculation O(n²)
- **Problem:** Finding aspects uses nested loops over all planet pairs
- **Files:** `src/aspects.rs:168-196`
- **Cause:** `O(n²)` complexity with n=14 planets is acceptable but not optimal
- **Improvement path:** Consider spatial indexing if planet count increases

### PNG Encoding Allocations
- **Problem:** `export_png()` allocates large intermediate buffer
- **Files:** `src/renderer.rs:590-613`
- **Cause:** `rgba_data` Vec allocated fresh for each export
- **Improvement path:** Reuse buffer or write directly to encoder

## Fragile Areas

### Swiss Ephemeris Integration
- **Files:** `src/swiss_eph_impl.rs`, `src/ephemeris/mod.rs`
- **Why fragile:** Depends on external C library (swiss-eph crate) and binary data files in `data/`
- **Safe modification:** Always test with `cargo test` after changes, verify data files present
- **Test coverage:** Good - has integration tests

### Font Loading System
- **Files:** `src/renderer.rs:12-46`
- **Why fragile:** Hardcoded system paths for symbol fonts, falls back silently
- **Safe modification:** Test on multiple Linux distributions, consider bundling symbol font
- **Test coverage:** Limited - only tests basic font loading

### Database Schema (Optional Feature)
- **Files:** `src/database/schema.rs`, `src/database/pool.rs`
- **Why fragile:** Minimal implementation, not integrated with main application flow
- **Safe modification:** Feature-gated with `#[cfg(feature = "db")]`
- **Test coverage:** None - no database tests in test suite

### Renderer Path Operations
- **Files:** `src/renderer.rs:446`, `src/renderer.rs:470`, `src/renderer.rs:496`
- **Why fragile:** `path.finish().unwrap()` will panic on invalid path data
- **Safe modification:** Validate path construction, handle None case gracefully
- **Test coverage:** Tests exist but may not cover edge cases

## Scaling Limits

### Server Concurrency
- **Current capacity:** Single-threaded Axum server with default settings
- **Limit:** No connection pooling or rate limiting implemented
- **Scaling path:** Add `tower::limit` middleware, consider horizontal scaling

### Chart Rendering
- **Current capacity:** Synchronous rendering in request handler
- **Limit:** Large chart dimensions (up to 4096x4096) could block async runtime
- **Scaling path:** Move rendering to blocking thread pool using `tokio::task::spawn_blocking`

### Database Connections
- **Current capacity:** 5 connections in pool (`src/database/pool.rs:13`)
- **Limit:** Hardcoded limit may be insufficient for high load
- **Scaling path:** Make configurable via `AppConfig`

## Dependencies at Risk

### swiss-eph (v0.2.1)
- **Risk:** Niche astronomy crate with potentially limited maintenance
- **Impact:** Core functionality depends on this for all calculations
- **Migration plan:** Monitor for updates, consider vendoring or forking if abandoned

### tiny-skia (v0.8)
- **Risk:** Version 0.8 is not latest (0.11 available)
- **Impact:** Missing bug fixes and performance improvements
- **Migration plan:** Test upgrade path, API changes likely minimal

### fontdue (v0.8)
- **Risk:** Font rasterization library, potential for rendering edge cases
- **Impact:** Text rendering quality depends on this
- **Migration plan:** Monitor for updates, test with various Unicode ranges

## Missing Critical Features

### Server Time Parsing
- **Problem:** Server endpoint cannot accept custom dates/times
- **Blocks:** Any server-based use case requiring specific chart dates
- **Priority:** High

### Configuration Hot-Reload
- **Problem:** Config file changes require application restart
- **Blocks:** Dynamic configuration updates in long-running server
- **Priority:** Medium

### Database Integration
- **Problem:** Database feature exists but is not wired into CLI or server
- **Blocks:** Chart persistence, historical queries
- **Priority:** Medium

### Logging Configuration
- **Problem:** Log level set at startup, no runtime adjustment
- **Blocks:** Debugging production issues without restart
- **Priority:** Low

## Test Coverage Gaps

### Database Module
- **What's not tested:** All database functionality (feature-gated, no tests)
- **Files:** `src/database/pool.rs`, `src/database/schema.rs`
- **Risk:** Database code could break silently
- **Priority:** Medium

### Server Error Handling
- **What's not tested:** Error responses for invalid inputs, calculator failures
- **Files:** `src/server/mod.rs:72-110`
- **Risk:** Server may return 500 errors instead of proper error responses
- **Priority:** High

### Edge Cases in Rendering
- **What's not tested:** Very large/small chart sizes, invalid planet data, missing fonts
- **Files:** `src/renderer.rs`
- **Risk:** Panics or incorrect rendering in edge cases
- **Priority:** Medium

### Aspect Detection Edge Cases
- **What's not tested:** Planets at exactly 0°/360° boundary, empty planet lists
- **Files:** `src/aspects.rs`
- **Risk:** Incorrect aspect calculations at zodiac boundaries
- **Priority:** Low

---

*Concerns audit: 2026-02-24*
