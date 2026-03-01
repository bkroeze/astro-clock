---
phase: 05-job-infrastructure
plan: 02
subsystem: jobs

tags: [jobs, repository, sqlx, postgresql, strum, uuid]

requires:
  - phase: 05-01
    provides: Database schema (jobs and loaded_days tables with migrations)

provides:
  - JobError enum with thiserror for structured error handling
  - JobResult type alias for standardized results
  - JobType enum (Load, Query) with strum derives
  - JobStatus enum (Pending, InProcess, Complete, Failed) with state machine
  - Job struct with UUID id, JSONB fields, and timestamp tracking
  - JobRepository with race-free job claiming using FOR UPDATE SKIP LOCKED
  - LoadedDaysRepository for date range tracking and gap detection
  - State machine validation via can_transition_to method

affects:
  - 05-03 (Job executor will use these types and repositories)
  - 06-data-loading (Load job handler will use LoadedDaysRepository)
  - 07-named-queries (Query job handler will use JobRepository)

tech-stack:
  added: [thiserror, strum, strum_macros, uuid]
  patterns:
    - Repository pattern for database access
    - Type-safe error handling with thiserror
    - FOR UPDATE SKIP LOCKED for race-free concurrency
    - State machine pattern for job status transitions
    - Strum derives for enum string conversions

key-files:
  created:
    - src/jobs/mod.rs - Module declarations and public exports
    - src/jobs/error.rs - JobError enum and JobResult type
    - src/jobs/types.rs - Job, JobType, JobStatus definitions
    - src/jobs/repository.rs - JobRepository and LoadedDaysRepository
  modified: []

key-decisions:
  - "Used String for job_type and status in DB struct for sqlx compatibility, with helper methods for enum conversion"
  - "Implemented FOR UPDATE SKIP LOCKED pattern for race-free job claiming (prevents multiple workers claiming same job)"
  - "State machine enforced via can_transition_to method before database update"
  - "LoadedDaysRepository uses PostgreSQL generate_series for efficient missing date queries"

patterns-established:
  - "Repository Pattern: Database access abstracted behind repository structs with Pool<Postgres> dependency"
  - "Error Type Pattern: Domain-specific error enum with thiserror + Result type alias"
  - "State Machine Pattern: Enum with can_transition_to method for valid transitions"
  - "String-Enum Bridge: DB uses strings, domain uses enums with parse methods"

requirements-completed:
  - JOB-03
  - JOB-04
  - JOB-05

duration: 2 min
completed: 2026-03-01
---

# Phase 5 Plan 2: Job Types and Repository Layer Summary

**Type-safe job system with state machine enforcement, race-free job claiming via FOR UPDATE SKIP LOCKED, and date range tracking for incremental loading.**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-01T15:44:19Z
- **Completed:** 2026-03-01T15:46:32Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments

- Created complete jobs module with error handling, types, and repositories
- Implemented JobError enum covering database, serialization, and state transition errors
- Defined JobType (Load, Query) and JobStatus (Pending, InProcess, Complete, Failed) enums with strum derives
- Built Job struct with UUID, JSONB payload/result/error fields, and timestamp tracking
- Implemented race-free job claiming using FOR UPDATE SKIP LOCKED PostgreSQL pattern
- Created LoadedDaysRepository for tracking loaded dates and detecting gaps
- State machine validation ensures valid status transitions (Pending→InProcess→Complete/Failed)

## Task Commits

Each task was committed atomically:

1. **Task 1: Create jobs module structure and error types** - `fe9b645` (feat)
2. **Task 2: Create job types (Job, JobStatus, JobType)** - `61c62d0` (feat)
3. **Task 3: Create JobRepository with race-free job claiming** - `8fc9352` (feat)

## Files Created/Modified

- `src/jobs/mod.rs` - Module declarations and public exports
- `src/jobs/error.rs` - JobError enum with thiserror, JobResult type alias
- `src/jobs/types.rs` - Job, JobType, JobStatus with strum derives and state machine
- `src/jobs/repository.rs` - JobRepository and LoadedDaysRepository implementations

## Decisions Made

1. **String-Enum Bridge Pattern**: Stored job_type and status as strings in DB struct for sqlx compatibility, with helper methods (job_type_enum(), status_enum()) for domain enum conversion. This balances database compatibility with type safety.

2. **FOR UPDATE SKIP LOCKED**: Used PostgreSQL's row-level locking with SKIP LOCKED to implement race-free job claiming. This ensures only one worker can claim each job even under concurrent access, without blocking other workers.

3. **State Machine in Code**: Implemented can_transition_to method on JobStatus to validate transitions before database updates. This complements the CHECK constraint at the database level.

4. **generate_series for Date Gaps**: Used PostgreSQL's generate_series function in LoadedDaysRepository::get_missing_dates for efficient detection of unloaded dates in a range.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - all tasks completed successfully on first attempt.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Job types and repositories complete and ready for 05-03 (Job executor)
- JobExecutor will use JobRepository for job claiming and status updates
- State machine validation in place for executor to use
- LoadedDaysRepository ready for incremental loading implementation
- All JOB-03, JOB-04, JOB-05 requirements satisfied

---
*Phase: 05-job-infrastructure*
*Completed: 2026-03-01*

## Self-Check: PASSED

- [x] All created files exist on disk:
  - src/jobs/mod.rs
  - src/jobs/error.rs
  - src/jobs/types.rs
  - src/jobs/repository.rs
- [x] All commits verified in git log
- [x] cargo check --features db passes
- [x] STATE.md updated with current position
- [x] ROADMAP.md updated with plan progress
- [x] REQUIREMENTS.md marked JOB-03, JOB-04, JOB-05 complete
