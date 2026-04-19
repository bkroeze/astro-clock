# S03: DELETE endpoint + integration tests — UAT

**Milestone:** M003
**Written:** 2026-04-19T19:46:08.444Z

# S03: DELETE endpoint + integration tests — UAT

**Milestone:** M003
**Written:** 2026-04-19

## UAT Type

- UAT mode: artifact-driven
- Why this mode is sufficient: All functionality is API-level with structured request/response patterns. Integration tests provide deterministic verification against a real database.

## Preconditions

- TimescaleDB test database running at `TEST_PG_URL`
- Seed data loaded (39+ days of astrology data)
- `cargo test --features db` compiles cleanly

## Smoke Test

Run `TEST_PG_URL=<url> cargo test --features db --test api_integration -- --ignored --test-threads=1 delete_completed_job_returns_204` — should pass.

## Test Cases

### 1. DELETE completed job returns 204

1. POST `/api/v1/load?date=2025-02-01&days=1&sync=true` to create and complete a load job
2. Note the job `id` from the response
3. Send `DELETE /api/v1/jobs/{id}`
4. **Expected:** HTTP 204 No Content, empty body

### 2. DELETE nonexistent job returns 404

1. Generate a random UUID (e.g., `00000000-0000-0000-0000-000000000000`)
2. Send `DELETE /api/v1/jobs/{uuid}`
3. **Expected:** HTTP 404, body contains `{ "error": "not_found", "message": "Job {uuid} not found" }`

### 3. DELETE in_process job returns 409

1. Insert a pending job via SQL, then claim it with `repo.claim_next_job()` to set it to in_process
2. Send `DELETE /api/v1/jobs/{id}`
3. **Expected:** HTTP 409, body contains `{ "error": "conflict", "message": "Cannot delete job {id} in in_process status" }`

### 4. DELETE then GET confirms removal

1. Create a completed job via POST sync load
2. DELETE the job
3. GET `/api/v1/jobs/{id}`
4. **Expected:** HTTP 404

### 5. Multi-value status filter

1. GET `/api/v1/jobs?status=complete,failed`
2. **Expected:** All returned jobs have status either "complete" or "failed"

### 6. Multi-value job_type filter

1. GET `/api/v1/jobs?job_type=load,query`
2. **Expected:** All returned jobs have job_type either "load" or "query"

### 7. Invalid status filter returns 400

1. GET `/api/v1/jobs?status=invalid`
2. **Expected:** HTTP 400 with error message about invalid status values

### 8. Date range filter

1. GET `/api/v1/jobs?created_after=2025-01-01&created_before=2025-03-01`
2. **Expected:** All returned jobs have created_at within the date range

### 9. Cursor pagination first page

1. GET `/api/v1/jobs?count=3`
2. **Expected:** Response has `jobs` array (≤3 items), `next` URL present, `prev` is null

### 10. Cursor pagination follow next

1. GET `/api/v1/jobs?count=3` → extract `next` URL
2. Follow `next` URL
3. **Expected:** Response has both `next` and `prev` URLs present, jobs are different from page 1

### 11. Cursor pagination last page

1. Walk through all pages following `next` URLs
2. **Expected:** Eventually reach a page where `next` is null

### 12. Cursor pagination no overlap

1. Walk through all pages collecting job IDs
2. **Expected:** No duplicate job IDs across pages

### 13. Invalid cursor returns 400

1. GET `/api/v1/jobs?cursor=!!!invalid!!!`
2. **Expected:** HTTP 400

## Edge Cases

### Job disappears between get and delete (race condition)

1. This is handled gracefully — returns 404 if the job was deleted by another process between the get_job and delete_job calls.

### DELETE is idempotent

1. Deleting the same job twice returns 204 then 404 (not an error).

## Failure Signals

- DELETE returns 500 — database connectivity issue
- DELETE returns 200 instead of 204 — handler logic error
- 409 for completed jobs — status check inverted
- Integration tests fail with "Cannot start a runtime from within a runtime" — build_app() not async

## Not Proven By This UAT

- Concurrent DELETE requests on the same job (race condition correctness under real concurrency)
- DELETE performance under load
- `test_seed_data_loaded` test — this is a pre-existing test that validates seed data volume (expects 61+ days) and is unrelated to S03 functionality
