---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: Add FromStr derives and JobListFilters struct

Add strum `EnumString` derive to `JobStatus` and `JobType` so both implement `FromStr` for comma-separated parsing. Create a `JobListFilters` struct in `repository.rs` holding parsed `Vec<JobStatus>`, `Vec<JobType>`, `Option<DateTime<Utc>>` date bounds. This is the shared contract between handler and repository.

## Inputs

- ``src/jobs/types.rs` — current JobStatus (has Display, AsRefStr) and JobType (has Display only) enums`
- ``src/jobs/repository.rs` — where JobListFilters will live alongside JobRepository`

## Expected Output

- ``src/jobs/types.rs` — JobStatus and JobType both have EnumString derive (FromStr impl)`
- ``src/jobs/repository.rs` — new JobListFilters struct with status vec, job_type vec, created_after/created_before Option<DateTime<Utc>>`

## Verification

cargo check --features db && cargo test --features db --lib -- jobs::types
