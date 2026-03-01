# Roadmap: Astro Clock

**Created:** 2026-02-24
**Phases:** 4
**Requirements:** 22 v1 requirements mapped

---

## Phase Overview

| # | Name | Goal | Requirements | Success Criteria |
|---|------|------|--------------|------------------|
| 1 | Database Schema | Create optimized TimescaleDB schema for time-series astrological data | DB-01 to DB-06 | 3/3 complete | 2026-02-25 | 6 |
| 2 | Data Loading | Complete chunk-based loading with LRU cache and Swiss Ephemeris | LOAD-01 to LOAD-05 | 4/4 complete | 2026-02-25 | 5 |
| 3 | Query System | Build specialized query functions for electoral astrology | QUERY-01 to QUERY-05 | 5/5 complete | 2026-02-25 | 5 |
| 4 | Performance | Optimize storage, memory, and query performance | PERF-01 to PERF-04 | 2/5 in progress | 2026-03-01 | 4 |

---

## Phase 1: Database Schema

**Goal:** Create optimized TimescaleDB schema for time-series astrological data

**Requirements:** DB-01, DB-02, DB-03, DB-04, DB-05, DB-06

**Success Criteria:**
1. All hypertables created with proper chunk intervals (1 day)
2. Composite indexes cover common query patterns
3. Aspect summaries table eliminates correlated subqueries
4. House cusps normalized with locations table
5. Schema supports 1-minute resolution for Moon, coarser for outer planets
6. Migration scripts tested and documented

**Key Deliverables:**
- `migrations/001_create_hypertables.sql`
- `migrations/002_create_indexes.sql`
- `migrations/003_create_locations.sql`
- `migrations/004_create_aspect_summaries.sql`
- `migrations/verify_schema.sql`
- Database schema documentation

**Plans:** 3 plans in 3 waves

Plans:
- [x] 01-01-PLAN.md — Create core hypertables and indexes (Complete: 2026-02-25)
- [x] 01-02-PLAN.md — Set up sqlx migration tooling (Complete: 2026-02-25)
- [x] 01-03-PLAN.md — Verify schema and update Rust types (Complete: 2026-02-25)

---

## Phase 2: Data Loading

**Goal:** Implement chunk-based loading with LRU cache and Swiss Ephemeris integration

**Requirements:** LOAD-01, LOAD-02, LOAD-03, LOAD-04, LOAD-05

**Success Criteria:**
1. ChunkManager loads data in 1-day chunks
2. LRU cache holds 30 days of data (~30MB)
3. Database hits return cached data without recalculation
4. Cache misses trigger Swiss Ephemeris generation
5. Generated data persisted to database
6. Background pre-fetching loads adjacent chunks

**Key Deliverables:**
- `src/database/chunk.rs` — Compact chunk data structures
- `src/database/chunk_manager.rs` — LRU cache and database loading
- `src/database/chunk_generator.rs` — Swiss Ephemeris integration
- Chunk data structures with compact memory layout (~4.5× size reduction)

**Plans:** 4/4 plans complete

Plans:
- [x] 02-01-PLAN.md — Create compact chunk data structures and add LRU dependency (Complete: 2026-02-25)
- [x] 02-02-PLAN.md — Implement ChunkManager with LRU cache and database loading (Complete: 2026-02-25)
- [x] 02-03-PLAN.md — Implement Swiss Ephemeris generation and database persistence (Complete: 2026-02-25)
- [x] 02-04-PLAN.md — Add background pre-fetching of adjacent chunks (Complete: 2026-02-25)

---

## Phase 3: Query System

**Goal:** Build specialized query functions for electoral astrology

**Requirements:** QUERY-01, QUERY-02, QUERY-03, QUERY-04, QUERY-05

**Success Criteria:**
1. Wedding date query returns results in <100ms (cached)
2. Void-of-course Moon periods query working
3. Retrograde period query working
4. Exact aspect query working
5. All queries use aspect_summaries for performance

**Key Deliverables:**
- `src/queries/wedding.rs`
- `src/queries/voc.rs`
- `src/queries/retrograde.rs`
- `src/queries/aspects.rs`
- `src/queries/benchmark.rs`
- Query benchmarks and performance tests

**Plans:** 3 plans in 2 waves

Plans:
- [x] 03-01-PLAN.md — Create query infrastructure (types, errors, criteria) (Complete: 2026-02-25)
- [x] 03-02-PLAN.md — Implement wedding and VoC queries with retrograde migration (Complete: 2026-02-25)
- [x] 03-03-PLAN.md — Implement retrograde and aspect queries with benchmarks (Complete: 2026-02-25)

---

## Phase 4: Performance

**Goal:** Optimize storage, memory, and query performance

**Requirements:** PERF-01, PERF-02, PERF-03, PERF-04

**Success Criteria:**
1. Multi-resolution storage implemented (1min Moon, 5min inner, 1hr outer)
2. Memory usage stays under 30MB for 30-day cache
3. Annual storage under 50GB
4. Wedding query 51× faster than baseline (2.3s → 45ms)

**Key Deliverables:**
- Continuous aggregates for lower resolutions
- Compact data structures (packed structs)
- Aspect filtering to reduce storage
- Performance benchmark suite
- Memory profiling results

**Plans:** 5 plans in 3 waves

Plans:
- [x] 04-01-PLAN.md — Create TimescaleDB continuous aggregates for multi-resolution storage (Complete: 2026-02-28)
- [x] 04-02-PLAN.md — Implement memory-aware cache eviction with sysinfo monitoring (Complete: 2026-03-01)
- [ ] 04-03-PLAN.md — Add interpolation for outer planets and major aspect filtering
- [ ] 04-04-PLAN.md — Create automated benchmark runner with regression detection
- [ ] 04-05-PLAN.md — Integrate memory monitoring with ChunkManager for eviction

---

## Dependencies Between Phases

```
Phase 1 (Schema)
    ↓
Phase 2 (Data Loading) — depends on schema tables
    ↓
Phase 3 (Query System) — depends on data loading
    ↓
Phase 4 (Performance) — depends on query system for benchmarking
```

**No parallel phases** — each phase builds on the previous.

---

## Risk Mitigation

| Risk | Phase | Mitigation |
|------|-------|------------|
| Cache misses slow queries | 2, 3 | Pre-load current year on startup |
| Memory pressure | 2, 4 | Configurable cache size with LRU eviction |
| Database downtime | 2, 3 | Fallback to Swiss Ephemeris generation |
| Data consistency | 2 | Version chunks with checksums |
| Query performance regression | 3, 4 | Benchmark suite with CI checks |

---

## Definition of Done

**Project complete when:**
- All 22 v1 requirements implemented and verified
- Wedding query completes in <100ms for 60-day ranges
- Memory usage <30MB for standard operations
- All tests passing
- Documentation complete

---

*Roadmap created: 2026-02-24*
*Last updated: 2026-03-01 after completing 04-02 Memory Monitoring*
