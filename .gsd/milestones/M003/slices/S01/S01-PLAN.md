# S01: Enhanced list filters

**Goal:** Add multi-value comma-separated filtering for status and job_type, plus date range filtering (created_after/created_before), to GET /api/v1/jobs. Backward-compatible — existing single-value status filter still works, no response shape changes.
**Demo:** GET /api/v1/jobs?status=complete,failed&job_type=load,query&created_after=2025-01-01&created_before=2025-03-01 returns filtered results; invalid values return 400

## Must-Haves

- GET /api/v1/jobs?status=complete,failed returns only jobs matching those statuses
- GET /api/v1/jobs?job_type=load,query returns only jobs matching those types
- GET /api/v1/jobs?created_after=2025-01-01&created_before=2025-03-01 returns jobs in that date range
- All filters can be combined in a single request
- Invalid status/job_type values return 400 with a descriptive error
- Invalid date formats return 400
- Empty filter params (no status, no job_type, no dates) returns all jobs (existing behavior)
- Existing response shape unchanged (jobs, total, limit, offset)
- CLI list command continues to work with single status filter

## Proof Level

- This slice proves: contract

## Integration Closure

- Upstream surfaces consumed: `JobStatus` and `JobType` enums from `src/jobs/types.rs`, `JobRepository` from `src/jobs/repository.rs`, route handler in `src/server/routes/jobs.rs`
- New wiring introduced: `JobListFilters` struct serves as the contract between handler and repository, replacing `Option<JobStatus>`
- What remains: S02 will add cursor-based pagination (replacing offset), next/prev URLs, and a `count` parameter. S01 keeps offset/limit untouched.

## Verification

- Structured tracing already exists on error paths in the handler. No new observability surfaces needed — the validation errors (400 responses) are the primary diagnostic signal for filter issues.

## Tasks

- [x] **T01: Add FromStr derives and JobListFilters struct** `est:20m`
  Add strum `EnumString` derive to `JobStatus` and `JobType` so both implement `FromStr` for comma-separated parsing. Create a `JobListFilters` struct in `repository.rs` holding parsed `Vec<JobStatus>`, `Vec<JobType>`, `Option<DateTime<Utc>>` date bounds. This is the shared contract between handler and repository.
  - Files: `src/jobs/types.rs`, `src/jobs/repository.rs`
  - Verify: cargo check --features db && cargo test --features db --lib -- jobs::types

- [ ] **T02: Rewrite list_jobs and count_jobs to use JobListFilters with dynamic WHERE** `est:30m`
  Update `JobRepository::list_jobs` and `count_jobs` to accept `JobListFilters` instead of `Option<JobStatus>`. Use QueryBuilder to dynamically push WHERE conditions: `status IN (...)` for multi-value status, `job_type IN (...)` for multi-value job_type, `created_at >= $N` and `created_at <= $N` for date range. Update the CLI app's call site in `src/cli/app.rs` to construct a JobListFilters with a single status value (backward-compatible).
  - Files: `src/jobs/repository.rs`, `src/cli/app.rs`
  - Verify: cargo check --features db

- [ ] **T03: Update handler with multi-value filter parsing, date support, and unit tests** `est:45m`
  Update `ListJobsRequest` to add `job_type: Option<String>`, `created_after: Option<String>`, `created_before: Option<String>`. Rewrite `list_jobs_handler` to parse comma-separated status/job_type values using FromStr, parse dates (RFC3339 first, fall back to YYYY-MM-DD), build JobListFilters, and pass to repository. Return 400 for any invalid values. Add comprehensive unit tests covering: multi-value parsing, invalid values, date format flexibility, combined filters, backward compat with single status.
  - Files: `src/server/routes/jobs.rs`
  - Verify: cargo test --features db --lib -- server::routes::jobs

## Files Likely Touched

- src/jobs/types.rs
- src/jobs/repository.rs
- src/cli/app.rs
- src/server/routes/jobs.rs
