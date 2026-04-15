---
id: T02
parent: S02
milestone: M002
key_files:
  - tests/common/mod.rs
  - tests/api_integration.rs
key_decisions:
  - Created api_integration.rs as the integration test entry point that uses mod common — this is the file referenced by the task verification command
  - Used #![allow(dead_code)] in common/mod.rs since it's a public API for future integration tests, not all functions are used yet
  - SEED_DAY_COUNT=60 represents the date span (Mar 2 minus Jan 1 = 60 days), not the inclusive count (61)
duration: 
verification_result: passed
completed_at: 2026-04-15T22:30:32.720Z
blocker_discovered: false
---

# T02: Create test helper module with seed data constants, DB connection helper, and fixture functions

**Create test helper module with seed data constants, DB connection helper, and fixture functions**

## What Happened

Created the test helper module at `tests/common/mod.rs` and a corresponding integration test skeleton at `tests/api_integration.rs`.

**Test helper module (`tests/common/mod.rs`):**
- **Database connection helper**: `test_pool()` reads `TEST_PG_URL` from the environment and creates a sqlx PgPool with max 5 connections. Also provides `pool_from_url()` for parameterized tests.
- **Seed data constants**: Date range (2025-01-01 to 2025-03-02, 60-day span), body count (10), zodiac sign count (12), aspect type count (5). Helper functions `seed_start_date()`, `seed_end_date()`, `seed_start_utc()`, `seed_end_utc()` for typed access. Constants for body IDs, aspect types, zodiac signs, and moon phases matching the database schema.
- **Known astronomical values**: Placeholder constants for retrograde bodies and planetary longitude ranges. These will be refined in T03 after querying the seeded database.
- **Fixture functions**: `verify_seed_data_loaded()`, `count_positions_for_body()`, `count_aspects()`, `count_retrograde_periods()`, `count_lunar_conditions()`, `count_voc_periods()`, `moon_signs_in_range()`, `position_counts_per_body()`. Assertion helpers: `assert_day_coverage()`, `assert_all_bodies_present()`.

**Integration test skeleton (`tests/api_integration.rs`):**
- 4 pure-constant tests (pass without DB): `test_helper_constants_are_valid`, `test_body_id_constants`, `test_aspect_type_constants`, `test_zodiac_sign_constants`.
- 4 DB-dependent tests (marked `#[ignore]`): `test_seed_data_loaded`, `test_aspects_present`, `test_retrograde_periods_present`, `test_lunar_conditions_present`.

**Day count fix**: Initial test had an off-by-one — Jan 1 to Mar 2 inclusive is 61 days, but SEED_DAY_COUNT=60 represents the span (delta). Fixed the assertion to compare span rather than inclusive count.

## Verification

Verified compilation: `cargo test --features db --test api_integration 2>&1 | grep 'error' | wc -l` returns 0. All 4 non-ignored tests pass (constant validation), 4 DB-dependent tests correctly ignored. Full test suite still passes (13 tests, 0 failures).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --features db --test api_integration 2>&1 | grep 'error' | wc -l` | 0 | ✅ pass | 8000ms |
| 2 | `cargo test --features db --test api_integration 2>&1 | grep 'test result'` | 0 | ✅ pass — 4 passed, 0 failed, 4 ignored | 8000ms |
| 3 | `cargo test --features db 2>&1 | grep 'test result'` | 0 | ✅ pass — full suite still passes (13 tests) | 30000ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `tests/common/mod.rs`
- `tests/api_integration.rs`
