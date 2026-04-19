---
id: S01
parent: M003
milestone: M003
provides:
  - ["JobListFilters struct with multi-value status/job_type and date range filtering", "parse_comma_separated<T: FromStr>() generic CSV parser in handler", "parse_date_param() RFC3339/YYYY-MM-DD date parser in handler", "list_jobs and count_jobs accepting JobListFilters with dynamic WHERE clauses"]
requires:
  []
affects:
  - ["S02"]
key_files:
  - ["src/jobs/types.rs", "src/jobs/repository.rs", "src/cli/app.rs", "src/server/routes/jobs.rs"]
key_decisions:
  - ["Used strum EnumString derive for FromStr impls rather than manual implementations — leverages existing strum dependency and snake_case serialization attribute", "Placed JobListFilters in repository.rs (not types.rs) because it is the boundary struct between handler and repository layers", "Used sqlx QueryBuilder with separated.push_bind() for IN clauses — generates parameterized queries preserving SQL injection safety", "Used generic parse_comma_separated<T: FromStr>() helper instead of inline parsing — eliminates duplication between status and job_type parsing", "parse_date_param tries RFC3339 first then falls back to YYYY-MM-DD at midnight UTC — gives maximum flexibility for API consumers"]
patterns_established:
  - ["Generic parse_comma_separated<T: FromStr>() for CSV query parameter parsing with error collection", "JobListFilters struct as the handler↔repository contract for complex query parameters", "sqlx QueryBuilder with dynamic WHERE chaining via has_where boolean tracker", "RFC3339-first date parsing with YYYY-MM-DD fallback for API date parameters"]
observability_surfaces:
  - ["400 response with descriptive error messages listing invalid values — primary diagnostic signal for filter validation issues"]
drill_down_paths:
  - [".gsd/milestones/M003/slices/S01/tasks/T01-SUMMARY.md", ".gsd/milestones/M003/slices/S01/tasks/T02-SUMMARY.md", ".gsd/milestones/M003/slices/S01/tasks/T03-SUMMARY.md"]
duration: ""
verification_result: passed
completed_at: 2026-04-19T19:06:18.786Z
blocker_discovered: false
---

# S01: Enhanced list filters

**Multi-value comma-separated status/job_type filters and date range filtering (created_after/created_before) on GET /api/v1/jobs, fully backward compatible**

## What Happened

This slice enhanced the GET /api/v1/jobs endpoint with multi-value comma-separated filtering for status and job_type query parameters, plus date range filtering via created_after/created_before parameters.

**T01** added `strum_macros::EnumString` derives to `JobStatus` and `JobType` enums, giving each a `FromStr` implementation matching the existing snake_case serialization. It also created the `JobListFilters` struct in `src/jobs/repository.rs` as the shared contract between handler and repository layers, with fields for `Vec<JobStatus>`, `Vec<JobType>`, and `Option<DateTime<Utc>>` date bounds.

**T02** rewrote `list_jobs` and `count_jobs` in `JobRepository` to accept `JobListFilters` instead of the old `Option<JobStatus>`. Both methods use `sqlx::QueryBuilder` to dynamically construct WHERE clauses with parameterized `IN (...)` clauses for multi-value enums and `>=`/`<=` comparisons for date ranges. The CLI call site was updated to construct `JobListFilters` from its single status value, preserving backward compatibility.

**T03** updated the `ListJobsRequest` struct with new fields (job_type, created_after, created_before) and rewrote `list_jobs_handler` with a generic `parse_comma_separated<T: FromStr>()` helper for CSV parsing and a `parse_date_param` helper that tries RFC3339 first then falls back to YYYY-MM-DD at midnight UTC. The handler validates all parameters and returns 400 with descriptive error messages for invalid values. 21 new unit tests were added covering multi-value parsing, invalid values, date format flexibility, combined filters, empty filters, and backward compatibility. All 188 project tests pass.

## Verification

All slice-level verification passed:

1. **Unit tests — handler layer:** `cargo test --features db --lib -- server::routes::jobs` — 36/36 tests pass (21 new filter tests + 15 pre-existing)
2. **Unit tests — CLI layer:** `cargo test --features db --lib -- cli::app` — 9/9 tests pass, confirming backward compatibility
3. **Full test suite:** `cargo test --features db --lib` — 188/188 tests pass, 0 failures
4. **Backward compatibility confirmed:** Existing single-value `?status=complete` requests parse identically through the same code path. Response shape (jobs, total, limit, offset) unchanged.
5. **Validation coverage:** Invalid enum values → 400, invalid date formats → 400, whitespace trimming, RFC3339 with timezone offsets, YYYY-MM-DD fallback, empty filters return all jobs.

## Requirements Advanced

- R004 — Implemented multi-value comma-separated parsing for status and job_type query parameters with 400 error responses for invalid values. Validated by 21 new unit tests.
- R005 — Implemented created_after/created_before date range filtering with RFC3339 and YYYY-MM-DD format support. Validated by date parsing and filter combination tests.

## Requirements Validated

- R004 — 21 unit tests covering multi-value CSV parsing, invalid value rejection, whitespace handling, combined filters, and backward compatibility. Full suite 188/188 pass.
- R005 — Unit tests covering RFC3339 parsing, YYYY-MM-DD fallback, timezone offset handling, invalid date rejection, and date range filter integration. Full suite 188/188 pass.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

Date range filtering currently does no timezone conversion — `created_after=2025-01-01` is treated as midnight UTC. API consumers in non-UTC timezones should provide RFC3339 timestamps with explicit offsets for precise boundaries.

## Follow-ups

None. S02 (cursor-based pagination) will build on the JobListFilters struct and may need to extend the response shape.

## Files Created/Modified

- `src/jobs/types.rs` — Added EnumString and AsRefStr derives to JobStatus and JobType for FromStr and as_ref() support
- `src/jobs/repository.rs` — Created JobListFilters struct; rewrote list_jobs and count_jobs to use dynamic QueryBuilder WHERE clauses
- `src/cli/app.rs` — Updated CLI job list handler to construct JobListFilters from single status value (backward compatible)
- `src/server/routes/jobs.rs` — Added job_type/created_after/created_before fields to ListJobsRequest; rewrote handler with parse_comma_separated and parse_date_param helpers; added 21 unit tests
