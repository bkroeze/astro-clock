---
phase: 05-job-infrastructure
plan: 03
subsystem: jobs
tags: [tokio, spawn-blocking, oneshot, async-trait, executor]

requires:
  - phase: 05-01
    provides: [Job types, database schema]
  - phase: 05-02
    provides: [JobRepository, LoadedDaysRepository]

provides:
  - JobHandler trait for pluggable job handlers
  - JobExecutor with sync/async execution modes
  - spawn_blocking + oneshot pattern for CPU-intensive work
  - Background job processing with tokio::spawn
  - Job status polling API

affects:
  - 05-04 (Worker pool)
  - 06-01 (LoadJobHandler)
  - 07-01 (QueryJobHandler)

tech-stack:
  added: [async-trait]
  patterns:
    - "spawn_blocking + oneshot channel for CPU-intensive FFI calls"
    - "Handler registry pattern with HashMap<JobType, Arc<dyn JobHandler>>"
    - "Dual execution modes: sync (blocking) and async (background)"

key-files:
  created:
    - src/jobs/executor.rs
  modified:
    - src/jobs/repository.rs
    - src/jobs/mod.rs
    - src/jobs/types.rs
    - src/lib.rs
    - Cargo.toml

key-decisions:
  - "JobExecutor::execute_sync uses spawn_blocking to avoid blocking async runtime during CPU-intensive Swiss Ephemeris FFI calls"
  - "JobExecutor::execute_async returns job-id immediately and spawns background task for API use cases"
  - "Handler registry uses Arc<dyn JobHandler> for thread-safe shared ownership"

patterns-established:
  - "spawn_blocking + oneshot: CPU-intensive work runs in spawn_blocking with oneshot channel for result communication"
  - "Handler registration: JobHandler trait implementations registered by JobType in HashMap"
  - "State machine enforcement: Jobs progress pending → in_process → complete|failed"

requirements-completed: [JOB-06, RESULT-01, RESULT-02]

duration: 4min
completed: 2026-03-01
---

# Phase 05 Plan 03: Job Executor Summary

**JobExecutor with sync/async execution modes using spawn_blocking + oneshot pattern for CPU-intensive FFI calls**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-01T15:49:15Z
- **Completed:** 2026-03-01T15:50:30Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments

- JobHandler trait for pluggable job type handlers (LoadJob, QueryJob in later phases)
- JobExecutor with execute_sync (blocking) and execute_async (background) modes
- spawn_blocking + oneshot channel pattern for CPU-intensive work isolation
- Background job processing with proper error handling and state updates
- poll_job_status API for async job completion polling

## Task Commits

Each task was committed atomically:

1. **Task 1: Create JobHandler trait and executor structure** - `e1f2766` (feat)
2. **Task 2: Update JobRepository to expose pool access** - `c789566` (feat)
3. **Task 3: Update module exports and lib.rs** - `b118fb4` (feat)

**Plan metadata:** `7a0937d` (docs: complete plan)

## Files Created/Modified

- `src/jobs/executor.rs` - JobHandler trait and JobExecutor with sync/async execution
- `src/jobs/repository.rs` - Added pool() method, fixed lifetime issue in list_jobs
- `src/jobs/mod.rs` - Exported executor module and JobExecutor/JobHandler
- `src/jobs/types.rs` - Removed conflicting Display impl, added Hash derive to JobType
- `src/lib.rs` - Added jobs module with feature gate
- `Cargo.toml` - Added async-trait and uuid dependencies, enabled sqlx uuid feature

## Decisions Made

1. **spawn_blocking for CPU-intensive work**: Swiss Ephemeris FFI calls are CPU-intensive and must not block the async runtime. Using spawn_blocking with oneshot channel isolates this work.

2. **Dual execution modes**: execute_sync blocks until completion (for CLI), execute_async returns immediately with job-id (for API).

3. **Arc<dyn JobHandler> for registry**: Thread-safe shared ownership allows handlers to be registered once and used across multiple concurrent job executions.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added uuid dependency with sqlx feature**

- **Found during:** Task 3 (compilation)
- **Issue:** uuid crate not in Cargo.toml, sqlx couldn't decode Uuid type
- **Fix:** Added `uuid = { version = "1", features = ["v4", "serde"] }` and enabled `"uuid"` feature in sqlx
- **Files modified:** Cargo.toml
- **Verification:** cargo check --features db passes
- **Committed in:** b118fb4 (Task 3 commit)

**2. [Rule 1 - Bug] Fixed conflicting Display implementations in JobStatus**

- **Found during:** Task 3 (compilation)
- **Issue:** Both strum_macros::Display and manual impl Display for JobStatus
- **Fix:** Removed manual impl fmt::Display, kept strum derive
- **Files modified:** src/jobs/types.rs
- **Verification:** cargo check --features db passes
- **Committed in:** b118fb4 (Task 3 commit)

**3. [Rule 1 - Bug] Added Hash derive to JobType for HashMap usage**

- **Found during:** Task 3 (compilation)
- **Issue:** JobType used as HashMap key but didn't implement Hash
- **Fix:** Added `#[derive(..., Hash, ...)]` to JobType enum
- **Files modified:** src/jobs/types.rs
- **Verification:** cargo check --features db passes
- **Committed in:** b118fb4 (Task 3 commit)

**4. [Rule 1 - Bug] Fixed lifetime issue in list_jobs**

- **Found during:** Task 3 (compilation)
- **Issue:** `s.as_ref()` borrows from `s` which doesn't live long enough for QueryBuilder
- **Fix:** Changed to `s.to_string()` to own the string
- **Files modified:** src/jobs/repository.rs
- **Verification:** cargo check --features db passes
- **Committed in:** b118fb4 (Task 3 commit)

---

**Total deviations:** 4 auto-fixed (3 Rule 1 bugs, 1 Rule 2 missing critical)

**Impact on plan:** All auto-fixes necessary for correctness and compilation. No scope creep.

## Issues Encountered

None beyond the auto-fixed compilation issues above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Job executor complete, ready for worker pool implementation (05-04)
- Handler trait defined, ready for LoadJobHandler (Phase 6) and QueryJobHandler (Phase 7)
- All job infrastructure foundation complete

---

_Phase: 05-job-infrastructure_
_Completed: 2026-03-01_
