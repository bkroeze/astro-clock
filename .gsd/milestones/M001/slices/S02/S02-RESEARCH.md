# Phase 2: Data Loading - Research

**Research Date:** 2026-02-25  
**Status:** Complete — Ready for Planning  
**Phase:** 02-data-loading  

---

## What I Need to Know to Plan This Phase Well

### 1. Requirements to Address (LOAD-01 through LOAD-05)

| Requirement | Description | Key Decisions Needed |
|-------------|-------------|---------------------|
| **LOAD-01** | Implement ChunkManager with LRU cache | Cache size (30 chunks), eviction strategy, thread-safety |
| **LOAD-02** | Load data from database when available | Query patterns, batch loading, connection pooling |
| **LOAD-03** | Generate data from Swiss Ephemeris when not in database | Chunk generation algorithm, aspect calculation during generation |
| **LOAD-04** | Save generated data to database for future queries | Batch insert strategy, transaction handling, error recovery |
| **LOAD-05** | Background pre-fetching of adjacent chunks | Tokio task spawning, fire-and-forget pattern, failure handling |

### 2. Technical Context

#### Existing Infrastructure
- **Database Pool:** `src/database/pool.rs` — sqlx PgPool with 5 max connections
- **Schema Types:** `src/database/schema.rs` — All 6 table types defined with FromRow derives
- **Swiss Ephemeris:** `src/swiss_eph_impl.rs` — SwissEphChartCalculator with planet/house calculation
- **Ephemeris Utils:** `src/ephemeris/mod.rs` — Julian day conversions, DateTime struct
- **Connection String:** `PG_URL` environment variable

#### Tech Stack Decisions
- **Async Runtime:** Tokio with full features
- **Database:** PostgreSQL with TimescaleDB (hypertables with 1-day chunks)
- **ORM:** sqlx 0.8 with compile-time query checking
- **LRU Cache:** Decision needed — `lru` crate vs custom implementation
- **Swiss Ephemeris:** `swiss-eph` 0.2.1 crate already in Cargo.toml

### 3. Domain Knowledge Required

#### Chunk Data Structure
```rust
// From implementation_plan.md
pub const CHUNK_SIZE_DAYS: i64 = 1;
pub const CACHE_SIZE_CHUNKS: usize = 30; // ~1 month in memory, ~30MB target

pub struct ChunkKey {
    pub date: NaiveDate,
    // All bodies in one chunk (not separate per body)
}

pub struct ChunkData {
    pub date: NaiveDate,
    pub planet_positions: Vec<PlanetPosition>, // All bodies, 1-minute resolution
    pub aspects: Vec<Aspect>,                  // Calculated during generation
    pub lunar_conditions: Vec<LunarCondition>, // Moon phase, VoC status
    pub loaded_at: Instant,
}
```

#### Data Resolution per Chunk (1 day)
- **Moon:** 1,440 records (1-minute intervals) — most frequently queried
- **Other planets:** 1,440 records each (1-minute for Phase 2, coarser in Phase 4)
- **Aspects:** Calculated during generation, stored if within orb threshold
- **Lunar conditions:** 1,440 records (1-minute intervals)

#### Cache Lookup Order
1. **LRU Cache** — O(1) lookup, 30-day capacity
2. **Database** — Query TimescaleDB hypertable
3. **Swiss Ephemeris** — Generate on-demand, then persist

### 4. Memory Requirements

| Metric | Target | Calculation |
|--------|--------|-------------|
| Cache capacity | 30 chunks (days) | Configurable |
| Memory per day | ~1MB | Compact data structures |
| Total cache memory | <30MB | 30 days × 1MB |
| Compact position size | ~16 bytes | vs ~72 bytes full struct |

**Compact Data Structure (from implementation_plan.md):**
```rust
#[repr(C, packed)]
pub struct CompactPlanetPosition {
    pub timestamp_minutes: u32,      // Minutes since chunk start (0-1439)
    pub longitude_millidegrees: i32, // 0-360000 (0.001° precision)
    pub speed_millidegrees: i16,     // Daily motion
    pub zodiac_sign: u8,             // 0-11
    pub is_retrograde: bool,         // 1 byte
}
// Total: ~16 bytes vs ~72 bytes = 4.5× reduction
```

### 5. Swiss Ephemeris Integration

#### Generation Algorithm
```rust
// For each minute in the day (0..1440):
// 1. Calculate Julian day for timestamp
// 2. For each body (0-9):
//    - Call swiss_eph::safe::calc(jd, planet, flags)
//    - Extract longitude, latitude, speed, retrograde
// 3. Calculate aspects between all body pairs
// 4. Calculate lunar conditions (phase, VoC)
```

#### Aspect Calculation During Generation
- Calculate all body pair combinations (45 pairs for 10 bodies)
- Check orb against major aspects (0°, 60°, 90°, 120°, 180°)
- Store if orb < threshold (default 10°)
- Mark as applying if orb decreasing

#### Existing Swiss Ephemeris Usage
```rust
// From src/swiss_eph_impl.rs
swiss_eph::safe::calc(julian_day, planet, flags)
    -> Result<CalcResult, SwissEphError>

// Flags needed:
// - swiss_eph::safe::CalcFlags::new().with_speed()
```

### 6. Database Operations

#### Load from Database
```sql
-- Planet positions for a day
SELECT * FROM planet_positions 
WHERE time >= $1 AND time < $2 
ORDER BY time, body_id;

-- Aspects for a day
SELECT * FROM aspects 
WHERE time >= $1 AND time < $2;

-- Lunar conditions for a day
SELECT * FROM lunar_conditions 
WHERE time >= $1 AND time < $2;
```

#### Batch Insert (after generation)
```rust
// Use sqlx query builder or COPY for performance
// Target: Insert 14,400 planet positions + aspects + lunar conditions
// Transaction per chunk for atomicity
```

### 7. Pre-fetching Strategy

#### Background Task Pattern
```rust
// From implementation_plan.md
let prefetch_chunks = self.get_adjacent_chunks(&chunks);
tokio::spawn(async move {
    for chunk in prefetch_chunks {
        let _ = self.load_chunk(&chunk).await; // Fire-and-forget
    }
});
```

#### Pre-fetch Rules
- **Scope:** Only immediate neighbors (1 day before, 1 day after)
- **Behavior:** Fire-and-forget, don't wait for completion
- **Failure handling:** Silent — don't affect main query flow
- **Trigger:** After loading requested chunks

### 8. Error Handling Strategy

| Error Type | Behavior | Rationale |
|------------|----------|-----------|
| Database load error | Log warning, fall back to Swiss Ephemeris | Graceful degradation |
| Swiss Ephemeris error | Return error to caller | Can't proceed without data |
| Cache error | Non-fatal, log and continue | Cache is optimization |
| Background pre-fetch error | Silent, no logging | Don't spam logs |
| Database write error | Log warning, continue | Data still in cache |

### 9. LRU Cache Implementation Options

#### Option A: `lru` crate (recommended)
```rust
use lru::LruCache;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ChunkManager {
    cache: RwLock<LruCache<ChunkKey, Arc<ChunkData>>>,
    // ...
}
```
- **Pros:** Battle-tested, O(1) operations, configurable capacity
- **Cons:** Additional dependency

#### Option B: Custom LRU
- **Pros:** No extra dependency, tailored to needs
- **Cons:** More code to write and test

**Decision:** Use `lru` crate (approved in context as "Claude's discretion")

### 10. Thread Safety Considerations

```rust
pub struct ChunkManager {
    // Thread-safe cache access
    cache: RwLock<LruCache<ChunkKey, Arc<ChunkData>>>,
    
    // Database pool is already Arc internally
    db_pool: DatabasePool,
    
    // Swiss Ephemeris needs protection (not Send/Sync)
    // Option 1: Create new ephemeris instance per generation
    // Option 2: Use thread-local storage
}
```

### 11. Key Design Decisions (from Context)

#### Cache Eviction
- **Strategy:** LRU (Least Recently Used)
- **No time-based expiration** — chunks stay until evicted
- **Capacity:** 30 chunks, eviction only at capacity

#### Chunk Granularity
- **Size:** 1 day (aligned with TimescaleDB chunk boundaries)
- **Contents:** All bodies in single chunk (not per-body chunks)
- **Resolution:** Full 1-minute for all bodies (multi-resolution deferred to Phase 4)

#### Data Persistence
- **When:** Immediately after Swiss Ephemeris generation
- **How:** Batch inserts for performance
- **Failure:** Non-fatal — data still available in cache

### 12. Dependencies on Other Phases

| Item | Depends On | Impact |
|------|-----------|--------|
| Database schema | Phase 1 ✓ Complete | Tables and indexes ready |
| Query functions | Phase 3 (Query System) | ChunkManager API must support query needs |
| Multi-resolution | Phase 4 (Performance) | Chunk structure supports, implemented later |

### 13. Open Questions for Planning

1. **LRU Implementation:**
   - Use `lru` crate or custom?
   - RwLock or Mutex for cache access?
   - Pre-allocate chunk data capacity?

2. **Swiss Ephemeris Thread Safety:**
   - Create new ephemeris per generation task?
   - Use thread-local storage?
   - Is swiss-eph crate Send/Sync?

3. **Batch Insert Strategy:**
   - sqlx query builder with UNNEST?
   - COPY protocol for bulk insert?
   - Transaction per table or per chunk?

4. **Chunk Data Layout:**
   - Vec<CompactPlanetPosition> or HashMap<body_id, Vec<...>>?
   - Separate arrays per body for cache locality?
   - How to handle aspect storage efficiently?

5. **Error Types:**
   - Custom error enum for ChunkManager?
   - Use anyhow for simplicity?
   - How to propagate Swiss Ephemeris errors?

6. **Configuration:**
   - Cache size configurable via config.ron?
   - Pre-fetching enabled/disabled flag?
   - Chunk generation parallelization?

### 14. Success Criteria Verification

From REQUIREMENTS.md:
- ✓ **LOAD-01:** ChunkManager with LRU cache — **PLAN**
- ✓ **LOAD-02:** Load from database when available — **PLAN**
- ✓ **LOAD-03:** Generate from Swiss Ephemeris when needed — **PLAN**
- ✓ **LOAD-04:** Save generated data to database — **PLAN**
- ✓ **LOAD-05:** Background pre-fetching — **PLAN**

From implementation_plan.md:
- ✓ 30MB memory target for 30-day cache — **PLAN** (compact data structures)
- ✓ 51× query speedup (2.3s → 45ms) — **DEFER to Phase 3** (query optimization)

---

## Research Summary

**What I need to know to plan well:**

1. **LRU cache implementation** — Use `lru` crate with RwLock for thread-safe access
2. **Swiss Ephemeris thread safety** — Need to verify if crate is Send/Sync or create instances per task
3. **Batch insert SQL** — sqlx UNNEST pattern or COPY protocol for 14K+ rows per chunk
4. **Chunk data structure layout** — Vec vs HashMap, compact representations for memory efficiency
5. **Tokio task spawning** — Fire-and-forget pre-fetch pattern with proper error isolation
6. **Error handling strategy** — Graceful degradation from DB → Ephemeris, silent pre-fetch failures

**Next Step:** Create detailed implementation plan with ChunkManager struct, cache implementation, generation algorithm, and database operations.

---

*Research complete: Ready for planning phase*