---
estimated_steps: 56
estimated_files: 1
skills_used: []
---

# T02: Fix broken integration tests and add comprehensive test suite

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

## Inputs

- ``tests/api_integration.rs` — existing integration tests to fix and extend`
- ``tests/common/mod.rs` — shared test helpers (test_pool, etc.)`
- ``src/server/routes/jobs.rs` — delete_job_handler from T01`
- ``src/server/mod.rs` — route wiring from T01`
- ``src/jobs/repository.rs` — JobRepository methods including delete_job from T01`

## Expected Output

- ``tests/api_integration.rs` — fixed existing tests + ~15 new integration tests covering multi-value filters, cursor pagination, DELETE behavior, and edge cases`

## Verification

cargo test --features db --test api_integration -- --ignored --test-threads=1
