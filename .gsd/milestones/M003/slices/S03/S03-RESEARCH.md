# S03: DELETE endpoint + integration tests — Research

**Date:** 2026-04-19

## Summary

This slice adds the `DELETE /api/v1/jobs/:id` endpoint (R008) and comprehensive integration tests (R009) covering all M003 functionality. The DELETE handler is straightforward — it checks job status, returns 409 for in_process, 404 for missing, and 204 for success. The integration tests need more attention because the existing `tests/api_integration.rs` tests still assert on the old `limit`/`offset`/`total` response shape that was replaced in S02 with `next`/`prev` pagination. These tests must be updated before new ones are added.

The work splits naturally into two tasks: (1) implement the DELETE endpoint (repository method + handler + route wiring), and (2) write integration tests covering multi-value filters, cursor pagination, DELETE behavior, and edge cases — while fixing the existing broken integration tests.

## Recommendation

Implement DELETE endpoint first (repository → handler → route) since it's self-contained and quick. Then fix the existing broken integration tests (S02 changed the response shape) and add new integration tests. The existing test patterns in `tests/api_integration.rs` are well-established — `build_app()`, `post_json()`, `get_uri()` helpers, tower::ServiceExt — follow them exactly. Add a `delete_uri()` helper alongside the existing ones.

## Implementation Landscape

### Key Files

- `src/jobs/repository.rs` — Add `delete_job(&self, id: Uuid) -> JobResult<DeleteOutcome>` method. SQL: `DELETE FROM jobs WHERE id = $1 RETURNING status`. Returns `DeleteOutcome::Deleted(status)`, `DeleteOutcome::NotFound`, or `DeleteOutcome::InProcess`. Use existing `JobError` variants.
- `src/jobs/error.rs` — May need a `Conflict(String)` variant for the 409 case, OR handle it in the handler by checking status before deletion. Simplest: repository returns the job's status on delete, handler maps it.
- `src/server/routes/jobs.rs` — Add `delete_job_handler` function. Pattern: fetch job (reuse `get_job` logic), check status, delete, return 204/404/409. Reuse existing `ErrorResponse` type.
- `src/server/mod.rs` — Wire DELETE route: change `.route("/api/v1/jobs/:id", get(routes::get_job_handler))` to `.route("/api/v1/jobs/:id", get(routes::get_job_handler).delete(routes::delete_job_handler))`. Add `delete` to the `use axum::routing` import.
- `tests/api_integration.rs` — Fix 3 broken tests (see below) and add ~15 new tests. Add `delete_uri()` helper. Add DELETE route to `build_app()`.

### Repository Delete Design

Two approaches, recommend approach A:

**A) Two-step (fetch then delete):** Use existing `get_job()` to check existence and status, then `DELETE FROM jobs WHERE id = $1` if allowed. This avoids adding new error variants and keeps the handler logic clear. Race condition risk is minimal since no worker should be processing a completed/failed job.

**B) Single SQL with CASE:** Complex SQL that conditionally deletes. Over-engineering for a simple endpoint.

Use approach A. The handler calls `get_job()`, checks the result, and only calls the delete query if appropriate.

### Existing Integration Tests That Need Fixing

Three tests in `tests/api_integration.rs` reference the old response shape:

1. **`list_jobs_returns_paginated_results`** (line ~504) — Asserts `json["total"]`, `json["limit"]`, `json["offset"]`. Must change to assert `json["next"]`, `json["prev"]`, and remove old field checks. Query uses `?limit=10&offset=0` — change to `?count=10`.

2. **`list_jobs_pagination_works`** (line ~566) — Uses `?limit=2&offset=0` and `?limit=2&offset=2`. Must rewrite to use cursor-based pagination: fetch first page, extract `next` URL, fetch second page via `next`.

3. **`list_jobs_with_status_filter`** (line ~542) — Simple GET, just verifies `status=complete` works. This one should still pass since the response still has `jobs` array. Verify it doesn't assert on old fields.

### New Integration Tests Needed

Per R009 and the roadmap acceptance criteria:

**Multi-value filters:**
- `?status=complete,failed` returns only matching jobs
- `?job_type=load,query` returns only matching jobs
- `?status=invalid` returns 400
- `?job_type=bad` returns 400
- `?created_after=2025-01-01&created_before=2025-03-01` returns filtered results
- `?created_after=not-a-date` returns 400

**Cursor pagination:**
- First page: `?count=3` returns jobs with `next` URL, `prev` is null
- Follow `next` URL: returns next page, both `next` and `prev` present
- Last page: `next` is null
- Invalid cursor returns 400
- Pages are non-overlapping (collect all job IDs across pages, assert no duplicates)

**DELETE endpoint:**
- DELETE on completed job → 204
- DELETE on nonexistent UUID → 404 with `error: "not_found"`
- DELETE on in_process job → 409 with `error: "conflict"`
- DELETE then GET same ID → 404 (confirm deletion)

### Build Order

1. **T01: DELETE endpoint** — repository method + handler + route wiring. Quick, unblocks testing.
2. **T02: Integration tests** — Fix broken existing tests, add new DELETE tests, add filter/pagination tests. This is the bulk of the work.

### Verification Approach

1. `cargo test --features db --lib` — All unit tests pass (no new unit tests needed; DELETE is thin)
2. `cargo check --features db` — Clean compilation
3. `cargo test --features db -- --ignored --test-threads=1` — All integration tests pass (requires TEST_PG_URL)

## Constraints

- All new code behind `#[cfg(feature = "db")]` feature gate
- Must use `tower::ServiceExt` pattern for integration tests (no HTTP client)
- Integration tests require `TEST_PG_URL` env var and running TimescaleDB
- Integration tests are `#[ignore]` by default, run with `--ignored` flag
- Route path uses axum's `:id` syntax (already registered for GET)
- No `deny_unknown_fields` on `ListJobsRequest` — old `limit`/`offset` query params silently ignored

## Common Pitfalls

- **Existing integration tests are broken** — The S02 slice changed the list response shape from `{jobs, total, limit, offset}` to `{jobs, next, prev}`, but didn't update integration tests. Fixing these is prerequisite for adding new tests.
- **204 No Content has no body** — DELETE success returns `(StatusCode::NO_CONTENT, ().into_response())`. Don't try to parse JSON from it.
- **in_process job creation for testing** — To test 409, need a job in `in_process` state. Use `claim_next_job` via the repository directly (not through the API), or manually update via SQL in the test setup.
- **`oneshot` consumes the app** — Each test needs its own `build_app()` call. The `app` is consumed by `oneshot()`, so multi-step tests (create then delete) need to rebuild the app between steps OR use `tower::ServiceBuilder` differently. Current pattern rebuilds for each test — follow that.
