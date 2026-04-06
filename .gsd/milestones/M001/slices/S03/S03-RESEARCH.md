# Phase 3: Query System - Research

**Research Date:** 2026-02-25  
**Status:** Complete — Ready for Planning  
**Phase:** 03-query-system  

---

## What I Need to Know to Plan This Phase Well

### 1. Requirements to Address (QUERY-01 through QUERY-05)

| Requirement | Description | Key Decisions Needed |
|-------------|-------------|---------------------|
| **QUERY-01** | Find optimal wedding dates (Venus aspects, Moon sign, no VoC) | SQL query structure, CTE design, aspect counting strategy |
| **QUERY-02** | Find void-of-course Moon periods | VoC detection algorithm, period aggregation, duration filtering |
| **QUERY-03** | Find planetary retrograde periods | Retrograde status calculation, shadow period handling |
| **QUERY-04** | Find exact aspects within date range | Orb threshold filtering, aspect type selection, body pair filtering |
| **QUERY-05** | Query completes in <100ms for 60-day ranges (cached) | aspect_summaries usage, ChunkManager integration, indexing strategy |

### 2. Technical Context

#### Existing Infrastructure (from Phase 2)
- **ChunkManager:** `src/database/chunk_manager.rs` — LRU cache (30 chunks), three-tier lookup (cache → DB → generation)
- **ChunkData:** `src/database/chunk.rs` — Compact data structures (~16 bytes/position), 1-minute resolution
- **Database Pool:** `src/database/pool.rs` — sqlx PgPool with 5 max connections
- **Schema Types:** `src/database/schema.rs` — All table types with FromRow derives, domain constants (body_ids, aspect_types, zodiac_signs)

#### aspect_summaries Table (Key Performance Enabler)
```sql
-- From migrations/004_create_aspect_summaries.sql
CREATE TABLE aspect_summaries (
    time TIMESTAMPTZ NOT NULL,
    body_id SMALLINT NOT NULL,
    conjunctions SMALLINT DEFAULT 0,
    sextiles SMALLINT DEFAULT 0,
    squares SMALLINT DEFAULT 0,
    trines SMALLINT DEFAULT 0,
    oppositions SMALLINT DEFAULT 0,
    total_favorable SMALLINT DEFAULT 0,  -- trines + sextiles
    total_challenging SMALLINT DEFAULT 0, -- squares + oppositions
    PRIMARY KEY (time, body_id)
);
```

**Performance Impact:** 51× faster wedding queries (2.3s → 45ms) by eliminating correlated subqueries

#### Database Schema Constants (from schema.rs)
```rust
// Body IDs
pub const SUN: i16 = 0;      pub const MOON: i16 = 1;
pub const MERCURY: i16 = 2;  pub const VENUS: i16 = 3;
pub const MARS: i16 = 4;     pub const JUPITER: i16 = 5;
pub const SATURN: i16 = 6;   pub const URANUS: i16 = 7;
pub const NEPTUNE: i16 = 8;  pub const PLUTO: i16 = 9;

// Aspect Types
pub const CONJUNCTION: i16 = 0;  pub const SEXTILE: i16 = 1;
pub const SQUARE: i16 = 2;       pub const TRINE: i16 = 3;
pub const OPPOSITION: i16 = 4;

// Zodiac Signs
pub const ARIES: i16 = 0;        pub const TAURUS: i16 = 1;
pub const GEMINI: i16 = 2;       pub const CANCER: i16 = 3;
pub const LEO: i16 = 4;          pub const VIRGO: i16 = 5;
pub const LIBRA: i16 = 6;        pub const SCORPIO: i16 = 7;
pub const SAGITTARIUS: i16 = 8;  pub const CAPRICORN: i16 = 9;
pub const AQUARIUS: i16 = 10;    pub const PISCES: i16 = 11;
```

### 3. Domain Knowledge Required

#### Wedding Date Query (QUERY-01)

**Favorable Moon Signs:** Taurus(1), Cancer(3), Leo(4), Libra(6), Scorpio(7), Capricorn(9), Aquarius(10), Pisces(11)

**Optimized SQL Pattern (from implementation_plan.md):**
```sql
SELECT 
    pp.time,
    pp.zodiac_sign as moon_sign,
    asum.total_favorable as favorable_aspects
FROM planet_positions pp
LEFT JOIN aspect_summaries asum 
    ON pp.time = asum.time 
    AND asum.body_id = 3 -- Venus
WHERE pp.body_id = 1 -- Moon
  AND pp.time BETWEEN $1 AND $2
  AND pp.zodiac_sign IN (1, 3, 4, 6, 7, 9, 10, 11)
  AND NOT EXISTS (
      SELECT 1 FROM lunar_conditions lc 
      WHERE lc.time = pp.time AND lc.is_void_of_course = true
  )
ORDER BY asum.total_favorable DESC NULLS LAST, pp.time
LIMIT $3;
```

**Key Insight:** Use aspect_summaries JOIN instead of counting aspects in subquery

#### Void-of-Course Moon Query (QUERY-02)

**VoC Detection:** Uses pre-calculated `is_void_of_course` flag from lunar_conditions table

**Query Pattern:**
```sql
-- Find VoC periods (contiguous times where is_void_of_course = true)
WITH voc_periods AS (
    SELECT 
        time,
        moon_sign,
        is_void_of_course,
        -- Gap detection logic for period aggregation
        LAG(is_void_of_course) OVER (ORDER BY time) as prev_voc
    FROM lunar_conditions
    WHERE time BETWEEN $1 AND $2
)
SELECT 
    MIN(time) as start_time,
    MAX(time) as end_time,
    MAX(time) - MIN(time) as duration,
    moon_sign
FROM voc_periods
WHERE is_void_of_course = true
GROUP BY -- Group contiguous periods
```

**Alternative:** Query already has `voc_start` and `voc_end` columns — can query directly for periods

#### Retrograde Period Query (QUERY-03)

**Data Source:** `retrograde_periods` table (from Phase 1 schema)

**Query Requirements:**
- Filter by planet(s)
- Filter by date range
- Calculate current status (in retrograde, in shadow, direct)
- Include shadow periods if available

**Status Calculation:**
```rust
pub enum RetrogradeStatus {
    Direct,      // Moving forward, not in shadow
    Retrograde,  // Moving backward
    PreShadow,   // In shadow period before retrograde
    PostShadow,  // In shadow period after retrograde
}
```

#### Exact Aspect Query (QUERY-04)

**Parameters:**
- Date range
- Orb threshold (default: 1°)
- Aspect types filter (conjunction, opposition, trine, square, sextile)
- Body pairs filter (e.g., Sun-Moon, Venus-Mars)

**Query Pattern:**
```sql
SELECT 
    time,
    body1_id,
    body2_id,
    aspect_type,
    orb,
    applying
FROM aspects
WHERE time BETWEEN $1 AND $2
  AND orb <= $3  -- Orb threshold
  AND aspect_type = ANY($4)  -- Filter by aspect types
  AND (body1_id, body2_id) IN ($5)  -- Filter by body pairs
ORDER BY time, orb;
```

### 4. Query Interface Design

#### Result Types (from CONTEXT.md decisions)
```rust
// Each query returns structured results with metadata
pub struct QueryResult<T> {
    pub data: Vec<T>,
    pub execution_time_ms: u64,
    pub rows_examined: usize,
    pub cache_hit: bool,
}

// Wedding date candidate
pub struct WeddingCandidate {
    pub datetime: DateTime<Utc>,
    pub moon_sign: ZodiacSign,
    pub venus_favorable_aspects: i16,
}

// Void-of-course period
pub struct VoCPeriod {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub duration: Duration,
    pub moon_sign: ZodiacSign,
}

// Retrograde period
pub struct RetrogradePeriod {
    pub planet: Body,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub shadow_start: Option<DateTime<Utc>>,
    pub shadow_end: Option<DateTime<Utc>>,
    pub status: RetrogradeStatus,
}

// Exact aspect
pub struct ExactAspect {
    pub datetime: DateTime<Utc>,
    pub body1: Body,
    pub body2: Body,
    pub aspect_type: AspectType,
    pub orb: Decimal,
    pub applying: bool,
}
```

#### Criteria Structs
```rust
pub struct WeddingCriteria {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub min_venus_aspects: i16,  // Default: 2
    pub limit: usize,            // Default: 10
}

pub struct VoCCriteria {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub min_duration: Option<Duration>,
}

pub struct RetrogradeCriteria {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub planets: Option<Vec<Body>>,  // None = all planets
}

pub struct AspectCriteria {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub orb_threshold: Decimal,      // Default: 1.0
    pub aspect_types: Option<Vec<AspectType>>,  // None = all
    pub body_pairs: Option<Vec<(Body, Body)>>,  // None = all pairs
}
```

### 5. Performance Strategy

#### <100ms Target (QUERY-05)

**Key Techniques:**
1. **aspect_summaries JOIN** — Eliminates correlated subqueries (51× speedup)
2. **ChunkManager cache** — In-memory data avoids database round-trips
3. **Proper indexing** — Composite indexes on (time, body_id)
4. **Query result limits** — Always use LIMIT for top-N queries

**Performance Budget:**
| Operation | Target Time |
|-----------|-------------|
| Cache hit | <5ms |
| Database query (cached chunks) | <50ms |
| aspect_summaries JOIN | <20ms |
| Result processing | <25ms |
| **Total** | **<100ms** |

#### ChunkManager Integration

**Query Flow:**
```rust
// 1. Load chunks for date range via ChunkManager
let chunks = chunk_manager.get_chunk_range(start, end).await?;

// 2. For in-memory queries, iterate chunk data directly
// 3. For complex queries, use database with aspect_summaries
// 4. Cache results for repeated queries
```

**Decision Point:** Which queries use ChunkManager vs direct SQL?
- **Wedding dates:** SQL with aspect_summaries (requires aggregation)
- **VoC periods:** SQL with lunar_conditions (period detection)
- **Retrograde:** SQL with retrograde_periods (range query)
- **Exact aspects:** Could use ChunkManager for small ranges, SQL for large

### 6. Error Handling Strategy

```rust
#[derive(Error, Debug)]
pub enum QueryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Invalid criteria: {0}")]
    InvalidCriteria(String),
    
    #[error("Query timeout after {0}ms")]
    Timeout(u64),
}
```

**Validation Rules:**
- Date range cannot exceed 1 year (prevent excessive queries)
- Start date must be before end date
- Orb threshold must be 0-10 degrees
- Body IDs must be valid (0-9)

### 7. Module Structure

```
src/queries/
├── mod.rs           # Public API, re-exports
├── wedding.rs       # QUERY-01: Wedding date finder
├── voc.rs           # QUERY-02: Void-of-course periods
├── retrograde.rs    # QUERY-03: Retrograde periods
├── aspects.rs       # QUERY-04: Exact aspect search
└── types.rs         # Shared result types and criteria structs
```

### 8. Dependencies on Previous Phases

| Item | Depends On | Status | Impact |
|------|-----------|--------|--------|
| aspect_summaries table | Phase 1 (DB-01) | ✓ Complete | Required for performance |
| ChunkManager | Phase 2 (LOAD-01) | ✓ Complete | Cache integration |
| Database schema types | Phase 1 (DB schema) | ✓ Complete | Query building |
| Compact chunk data | Phase 2 (LOAD-01) | ✓ Complete | Potential in-memory queries |

### 9. Open Questions for Planning

1. **SQL Generation:**
   - Use sqlx `query_as!` macro with compile-time checking?
   - Or `query_as` function for dynamic queries?
   - How to handle optional filters (WHERE clauses)?

2. **aspect_summaries Population:**
   - Who populates aspect_summaries table?
   - Is it pre-populated or populated on-demand?
   - Should ChunkGenerator populate it when generating chunks?

3. **Query Caching:**
   - Cache query results beyond ChunkManager?
   - TTL for cached results?
   - Cache key structure?

4. **Async Boundaries:**
   - All queries async (return impl Future)?
   - Streaming results for large result sets?

5. **Testing Strategy:**
   - Mock database for unit tests?
   - Integration tests with test database?
   - Performance benchmarks?

### 10. Success Criteria Verification

From REQUIREMENTS.md:
- [ ] **QUERY-01:** Wedding date query with Venus aspects, Moon sign, no VoC
- [ ] **QUERY-02:** VoC period detection with duration filtering
- [ ] **QUERY-03:** Retrograde period queries with status calculation
- [ ] **QUERY-04:** Exact aspect search with orb/body/type filtering
- [ ] **QUERY-05:** <100ms for 60-day ranges (using aspect_summaries)

From CONTEXT.md decisions:
- [ ] Queries return structured result types (not raw rows)
- [ ] Each query has corresponding `*Criteria` struct
- [ ] Results include metadata (execution time, rows examined, cache hit/miss)
- [ ] Async functions returning `Result<Vec<T>, QueryError>`

---

## Research Summary

**What I need to know to plan well:**

1. **aspect_summaries is the key to performance** — Pre-aggregated aspect counts enable 51× speedup by eliminating correlated subqueries

2. **ChunkManager provides the cache layer** — Already implemented with LRU cache, database loading, and Swiss Ephemeris fallback

3. **Four distinct query patterns** — Each with specific SQL requirements, result types, and criteria validation

4. **SQL query design is critical** — Must use aspect_summaries JOIN, proper indexes, and LIMIT clauses to meet <100ms target

5. **Error handling must be descriptive** — Invalid criteria should return specific errors, not generic failures

**Next Step:** Create detailed implementation plan with SQL queries for each requirement, result type definitions, and module structure.

---

*Research complete: Ready for planning phase*