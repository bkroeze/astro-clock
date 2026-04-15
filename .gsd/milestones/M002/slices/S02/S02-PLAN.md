# S02: Test database infrastructure

**Goal:** Create reproducible test database infrastructure with deterministic seed data covering a 60-day range
**Demo:** just test-db-setup creates a seeded test DB; a test binary can connect, query planet_positions, and assert known values

## Must-Haves

- just test-db-setup recipe creates test DB from scratch\nRecipe is idempotent — re-running produces same state\nSeed data includes at least one Mercury retrograde period\nSeed data includes VoC Moon periods\nplanet_positions has data for all 10 bodies across the 60-day range\naspect_summaries has aggregated data for query optimization

## Proof Level

- This slice proves: integration

## Integration Closure

Test helper module and seed data constants available for S03 integration tests

## Verification

- Test helper module exposes seed data metadata (date range, known values) for test assertions

## Tasks

- [x] **T01: Add just test-db-setup and test-integration recipes** `est:30 min`
  Add `test-db-setup` and `test-integration` recipes to Justfile. test-db-setup: drops and recreates the test database using TEST_PG_URL, runs migrations via sqlx, loads 60 days of seed data via the CLI load command. test-integration: runs cargo test with the integration tests enabled. Also update .env.example to include TEST_PG_URL.
  - Files: `Justfile`, `.env.example`
  - Verify: just test-db-setup exits 0 (requires TEST_PG_URL set and TimescaleDB available)

- [x] **T02: Create test helper module with seed data constants** `est:45 min`
  Create `tests/common/mod.rs` as a shared test helper module. Provides: database connection helper (reads TEST_PG_URL, creates pool), seed data constants (date range, known planet positions, expected query results), test fixture functions (verify seed data loaded, get specific position assertions). This module is shared across integration test files.
  - Files: `tests/common/mod.rs`
  - Verify: cargo test --features db --test api_integration 2>&1 | grep 'error' | wc -l returns 0 (compiles, may fail at runtime without DB)

- [ ] **T03: Load seed data and discover deterministic values** `est:30 min`
  Run just test-db-setup to load 60 days of seed data. Query the database to discover and document the deterministic values: how many retrograde periods exist, which bodies are retrograde, what Moon signs appear, VoC periods present, aspect summary counts. Record these as constants in the test helper module so integration tests can assert against them.
  - Files: `tests/common/mod.rs`
  - Verify: Queries against the test database return the documented seed data values

## Files Likely Touched

- Justfile
- .env.example
- tests/common/mod.rs
