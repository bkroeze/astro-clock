---
id: S07
parent: M001
milestone: M001
provides:
  - QueryJobHandler for executing named queries as jobs
  - QueryTemplateRegistry for query name to function dispatch
  - QueryJobPayload and QueryJobResult structs
  - Automatic data pre-loading before query execution
  - Wedding query integration via registry
  - ProjectCriteria struct with validation
  - TravelCriteria struct with validation
  - ProjectCandidate result struct
  - TravelCandidate result struct
  - find_project_dates() query function
  - find_travel_dates() query function
  - Updated QueryTemplateRegistry with actual query functions
  - FAVORABLE_PROJECT_SIGNS constant
  - FAVORABLE_TRAVEL_SIGNS constant
  - TravelPurpose enum
  - POST /api/v1/query/:query_name HTTP endpoint
  - QueryRequest/QuerySyncResponse/QueryAsyncResponse types
  - Synchronous and asynchronous query execution modes
  - Input validation for query parameters
  - Integration tests for query job execution
requires: []
affects: []
key_files: []
key_decisions:
  - Project and travel queries use placeholder implementations in 07-01, full implementation in 07-02
  - QueryTemplateRegistry uses type-erased futures (Pin<Box<dyn Future>>) to store different query types
  - Auto-loading continues on partial failure with warnings in result
  - QueryJobResult includes total_results, execution_time_ms, and optional warnings
  - Clone pool and payload data before async blocks to resolve lifetime issues
  - Use Mercury direct criteria (not in retrograde_periods) for project/travel
  - Exclude Scorpio and Capricorn from favorable signs (intensity/restriction)
  - Include Gemini in travel signs (movement) but Aries in project signs (initiation)
  - Include VoC status in travel results for traveler awareness
  - Used 202 ACCEPTED status for async mode to indicate job acceptance
  - Used 200 OK for sync mode to indicate immediate success
  - Added is_valid_date helper for YYYY-MM-DD format validation
  - Query names validated against whitelist: wedding, project, travel
patterns_established:
  - Query dispatch via registry pattern: registry.get(query_name) -> execute
  - Pre-flight data loading: ensure_data_loaded() before query execution
  - Structured JSON results with common metadata wrapper
  - Route handlers follow same pattern as jobs.rs for consistency
  - Feature-gated queries module behind db feature flag
  - Integration tests in nested module within #[cfg(test)] block
observability_surfaces: []
drill_down_paths: []
duration: 7min
verification_result: passed
completed_at: 2026-03-02
blocker_discovered: false
---
# S07: Named Queries

**# Phase 7 Plan 1: QueryJobHandler and QueryTemplateRegistry Summary**

## What Happened

# Phase 7 Plan 1: QueryJobHandler and QueryTemplateRegistry Summary

**Query job infrastructure with automatic data loading and registry-based dispatch for wedding, project, and travel queries**

## Performance

- **Duration:** 11 min
- **Started:** 2026-03-01T23:43:00Z
- **Completed:** 2026-03-01T23:53:34Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments

- QueryJobHandler implementing JobHandler trait for JobType::Query
- QueryTemplateRegistry mapping query names (wedding, project, travel) to execution functions
- Automatic data loading via ensure_data_loaded() using LoadedDaysRepository and ChunkGenerator
- Wedding query fully integrated via registry
- Project and travel query placeholders (to be implemented in 07-02)
- Comprehensive unit tests for serialization and registry behavior

## Task Commits

Each task was committed atomically:

1. **Task 1: Create QueryJobHandler** - `ecefdc9` (feat)
   - QueryJobPayload and QueryJobResult structs with serde derives
   - parse_payload() validation for query_name, start_date, days
   - ensure_data_loaded() with partial failure handling
   - JobHandler trait implementation

2. **Task 2: Create QueryTemplateRegistry** - `2c8efbf` (feat)
   - QueryTemplate and QueryTemplateRegistry structs
   - Type-erased future pattern for query dispatch
   - Wedding query integration, project/travel placeholders
   - Unit tests for registry operations

3. **Task 3: Module exports** - `8288c12` (feat)
   - Exports from src/jobs/mod.rs
   - Exports from src/jobs/handlers/mod.rs
   - Fixed queries module exports for Project/Travel types

**Plan metadata:** (included in commits)

## Files Created/Modified

- `src/jobs/handlers/query.rs` (369 lines) - QueryJobHandler with payload parsing, data pre-loading, and execution
- `src/jobs/registry.rs` (396 lines) - QueryTemplateRegistry with wedding, project, travel templates
- `src/jobs/mod.rs` - Added exports for QueryJobHandler, QueryJobPayload, QueryJobResult, QueryTemplateRegistry, QueryTemplate
- `src/jobs/handlers/mod.rs` - Added query module and exports
- `src/queries/mod.rs` - Fixed exports for ProjectCandidate, ProjectCriteria, TravelCandidate, TravelCriteria

## Decisions Made

1. **Placeholder queries for project/travel**: Following the plan's must_haves key_links note, project and travel templates return "not yet implemented" errors in 07-01, to be replaced with actual implementations in 07-02.

2. **Type-erased futures for registry**: Used `Pin<Box<dyn Future<Output = JobResult<JsonValue>> + Send>>` to allow storing different query functions in the same HashMap. This provides flexibility while maintaining type safety at the registry boundary.

3. **Continue on partial data load failure**: Rather than failing the entire job when some dates fail to load, the handler collects warnings and proceeds with available data. This follows the LoadJobHandler pattern for resilience.

4. **Structured result wrapper**: QueryJobResult includes common metadata (query_name, start_date, days, total_results, execution_time_ms) alongside the actual results array, providing consistent response format across query types.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

1. **Module export errors for project/travel types**: The queries/mod.rs was trying to export ProjectCandidate, ProjectCriteria from project.rs module, but they were defined in types.rs. Fixed by exporting from types.rs directly.

2. **Lifetime issues in registry closures**: The closure in QueryExecuteFn captures references that need to outlive the async block. Fixed by cloning pool and payload data before moving into the async block.

## Next Phase Readiness

- QueryJobHandler is ready for use by JobExecutor
- QueryTemplateRegistry supports wedding queries fully
- Project and travel query placeholders ready for 07-02 implementation
- All modules compile with --features db
- 115 tests passing

---
*Phase: 07-named-queries*
*Completed: 2026-03-01*

# Phase 07 Plan 02: Project and Travel Queries Summary

**Implemented project and travel query functions with Mercury direct + favorable Moon sign criteria, completing the named query set alongside wedding queries.**

## Performance

- **Duration:** 9 min
- **Started:** 2026-03-01T23:43:11Z
- **Completed:** 2026-03-01T23:52:54Z
- **Tasks:** 4
- **Files modified:** 7

## Accomplishments

- Created `find_project_dates()` with Mercury direct + favorable Moon (Taurus, Cancer, Leo, Libra, Aquarius, Pisces, Aries) criteria
- Created `find_travel_dates()` with Mercury direct + favorable Moon (Taurus, Cancer, Leo, Libra, Aquarius, Pisces, Gemini) criteria + VoC tracking
- Added `ProjectCriteria` and `TravelCriteria` structs with validation (date range, limits)
- Added `ProjectCandidate` and `TravelCandidate` result structs with full Serialize/Deserialize support
- Updated `QueryTemplateRegistry` to execute actual query functions instead of placeholder errors
- Exported new modules and types from `src/queries/mod.rs` and `src/jobs/mod.rs`
- All 115 tests pass

## Task Commits

Each task was committed atomically:

1. **Task 1: Add ProjectCriteria and TravelCriteria to types.rs** - `b31eda9` (feat)
2. **Task 2: Implement find_project_dates query function** - `43ca88f` (feat)
3. **Task 3: Implement find_travel_dates query function** - `81912fa` (feat)
4. **Task 4: Update QueryTemplateRegistry with actual query functions** - `14ead17` (feat)

## Files Created/Modified

- `src/queries/types.rs` - Added ProjectCriteria, TravelCriteria, ProjectCandidate, TravelCandidate, TravelPurpose enum, favorable signs constants, unit tests
- `src/queries/project.rs` - find_project_dates() with Mercury direct + favorable Moon SQL query
- `src/queries/travel.rs` - find_travel_dates() with VoC tracking + Mercury direct SQL query
- `src/queries/mod.rs` - Export project and travel modules
- `src/jobs/registry.rs` - Updated project and travel templates to use actual query functions
- `src/jobs/mod.rs` - Export registry and query handler types
- `src/jobs/handlers/mod.rs` - Export QueryJobHandler and QueryJobPayload

## Decisions Made

1. **Clone pool and payload before async blocks**: Resolved lifetime issues in QueryTemplateRegistry closures by cloning DatabasePool and payload fields before moving into async blocks.

2. **Mercury direct criteria**: Both project and travel queries filter for Mercury direct periods (not in retrograde_periods table), essential for clear planning and smooth travel.

3. **Favorable sign selection**: 
   - Project: Aries (initiation), Taurus, Cancer, Leo, Libra, Aquarius, Pisces
   - Travel: Gemini (movement), Taurus, Cancer, Leo, Libra, Aquarius, Pisces
   - Both exclude Scorpio (intensity) and Capricorn (restriction)

4. **VoC status in travel**: Travel query includes void-of-course status in results for traveler awareness, while project query excludes VoC periods entirely.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

1. **Lifetime issues in registry closures**: The original closure pattern captured pool and payload by reference, causing lifetime errors when moved into async blocks.
   - **Resolution**: Clone pool and payload fields (start_date, days) before the async block to ensure owned data is moved.

2. **Missing imports in registry.rs**: After editing, the import statements for ProjectCriteria, TravelCriteria, and query functions were incomplete.
   - **Resolution**: Added complete import statements for all required types and functions.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- All named queries (wedding, project, travel) execute real queries through QueryTemplateRegistry
- QueryJobHandler can execute all query types via job system
- Ready for Phase 8: CLI & API Integration to expose queries via HTTP endpoints
- No blockers - QueryTemplateRegistry pattern proven working

---
*Phase: 07-named-queries*
*Completed: 2026-03-01*

# Phase 7 Plan 3: Query API Endpoints Summary

**HTTP API endpoints for named queries with sync/async execution modes and structured JSON responses.**

## Performance

- **Duration:** 7 min
- **Started:** 2026-03-01T23:57:37Z
- **Completed:** 2026-03-02T00:03:59Z
- **Tasks:** 4
- **Files modified:** 4

## Accomplishments

- Created query API route handler (`src/server/routes/queries.rs`) with sync/async support
- Exported queries module from routes with proper feature gating
- Integrated query routes into server router with QueryJobHandler registration
- Added 5 integration tests verifying wedding query end-to-end execution

## Task Commits

Each task was committed atomically:

1. **Task 1: Create query API route handler** - `b709a59` (feat)
2. **Task 2: Export queries module from routes** - `d779ff7` (feat)
3. **Task 3: Integrate query routes into server router** - `b5ab6a7` (feat)
4. **Task 4: Add end-to-end integration tests** - `d17e6a4` (feat)

**Plan metadata:** TBD (docs commit)

## Files Created/Modified

- `src/server/routes/queries.rs` - New query API route handler with POST /api/v1/query/:query_name endpoint
- `src/server/routes/mod.rs` - Added queries module export behind db feature gate
- `src/server/mod.rs` - Added query route and QueryJobHandler to executor
- `src/jobs/handlers/query.rs` - Added 5 integration tests for query execution

## Decisions Made

- Used 202 ACCEPTED status code for async mode to indicate job acceptance (vs 200 OK for sync)
- Validated query names against explicit whitelist (wedding, project, travel) for security
- Added date format validation using chrono::NaiveDate::parse_from_str
- Maintained consistency with jobs.rs patterns for request/response types
- Placed integration tests in nested module for organization

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - all compilation and tests passed on first attempt.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Query API endpoints are ready for Phase 8 (CLI & API Integration)
- HTTP clients can now execute wedding, project, and travel queries
- Both sync (blocking) and async (polling) modes available
- All 127 tests passing

---
*Phase: 07-named-queries*
*Completed: 2026-03-02*
