---
phase: 07-named-queries
plan: 02
subsystem: queries
tags: [sqlx, queries, project, travel, mercury-direct, moon-signs]

requires:
  - phase: 07-named-queries
    provides: [WeddingCriteria pattern, QueryResult type, body_ids constants]

provides:
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

affects:
  - 07-named-queries
  - 08-cli-api-integration

tech-stack:
  added: []
  patterns:
    - "Mercury direct + favorable Moon signs criteria for project/travel queries"
    - "SQL JOIN with aspect_summaries for performance"
    - "NOT EXISTS subquery for retrograde period filtering"

key-files:
  created:
    - src/queries/project.rs
    - src/queries/travel.rs
  modified:
    - src/queries/types.rs
    - src/queries/mod.rs
    - src/jobs/registry.rs
    - src/jobs/mod.rs
    - src/jobs/handlers/mod.rs

key-decisions:
  - "Clone pool and payload data before async blocks to resolve lifetime issues"
  - "Use Mercury direct criteria (not in retrograde_periods) for project/travel"
  - "Exclude Scorpio and Capricorn from favorable signs (intensity/restriction)"
  - "Include Gemini in travel signs (movement) but Aries in project signs (initiation)"
  - "Include VoC status in travel results for traveler awareness"

requirements-completed: [QUERY-07, QUERY-08]

duration: 9min
completed: 2026-03-01
---

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
