# Phase 3: Query System - Context

**Gathered:** 2026-02-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Build specialized query functions for electoral astrology. This includes wedding date queries (Moon sign + Venus aspects + no VoC), void-of-course Moon period detection, planetary retrograde queries, and exact aspect searches. All queries must complete in <100ms using aspect_summaries for performance.

</domain>

<decisions>
## Implementation Decisions

### Query interface design
- Queries return structured result types (not just raw database rows)
- Each query has a corresponding `*Criteria` struct for parameters
- Results include metadata: execution time, rows examined, cache hit/miss
- Async functions returning `Result<Vec<T>, QueryError>`

### Wedding date query criteria
- Moon must be in favorable signs (Taurus, Cancer, Leo, Libra, Scorpio, Capricorn, Aquarius, Pisces)
- Minimum Venus favorable aspects threshold (configurable, default: 2)
- Exclude void-of-course Moon periods
- Sort by Venus aspect count descending, then by date ascending
- Return top N results (configurable, default: 10)

### Void-of-course Moon detection
- Use pre-calculated `is_void_of_course` flag from lunar_conditions table
- Query returns start time, end time, duration, Moon sign for each VoC period
- Option to filter by minimum duration (configurable, default: no filter)

### Retrograde period query
- Query retrograde_periods table for date ranges
- Return planet, start time, end time, shadow periods (if available)
- Option to filter by specific planets or date range
- Include current status (in retrograde, in shadow, direct)

### Exact aspect query
- Search for exact aspects within an orb threshold (configurable, default: 1°)
- Filter by aspect types (conjunction, opposition, trine, square, sextile)
- Filter by body pairs (e.g., Sun-Moon, Venus-Mars)
- Return aspect details: exact time, orb, applying/separating

### Performance requirements
- All queries must complete in <100ms for 60-day ranges (cached)
- Use aspect_summaries table for aspect counting (not raw aspects table)
- Leverage ChunkManager for data loading (cache-first)
- SQL queries should use indexes effectively

### Error handling
- QueryError enum with variants: Database, InvalidCriteria, Timeout
- Invalid criteria return descriptive error (not just generic failure)
- Database timeouts return partial results if possible

### Claude's Discretion
- Exact SQL query structure and CTE design
- Result struct field names and types
- Criteria validation logic
- Query benchmarking approach
- Whether to use sqlx query_as! macro or query_as function

</decisions>

<specifics>
## Specific Ideas

- Implementation plan already has detailed SQL for wedding query using CTEs
- Target: Wedding query 51× faster (2.3s → 45ms) using aspect_summaries JOIN
- Use existing ChunkManager for data access (not direct database queries)
- Query module structure: src/queries/{wedding,void_of_course,retrograde,aspects}.rs
- All queries should work with the existing database schema from Phase 1

</specifics>

<deferred>
## Deferred Ideas

- Multi-resolution storage optimization — Phase 4
- Aspect filtering to reduce storage — Phase 4  
- Continuous aggregates for lower resolutions — Phase 4
- Advanced pattern detection (grand trines, T-squares) — future phase
- Transit calculations relative to natal charts — future phase
- API endpoint wrappers for queries — future phase
- Query result caching beyond ChunkManager — future enhancement

</deferred>

---

*Phase: 03-query-system*
*Context gathered: 2026-02-25*
