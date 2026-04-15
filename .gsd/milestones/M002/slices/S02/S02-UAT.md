# S02: Test database infrastructure — UAT

**Milestone:** M002
**Written:** 2026-04-15T22:59:07.117Z

## UAT Type

- UAT mode: artifact-driven
- Why this mode is sufficient: The slice produces buildable code, a database provisioning recipe, and executable tests. All verification is automated.

## Preconditions

- PostgreSQL with TimescaleDB extension available at `$TEST_PG_URL`
- `.env` file sourced (provides PG_URL)
- TEST_PG_URL exported: `export TEST_PG_URL=postgresql://astro:barnlab@10.0.10.50:5432/astrology_test`

## Smoke Test

```bash
source .env && export TEST_PG_URL=postgresql://astro:barnlab@10.0.10.50:5432/astrology_test
just test-db-setup && just test-integration
```

Both commands exit 0.

## Test Cases

### 1. Justfile recipe test-db-setup is idempotent

1. Run `just test-db-setup`
2. Run `just test-db-setup` again
3. **Expected:** Both runs complete successfully with exit code 0. Second run produces same database state (drop+recreate ensures idempotency).

### 2. Test helper module compiles and constant tests pass

1. Run `cargo test --features db --test api_integration`
2. **Expected:** 5 tests pass, 4 ignored (DB-dependent). Exit code 0. No compilation errors.

### 3. DB-dependent integration tests pass with seed data

1. Run `just test-db-setup` (ensure fresh seed data)
2. Run `cargo test --features db --test api_integration -- --ignored --test-threads=1`
3. **Expected:** 4 tests pass (test_seed_data_loaded, test_aspects_present, test_retrograde_bodies_match_known, test_lunar_conditions_match_known). All assertions match exact counts from seed data.

### 4. Seed data contains expected record counts

1. Run `just test-db-setup`
2. Query: `psql $TEST_PG_URL -c "SELECT COUNT(*) FROM planet_positions;"`
3. **Expected:** 864,000 positions
4. Query: `psql $TEST_PG_URL -c "SELECT COUNT(*) FROM aspects;"`
5. **Expected:** 1,465,067 aspects
6. Query: `psql $TEST_PG_URL -c "SELECT COUNT(*) FROM lunar_conditions;"`
7. **Expected:** 86,400 lunar conditions

### 5. Seed data contains expected retrograde bodies

1. Query test database: `psql $TEST_PG_URL -c "SELECT body_id, SUM(CASE WHEN retrograde THEN 1 ELSE 0 END) as retrograde_minutes FROM planet_positions GROUP BY body_id HAVING SUM(CASE WHEN retrograde THEN 1 ELSE 0 END) > 0 ORDER BY body_id;"`
2. **Expected:** Venus (3): 1,402 min, Mars (4): 76,441 min, Jupiter (5): 48,102 min, Uranus (7): 41,304 min

### 6. Full test suite passes

1. Run `cargo test --features db`
2. **Expected:** All test binaries report 0 failures. Total ~195 tests pass.

## Edge Cases

### Jan 29 seed data failure

1. Load includes Jan 29 which fails due to moon_phase_angle constraint
2. **Expected:** Load reports "Dates failed: 1" for Jan 29 but completes with 60/61 days. Tests pass regardless since assertions use >= comparisons or exact counts from successful days.

### TEST_PG_URL not set

1. Run `just test-db-setup` without TEST_PG_URL set
2. **Expected:** Clear error message: "Error: TEST_PG_URL environment variable is not set". Exit code 1.

## Failure Signals

- `just test-db-setup` exits non-zero (database creation or seed data loading failed)
- Integration tests report count mismatches (seed data different from constants)
- Compilation errors in tests/common/mod.rs or tests/api_integration.rs

## Not Proven By This UAT

- Migration 006 continuous aggregates fully working (known TimescaleDB issue with IF NOT EXISTS)
- Server API endpoints (deferred to S03)
- CLI query commands against test database (deferred to S03)
- Concurrent test execution safety (tests run single-threaded via --test-threads=1)

## Notes for Tester

- Migration 6 errors in test-db-setup output are expected and handled (continuous aggregate re-creation issue).
- Jan 29 seed data failure is a known edge case in Swiss Ephemeris moon phase calculation and does not affect test validity.
- The test-integration recipe uses `--ignored` flag which only runs DB-dependent tests. To run ALL tests including non-DB tests, use `cargo test --features db` directly.
