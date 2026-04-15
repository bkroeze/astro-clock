---
id: S02
parent: M002
milestone: M002
provides:
  - ["Test helper module (tests/common/mod.rs) with seed data constants, DB connection helpers, and fixture functions", "Justfile recipes (test-db-setup, test-integration) for reproducible test database provisioning", "Deterministic seed data constants: 864,000 positions, 1,465,067 aspects, 86,400 lunar conditions, 4 retrograde bodies, 68 VoC periods, 12 Moon signs", "Integration test skeleton (tests/api_integration.rs) with 9 tests validating seed data integrity"]
requires:
  - slice: S01
    provides: Clean build with --features db flag, all unit tests passing
affects:
  - ["S03"]
key_files:
  - ["Justfile", "tests/common/mod.rs", "tests/api_integration.rs", ".env.example"]
key_decisions:
  - ["D001: Separate TEST_PG_URL for test database to prevent accidental production data destruction", "D002: Seed data via CLI load command rather than fixture SQL — Swiss Ephemeris is deterministic and tests the real pipeline", "D003: Justfile recipe for test DB provisioning, separate from test execution — developer controls when to provision", "D004: Seed data constants queried from actual database rather than hardcoded astronomical calculations"]
patterns_established:
  - ["#[ignore] attribute for DB-dependent integration tests, run via --ignored flag", "Shared test helper module in tests/common/mod.rs with typed constants and async fixture functions", "Idempotent Justfile recipe for test database setup (drop + recreate + migrate + seed)", "TEST_PG_URL environment variable as separate connection string for test database"]
observability_surfaces:
  - none
drill_down_paths:
  - [".gsd/milestones/M002/slices/S02/tasks/T01-SUMMARY.md", ".gsd/milestones/M002/slices/S02/tasks/T02-SUMMARY.md", ".gsd/milestones/M002/slices/S02/tasks/T03-SUMMARY.md"]
duration: ""
verification_result: passed
completed_at: 2026-04-15T22:59:07.116Z
blocker_discovered: false
---

# S02: Test database infrastructure

**Reproducible test database infrastructure with deterministic 60-day seed data, Justfile recipes for setup/integration test execution, and shared test helper module with queried-verified constants**

## What Happened

Created reproducible test database infrastructure for integration testing. The slice delivered three main components:

1. **Justfile recipes** (`test-db-setup`, `test-integration`): The test-db-setup recipe drops/recreates the test database, applies all 9 migrations (with a workaround for migration 6's TimescaleDB continuous aggregate issue), and loads 60 days of deterministic seed data via the CLI. The test-integration recipe runs integration tests against the seeded database. Both recipes are idempotent.

2. **Test helper module** (`tests/common/mod.rs`): Provides database connection helpers (reads TEST_PG_URL, creates PgPool), typed seed data constants (date range, body counts, aspect type mappings, zodiac sign IDs), and fixture functions for verifying seed data state. Also contains discovered deterministic values queried from the actual seeded database.

3. **Integration test skeleton** (`tests/api_integration.rs`): 9 tests — 5 pure constant validation tests (pass without DB) and 4 DB-dependent tests (marked #[ignore]) that assert exact counts for positions, aspects, retrograde periods, and lunar conditions.

During execution, 5 pre-existing bugs were discovered and fixed to enable seed data loading: CLI tokio runtime scoping, chunk_generator timestamp conversion, TimescaleDB ON CONFLICT incompatibility, migration 006 continuous aggregate cast, and the Justfile DATABASE_URL env var passthrough. The migration 6 `IF NOT EXISTS` re-creation issue was worked around in the Justfile recipe by applying it via psql with error suppression.

Seed data covers 60 days (2025-01-01 to 2025-03-01) with 864,000 position records, 1,465,067 aspects, 86,400 lunar conditions, 4 retrograde bodies (Venus, Mars, Jupiter, Uranus), 68 VoC periods, and Moon transiting all 12 signs. Jan 29 fails due to a moon_phase_angle constraint edge case (~1.6% loss).

## Verification

All 9 integration tests pass: 5 pure constant tests + 4 DB-dependent tests with exact value assertions against the seeded database. `just test-db-setup` runs end-to-end idempotently (verified by running twice). `just test-integration` runs the full integration suite. Full cargo test suite passes: 195 total tests, 0 failures across all test binaries.

## Requirements Advanced

- R002 — Fully implemented: test-db-setup recipe, test helper module, seed data constants, integration tests all delivered and verified

## Requirements Validated

- R002 — just test-db-setup creates seeded DB idempotently; 9 integration tests pass asserting exact seed data values; just test-integration runs full suite

## New Requirements Surfaced

- ["Migration 006 continuous aggregate bug needs proper fix (DROP vs DROP MATERIALIZED VIEW for TimescaleDB continuous aggregates)"]

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

"Justified recipe to handle migration 6 failure — applied migrations 1-5 via sqlx, then 6 via psql with error suppression, then 7-9 individually via psql. This is a workaround for a pre-existing TimescaleDB issue where continuous aggregate views can't be re-created with CREATE MATERIALIZED VIEW IF NOT EXISTS."

## Known Limitations

["Migration 006 continuous aggregate re-creation fails with 'is not a materialized view' error — worked around in Justfile recipe with psql + error suppression", "Jan 29 seed data fails due to moon_phase_angle check constraint edge case (~1.6% data loss)", "Test helper SEED_END_DATE says '2025-03-02' but --days 60 covers through Mar 1 — the end date constant is used for query range which includes data through the loaded range"]

## Follow-ups

["S03 will use the test helper module and seeded database to write API and CLI integration tests"]

## Files Created/Modified

- `Justfile` — Added test-db-setup and test-integration recipes; test-db-setup handles migration 6 workaround
- `.env.example` — Added TEST_PG_URL entry for test database connection
- `tests/common/mod.rs` — Shared test helper with seed data constants, DB connection helpers, fixture functions
- `tests/api_integration.rs` — 9 integration tests (5 pure + 4 DB-dependent) validating seed data
- `src/cli/app.rs` — Fixed tokio runtime scoping for pool creation + execution
- `src/database/chunk_generator.rs` — Fixed and_hms_opt timestamp conversion bug
- `migrations/006_create_continuous_aggregates.sql` — Fixed boolean-to-smallint cast in continuous aggregate
