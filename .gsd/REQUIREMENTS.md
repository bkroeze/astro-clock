# Requirements

This file is the explicit capability and coverage contract for the project.

## Active

### QUERY-07 — Untitled
- Status: active
- Primary owning slice: M002/S03
- Validation: mapped
- Notes: Project query implementation exists in src/queries/project.rs. Will be validated by integration tests in M002/S03.

### QUERY-08 — Untitled
- Status: active
- Primary owning slice: M002/S03
- Validation: mapped
- Notes: Travel query implementation exists in src/queries/travel.rs. Will be validated by integration tests in M002/S03.

### QUERY-10 — Untitled
- Status: active
- Primary owning slice: M002/S03
- Validation: mapped
- Notes: Auto-loading implemented in QueryJobHandler::ensure_data_loaded(). Will be validated by integration tests in M002/S03.

### QUERY-11 — Untitled
- Status: active
- Primary owning slice: M002/S03
- Validation: mapped
- Notes: Sync/async modes implemented in JobExecutor::execute_sync/execute_async and server routes. Will be validated by integration tests in M002/S03.

### R002 — Justfile recipe (test-db-setup) that idempotently drops, recreates, migrates, and seeds a test database using TEST_PG_URL. Seed data covers a 60-day range loaded via the ephemeris data-range filling functions, producing deterministic planet_positions, aspects, aspect_summaries, lunar_conditions, and retrograde_periods.
- Class: operability
- Status: active
- Description: Justfile recipe (test-db-setup) that idempotently drops, recreates, migrates, and seeds a test database using TEST_PG_URL. Seed data covers a 60-day range loaded via the ephemeris data-range filling functions, producing deterministic planet_positions, aspects, aspect_summaries, lunar_conditions, and retrograde_periods.
- Why it matters: Integration tests need a reproducible known-state database. Deterministic seed data enables tests to assert against specific expected values.
- Source: user
- Primary owning slice: M002/S02
- Validation: unmapped

### R003 — Integration tests for API routes (load, query/wedding, query/project, query/travel, jobs status, jobs list) and CLI commands (load --sync, query wedding/project/travel --sync, job status, job list) run against the seeded test database and assert correct behavior including happy paths, validation errors, and job lifecycle.
- Class: quality-attribute
- Status: active
- Description: Integration tests for API routes (load, query/wedding, query/project, query/travel, jobs status, jobs list) and CLI commands (load --sync, query wedding/project/travel --sync, job status, job list) run against the seeded test database and assert correct behavior including happy paths, validation errors, and job lifecycle.
- Why it matters: The existing integration tests are hollow shells with commented-out code. Real integration tests prove the db-gated code paths actually work end-to-end.
- Source: user
- Primary owning slice: M002/S03
- Validation: unmapped

## Validated

### R001 — cargo build --features db compiles with zero errors. Currently blocked by missing `axum::routing::post` import (5 errors) and Rust 2024 edition string concatenation issues in svg_renderer.rs (2 errors).
- Class: quality-attribute
- Status: validated
- Description: cargo build --features db compiles with zero errors. Currently blocked by missing `axum::routing::post` import (5 errors) and Rust 2024 edition string concatenation issues in svg_renderer.rs (2 errors).
- Why it matters: The db feature gate hides a significant chunk of the server, jobs, and query code from normal builds. Build failures in this path must not recur.
- Source: user
- Primary owning slice: M002/S01
- Validation: cargo build --features db compiles with zero errors (verified via just verify-full). All 197 tests pass with --all-features. The missing post import (T01) and Rust 2024 &&str deref issue (T02) are both fixed.
- Notes: Covers BUILD-01, BUILD-02, BUILD-03

## Traceability

| ID | Class | Status | Primary owner | Supporting | Proof |
|---|---|---|---|---|---|
| QUERY-07 |  | active | M002/S03 | none | mapped |
| QUERY-08 |  | active | M002/S03 | none | mapped |
| QUERY-10 |  | active | M002/S03 | none | mapped |
| QUERY-11 |  | active | M002/S03 | none | mapped |
| R001 | quality-attribute | validated | M002/S01 | none | cargo build --features db compiles with zero errors (verified via just verify-full). All 197 tests pass with --all-features. The missing post import (T01) and Rust 2024 &&str deref issue (T02) are both fixed. |
| R002 | operability | active | M002/S02 | none | unmapped |
| R003 | quality-attribute | active | M002/S03 | none | unmapped |

## Coverage Summary

- Active requirements: 6
- Mapped to slices: 6
- Validated: 1 (R001)
- Unmapped active requirements: 0
