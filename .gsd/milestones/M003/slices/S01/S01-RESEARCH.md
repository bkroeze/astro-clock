# S01: Enhanced List Filters — Research

**Date:** 2026-04-19

## Summary

Slice S01 adds multi-value comma-separated filtering for `status` and `job_type` parameters, plus `created_after`/`created_before` date range filtering, to the existing `GET /api/v1/jobs` endpoint. The current handler accepts a single `status` string and does offset/limit pagination. This slice replaces the single-value status filter with multi-value support for both status and job_type, adds date range parameters, and updates the repository's `list_jobs` and `count_jobs` methods to build dynamic WHERE clauses with `sqlx::QueryBuilder`. The work is a straightforward extension of existing patterns — no new dependencies, no architectural changes.

**Requirements owned:** R004 (multi-value status/job_type filters), R005 (date range filters).

## Recommendation

Extend the existing `ListJobsRequest` struct with new fields (`status` becomes comma-separated, new `job_type`, `created_after`, `created_before` fields). Replace the current manual string-matching status parser with strum-based `FromStr` parsing for both `JobStatus` and `JobType`. Update `JobRepository::list_jobs` and `count_jobs` to accept a structured filter type and build dynamic WHERE clauses with QueryBuilder. This keeps the approach consistent with the existing codebase while adding the needed flexibility.

## Implementation Landscape

### Key Files

- `src/server/routes/jobs.rs` — Route handlers (`list_jobs_handler`), request/response types (`ListJobsRequest`, `ListJobsResponse`). The `ListJobsRequest` struct needs new fields and the handler needs updated validation logic.
- `src/jobs/repository.rs` — `JobRepository::list_jobs()` and `count_jobs()` methods. Both currently accept `Option<JobStatus>` + limit/offset. Need to accept a richer filter struct and use QueryBuilder for dynamic multi-condition WHERE clauses.
- `src/jobs/types.rs` — `JobStatus` and `JobType` enums with strum derives (`Display`, `AsRefStr`). Need `FromStr` derive added so comma-separated values can be parsed. Currently `JobType` has `Display` only, not `AsRefStr`.
- `src/jobs/error.rs` — `JobError` enum. May add a filter validation error variant, or handlers can return 400 directly (current pattern).

### Build Order

1. **Add `FromStr` to `JobStatus` and `JobType` in `types.rs`** — strum already provides `EnumString` derive. This unblocks filter parsing. Also add `AsRefStr` to `JobType` for consistency with `JobStatus`.
2. **Create a `JobListFilters` struct in `repository.rs` (or a new module)** — holds parsed `Vec<JobStatus>`, `Vec<JobType>`, `Option<DateTime<Utc>>` for date bounds. This is the clean contract between handler and repository.
3. **Update `list_jobs` and `count_jobs` in repository** — Accept `JobListFilters` instead of `Option<JobStatus>`. Use QueryBuilder to dynamically push WHERE conditions for status IN (...), job_type IN (...), created_at >= $N, created_at <= $N.
4. **Update `ListJobsRequest` in routes/jobs.rs** — Add `job_type: Option<String>`, `created_after: Option<String>`, `created_before: Option<String>`. Change status validation to parse comma-separated values.
5. **Update `list_jobs_handler`** — Parse all filter params into `JobListFilters`, return 400 for invalid values, pass to repository.
6. **Add unit tests** — Request deserialization, filter parsing (valid/invalid status, valid/invalid job_type, date parsing), QueryBuilder output.

### Verification Approach

- `cargo test --features db` — unit tests for deserialization, filter parsing, error cases
- `cargo check --features db` — compilation check
- `cargo test --features db --test api_integration -- --ignored` — integration tests (existing list tests should still pass; new filter tests will be added in S03)
- Manual: `cargo run --features db` then `curl "http://localhost:3030/api/v1/jobs?status=complete,failed&job_type=load"`

## Constraints

- All new code must be behind `#[cfg(feature = "db")]` — the handler file already is, types.rs is not (it's shared). The `FromStr` derive on enums is not db-gated, which is fine — it's pure data logic.
- Existing indexes `idx_jobs_status_type(status, job_type)` and `idx_jobs_created_at(created_at DESC)` cover the query patterns. Status IN + job_type IN can use the composite index. Date range uses the created_at index. No new indexes needed.
- `status` column has a CHECK constraint (`IN ('pending', 'in_process', 'complete', 'failed')`) and `job_type` has no explicit CHECK but values are 'load'/'query'. The filters are validated at the application layer before reaching SQL.
- Response shape must remain backward-compatible — existing `jobs`, `total`, `limit`, `offset` fields stay. S02 will add `next`/`prev` later. S01 does NOT change the response shape.

## Common Pitfalls

- **Comma-separated query params with URL encoding** — Axum's `Query` extractor will give the raw query string value. `status=complete,failed` arrives as a single string `"complete,failed"`. Split on `,`, trim whitespace, parse each. Don't URL-decode manually — Axum handles that.
- **QueryBuilder bind ordering** — When pushing multiple dynamic conditions with `push_bind`, the parameter indices are implicit and sequential. The order conditions are pushed must match the order they appear in the SQL. This is already handled correctly by the QueryBuilder pattern.
- **Date parsing flexibility** — The milestone context says ISO 8601 timestamps or YYYY-MM-DD dates. Use `chrono::DateTime::parse_from_rfc3339` first, fall back to `NaiveDate::parse_from_str("%Y-%m-%d")` + midnight UTC. Both must work.
- **Empty filter = all jobs** — If no filters are provided, return all jobs (existing behavior). The filter struct should have all optional fields, and `list_jobs` builds no WHERE clause when all are None/empty.
- **`IN` clause with empty Vec** — If `status` or `job_type` param is provided but results in an empty vec after parsing invalid entries, return 400 rather than building `IN ()` which is invalid SQL.

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| Enum string parsing | strum `EnumString` derive (`FromStr`) | Already have strum in deps with `Display`/`AsRefStr` derives. Adding `EnumString` gives `FromStr` for free — no need for manual match arms. |
| Dynamic SQL building | sqlx `QueryBuilder` | Already used in `list_jobs`. Push conditions dynamically, bind params safely — no SQL injection risk. |
