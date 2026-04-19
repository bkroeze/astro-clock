---
id: T02
parent: S03
milestone: M003
key_files:
  - tests/api_integration.rs
key_decisions:
  - cursor_pagination_last_page uses a walk-all-pages approach rather than assuming exact job counts, since the test DB accumulates jobs from previous test runs
  - 409 test creates pending job via raw SQL + claim_next_job instead of relying on async job creation, because the executor completes sync jobs before the test can intercept them
duration: 
verification_result: passed
completed_at: 2026-04-19T19:40:56.486Z
blocker_discovered: false
---

# T02: Fix broken integration tests and add comprehensive test suite (36 tests total: 3 fixed + 15 new)

**Fix broken integration tests and add comprehensive test suite (36 tests total: 3 fixed + 15 new)**

## What Happened

Fixed 3 integration tests broken by S02's response shape change (offset/limit → cursor pagination), added `delete_uri` helper, wired the DELETE route into `build_app()`, and added 15 new integration tests.

**Fixed tests:**
1. `list_jobs_returns_paginated_results` — Changed from `?limit=10&offset=0` to `?count=10`, removed assertions on legacy `total`/`limit`/`offset` fields, added assertions on `next`/`prev` fields.
2. `list_jobs_pagination_works` — Rewrote from offset-based pagination to cursor-based: fetches first page with `?count=2`, extracts `next` URL, follows it to get page 2, verifies both pages have correct `next`/`prev` values.
3. `list_jobs_with_status_filter` — Quick review only; already passed because it only asserts `json["jobs"].is_array()`.

**New tests (15):**
- Multi-value filters: `list_jobs_multi_status_filter`, `list_jobs_multi_job_type_filter`, `list_jobs_invalid_status_in_multi_returns_400`, `list_jobs_invalid_job_type_returns_400`, `list_jobs_date_range_filter`, `list_jobs_invalid_date_filter_returns_400`
- Cursor pagination: `cursor_pagination_first_page`, `cursor_pagination_follow_next`, `cursor_pagination_last_page`, `cursor_pagination_invalid_cursor_returns_400`, `cursor_pagination_no_overlap`
- DELETE endpoint: `delete_completed_job_returns_204`, `delete_nonexistent_job_returns_404`, `delete_in_process_job_returns_409`, `delete_then_get_returns_404`

**Infrastructure additions:**
- Added `delete_uri()` helper for DELETE requests (handles 204 No Content → Value::Null)
- Added `delete_job_handler` import and DELETE route to `build_app()` 

**Key implementation detail for 409 test:** Creates a pending job via raw SQL INSERT, then uses `repo.claim_next_job()` to transition it to in_process status — this bypasses the async job queue which would execute the job before the test can claim it.

**Flaky test fix:** `cursor_pagination_last_page` was initially brittle (assumed exactly 3 jobs in DB). Rewrote to walk all pages until finding one with `next=null`, with a 50-page safety limit.

## Verification

All 36 integration tests pass with `cargo test --features db --test api_integration -- --ignored --test-threads=1` (including the 15 new tests, 3 fixed tests, and 18 pre-existing tests). All 206 lib tests continue to pass. Compilation is clean with no test-level warnings.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check --features db --test api_integration` | 0 | ✅ pass | 600ms |
| 2 | `cargo test --features db --lib` | 0 | ✅ pass | 400ms |
| 3 | `TEST_PG_URL=... cargo test --features db --test api_integration -- --ignored --test-threads=1` | 0 | ✅ pass | 9940ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `tests/api_integration.rs`
