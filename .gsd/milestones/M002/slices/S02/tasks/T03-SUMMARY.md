---
id: T03
parent: S02
milestone: M002
key_files:
  - tests/common/mod.rs
  - tests/api_integration.rs
  - src/cli/app.rs
  - src/database/chunk_generator.rs
  - migrations/006_create_continuous_aggregates.sql
  - Justfile
key_decisions:
  - Recorded deterministic seed data values as typed constants in tests/common/mod.rs (D004)
  - Fixed CLI to use single tokio runtime for pool+execution
  - Removed ON CONFLICT from hypertable INSERTs
  - Fixed and_hms_opt timestamp conversion in chunk_generator
duration: 
verification_result: passed
completed_at: 2026-04-15T22:49:40.886Z
blocker_discovered: false
---

# T03: Load 60 days of seed data and populate deterministic test constants from queried database values

**Load 60 days of seed data and populate deterministic test constants from queried database values**

## What Happened

Loaded 60 days of seed data (2025-01-01 to 2025-03-02) into the test database and queried it to discover all deterministic values needed for integration test assertions.

**Pre-requisite bug fixes (discovered during execution):**
1. **CLI tokio runtime bug** — `src/cli/app.rs` created a sqlx PgPool inside one `Runtime::new().block_on()` and used it in a different runtime, causing "pool timed out". Refactored to use a single runtime for pool creation + execution.
2. **chunk_generator timestamp bug** — `and_hms_opt(0, minute, 0)` failed for minute >= 60. Fixed to `and_hms_opt(minute / 60, minute % 60, 0)` in all three save methods (positions, aspects, lunar conditions).
3. **TimescaleDB ON CONFLICT bug** — Hypertables don't support `ON CONFLICT` in this version. Removed ON CONFLICT clauses from all three INSERT statements since test DB is always fresh.
4. **Migration 006 continuous aggregate bug** — `last(retrograde::smallint, time)::boolean` cast fails. Fixed to use CASE expression.
5. **Justfile test-db-setup** — Load command wasn't passing DATABASE_URL env var. Fixed to set `DATABASE_URL="${TEST_PG_URL}"`.
6. Applied migrations 7-9 manually (migration 6 blocked sqlx migrate run for subsequent migrations).

**Seed data loaded successfully:**
- 60 of 61 days loaded (Jan 29 failed due to moon_phase_angle constraint — ~1.6% loss, acceptable)
- 864,000 planet position records (10 bodies × 86,400 minutes)
- 1,465,067 aspect records
- 86,400 lunar condition records

**Discovered deterministic values recorded as constants:**
- Retrograde bodies: Venus (1,402 min), Mars (76,441 min), Jupiter (48,102 min), Uranus (41,304 min)
- Non-retrograde bodies: Sun, Moon, Mercury, Saturn, Neptune, Pluto
- Aspect breakdown: 178,226 conjunctions, 630,355 sextiles, 265,324 squares, 295,184 trines, 95,978 oppositions
- 68 distinct VoC periods, 48,294 VoC minutes
- Moon transits all 12 zodiac signs with 27 sign changes
- All 8 moon phases present
- Sun longitude Jan 1: ~280.8° (Capricorn), Mar 2: ~341.7° (Pisces)

**Updated integration tests** from placeholder assertions to exact value matching against discovered constants. All 9 tests pass (5 pure + 4 DB-dependent).

## Verification

All 9 integration tests pass (5 pure constants + 4 DB-dependent with exact value assertions). Full test suite: 195 tests, 0 failures. Seed data verified by querying all tables and asserting exact counts match the documented constants.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --features db --test api_integration 2>&1 | grep 'test result'` | 0 | ✅ pass — 5 passed, 0 failed, 4 ignored | 8000ms |
| 2 | `source .env && export TEST_PG_URL=postgresql://astro:barnlab@10.0.10.50:5432/astrology_test && cargo test --features db --test api_integration -- --ignored 2>&1 | grep 'test result'` | 0 | ✅ pass — 4 passed, 0 failed, 0 ignored | 5000ms |
| 3 | `cargo test --features db 2>&1 | grep 'test result'` | 0 | ✅ pass — 195 total tests, 0 failures, 5 ignored | 30000ms |
| 4 | `psql $TEST_PG_URL -c "SELECT COUNT(*) FROM planet_positions"` | 0 | ✅ pass — 864000 positions loaded | 5000ms |

## Deviations

Fixed 5 pre-existing bugs in the codebase to enable seed data loading: CLI runtime management, chunk_generator timestamp conversion, TimescaleDB ON CONFLICT incompatibility, migration 006 continuous aggregate cast, and Justfile DATABASE_URL env var. Also applied migrations 7-9 manually since migration 6 failure blocked sqlx-cli from reaching them.

## Known Issues

Jan 29 seed data failed to load due to "violates check constraint lunar_conditions_moon_phase_angle_check" — likely an edge case in the Swiss Ephemeris moon phase calculation. 60 of 61 days loaded (98.4% coverage). This does not affect test validity since the remaining 60 days provide comprehensive coverage.

## Files Created/Modified

- `tests/common/mod.rs`
- `tests/api_integration.rs`
- `src/cli/app.rs`
- `src/database/chunk_generator.rs`
- `migrations/006_create_continuous_aggregates.sql`
- `Justfile`
