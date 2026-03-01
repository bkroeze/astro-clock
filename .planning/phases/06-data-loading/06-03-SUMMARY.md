---
phase: 06-data-loading
plan: 03
subsystem: api
tags: [axum, rest-api, job-endpoints, load-api]

requires:
  - phase: 06-01
    provides: [LoadJobHandler, JobExecutor with sync/async modes]
  - phase: 05-03
    provides: [JobHandler trait, JobExecutor implementation]

provides:
  - POST /api/v1/load endpoint with sync/async execution modes
  - GET /api/v1/jobs/{job-id} endpoint for job status polling
  - AppState struct for sharing executor and pool across handlers
  - Input validation (date format, days range) with 400 Bad Request
  - Structured error responses with code and message fields
  - Job response serialization with all timestamps and payload/result/error

affects:
  - 08-01 (CLI integration - both CLI and API use same JobExecutor)
  - 08-02 (API documentation - these are the endpoints to document)

tech-stack:
  added: []
  patterns:
    - "axum State extractor pattern for dependency injection"
    - "impl IntoResponse for flexible handler return types"
    - "serde_json::json!() for dynamic response building"
    - "Feature-gated server routes for optional db support"

key-files:
  created:
    - src/server/routes/mod.rs - Routes module structure with re-exports
    - src/server/routes/jobs.rs - Load and job status handlers with request/response types
    - src/server/state.rs - AppState with executor and pool for axum State
  modified:
    - src/server/mod.rs - Integrated routes, added database initialization, dual build paths

key-decisions:
  - "Use impl IntoResponse instead of Result<impl IntoResponse, StatusCode> for cleaner handler signatures"
  - "Store both executor and pool in AppState - pool needed for JobRepository in handlers"
  - "Parse and validate request parameters before creating jobs - fail fast for bad input"
  - "Feature-gate database-dependent server code - supports builds without db feature"

patterns-established:
  - "Handler validation before executor: Validate inputs in handler, executor trusts validated data"
  - "Consistent error response format: {error: code, message: description} across all endpoints"
  - "Private fields with accessor methods: AppState uses private fields with executor() and get_pool()"

requirements-completed: [LOAD-07, RESULT-03, RESULT-04]

duration: 4min
completed: 2026-03-01
---

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
