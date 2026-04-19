# S03: DELETE endpoint + integration tests

**Goal:** Add DELETE /api/v1/jobs/:id endpoint (R008) returning 204/404/409, and comprehensive integration tests (R009) covering multi-value filters, cursor pagination, DELETE behavior, and all edge cases.
**Demo:** DELETE /api/v1/jobs/:id returns 204 for completed job, 404 for missing, 409 for in_process; full integration test suite covers filters, pagination, and delete

## Must-Haves

- DELETE /api/v1/jobs/:id returns 204 for completed/failed/pending jobs, 404 for missing, 409 for in_process\n- All existing integration tests pass with updated response shape assertions\n- New integration tests cover: multi-value CSV filters (status, job_type), date range filters, cursor pagination (forward, last page, invalid cursor, non-overlapping pages), DELETE success/404/409, DELETE-then-GET confirms deletion\n- Full unit test suite (cargo test --features db --lib) still passes with no regressions\n- cargo check --features db is clean

## Proof Level

- This slice proves: integration

## Integration Closure

Upstream surfaces consumed: JobRepository (get_job, delete_job, claim_next_job), routes (get_job_handler, list_jobs_handler, delete_job_handler), AppState/JobExecutor wiring\nNew wiring: DELETE method on /api/v1/jobs/:id route, delete_job_handler in route table\nWhat remains: nothing — this is the final slice in M003

## Verification

- Signals added: 409 Conflict response with descriptive message for in_process deletion attempts; 404 with not_found error code\nInspection: standard ErrorResponse JSON body on error responses (error + message fields)\nFailure state: DELETE on in_process job returns structured error explaining why deletion was refused

## Tasks

- [x] **T01: Implement DELETE /api/v1/jobs/:id endpoint** `est:30m`
  Add a delete_job method to JobRepository and a delete_job_handler to the routes, wired into the existing /api/v1/jobs/:id route with axum's .delete() modifier.

## Design

**Two-step approach (fetch then delete):** Use the existing `get_job()` to check existence and status, then perform `DELETE FROM jobs WHERE id = $1` if the job is not in_process. This avoids adding new error variants — the handler maps get_job results directly to HTTP status codes.

**Repository method:** `delete_job(&self, id: Uuid) -> JobResult<bool>` — executes `DELETE FROM jobs WHERE id = $1`, returns true if a row was deleted, false if not found. The handler calls get_job first to check status, then delete_job to perform the deletion.

**Handler logic:**
1. Call `repository.get_job(job_id)`
2. If None → return 404 with `ErrorResponse { error: "not_found", message: "Job {id} not found" }`
3. If Some(job) and `job.status == "in_process"` → return 409 with `ErrorResponse { error: "conflict", message: "Cannot delete job {id} in in_process status" }`
4. Otherwise → call `repository.delete_job(job_id)`, return 204 No Content

**Route wiring:** In `src/server/mod.rs`, change the `/api/v1/jobs/:id` route from `.route("/api/v1/jobs/:id", get(routes::get_job_handler))` to include `.delete(routes::delete_job_handler)`. Also update `tests/api_integration.rs` `build_app()` to include the DELETE route.

**No new error variants needed** — handler uses inline status checks and the existing ErrorResponse type.

## Constraints
- All new code behind `#[cfg(feature = "db")]`
- Import `delete` in `use axum::routing::{get, post}` → `use axum::routing::{get, post, delete}`
- 204 No Content returns `StatusCode::NO_CONTENT` with empty body
- Handler signature: `pub async fn delete_job_handler(State(state): State<AppState>, Path(job_id): Path<Uuid>) -> impl IntoResponse`
- Export delete_job_handler from routes module (add to `src/server/routes/mod.rs` if it exists, or ensure pub visibility)

## Route file structure
Check if `src/server/routes/mod.rs` exists — if routes are in `src/server/routes/jobs.rs`, the handler goes there. If there's a mod.rs re-exporting, update it too.
  - Files: `src/jobs/repository.rs`, `src/server/routes/jobs.rs`, `src/server/mod.rs`, `tests/api_integration.rs`
  - Verify: cargo check --features db && cargo test --features db --lib

- [x] **T02: Fix broken integration tests and add comprehensive test suite** `est:1h30m`
  Fix 3 existing integration tests broken by S02's response shape change, then add ~15 new integration tests covering multi-value filters, cursor pagination, DELETE behavior, and edge cases.

## Broken Tests to Fix

1. **list_jobs_returns_paginated_results** — Asserts `json["total"]`, `json["limit"]`, `json["offset"]`. Must change to assert `json["next"]`, `json["prev"]`, remove old field checks. Query uses `?limit=10&offset=0` → change to `?count=10`.

2. **list_jobs_pagination_works** — Uses `?limit=2&offset=0` and `?limit=2&offset=2`. Rewrite to use cursor pagination: fetch first page with `?count=2`, extract `next` URL, fetch second page via `next`.

3. **list_jobs_with_status_filter** — Verify it doesn't assert on old fields. It just checks `json["jobs"].is_array()` which should still pass. Quick review only.

## New Tests

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
- Pages are non-overlapping (collect all job IDs, assert no duplicates)

**DELETE endpoint:**
- DELETE on completed job → 204
- DELETE on nonexistent UUID → 404 with `error: "not_found"`
- DELETE on in_process job → 409 with `error: "conflict"`
- DELETE then GET same ID → 404 (confirm deletion)

## Test Helpers Needed

Add `delete_uri()` helper alongside existing `post_json()` and `get_uri()`:
```rust
async fn delete_uri(app: &mut axum::Router, uri: &str) -> (StatusCode, serde_json::Value) {
    let response = app.oneshot(
        Request::builder().method("DELETE").uri(uri).body(Body::empty()).unwrap()
    ).await.unwrap();
    let status = response.status();
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json = serde_json::from_slice(&body_bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}
```

Also add DELETE route to `build_app()`:
```rust
.route("/api/v1/jobs/:id", get(get_job_handler).delete(delete_job_handler))
```
Import `delete_job_handler` from routes.

## In-Process Job for 409 Test

To create an in_process job for the 409 DELETE test, use the repository directly:
```rust
let pool = common::test_pool().await;
let repo = JobRepository::new(pool.clone());
let in_process_job = repo.claim_next_job("test-worker").await.unwrap();
```
This requires creating a pending job first (via POST /api/v1/load async), then claiming it. If claim_next_job returns None (race condition with async execution), insert a job directly via SQL as fallback.

## Constraints
- All tests are `#[tokio::test]` + `#[ignore]`
- Use `tower::ServiceExt` pattern (no HTTP client)
- Each test gets its own `build_app()` call since `oneshot()` consumes the app
- Integration tests require `TEST_PG_URL` env var
- For 204 No Content, `serde_json::from_slice` will return `Value::Null` — the `delete_uri` helper already handles this with `unwrap_or(Value::Null)`
  - Files: `tests/api_integration.rs`
  - Verify: cargo test --features db --test api_integration -- --ignored --test-threads=1

## Files Likely Touched

- src/jobs/repository.rs
- src/server/routes/jobs.rs
- src/server/mod.rs
- tests/api_integration.rs
