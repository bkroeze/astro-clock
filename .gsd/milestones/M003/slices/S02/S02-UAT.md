# S02: Cursor-based pagination with next/prev URLs — UAT

**Milestone:** M003
**Written:** 2026-04-19T19:28:18.446Z

# UAT: Cursor-based Pagination (S02)

## Preconditions
- Application built with `cargo build --features db`
- Test database available with seed data (60 days of jobs via `just test-db-setup`)
- `TEST_PG_URL` environment variable set

## Test Cases

### TC1: First page returns correct shape
1. Start the server: `cargo run --features db -- serve`
2. `GET /api/v1/jobs?count=5`
3. **Expected**: Response has `{ "jobs": [...], "next": "http://...cursor=...", "prev": null }`
4. **Expected**: `jobs` array has ≤ 5 items
5. **Expected**: Each job has `id`, `status`, `job_type`, `created_at` fields
6. **Expected**: `prev` is `null` (first page)

### TC2: Following next URL returns stable next page
1. From TC1, copy the `next` URL
2. `GET <next_url>`
3. **Expected**: Response has `{ "jobs": [...], "next": "...", "prev": "..." }`
4. **Expected**: No job IDs overlap with TC1 results (pages are disjoint)
5. **Expected**: Jobs are in `created_at DESC` order (newest first)

### TC3: Inserting jobs between page fetches doesn't shift results
1. `GET /api/v1/jobs?count=3` — note the job IDs on this page
2. Trigger a new job (e.g., `POST /api/v1/load` with async=true)
3. `GET /api/v1/jobs?count=3` again with same parameters
4. **Expected**: Same 3 job IDs as step 1 (new job appears in later pages)

### TC4: Backward pagination via prev URL
1. `GET /api/v1/jobs?count=3` — get first page, note next URL
2. Follow `next` URL — get second page
3. Follow `prev` URL from second page
4. **Expected**: Returns the same jobs as first page (step 1)

### TC5: Filter parameters preserved in pagination URLs
1. `GET /api/v1/jobs?count=5&status=complete,failed`
2. **Expected**: `next` URL contains `status=complete%2Cfailed` (or equivalent encoding)
3. Follow `next` URL
4. **Expected**: Results only include jobs with status `complete` or `failed`
5. **Expected**: `next` and `prev` URLs also contain the status filter

### TC6: count parameter validation
1. `GET /api/v1/jobs?count=1` — **Expected**: Returns 0-1 jobs
2. `GET /api/v1/jobs?count=100` — **Expected**: Returns up to 100 jobs
3. `GET /api/v1/jobs?count=0` — **Expected**: Treated as 1 (minimum)
4. `GET /api/v1/jobs?count=200` — **Expected**: Treated as 100 (maximum)
5. `GET /api/v1/jobs` (no count) — **Expected**: Default 20 jobs

### TC7: Invalid cursor returns 400
1. `GET /api/v1/jobs?cursor=not-valid-base64!!!`
2. **Expected**: HTTP 400 with error message containing "Invalid cursor"
3. `GET /api/v1/jobs?cursor=eyJpbnZhbGlkIjoxfQ` (valid base64, invalid JSON structure)
4. **Expected**: HTTP 400 with error message describing the decode failure

### TC8: CLI uses count parameter
1. `cargo run --features db -- job list --count 3`
2. **Expected**: Displays up to 3 jobs in terminal
3. `cargo run --features db -- job list` (no count)
4. **Expected**: Displays up to 20 jobs (default)
5. `cargo run --features db -- job list --help`
6. **Expected**: Help text shows `--count` flag, no `--limit` or `--offset`

### TC9: Empty results
1. `GET /api/v1/jobs?status=nonexistent_status_value`
2. **Expected**: HTTP 200 with `{ "jobs": [], "next": null, "prev": null }`

### TC10: Single page (no more results)
1. `GET /api/v1/jobs?count=10000` (exceeds total jobs)
2. **Expected**: All jobs returned, `next` is `null`, `prev` is `null`
