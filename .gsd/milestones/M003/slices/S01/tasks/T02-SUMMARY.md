---
id: T02
parent: S01
milestone: M003
key_files:
  - src/jobs/repository.rs
  - src/jobs/types.rs
  - src/cli/app.rs
  - src/server/routes/jobs.rs
key_decisions:
  - Used sqlx::QueryBuilder with separated.push_bind() for IN clauses — generates parameterized queries ($1, $2, ...) rather than string interpolation, preserving SQL injection safety.
  - Tracked WHERE clause state with a mutable boolean to correctly chain AND conditions without duplicating WHERE keywords.
duration: 
verification_result: passed
completed_at: 2026-04-19T19:00:35.628Z
blocker_discovered: false
---

# T02: Rewrite list_jobs and count_jobs to use JobListFilters with dynamic QueryBuilder WHERE clauses

**Rewrite list_jobs and count_jobs to use JobListFilters with dynamic QueryBuilder WHERE clauses**

## What Happened

Replaced `list_jobs(Option<JobStatus>, limit, offset)` and `count_jobs(Option<JobStatus>)` with versions that accept `JobListFilters`. Both methods now use `sqlx::QueryBuilder` to dynamically construct WHERE clauses: `status IN (...)` for multi-value status filtering, `job_type IN (...)` for multi-value job_type filtering, and `created_at >= $N` / `created_at <= $N` for date range filtering. The query correctly chains AND conditions, only adding WHERE when the first condition appears.

Updated the CLI call site in `src/cli/app.rs` to construct a `JobListFilters` from the single status value using `status.into_iter().collect()` — backward-compatible. Also adapted the server route handler in `src/server/routes/jobs.rs` with the same pattern so it continues to compile until T03 properly rewrites it with multi-value support.

Added `strum_macros::AsRefStr` derive to `JobType` (it was already on `JobStatus`) so the enum values can be bound as `&str` in the SQL query. Added `#[allow(unused_assignments)]` on both methods to suppress false-positive warnings about the `has_where` tracking variable.

## Verification

Verified with `cargo check --features db` (clean compile, only 2 pre-existing warnings), `cargo test --features db --lib -- server::routes::jobs` (16/16 pass), and `cargo test --features db --lib -- cli::app` (9/9 pass). The dynamic WHERE building compiles correctly and existing callers are adapted to the new signature.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check --features db` | 0 | ✅ pass | 1140ms |
| 2 | `cargo test --features db --lib -- server::routes::jobs` | 0 | ✅ pass | 5660ms |
| 3 | `cargo test --features db --lib -- cli::app` | 0 | ✅ pass | 140ms |

## Deviations

Also added `AsRefStr` derive to `JobType` in `src/jobs/types.rs` (not mentioned in task plan) because the new query binding code requires `as_ref()` on JobType values. Also updated `src/server/routes/jobs.rs` (not in task plan scope) to use `JobListFilters` so it compiles — T03 will properly rewrite the handler with full multi-value support.

## Known Issues

None.

## Files Created/Modified

- `src/jobs/repository.rs`
- `src/jobs/types.rs`
- `src/cli/app.rs`
- `src/server/routes/jobs.rs`
