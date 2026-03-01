# Project State: Astro Clock

**Status:** In Progress
**Last Updated:** 2026-03-01

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-02-24)

**Core value:** Generate accurate, visually appealing astrological charts from any date/time/location with minimal configuration
**Current focus:** Phase 4 — Performance Optimization (Plan 04 in progress)

## Phase Status

| Phase | Status | Requirements | Progress |
|-------|--------|--------------|----------|
| 1 — Database Schema | ✓ Complete | 6 | 100% |
| 2 — Data Loading | ✓ Complete | 5 | 100% |
| 3 — Query System | ✓ Complete | 5 | 100% |
| 4 — Performance | ○ In Progress | 4 | 25% |

## Current Phase

**Phase 4: Performance** ○ IN PROGRESS

Goal: Memory monitoring, cache management, and interpolation for optimal performance

Requirements: PERF-01 to PERF-04 (1 of 4 complete)

**Completed Plans:**
- ✓ 04-01: Multi-Resolution Chunk Manager (2026-02-28)
  - Continuous aggregates for 1min/5min/60min resolutions
  - Resolution selection by body movement speed
  - MultiResolutionManager with automatic resolution selection
- ✓ 04-02: Memory Monitoring (2026-03-01)
  - MemoryMonitor with configurable soft/hard limits
  - Memory pressure detection (Normal, Elevated, High, Critical)
  - Default limits: 30MB soft, 50MB hard, 90% eviction threshold
- ✓ 04-03: Interpolation Module (2026-03-01)
  - Linear interpolation with 360° wraparound handling
  - Major aspect filtering (conjunction, sextile, square, trine, opposition)
  - ~80% storage reduction through aspect filtering
- ✓ 04-05: ChunkManager Integration (2026-03-01)
  - Memory-aware cache eviction in ChunkManager
  - Automatic eviction at 90% threshold (45MB)
  - Evict 25% at High pressure, 50% at Critical

**Pending Plans:**
- 04-04: Cache Eviction Strategies

Next step: Continue with 04-04 — Cache Eviction Strategies

## Completed Work

- ✓ Project initialized
- ✓ Requirements defined (22 v1 requirements)
- ✓ Roadmap created (4 phases)
- ✓ Codebase mapped (7 documents)
- ✓ Phase 1 context gathered (2026-02-24)
- ✓ Plan 01-01: TimescaleDB hypertables and indexes (2026-02-25)
  - 4 hypertables with 1-day chunks
  - 13 composite/partial indexes
  - Normalized locations table
  - Aspect summaries table for 51× query speedup
- ✓ Plan 01-02: sqlx Migration Tooling (2026-02-25)
  - Justfile migrate commands (migrate, migrate-create, migrate-revert, migrate-info, db-setup)
  - .env.example with documented environment variables
  - Migration README with sqlx conventions
- ✓ Plan 01-03: Schema Verification and Rust Types (2026-02-25)
  - Schema verification SQL script (206 lines)
  - Rust types for all 6 tables with sqlx FromRow derives
  - Domain constants for type safety
- ✓ Phase 2 context gathered (2026-02-25)
- ✓ Phase 3 context gathered (2026-02-25)
- ✓ Plan 02-01: Compact Chunk Data Structures (2026-02-25)
  - ChunkKey, ChunkData, CompactPlanetPosition, CompactAspect, CompactLunarCondition
  - ~4.5× memory reduction (16 bytes vs 72 bytes per position)
  - lru crate for LRU cache implementation
- ✓ Plan 02-02: ChunkManager with LRU Cache (2026-02-25)
  - ChunkManager with RwLock-protected LRU cache
  - Database loading from three hypertables
  - f64 to Decimal conversion for sqlx compatibility

## Decisions

1. **SMALLINT for body IDs** (2026-02-25): Saves 2 bytes per row vs INTEGER, sufficient for 0-9 range
2. **DECIMAL(8,4) for coordinates** (2026-02-25): 0.0001° precision adequate for astrological calculations
3. **1-day chunk intervals** (2026-02-25): Aligns with chunk manager design and natural daily query patterns
4. **PG_URL environment variable** (2026-02-25): Required for all migration commands, validated in Justfile recipes
5. **Kept legacy structs for backward compatibility** (2026-02-25): ChartRecord and PlanetPositionRecord preserved during transition
6. **rust_decimal for DECIMAL types** (2026-02-25): Preserves exact precision required for astrological calculations
- [Phase 02-data-loading]: Used lru crate instead of custom implementation for cache correctness — Per CONTEXT.md discretion, using battle-tested crate saves development time and ensures O(1) operations
- [Phase 02-data-loading]: Packed struct representation with integer encoding for memory efficiency — Millidegrees and permille encoding achieves ~4.5× size reduction while maintaining sufficient precision
- [Phase 02-data-loading]: Manual f64 to Decimal conversion for sqlx compatibility — rust_decimal doesn't implement sqlx traits, so we query as f64 and convert using Decimal::from_f64_retain()
- [Phase 02-data-loading]: f64 for UNNEST batch inserts — rust_decimal doesn't implement sqlx array traits, so we use f64 arrays and let PostgreSQL cast to DECIMAL(8,4)
- [Phase 02-data-loading]: Fire-and-forget background database persistence — Database writes happen in tokio::spawn after returning chunk, ensuring low latency
- [Phase 02-data-loading]: Spawn pre-fetching in dedicated task to avoid Send bound issues — Recursive async calls create Send bound problems with tokio::spawn; internal task spawning in pre_fetch_adjacent_chunks solves this cleanly
- [Phase 02-data-loading]: AtomicU64 with Relaxed ordering for statistics — Statistics are for monitoring only, so strict memory ordering isn't required. Relaxed ordering provides better performance than SeqCst.
- [Phase 04-performance]: Used last() aggregation for continuous aggregates to capture most recent value in each bucket
- [Phase 04-performance]: Resolution mapping by body movement speed: Moon at 1-minute, inner planets at 5-minute, outer planets at 60-minute

## Blockers

None

## Decisions

1. **SMALLINT for body IDs** (2026-02-25): Saves 2 bytes per row vs INTEGER, sufficient for 0-9 range
2. **DECIMAL(8,4) for coordinates** (2026-02-25): 0.0001° precision adequate for astrological calculations
3. **1-day chunk intervals** (2026-02-25): Aligns with chunk manager design and natural daily query patterns
4. **PG_URL environment variable** (2026-02-25): Required for all migration commands, validated in Justfile recipes
5. **Kept legacy structs for backward compatibility** (2026-02-25): ChartRecord and PlanetPositionRecord preserved during transition
6. **rust_decimal for DECIMAL types** (2026-02-25): Preserves exact precision required for astrological calculations
7. **[Phase 02-data-loading]: Used lru crate instead of custom implementation for cache correctness** — Per CONTEXT.md discretion, using battle-tested crate saves development time and ensures O(1) operations
8. **[Phase 02-data-loading]: Packed struct representation with integer encoding for memory efficiency** — Millidegrees and permille encoding achieves ~4.5× size reduction while maintaining sufficient precision
9. **[Phase 02-data-loading]: Manual f64 to Decimal conversion for sqlx compatibility** — rust_decimal doesn't implement sqlx traits, so we query as f64 and convert using Decimal::from_f64_retain()
10. **[Phase 02-data-loading]: f64 for UNNEST batch inserts** — rust_decimal doesn't implement sqlx array traits, so we use f64 arrays and let PostgreSQL cast to DECIMAL(8,4)
11. **[Phase 02-data-loading]: Fire-and-forget background database persistence** — Database writes happen in tokio::spawn after returning chunk, ensuring low latency
12. **[Phase 02-04]: Spawn pre-fetching in dedicated task to avoid Send bound issues** — Recursive async calls create Send bound problems; internal task spawning solves this
13. **[Phase 02-04]: AtomicU64 with Relaxed ordering for statistics** — Sufficient for monitoring, better performance than strict ordering
14. **[Phase 03-02]: Made database modules public for query access** — Required for query modules to access DatabasePool and schema constants
15. **[Phase 03-02]: Used gap-and-island pattern for VoC aggregation** — CTE with window function efficiently aggregates contiguous VoC periods
16. **[Phase 03-03]: Dynamic SQL construction for optional filters** — Used String-based query building instead of query_as! macro to handle variable WHERE clauses for aspect types and body pairs
17. **[Phase 03-03]: Runtime retrograde status calculation** — Calculate status (Direct, Retrograde, PreShadow, PostShadow) at query time based on date range overlap rather than storing status in database
18. **[Phase 04-02]: Used sysinfo 0.30 for cross-platform memory monitoring** — Process memory tracking with configurable soft/hard limits for cache eviction decisions
19. **[Phase 04-05]: Integrated MemoryMonitor into ChunkManager** — Memory-aware cache eviction with 25% eviction at High pressure, 50% at Critical
20. **[Phase 04-03]: Used 8° orb for major aspect filtering** — Matches standard astrological conventions while reducing storage by ~80%
21. **[Phase 04-03]: Shortest-path interpolation for longitude** — Handles 360° wraparound correctly (e.g., 350° to 10° goes forward through 360°)

## Notes

- This is a brownfield project with existing chart generation capabilities
- Electoral astrology features are the new development focus
- Database connection string in `.env` as `PG_URL`
- Implementation plan already documented in `implementation_plan.md`
- Phase 1 complete: All 6 database requirements (DB-01 to DB-06) satisfied
- Phase 2 complete: 4 of 4 plans complete (LOAD-01 through LOAD-05 satisfied)
- Phase 3 complete: 5 of 5 requirements complete (QUERY-01 through QUERY-05)
  - Wedding query with aspect_summaries JOIN for 51× performance
  - VoC query with gap-and-island pattern for period aggregation
  - Retrograde query with status calculation (Direct, Retrograde, PreShadow, PostShadow)
  - Exact aspect query with orb/type/body pair filtering
  - Performance benchmarks verifying <100ms for 60-day ranges
- Phase 4 in progress: 3 of 4 requirements complete (PERF-02, PERF-03)
  - Memory monitoring with sysinfo crate
  - Configurable soft/hard limits (30MB/50MB)
  - Memory pressure detection for cache eviction
  - Memory-aware eviction integrated into ChunkManager
  - Interpolation module with 360° wraparound handling
  - Major aspect filtering for ~80% storage reduction

---

*State tracking started: 2026-02-24*
*Phase 1 completed: 2026-02-25*
*Phase 2 completed: 2026-02-25*
*Phase 3 completed: 2026-02-25*
*Phase 4 started: 2026-03-01*
