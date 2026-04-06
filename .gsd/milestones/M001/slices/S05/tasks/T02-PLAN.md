# T02: 05-job-infrastructure 02

**Slice:** S05 — **Milestone:** M001

## Description

Create job types (Job, JobStatus, JobType) and repository layer (JobRepository, LoadedDaysRepository) with proper state machine and race-free job claiming.

Purpose: Define the domain types and data access layer for the job system. This provides type-safe abstractions over the database schema with proper error handling, JSON serialization for payloads/results, and efficient queries for job claiming and date range tracking.

Output: src/jobs/types.rs, src/jobs/repository.rs, src/jobs/error.rs, and src/jobs/mod.rs with complete type definitions and repository implementations.

## Must-Haves

- [ ] "Job struct exists with UUID id, status enum, payload/result/error JSONB"
- [ ] "JobStatus enum has valid state machine: pending → in_process → complete | failed"
- [ ] "JobRepository provides CRUD operations and race-free job claiming"
- [ ] "LoadedDaysRepository tracks date ranges and detects missing dates"
- [ ] "All repositories use proper error handling with thiserror"

## Files

- `src/jobs/mod.rs`
- `src/jobs/types.rs`
- `src/jobs/repository.rs`
- `src/jobs/error.rs`
