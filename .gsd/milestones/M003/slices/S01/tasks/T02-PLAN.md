---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T02: Rewrite list_jobs and count_jobs to use JobListFilters with dynamic WHERE

Update `JobRepository::list_jobs` and `count_jobs` to accept `JobListFilters` instead of `Option<JobStatus>`. Use QueryBuilder to dynamically push WHERE conditions: `status IN (...)` for multi-value status, `job_type IN (...)` for multi-value job_type, `created_at >= $N` and `created_at <= $N` for date range. Update the CLI app's call site in `src/cli/app.rs` to construct a JobListFilters with a single status value (backward-compatible).

## Inputs

- ``src/jobs/repository.rs` — current list_jobs(Option<JobStatus>, limit, offset) and count_jobs(Option<JobStatus>) signatures`
- ``src/cli/app.rs` — calls repo.list_jobs(status, limit, offset) around line 919`

## Expected Output

- ``src/jobs/repository.rs` — list_jobs and count_jobs accept JobListFilters, build dynamic WHERE with QueryBuilder`
- ``src/cli/app.rs` — updated to construct JobListFilters from single status value`

## Verification

cargo check --features db
