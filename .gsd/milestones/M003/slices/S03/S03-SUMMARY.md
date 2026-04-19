---
id: S03
parent: M003
milestone: M003
provides:
  - ["DELETE /api/v1/jobs/:id endpoint with 204/404/409 status codes", "delete_job repository method", "Comprehensive integration test suite (36 tests) covering all M003 functionality", "delete_uri test helper for DELETE requests in integration tests"]
requires:
  - slice: S01
    provides: Multi-value filter query parsing (parse_comma_separated, parse_date_param), ErrorResponse type
  - slice: S02
    provides: Cursor pagination response shape (next/prev fields), list_jobs_handler, build_app() test infrastructure
affects:
  - []
key_files:
  - ["src/jobs/repository.rs", "src/server/routes/jobs.rs", "src/server/mod.rs", "src/server/routes/mod.rs", "tests/api_integration.rs"]
key_decisions:
  - ["Two-step fetch-then-delete approach (get_job + delete_job) keeps business logic in the handler layer and reuses existing repository methods", "409 test uses raw SQL INSERT + claim_next_job to create in_process jobs, since async job executor completes sync jobs before tests can intercept", "cursor_pagination_last_page walks all pages with 50-page safety limit rather than assuming exact job counts"]
patterns_established:
  - ["Two-step fetch-then-delete pattern for status-gated deletion (get_job for status check, delete_job for actual deletion)", "DELETE handler returns 404 for race condition where job disappears between get and delete (idempotent behavior)", "Creating in_process test jobs via raw SQL INSERT + claim_next_job to bypass async executor"]
observability_surfaces:
  - ["409 Conflict response with structured ErrorResponse { error: \"conflict\", message: \"Cannot delete job {id} in in_process status\" } for in_process deletion attempts"]
drill_down_paths:
  - [".gsd/milestones/M003/slices/S03/tasks/T01-SUMMARY.md", ".gsd/milestones/M003/slices/S03/tasks/T02-SUMMARY.md"]
duration: ""
verification_result: passed
completed_at: 2026-04-19T19:46:08.444Z
blocker_discovered: false
---

# S03: DELETE endpoint + integration tests

**DELETE /api/v1/jobs/:id endpoint with 204/404/409 responses and comprehensive integration test suite (36 tests covering filters, pagination, and DELETE)**

## What Happened

S03 added the final piece of M003: a DELETE endpoint for job cleanup and a comprehensive integration test suite that validates the entire milestone's work.

**T01 (DELETE endpoint):** Implemented `DELETE /api/v1/jobs/:id` using a two-step fetch-then-delete approach. The handler calls `get_job()` first to check existence and status, then performs deletion only for non-in_process jobs. Returns 204 No Content on success, 404 with `ErrorResponse { error: "not_found" }` for missing jobs, and 409 with `ErrorResponse { error: "conflict" }` for in_process jobs. Added `delete_job()` repository method executing `DELETE FROM jobs WHERE id = $1`. Race condition handling: if a job disappears between get and delete, returns 404 (idempotent).

**T02 (Integration tests):** Fixed 3 integration tests broken by S02's response shape change (offset/limit → cursor pagination), added `delete_uri` helper, wired DELETE route into `build_app()`, and added 15 new integration tests:
- Multi-value filters: status CSV, job_type CSV, invalid values returning 400, date range filters, invalid dates returning 400
- Cursor pagination: first page, follow next URL, walk to last page (next=null), invalid cursor, non-overlapping pages
- DELETE: completed job → 204, nonexistent → 404, in_process → 409, delete-then-GET confirms removal

Key implementation detail for the 409 test: creates a pending job via raw SQL INSERT then uses `repo.claim_next_job()` to transition it to in_process, since the async job executor completes sync jobs before tests can intercept them.

Minor fix in this verification round: suppressed unused variable warning by prefixing `jobs` with underscore in `cursor_pagination_last_page` test.

## Verification

All 35 API integration tests pass (excluding pre-existing `test_seed_data_loaded` which fails due to seed data having only 39 days instead of 61+ — not a regression from S03). All 206 lib tests pass. `cargo check --features db` compiles cleanly with only pre-existing warnings. The 15 new S03 tests cover: multi-value CSV filters (status, job_type), date range filters, invalid filter validation, cursor pagination (first page, follow next, last page, invalid cursor, no overlap), DELETE success/404/409, and delete-then-GET confirmation.

## Requirements Advanced

- R008 — Implemented DELETE /api/v1/jobs/:id with 204 No Content, 404 not_found, and 409 conflict responses. Two-step fetch-then-delete guards against in_process deletion.
- R009 — Added 15 new integration tests covering multi-value filters, cursor pagination, DELETE behavior, and edge cases. Fixed 3 existing tests broken by S02's response shape change. Total suite: 36 tests.

## Requirements Validated

- R008 — Integration tests: delete_completed_job_returns_204, delete_nonexistent_job_returns_404, delete_in_process_job_returns_409, delete_then_get_returns_404 — all passing
- R009 — 36 integration tests total covering multi-value CSV filters, date ranges, cursor pagination (first page, follow next, last page, invalid cursor, no overlap), DELETE success/404/409, and delete-then-GET. All 35 non-seed tests pass.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

["test_seed_data_loaded integration test fails (expects 61+ days, found 39) — pre-existing issue unrelated to S03, likely needs test DB re-seeding"]

## Follow-ups

["Re-seed test database with full 60+ days of data to fix test_seed_data_loaded test"]

## Files Created/Modified

None.
