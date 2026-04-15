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

## Coverage Summary

- Active requirements: 0
- Mapped to slices: 0
- Validated: 7 (QUERY-07, QUERY-08, QUERY-10, QUERY-11, R001, R002, R003)
- Unmapped active requirements: 0
