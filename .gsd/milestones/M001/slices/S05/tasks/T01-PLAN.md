# T01: 05-job-infrastructure 01

**Slice:** S05 — **Milestone:** M001

## Description

Create database schema for the job system including the jobs table and loaded_days tracking table.

Purpose: Provide persistent storage for job state management and incremental data loading tracking. The jobs table enables tracking of asynchronous operations with their payloads, results, and error states. The loaded_days table enables intelligent incremental loading by tracking which date ranges have already been computed.

Output: Two migration files (008_create_jobs.sql, 009_create_loaded_days.sql) and updated Cargo.toml with strum dependencies.

## Must-Haves

- [ ] "Jobs table exists with UUID primary key and status tracking"
- [ ] "Loaded days table tracks date ranges with coverage"
- [ ] "strum crate is available for enum derive macros"
- [ ] "Both migrations are valid and can be applied by sqlx migrate"

## Files

- `Cargo.toml`
- `migrations/008_create_jobs.sql`
- `migrations/009_create_loaded_days.sql`
