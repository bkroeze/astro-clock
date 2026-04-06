---
id: T01
parent: S07
milestone: M001
provides:
  - QueryJobHandler for executing named queries as jobs
  - QueryTemplateRegistry for query name to function dispatch
  - QueryJobPayload and QueryJobResult structs
  - Automatic data pre-loading before query execution
  - Wedding query integration via registry
requires: []
affects: []
key_files: []
key_decisions: []
patterns_established: []
observability_surfaces: []
drill_down_paths: []
duration: 11min
verification_result: passed
completed_at: 2026-03-01
blocker_discovered: false
---
# T01: 07-named-queries 01

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
