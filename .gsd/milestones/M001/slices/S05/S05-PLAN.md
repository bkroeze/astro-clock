# S05: Job Infrastructure

**Goal:** Create database schema for the job system including the jobs table and loaded_days tracking table.
**Demo:** Create database schema for the job system including the jobs table and loaded_days tracking table.

## Must-Haves


## Tasks

- [x] **T01: 05-job-infrastructure 01** `est:1min`
  - Create database schema for the job system including the jobs table and loaded_days tracking table.

Purpose: Provide persistent storage for job state management and incremental data loading tracking. The jobs table enables tracking of asynchronous operations with their payloads, results, and error states. The loaded_days table enables intelligent incremental loading by tracking which date ranges have already been computed.

Output: Two migration files (008_create_jobs.sql, 009_create_loaded_days.sql) and updated Cargo.toml with strum dependencies.
- [x] **T02: 05-job-infrastructure 02** `est:2 min`
  - Create job types (Job, JobStatus, JobType) and repository layer (JobRepository, LoadedDaysRepository) with proper state machine and race-free job claiming.

Purpose: Define the domain types and data access layer for the job system. This provides type-safe abstractions over the database schema with proper error handling, JSON serialization for payloads/results, and efficient queries for job claiming and date range tracking.

Output: src/jobs/types.rs, src/jobs/repository.rs, src/jobs/error.rs, and src/jobs/mod.rs with complete type definitions and repository implementations.
- [x] **T03: 05-job-infrastructure 03** `est:4min`
  - Create JobExecutor that orchestrates job lifecycle with both synchronous and asynchronous execution modes.

Purpose: Provide the core orchestration layer that manages job execution, handles state transitions, and supports both blocking (CLI) and non-blocking (API) execution patterns. The executor bridges the job repository with handler implementations using spawn_blocking for CPU-intensive work to avoid blocking the async runtime.

Output: src/jobs/executor.rs with JobExecutor, JobHandler trait, and complete sync/async execution implementations.

## Files Likely Touched

- `Cargo.toml`
- `migrations/008_create_jobs.sql`
- `migrations/009_create_loaded_days.sql`
- `src/jobs/mod.rs`
- `src/jobs/types.rs`
- `src/jobs/repository.rs`
- `src/jobs/error.rs`
- `src/jobs/executor.rs`
- `src/jobs/mod.rs`
- `src/lib.rs`
