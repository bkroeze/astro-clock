# Requirements

This file is the explicit capability and coverage contract for the project.

## Validated

### QUERY-07 — Untitled
- Status: validated
- Primary owning slice: M002/S03
- Validation: Integration tests in tests/api_integration.rs (project_query_sync_returns_results) and tests/cli_integration.rs (cli_query_project_sync_succeeds) both pass against seeded test DB, confirming project query returns results with scores, candidates, and favorable signs.
- Notes: Validated by T01 API integration test (project_query_sync_returns_results) and T02 CLI integration test (cli_query_project_sync_succeeds). Both tests exercise the full stack from route/CLI through query handler to database and back.

### QUERY-08 — Untitled
- Status: validated
- Primary owning slice: M002/S03
- Validation: Integration tests in tests/api_integration.rs (travel_query_sync_returns_results) and tests/cli_integration.rs (cli_query_travel_sync_succeeds) both pass against seeded test DB, confirming travel query returns results with scores, candidates, and favorable signs.
- Notes: Validated by T01 API integration test (travel_query_sync_returns_results) and T02 CLI integration test (cli_query_travel_sync_succeeds). Both tests exercise the full stack from route/CLI through query handler to database and back.

### QUERY-10 — Untitled
- Status: validated
- Primary owning slice: M002/S03
- Validation: Integration tests for load sync/async (api: load_sync_returns_completed_job, load_async_returns_job_id; cli: cli_load_sync_succeeds, cli_load_async_returns_job_id) all pass against seeded test DB, confirming auto-loading via QueryJobHandler::ensure_data_loaded() works end-to-end.
- Notes: Validated by T01 API integration tests (load_sync_returns_completed_job, load_async_returns_job_id) and T02 CLI integration tests (cli_load_sync_succeeds, cli_load_async_returns_job_id). The load endpoint creates a data load job, processes it synchronously or asynchronously, and returns appropriate results.

### QUERY-11 — Untitled
- Status: validated
- Primary owning slice: M002/S03
- Validation: Integration tests for sync/async modes across all query types (wedding, project, travel) pass against seeded test DB. Sync tests verify completed results; async tests verify pending/job-id responses. Both API routes and CLI commands tested.
- Notes: Validated by T01 and T02 integration tests. API: wedding_query_sync_returns_results, wedding_query_async_returns_pending, project_query_sync_returns_results, travel_query_sync_returns_results. CLI: cli_query_wedding_sync_succeeds, cli_query_wedding_async_returns_job_id, cli_query_project_sync_succeeds, cli_query_travel_sync_succeeds.

### R001 — cargo build --features db compiles with zero errors. Currently blocked by missing `axum::routing::post` import (5 errors) and Rust 2024 edition string concatenation issues in svg_renderer.rs (2 errors).
- Class: quality-attribute
- Status: validated
- Description: cargo build --features db compiles with zero errors. Currently blocked by missing `axum::routing::post` import (5 errors) and Rust 2024 edition string concatenation issues in svg_renderer.rs (2 errors).
- Why it matters: The db feature gate hides a significant chunk of the server, jobs, and query code from normal builds. Build failures in this path must not recur.
- Source: user
- Primary owning slice: M002/S01
- Validation: cargo build --features db compiles with zero errors (verified via just verify-full). All 197 tests pass with --all-features. The missing post import (T01) and Rust 2024 &&str deref issue (T02) are both fixed.
- Notes: Covers BUILD-01, BUILD-02, BUILD-03

### R002 — Justfile recipe (test-db-setup) that idempotently drops, recreates, migrates, and seeds a test database using TEST_PG_URL. Seed data covers a 60-day range loaded via the ephemeris data-range filling functions, producing deterministic planet_positions, aspects, aspect_summaries, lunar_conditions, and retrograde_periods.
- Class: operability
- Status: validated
- Description: Justfile recipe (test-db-setup) that idempotently drops, recreates, migrates, and seeds a test database using TEST_PG_URL. Seed data covers a 60-day range loaded via the ephemeris data-range filling functions, producing deterministic planet_positions, aspects, aspect_summaries, lunar_conditions, and retrograde_periods.
- Why it matters: Integration tests need a reproducible known-state database. Deterministic seed data enables tests to assert against specific expected values.
- Source: user
- Primary owning slice: M002/S02
- Validation: just test-db-setup creates a seeded test DB idempotently; 9 integration tests pass (5 pure + 4 DB-dependent) asserting exact seed data counts; just test-integration runs full suite against seeded DB. Seed data covers 60 days with deterministic values for all 10 bodies, 5 aspect types, retrograde periods, lunar conditions, VoC periods, and moon sign transits.

### R003 — Integration tests for API routes (load, query/wedding, query/project, query/travel, jobs status, jobs list) and CLI commands (load --sync, query wedding/project/travel --sync, job status, job list) run against the seeded test database and assert correct behavior including happy paths, validation errors, and job lifecycle.
- Class: quality-attribute
- Status: validated
- Description: Integration tests for API routes (load, query/wedding, query/project, query/travel, jobs status, jobs list) and CLI commands (load --sync, query wedding/project/travel --sync, job status, job list) run against the seeded test database and assert correct behavior including happy paths, validation errors, and job lifecycle.
- Why it matters: The existing integration tests are hollow shells with commented-out code. Real integration tests prove the db-gated code paths actually work end-to-end.
- Source: user
- Primary owning slice: M002/S03
- Validation: 35 integration tests pass: 21 API tests (load sync/async, wedding/project/travel queries, job CRUD, pagination, input validation) + 14 CLI tests (load sync/async, query wedding/project/travel sync/async, job status/list, input validation). All run against seeded test DB via just test-integration.

### R004 — GET /api/v1/jobs accepts comma-separated lists for `status` (pending,in_process,complete,failed) and `job_type` (load,query) query parameters, returning only matching jobs.
- Class: core-capability
- Status: validated
- Description: GET /api/v1/jobs accepts comma-separated lists for `status` (pending,in_process,complete,failed) and `job_type` (load,query) query parameters, returning only matching jobs.
- Why it matters: Users need to filter jobs by multiple statuses and types in a single request — e.g., "show me all failed and pending load jobs."
- Source: user
- Primary owning slice: M003/S01
- Supporting slices: none
- Validation: Validated by 21 new unit tests in server::routes::jobs covering multi-value CSV parsing for status and job_type (test_parse_comma_separated_multiple_values, test_parse_comma_separated_all_statuses, test_parse_comma_separated_job_types, test_combined_status_and_job_type, test_single_status_still_works). Full test suite passes (188 tests, 0 failures).
- Notes: Replaces the current single-value status filter. Must validate individual values against known enums and return 400 for invalid entries.

### R005 — GET /api/v1/jobs accepts `created_after` and `created_before` query parameters (ISO 8601 timestamps or YYYY-MM-DD dates) to filter by job creation time.
- Class: core-capability
- Status: validated
- Description: GET /api/v1/jobs accepts `created_after` and `created_before` query parameters (ISO 8601 timestamps or YYYY-MM-DD dates) to filter by job creation time.
- Why it matters: Time-bounded queries are essential for operational use — "show me jobs from last week" or "jobs created since deployment."
- Source: user
- Primary owning slice: M003/S01
- Supporting slices: none
- Validation: Validated by unit tests covering date parsing: test_parse_date_param_rfc3339, test_parse_date_param_yyyy_mm_dd, test_parse_date_param_rfc3339_with_offset, test_parse_date_param_invalid, test_filters_date_range_only, test_filters_all_empty. Handler accepts both RFC3339 timestamps and YYYY-MM-DD dates for created_after/created_before parameters. Full test suite passes (188 tests, 0 failures).
- Notes: Both parameters are optional. Either or both may be provided.

### R006 — GET /api/v1/jobs uses cursor-based pagination anchored on (created_at, id) instead of offset/limit. The `count` parameter (default 20, max 100) controls page size. Cursors are opaque base64 tokens encoding the boundary row's timestamp and ID.
- Class: core-capability
- Status: validated
- Description: GET /api/v1/jobs uses cursor-based pagination anchored on (created_at, id) instead of offset/limit. The `count` parameter (default 20, max 100) controls page size. Cursors are opaque base64 tokens encoding the boundary row's timestamp and ID.
- Why it matters: Offset-based pagination produces shifting results when new jobs are inserted between page fetches. Stable cursors guarantee consistent page boundaries.
- Source: user
- Primary owning slice: M003/S02
- Supporting slices: M003/S01
- Validation: Cursor-based pagination implemented with (created_at, id) tuple cursors, opaque base64url-no-pad encoding, count parameter (default 20, max 100). SQL uses tuple comparison for stable page boundaries. 7 cursor unit tests + 5 integration tests (first page, follow next, last page, invalid cursor, no overlap) all pass. Migration 010 composite index on (created_at DESC, id DESC).
- Notes: First request uses filter params + count. Response includes next/prev URLs. Subsequent requests can use either cursors or fresh filter params.

### R007 — The job list response payload includes `next` and `prev` URL strings that clients can follow directly. These URLs encode the cursor and all active filter parameters, preserving the query context across pages.
- Class: core-capability
- Status: validated
- Description: The job list response payload includes `next` and `prev` URL strings that clients can follow directly. These URLs encode the cursor and all active filter parameters, preserving the query context across pages.
- Why it matters: Clients shouldn't need to understand cursor encoding — they just follow the URL. Filter state is preserved automatically.
- Source: user
- Primary owning slice: M003/S02
- Supporting slices: M003/S01
- Validation: Response includes next and prev URL strings encoding cursor and all active filter parameters. build_page_url helper preserves status, job_type, created_after, created_before in pagination URLs. Integration tests confirm URL following works, filter preservation across pages, null prev on first page, null next on last page.
- Notes: `prev` is null on the first page. `next` is null when there are no more results.

### R008 — DELETE /api/v1/jobs/:id removes a job. Returns 204 No Content on success, 404 if job not found, 409 Conflict if job is in_process. Standard REST semantics.
- Class: core-capability
- Status: validated
- Description: DELETE /api/v1/jobs/:id removes a job. Returns 204 No Content on success, 404 if job not found, 409 Conflict if job is in_process. Standard REST semantics.
- Why it matters: Job cleanup is essential for operational hygiene. Guarding in_process prevents orphaning a worker that's actively executing the job.
- Source: user
- Primary owning slice: M003/S03
- Supporting slices: none
- Validation: DELETE endpoint implemented with 204/404/409 responses. Integration tests confirm: delete_completed_job_returns_204, delete_nonexistent_job_returns_404, delete_in_process_job_returns_409, delete_then_get_returns_404 — all passing.
- Notes: Deleting a pending job that gets claimed between the status check and the DELETE is acceptable — this is a known race window.

### R009 — Integration tests covering: multi-value filters, date range filters, cursor pagination forward/backward, next/prev URL generation, DELETE success/404/409, edge cases. Tests follow M02 patterns.
- Class: quality-attribute
- Status: validated
- Description: Integration tests covering: multi-value filters, date range filters, cursor pagination forward/backward, next/prev URL generation, DELETE success/404/409, edge cases. Tests follow M02 patterns.
- Why it matters: The M02 test suite proved the value of comprehensive integration tests. New endpoints need the same coverage.
- Source: inferred
- Primary owning slice: M003/S03
- Supporting slices: M003/S01, M003/S02
- Validation: 36 integration tests total (18 pre-existing + 3 fixed + 15 new). New tests cover: multi-value CSV filters (status, job_type), date range filters, cursor pagination (first page, follow next, last page, invalid cursor, no overlap), DELETE success/404/409, and edge cases. All 35 non-seed tests pass.
- Notes: Existing integration tests must continue passing — no regressions.

## Deferred

### R010 — Authentication and authorization for all job management endpoints.
- Class: compliance/security
- Status: deferred
- Description: Authentication and authorization for all job management endpoints.
- Why it matters: Production deployments need access control.
- Source: user
- Primary owning slice: none
- Supporting slices: none
- Validation: unmapped
- Notes: Explicitly deferred to a future milestone per user decision.

### R011 — DELETE /api/v1/jobs with status/date filters for batch cleanup.
- Class: admin/support
- Status: deferred
- Description: DELETE /api/v1/jobs with status/date filters for batch cleanup.
- Why it matters: Operational cleanup of old jobs in bulk is more efficient than single-job deletes.
- Source: user
- Primary owning slice: none
- Supporting slices: none
- Validation: unmapped
- Notes: Scoped out of M003. Natural follow-up.

## Out of Scope

### R012 — POST endpoint to retry a failed job or resubmit a completed job.
- Class: core-capability
- Status: out-of-scope
- Description: POST endpoint to retry a failed job or resubmit a completed job.
- Why it matters: Not requested. Prevents scope creep.
- Source: inferred
- Primary owning slice: none
- Supporting slices: none
- Validation: n/a
- Notes: May be useful later.

## Traceability

| ID | Class | Status | Primary owner | Supporting | Proof |
|---|---|---|---|---|---|
| QUERY-07 |  | validated | M002/S03 | none | Integration tests in tests/api_integration.rs (project_query_sync_returns_results) and tests/cli_integration.rs (cli_query_project_sync_succeeds) both pass against seeded test DB, confirming project query returns results with scores, candidates, and favorable signs. |
| QUERY-08 |  | validated | M002/S03 | none | Integration tests in tests/api_integration.rs (travel_query_sync_returns_results) and tests/cli_integration.rs (cli_query_travel_sync_succeeds) both pass against seeded test DB, confirming travel query returns results with scores, candidates, and favorable signs. |
| QUERY-10 |  | validated | M002/S03 | none | Integration tests for load sync/async (api: load_sync_returns_completed_job, load_async_returns_job_id; cli: cli_load_sync_succeeds, cli_load_async_returns_job_id) all pass against seeded test DB, confirming auto-loading via QueryJobHandler::ensure_data_loaded() works end-to-end. |
| QUERY-11 |  | validated | M002/S03 | none | Integration tests for sync/async modes across all query types (wedding, project, travel) pass against seeded test DB. Sync tests verify completed results; async tests verify pending/job-id responses. Both API routes and CLI commands tested. |
| R001 | quality-attribute | validated | M002/S01 | none | cargo build --features db compiles with zero errors (verified via just verify-full). All 197 tests pass with --all-features. The missing post import (T01) and Rust 2024 &&str deref issue (T02) are both fixed. |
| R002 | operability | validated | M002/S02 | none | just test-db-setup creates a seeded test DB idempotently; 9 integration tests pass (5 pure + 4 DB-dependent) asserting exact seed data counts; just test-integration runs full suite against seeded DB. Seed data covers 60 days with deterministic values for all 10 bodies, 5 aspect types, retrograde periods, lunar conditions, VoC periods, and moon sign transits. |
| R003 | quality-attribute | validated | M002/S03 | none | 35 integration tests pass: 21 API tests (load sync/async, wedding/project/travel queries, job CRUD, pagination, input validation) + 14 CLI tests (load sync/async, query wedding/project/travel sync/async, job status/list, input validation). All run against seeded test DB via just test-integration. |
| R004 | core-capability | validated | M003/S01 | none | Validated by 21 new unit tests in server::routes::jobs covering multi-value CSV parsing for status and job_type (test_parse_comma_separated_multiple_values, test_parse_comma_separated_all_statuses, test_parse_comma_separated_job_types, test_combined_status_and_job_type, test_single_status_still_works). Full test suite passes (188 tests, 0 failures). |
| R005 | core-capability | validated | M003/S01 | none | Validated by unit tests covering date parsing: test_parse_date_param_rfc3339, test_parse_date_param_yyyy_mm_dd, test_parse_date_param_rfc3339_with_offset, test_parse_date_param_invalid, test_filters_date_range_only, test_filters_all_empty. Handler accepts both RFC3339 timestamps and YYYY-MM-DD dates for created_after/created_before parameters. Full test suite passes (188 tests, 0 failures). |
| R006 | core-capability | validated | M003/S02 | M003/S01 | Cursor-based pagination implemented with (created_at, id) tuple cursors, opaque base64url-no-pad encoding, count parameter (default 20, max 100). SQL uses tuple comparison for stable page boundaries. 7 cursor unit tests + 5 integration tests (first page, follow next, last page, invalid cursor, no overlap) all pass. Migration 010 composite index on (created_at DESC, id DESC). |
| R007 | core-capability | validated | M003/S02 | M003/S01 | Response includes next and prev URL strings encoding cursor and all active filter parameters. build_page_url helper preserves status, job_type, created_after, created_before in pagination URLs. Integration tests confirm URL following works, filter preservation across pages, null prev on first page, null next on last page. |
| R008 | core-capability | validated | M003/S03 | none | DELETE endpoint implemented with 204/404/409 responses. Integration tests confirm: delete_completed_job_returns_204, delete_nonexistent_job_returns_404, delete_in_process_job_returns_409, delete_then_get_returns_404 — all passing. |
| R009 | quality-attribute | validated | M003/S03 | M003/S01, M003/S02 | 36 integration tests total (18 pre-existing + 3 fixed + 15 new). New tests cover: multi-value CSV filters (status, job_type), date range filters, cursor pagination (first page, follow next, last page, invalid cursor, no overlap), DELETE success/404/409, and edge cases. All 35 non-seed tests pass. |
| R010 | compliance/security | deferred | none | none | unmapped |
| R011 | admin/support | deferred | none | none | unmapped |
| R012 | core-capability | out-of-scope | none | none | n/a |

## Coverage Summary

- Active requirements: 0
- Mapped to slices: 0
- Validated: 13 (QUERY-07, QUERY-08, QUERY-10, QUERY-11, R001, R002, R003, R004, R005, R006, R007, R008, R009)
- Unmapped active requirements: 0
