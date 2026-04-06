---
id: S02
parent: M001
milestone: M001
provides:
  - ChunkKey for LRU cache lookups by date
  - CompactPlanetPosition (~16 bytes vs ~72 bytes, ~4.5× size reduction)
  - CompactAspect for memory-efficient aspect storage
  - CompactLunarCondition for lunar data storage
  - ChunkData container for all daily astrological data
  - lru crate dependency for cache implementation
  - ChunkManager with LRU cache and database loading
  - ChunkManagerError for error handling
  - Cache-first lookup with database fallback
  - Thread-safe cache access via RwLock
  - ChunkGenerator for Swiss Ephemeris integration
  - Batch database persistence with UNNEST
  - Aspect calculation for 5 major aspects
  - Lunar condition calculation with moon phase and VoC detection
  - Three-tier lookup: cache → database → generation
  - Background pre-fetching of adjacent chunks (±1 day)
  - Configurable pre-fetching behavior via ChunkManagerConfig
  - Performance statistics tracking via ChunkManagerStats
  - Batch chunk loading via get_chunk_range method
requires: []
affects: []
key_files: []
key_decisions:
  - Used lru crate instead of custom implementation (per CONTEXT.md discretion) for correctness and time savings
  - Packed struct representation with #[repr(C, packed)] for optimal memory layout
  - Integer encoding: millidegrees for angles (0.001° precision), permille for illumination (0.001 precision)
  - Flat Vec storage in ChunkData (not HashMap) for cache efficiency
  - Body ID ordering in CompactAspect (body1_id < body2_id) to avoid duplicate storage
  - Used manual row mapping with f64 conversion instead of query_as due to rust_decimal/sqlx compatibility
  - Added sqlx chrono feature for DateTime support
  - Added bigdecimal dependency for future decimal support if needed
  - Used f64 instead of Decimal for UNNEST arrays since rust_decimal doesn't implement sqlx array traits
  - Background database persistence is fire-and-forget (doesn't block chunk return)
  - Database write failures don't fail chunk load (data is still in cache)
  - VoC detection uses simplified 2° orb check (full implementation would track until sign change)
  - Spawn pre-fetching in dedicated task to avoid Send bound issues with recursive async calls
  - Use AtomicU64 with Ordering::Relaxed for statistics - sufficient for monitoring, no need for strict ordering
  - Silent pre-fetch failures - pre-fetching is best-effort and shouldn't affect main query flow
  - Configurable pre-fetch range allows future extension to multi-day lookahead
patterns_established:
  - Compact representation: Use integer encoding (millidegrees, permille) instead of Decimal/f64 for memory efficiency
  - Conversion pattern: from_schema() methods with proper error handling using thiserror
  - Size verification: Compile-time tests to ensure memory targets are met
  - Module organization: Database types in schema.rs, compact cache types in chunk.rs
  - ChunkGenerator: Swiss Ephemeris integration with error handling
  - Batch inserts: UNNEST with f64 arrays for PostgreSQL compatibility
  - Fallback chain: Cache miss → Database miss → Generate → Background save
  - Background task spawning: Use tokio::spawn for fire-and-forget operations
  - Stats tracking: Atomic counters with snapshot pattern for thread-safe metrics
  - Config-driven behavior: Struct with Default impl for easy customization
observability_surfaces: []
drill_down_paths: []
duration: 4min
verification_result: passed
completed_at: 2026-02-25
blocker_discovered: false
---
# S02: Data Loading

**# Phase 02 Plan 01: Compact Chunk Data Structures Summary**

## What Happened

# Phase 02 Plan 01: Compact Chunk Data Structures Summary

**Memory-efficient chunk data structures with ~4.5× size reduction through packed representations and integer encoding, establishing the foundation for 30MB LRU cache target.**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-25T15:05:09Z
- **Completed:** 2026-02-25T15:07:55Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments

- Added lru crate dependency for battle-tested LRU cache implementation
- Created ChunkKey struct implementing Hash, Eq, Clone, Copy for cache key usage
- Implemented CompactPlanetPosition (~16 bytes vs ~72 bytes schema type)
- Implemented CompactAspect for memory-efficient aspect storage
- Implemented CompactLunarCondition for lunar condition storage
- Created ChunkData container aggregating all daily astrological data
- Added ChunkError type with thiserror for conversion failures
- Implemented from_schema() conversion methods for all compact types
- Added compile-time size verification tests

## Task Commits

Each task was committed atomically:

1. **Task 1: Add lru crate dependency** - `539b3d9` (chore)
2. **Task 2: Create compact chunk data structures** - `851f2a4` (feat)
3. **Task 3: Export chunk module** - `8c4a823` (feat)

**Plan metadata:** [to be committed]

## Files Created/Modified

- `src/database/chunk.rs` - All compact data structures (ChunkKey, CompactPlanetPosition, CompactAspect, CompactLunarCondition, ChunkData, ChunkError)
- `Cargo.toml` - Added `lru = "0.12"` dependency
- `src/database/mod.rs` - Exported chunk module and public types

## Decisions Made

1. **Used lru crate instead of custom implementation** (per CONTEXT.md "Claude's discretion") - Saves development time and ensures correctness with O(1) operations
2. **Packed struct representation** - `#[repr(C, packed)]` minimizes memory footprint
3. **Integer encoding strategy** - Millidegrees (0.001° precision) for angles, permille (0.001 precision) for illumination
4. **Flat Vec storage** - Not HashMap for cache efficiency and predictable memory layout
5. **Ordered body IDs in aspects** - body1_id < body2_id prevents duplicate storage

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added missing ToPrimitive trait import**
- **Found during:** Task 3 (Export chunk module)
- **Issue:** Decimal::to_i32(), to_i16(), to_u16() methods not available - ToPrimitive trait not in scope
- **Fix:** Added `use rust_decimal::prelude::ToPrimitive;` to chunk.rs imports
- **Files modified:** src/database/chunk.rs
- **Verification:** cargo check --features db passes
- **Committed in:** 8c4a823 (Task 3 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Minor import fix, no scope creep or architectural changes

## Issues Encountered

None - all compilation errors were expected trait resolution issues fixed immediately.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- ✓ Chunk data structures ready for ChunkManager implementation (02-02)
- ✓ Compact types provide ~4.5× memory reduction for 30MB cache target
- ✓ Conversion methods ready for Swiss Ephemeris integration (02-03)
- ✓ LRU cache dependency ready for ChunkManager

Ready for Plan 02-02: ChunkManager with LRU Cache

---

*Phase: 02-data-loading*
*Completed: 2026-02-25*

# Phase 02 Plan 02: ChunkManager with LRU Cache Summary

**ChunkManager with 30-chunk LRU cache, thread-safe RwLock access, and database fallback loading from three hypertables**

## Performance

- **Duration:** 7 min
- **Started:** 2026-02-25T15:10:06Z
- **Completed:** 2026-02-25T15:17:19Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- ChunkManager struct with LRU cache (30-chunk capacity) and database pool
- Thread-safe cache access using tokio::sync::RwLock
- Cache-first lookup: check cache → load from database on miss → populate cache
- Database loading from all three hypertables (planet_positions, aspects, lunar_conditions)
- ChunkManagerError enum with thiserror for comprehensive error handling
- Cache management methods: cache_stats, clear_cache, is_cached, cache_size, peek_cached

## Task Commits

Each task was committed atomically:

1. **Task 1: Create ChunkManager with LRU cache** - `f818427` (feat)
2. **Task 2: Export ChunkManager from database module** - `d7cad55` (feat)

**Plan metadata:** TBD (docs: complete plan)

## Files Created/Modified

- `src/database/chunk_manager.rs` - ChunkManager with LRU cache and database loading (215 lines)
- `src/database/mod.rs` - Added chunk_manager module and exports
- `Cargo.toml` - Added sqlx chrono feature and bigdecimal dependency

## Decisions Made

1. **Manual row mapping with f64 conversion** - rust_decimal doesn't implement sqlx's Decode/Type traits, so we query as f64 and convert to Decimal using Decimal::from_f64_retain()
2. **sqlx chrono feature** - Required for DateTime<Utc> support in database queries
3. **tokio::sync::RwLock** - Async-aware RwLock for non-blocking cache access

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed rust_decimal/sqlx compatibility**
- **Found during:** Task 1 (ChunkManager implementation)
- **Issue:** rust_decimal::Decimal doesn't implement sqlx::Decode or sqlx::Type traits, preventing use with query_as
- **Fix:** Used sqlx::query with manual row mapping, extracting f64 values and converting to Decimal using Decimal::from_f64_retain()
- **Files modified:** src/database/chunk_manager.rs, Cargo.toml
- **Verification:** cargo check --features db passes, cargo test --features db passes
- **Committed in:** f818427 (Task 1 commit)

**2. [Rule 3 - Blocking] Added missing sqlx chrono feature**
- **Found during:** Task 1 (ChunkManager implementation)
- **Issue:** chrono::DateTime<Utc> couldn't be decoded from PostgreSQL timestamps
- **Fix:** Added "chrono" feature to sqlx dependency in Cargo.toml
- **Files modified:** Cargo.toml
- **Verification:** cargo check --features db passes
- **Committed in:** f818427 (Task 1 commit)

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both fixes were necessary for database compatibility. No scope creep.

## Issues Encountered

None beyond the rust_decimal/sqlx compatibility issue which was resolved via f64 conversion.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- ChunkManager foundation complete, ready for Swiss Ephemeris integration
- Cache can be populated with calculated data from ephemeris
- Ready for Plan 02-03: Swiss Ephemeris Integration

---
*Phase: 02-data-loading*
*Completed: 2026-02-25*

# Phase 02 Plan 03: Swiss Ephemeris Integration Summary

**ChunkGenerator with Swiss Ephemeris integration, batch database persistence using UNNEST, and three-tier lookup with background saving.**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-25T15:20:21Z
- **Completed:** 2026-02-25T15:26:10Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments

- Created ChunkGenerator with Swiss Ephemeris integration for all 10 celestial bodies
- Implemented aspect calculation for 5 major aspects (conjunction, sextile, square, trine, opposition) with 10° orb
- Added lunar condition calculation including moon phase and void-of-course detection
- Built batch database persistence using PostgreSQL UNNEST for high-performance inserts
- Integrated ChunkGenerator into ChunkManager with three-tier lookup (cache → database → generation)
- Background database persistence using tokio::spawn (fire-and-forget pattern)

## Task Commits

Each task was committed atomically:

1. **Task 1: Create ChunkGenerator with Swiss Ephemeris integration** - `1c662ba` (feat)
2. **Task 2 & 3: Batch persistence and ChunkManager integration** - `ce45d77` (feat)

**Plan metadata:** [pending final commit]

## Files Created/Modified

- `src/database/chunk_generator.rs` - New ChunkGenerator with Swiss Ephemeris integration, aspect calculation, lunar conditions, and batch persistence (510 lines)
- `src/database/chunk_manager.rs` - Added ChunkGenerator field, updated get_chunk with generation fallback, added Generation error variant
- `src/database/mod.rs` - Exported chunk_generator module and types

## Decisions Made

1. **f64 for UNNEST arrays** - rust_decimal::Decimal doesn't implement sqlx traits for array binding, so we use f64 and let PostgreSQL cast to DECIMAL(8,4)
2. **Fire-and-forget background saves** - Database persistence happens in tokio::spawn after returning chunk to caller, ensuring low latency
3. **Database failures don't fail loads** - If background save fails, chunk is still in cache and can be used
4. **Simplified VoC detection** - Uses 2° orb check against major planets; full implementation would track until Moon leaves sign

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

**rust_decimal doesn't implement sqlx array traits** - When implementing batch inserts with UNNEST, discovered that `rust_decimal::Decimal` doesn't implement `sqlx::Encode` or `PgHasArrayType` for use in arrays. Fixed by using `f64` for the array values and binding to `float8[]`, letting PostgreSQL implicitly cast to `DECIMAL(8,4)`.

## Next Phase Readiness

- ChunkGenerator is ready for use in background pre-fetching (Plan 02-04)
- Three-tier lookup enables seamless data access regardless of database state
- Batch persistence ensures generated data is efficiently stored for future queries
- All requirements LOAD-03 and LOAD-04 satisfied

## Self-Check: PASSED

- [x] src/database/chunk_generator.rs exists (510 lines)
- [x] Task 1 commit 1c662ba exists (ChunkGenerator creation)
- [x] Task 2/3 commit ce45d77 exists (Batch persistence and ChunkManager integration)
- [x] Final commit 3b74911 exists (SUMMARY.md and metadata)
- [x] cargo check --features db passes with 0 errors
- [x] cargo test --features db --lib passes (60 tests)

---
*Phase: 02-data-loading*
*Completed: 2026-02-25*

# Phase 2 Plan 4: Background Pre-fetching Summary

**Background pre-fetching of adjacent day chunks with configurable behavior and performance statistics tracking**

## Performance

- **Duration:** 4 min
- **Started:** 2026-02-25T15:29:11Z
- **Completed:** 2026-02-25T15:33:25Z
- **Tasks:** 3
- **Files modified:** 1

## Accomplishments

- Implemented `pre_fetch_adjacent_chunks` method that loads ±1 day chunks in background
- Added `ChunkManagerConfig` with `enable_pre_fetching` and `pre_fetch_range` options
- Added `ChunkManagerStats` with atomic counters for cache hits, misses, DB loads, generations, and pre-fetches
- Added `get_chunk_range` method for efficient multi-day batch loading
- Pre-fetching triggers after every chunk load (cache hit, DB load, or generation)
- Pre-fetch failures are silent (best-effort, doesn't affect main query flow)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add pre-fetching method to ChunkManager** - `187df92` (feat)
2. **Task 2: Trigger pre-fetching after chunk load** - `2c727cb` (feat)
3. **Task 3: Add pre-fetching statistics and configuration** - (included in Task 1)

**Plan metadata:** `TBD` (docs: complete plan)

## Files Created/Modified

- `src/database/chunk_manager.rs` - Added pre-fetching, config, stats, and get_chunk_range

## Decisions Made

1. **Spawn pre-fetching in dedicated task** - The recursive nature of `pre_fetch_adjacent_chunks` calling `get_chunk` creates Send bound issues when spawned directly. By having `pre_fetch_adjacent_chunks` spawn its own internal task, we avoid these lifetime issues.

2. **AtomicU64 with Relaxed ordering** - Statistics are for monitoring only, so strict memory ordering isn't required. This provides better performance than SeqCst.

3. **Silent pre-fetch failures** - Pre-fetching is purely an optimization. If it fails (e.g., database unavailable), the main query should still succeed. Only successful pre-fetches are logged at debug level.

4. **Configurable pre-fetch range** - While currently hardcoded to 1 day, the config supports arbitrary ranges for future extension.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed Send bound issues with recursive async calls**
- **Found during:** Task 2
- **Issue:** `tokio::spawn` requires Send futures, but `pre_fetch_adjacent_chunks` calling `get_chunk` created non-Send futures due to RwLock guards
- **Fix:** Restructured `pre_fetch_adjacent_chunks` to spawn its own internal task, avoiding the Send requirement on the outer future
- **Files modified:** src/database/chunk_manager.rs
- **Verification:** `cargo check --features db` passes
- **Committed in:** 2c727cb (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Minor structural change to avoid Rust async lifetime issues. No behavioral changes.

## Issues Encountered

- **Send bound compilation error** - The original plan's approach of spawning `pre_fetch_adjacent_chunks` directly caused Send bound issues because the recursive call to `get_chunk` held non-Send types across await points. Fixed by restructuring the method to spawn internally.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 2 (Data Loading) is now complete
- All 5 requirements (LOAD-01 through LOAD-05) are satisfied
- Ready for Phase 3: Query System

---
*Phase: 02-data-loading*
*Completed: 2026-02-25*
