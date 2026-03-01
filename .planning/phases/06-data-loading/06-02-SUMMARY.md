---
phase: 06-data-loading
plan: 02
subsystem: cli
tags: [cli, load-command, data-loading, job-execution]

requires:
  - phase: 06-01
    provides: [LoadJobHandler, JobExecutor with sync/async modes]

provides:
  - 'astro-clock load' CLI command with --start, --days, --sync arguments
  - Date validation (YYYY-MM-DD format) with clear error messages
  - Days range validation (1-365) to prevent excessive load requests
  - Synchronous execution mode (--sync) with structured result display
  - Asynchronous execution mode returning job-id for polling
  - Database pool initialization from DATABASE_URL or config
  - Feature-gated compilation for db feature

affects:
  - 06-03 (API integration)
  - 07-01 (QueryJobHandler CLI command)
  - 08-01 (CLI polish)

tech-stack:
  added: []
  patterns:
    - "CLI subcommand with structured argument parsing using clap"
    - "Feature-gated database functionality with graceful fallback"
    - "Runtime creation for async operations in sync CLI context"
    - "Result deserialization for structured job output display"

key-files:
  created: []
  modified:
    - src/cli/app.rs - Added Load command to Commands enum and handler in App::run()
    - src/jobs/handlers/mod.rs - Exported LoadJobResult for CLI result parsing
    - src/jobs/mod.rs - Re-exported LoadJobResult from jobs module

key-decisions:
  - "Runtime-per-async-block pattern - Create new tokio runtime for each async block since App::run() is sync"
  - "Result deserialization for display - Parse LoadJobResult JSON for formatted terminal output"
  - "DATABASE_URL env var priority - Check environment before config for database URL"
  - "Feature-gate the entire load command - #[cfg(feature = db)] ensures clean compilation without db"

patterns-established:
  - "Runtime creation pattern: Use tokio::runtime::Runtime::new() in sync CLI context for async operations"
  - "Structured result display: Deserialize job result JSON for formatted CLI output"
  - "Environment variable priority: DATABASE_URL env var overrides config setting"

requirements-completed: [LOAD-06]

duration: 3min
completed: 2026-03-01
---

# Phase 06 Plan 02: CLI Load Command Summary

**CLI `astro-clock load` command with --start, --days, --sync arguments for synchronous and asynchronous planetary data loading**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-01T22:51:23Z
- **Completed:** 2026-03-01T22:53:58Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Extended CLI with `Load` subcommand supporting --start, --days, and --sync flags
- Implemented date validation with YYYY-MM-DD format checking
- Added days range validation ensuring 1-365 limit
- Created database pool initialization from DATABASE_URL or config
- Implemented synchronous execution mode with detailed result display
- Implemented asynchronous execution mode returning job-id
- Added feature-gated compilation for clean builds without db feature
- Exported LoadJobResult from jobs module for result parsing

## Task Commits

Each task was committed atomically:

1. **Task 1: Add Load command to CLI** - `aaac3cf` (feat)
   - Added Load variant to Commands enum
   - Implemented command handler with validation and execution
   - Added sync/async execution modes
   - Integrated database pool and JobExecutor
   - Added structured result display

2. **Task 2: Add database pool initialization** - (covered in Task 1)
   - Pool creation using DatabasePool::connect pattern
   - Environment variable and config-based URL resolution
   - Proper pool configuration and lifecycle management

**Plan metadata:** [to be committed]

## Files Created/Modified

- `src/cli/app.rs` - Added Load command variant and handler implementation (~139 lines added)
- `src/jobs/handlers/mod.rs` - Exported LoadJobResult type for CLI deserialization
- `src/jobs/mod.rs` - Re-exported LoadJobResult from jobs module root

## Decisions Made

1. **Runtime-per-async-block**: Since `App::run()` is synchronous, each async operation creates a new tokio runtime. This pattern is used throughout the existing CLI code (e.g., for server startup).

2. **Result deserialization**: Instead of displaying raw JSON, the CLI deserializes the LoadJobResult for formatted output showing loaded/skipped/failed counts and per-date error details.

3. **Environment variable priority**: DATABASE_URL environment variable takes precedence over config file setting, following 12-factor app principles.

4. **Feature-gate at command level**: The entire Load command is wrapped in `#[cfg(feature = "db")]` with a helpful error message when db is not enabled.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None significant. Minor borrow checker issue resolved:

- **Resolved**: Used `ref` pattern in `if let Some(ref result)` to avoid partial move, then cloned for deserialization since `serde_json::from_value` takes ownership.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- CLI load command ready for use with LoadJobHandler
- Pattern established for future job type CLI commands (QueryJobHandler in Phase 7)
- Database pool initialization pattern reusable for other CLI features
- Ready for Phase 6 Plan 3: API integration for load jobs

---
*Phase: 06-data-loading*
*Completed: 2026-03-01*
