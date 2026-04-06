---
id: S06
parent: M001
milestone: M001
provides:
  - LoadJobHandler implementing JobHandler trait for JobType::Load
  - Day-level incremental loading with gap detection via LoadedDaysRepository
  - Resume-capable data loading through loaded_days tracking table
  - Structured JSON results with loaded/skipped/failed statistics
  - Per-date error handling that tracks failures without failing entire job
  - 'astro-clock load' CLI command with --start, --days, --sync arguments
  - Date validation (YYYY-MM-DD format) with clear error messages
  - Days range validation (1-365) to prevent excessive load requests
  - Synchronous execution mode (--sync) with structured result display
  - Asynchronous execution mode returning job-id for polling
  - Database pool initialization from DATABASE_URL or config
  - Feature-gated compilation for db feature
  - POST /api/v1/load endpoint with sync/async execution modes
  - GET /api/v1/jobs/{job-id} endpoint for job status polling
  - AppState struct for sharing executor and pool across handlers
  - Input validation (date format, days range) with 400 Bad Request
  - Structured error responses with code and message fields
  - Job response serialization with all timestamps and payload/result/error
requires: []
affects: []
key_files: []
key_decisions:
  - Store Pool<Postgres> in handler and create DatabasePool on demand - maintains compatibility with existing repository patterns
  - Sequential day processing within job - Swiss Ephemeris requires thread isolation
  - Per-day failures tracked but don't fail job - enables partial success reporting
  - Mark job Complete if >=1 day loaded, only Failed if 0 progress - matches user expectations for resumable operations
  - Runtime-per-async-block pattern - Create new tokio runtime for each async block since App::run() is sync
  - Result deserialization for display - Parse LoadJobResult JSON for formatted terminal output
  - DATABASE_URL env var priority - Check environment before config for database URL
  - Feature-gate the entire load command - #[cfg(feature = db)] ensures clean compilation without db
  - Use impl IntoResponse instead of Result<impl IntoResponse, StatusCode> for cleaner handler signatures
  - Store both executor and pool in AppState - pool needed for JobRepository in handlers
  - Parse and validate request parameters before creating jobs - fail fast for bad input
  - Feature-gate database-dependent server code - supports builds without db feature
patterns_established:
  - Handler orchestration pattern: Handler coordinates repositories and generators, doesn't implement business logic directly
  - Structured result types: Define serde-serializable result structs for type-safe job outputs
  - Feature-gated handlers: #[cfg(feature = \"db\")] on handler modules for clean compilation without database
  - Runtime creation pattern: Use tokio::runtime::Runtime::new() in sync CLI context for async operations
  - Structured result display: Deserialize job result JSON for formatted CLI output
  - Environment variable priority: DATABASE_URL env var overrides config setting
  - Handler validation before executor: Validate inputs in handler, executor trusts validated data
  - Consistent error response format: {error: code, message: description} across all endpoints
  - Private fields with accessor methods: AppState uses private fields with executor() and get_pool()
observability_surfaces: []
drill_down_paths: []
duration: 4min
verification_result: passed
completed_at: 2026-03-01
blocker_discovered: false
---
# S06: Data Loading

**# Phase 06 Plan 01: LoadJobHandler Summary**

## What Happened

# Phase 06 Plan 01: LoadJobHandler Summary

**LoadJobHandler implementing JobHandler trait for day-level incremental planetary data loading with gap detection and resume capability**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-01T22:43:06Z
- **Completed:** 2026-03-01T22:48:23Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments

- LoadJobHandler implements JobHandler trait for JobType::Load with full orchestration logic
- Integrates ChunkGenerator for planetary data generation and persistence
- Uses LoadedDaysRepository for intelligent gap detection and resume capability
- Returns structured JSON results with detailed statistics (positions, aspects, lunar conditions)
- Per-date error handling tracks failures without failing the entire job
- Comprehensive unit tests for payload and result serialization

## Task Commits

Each task was committed atomically:

1. **Task 1: Create handlers module structure** - `3f83491` (feat)
2. **Task 2: Implement LoadJobHandler** - `fa3ad32` (feat)
3. **Task 3: Export LoadJobHandler from jobs module** - `3734fb6` (feat)

**Plan metadata:** [to be committed with SUMMARY.md]

## Files Created/Modified

- `src/jobs/handlers/mod.rs` - Handler module entry point with documentation and re-exports
- `src/jobs/handlers/load.rs` - LoadJobHandler with execute() logic, payload/result types, and tests
- `src/jobs/mod.rs` - Added handlers module with feature gate, re-exported LoadJobHandler
- `src/database/pool.rs` - Added `from_pool()` constructor to create DatabasePool from Pool<Postgres>

## Decisions Made

1. **Pool storage strategy**: Store `Pool<Postgres>` in handler and create `DatabasePool` on demand. This maintains compatibility with `LoadedDaysRepository` (takes Pool) and `ChunkGenerator` (takes DatabasePool).

2. **Sequential day processing**: Process days one at a time within the job to ensure Swiss Ephemeris thread safety. The JobExecutor already uses `spawn_blocking`, and sequential processing within that block ensures no concurrent access to the C library.

3. **Partial success handling**: Track per-date failures but continue processing remaining dates. Job is marked Complete if at least one day loads successfully, only Failed if zero progress is made. This matches user expectations for resumable operations.

4. **Feature gating**: Apply `#[cfg(feature = "db")]` to handler modules since they require database connectivity. This follows the existing pattern in the jobs module.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None significant. Minor compilation fix needed when linking modules:

- **Resolved**: Added `DatabasePool::from_pool()` constructor to enable creating DatabasePool from an existing Pool<Postgres> without reconnecting. This was necessary because ChunkGenerator requires DatabasePool while LoadedDaysRepository requires Pool<Postgres>.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- LoadJobHandler ready for CLI integration (Phase 6 Plan 2)
- LoadJobHandler ready for API integration (Phase 6 Plan 3)
- Pattern established for QueryJobHandler in Phase 7
- All data loading requirements (LOAD-08, LOAD-09, LOAD-10) satisfied

---
*Phase: 06-data-loading*
*Completed: 2026-03-01*

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

# Phase 06 Plan 03: HTTP API Endpoints Summary

**POST /api/v1/load and GET /api/v1/jobs/{id} endpoints with sync/async execution modes, input validation, and structured error responses using axum State extractor pattern**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-01T22:51:40Z
- **Completed:** 2026-03-01T22:55:27Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments

- Created modular server routes structure with src/server/routes/ module
- Implemented POST /api/v1/load endpoint supporting both sync and async execution modes
- Implemented GET /api/v1/jobs/{id} endpoint returning full job details with payload, result, error
- Added comprehensive input validation (date format YYYY-MM-DD, days range 1-365)
- Created AppState struct for sharing JobExecutor and database pool across handlers
- Integrated routes into main server with database pool initialization
- Added feature-gated compilation supporting both db and non-db builds
- Structured error responses with error code and human-readable message

## Task Commits

Each task was committed atomically:

1. **Task 1-2: Create server routes module and AppState** - `0dcf5ba` (feat)
2. **Task 3: Integrate routes into server** - `28c2bcb` (feat)

**Plan metadata:** [to be committed with SUMMARY.md]

## Files Created/Modified

- `src/server/routes/mod.rs` - Routes module entry point with documentation and re-exports
- `src/server/routes/jobs.rs` - Load and job status handlers with:
  - LoadRequest, LoadSyncResponse, LoadAsyncResponse, JobResponse structs
  - load_handler with sync/async execution and input validation
  - get_job_handler for job status polling
  - ErrorResponse type with code and message
  - Comprehensive unit tests for serialization/deserialization
- `src/server/state.rs` - AppState struct with Arc<JobExecutor> and Pool<Postgres>
- `src/server/mod.rs` - Updated server with:
  - build_app_with_db() creating pool, repository, handler, executor, state
  - build_app_without_db() for non-feature-gated builds
  - New API routes integrated with existing /health and /chart

## Decisions Made

1. **Use impl IntoResponse**: Changed from `Result<impl IntoResponse, StatusCode>` to just `impl IntoResponse` for cleaner handler signatures and more flexible response building with serde_json::json!().

2. **Store pool in AppState**: AppState holds both the executor and the pool directly, since handlers need to create JobRepository instances which require the pool.

3. **Validate before execute**: Input validation (date format, days range) happens in the handler before calling executor methods, providing fast failure with clear 400 Bad Request responses.

4. **Feature-gated server initialization**: Server routes that depend on the database are compiled only when the `db` feature is enabled, supporting builds without database connectivity.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

**Type inference issues with error responses** - Initially used `Result<impl IntoResponse, StatusCode>` return type with explicit Json wrapper types, which caused type mismatches. Fixed by:
- Switching to `impl IntoResponse` return type
- Using `serde_json::json!()` macro for response serialization
- This provided cleaner code and consistent response formatting

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- API endpoints ready for integration testing (Phase 8)
- LoadJobHandler accessible via HTTP API
- Job status polling endpoint enables progress tracking
- Pattern established for future query endpoints (Phase 7)
- All data loading API requirements (LOAD-07, RESULT-03, RESULT-04) satisfied

---
*Phase: 06-data-loading*
*Completed: 2026-03-01*
