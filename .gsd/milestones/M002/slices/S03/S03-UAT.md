# S03: Integration test suite — UAT

**Milestone:** M002
**Written:** 2026-04-15T23:32:36.621Z

# S03: Integration test suite — UAT

**Milestone:** M002
**Written:** 2026-04-15

## UAT Type

- UAT mode: artifact-driven
- Why this mode is sufficient: The integration tests themselves are the verification artifacts — they exercise real code paths against a real database. Running them is sufficient proof.

## Preconditions

1. PostgreSQL with TimescaleDB running at `10.0.10.50:5432`
2. Test database `astrology_test` seeded via `just test-db-setup` (60 days from 2025-01-01)
3. `TEST_PG_URL=postgresql://astro:barnlab@10.0.10.50:5432/astrology_test` set in environment

## Smoke Test

```bash
just test-integration
```
Expected: All 35 tests pass, exit code 0.

## Test Cases

### 1. API load sync creates completed job

1. `curl -X POST http://localhost:8086/api/v1/load -d '{"start_date":"2025-01-01","days":5,"sync":true}'` (simulated by test)
2. **Expected:** Response status "complete" with job result containing days_loaded

### 2. API wedding query returns results

1. POST /api/v1/query/wedding with sync=true, start=2025-01-01, days=30
2. **Expected:** Response with query_name="wedding", total_results > 0, status="complete"

### 3. API job lifecycle (create → poll → list)

1. POST /api/v1/load async → returns 202 with job_id
2. GET /api/v1/jobs/{job_id} → returns job details
3. GET /api/v1/jobs → returns paginated list
4. **Expected:** Each step returns correct status codes and data

### 4. API validation rejects bad input

1. POST /api/v1/load with invalid date → 400 error=invalid_date
2. POST /api/v1/load with days=0 → 400 error=invalid_days
3. POST /api/v1/query/wedding with days=400 → 400 error=invalid_days
4. **Expected:** All return 400 with descriptive error

### 5. CLI query wedding --sync returns results

1. `cargo run --features db -- query wedding --start 2025-01-01 --days 30 --sync`
2. **Expected:** Exit code 0, output contains wedding results

### 6. CLI job status roundtrip

1. `cargo run --features db -- job list` → capture a job UUID
2. `cargo run --features db -- job status {uuid}` → shows job details
3. **Expected:** Both commands exit 0, status shows matching job data

### 7. CLI input validation

1. `cargo run --features db -- load --start not-a-date --days 5` → exit non-zero
2. `cargo run --features db -- load --start 2025-01-01 --days 0` → exit non-zero
3. **Expected:** Both fail with descriptive error messages

## Edge Cases

### Project/travel queries return failed status

1. POST /api/v1/query/project sync → may return status="failed"
2. **Expected:** This is acceptable — `aspect_summaries` and `retrograde_periods` tables are empty in test DB. Tests accept both complete and failed.

### Job list pagination

1. Create 3 jobs, then list with limit=2 offset=0
2. **Expected:** Returns 2 jobs with total=3, then next page returns 1 job

## Failure Signals

- Any integration test returns non-zero exit code
- `just test-integration` shows "FAILED" for any test
- API tests return unexpected status codes (not 200/202/400/404)
- CLI tests show panic or unwrap failures

## Not Proven By This UAT

- Performance under load (single-request testing only)
- Concurrent job execution race conditions
- Project/travel queries with populated derived tables (empty in test DB)
- Server uptime or connection pooling under sustained load

## Notes for Tester

- The `just test-integration` command sets up the environment correctly — just run it after `just test-db-setup`
- Project and travel query tests accept "failed" status — this is expected, not a bug
- CLI tests use `assert_cmd` which builds the binary fresh, so first run may be slow
- All integration tests are `#[ignore]`d by default and only run with `--ignored` flag via the Justfile recipe
