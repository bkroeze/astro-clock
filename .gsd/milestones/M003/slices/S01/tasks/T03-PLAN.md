---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T03: Update handler with multi-value filter parsing, date support, and unit tests

Update `ListJobsRequest` to add `job_type: Option<String>`, `created_after: Option<String>`, `created_before: Option<String>`. Rewrite `list_jobs_handler` to parse comma-separated status/job_type values using FromStr, parse dates (RFC3339 first, fall back to YYYY-MM-DD), build JobListFilters, and pass to repository. Return 400 for any invalid values. Add comprehensive unit tests covering: multi-value parsing, invalid values, date format flexibility, combined filters, backward compat with single status.

## Inputs

- ``src/server/routes/jobs.rs` — current ListJobsRequest struct and list_jobs_handler`
- ``src/jobs/repository.rs` — JobListFilters struct from T01`
- ``src/jobs/types.rs` — FromStr-enabled enums from T01`

## Expected Output

- ``src/server/routes/jobs.rs` — updated ListJobsRequest with new fields, updated handler with multi-value parsing and date support, new unit tests for filter parsing and validation`

## Verification

cargo test --features db --lib -- server::routes::jobs
