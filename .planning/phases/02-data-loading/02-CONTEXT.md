# Phase 2: Data Loading - Context

**Gathered:** 2026-02-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement chunk-based loading with LRU cache and Swiss Ephemeris integration. This includes loading data from database when available, generating from Swiss Ephemeris when not, persisting generated data, and background pre-fetching of adjacent chunks.

</domain>

<decisions>
## Implementation Decisions

### Cache eviction strategy
- LRU (Least Recently Used) eviction when cache is full
- No time-based expiration — chunks stay in cache until evicted
- Eviction happens only when cache reaches capacity (30 chunks)

### Pre-fetching behavior
- Pre-fetch adjacent chunks (previous day and next day) in background
- Fire-and-forget background tasks — don't wait for completion
- Pre-fetching is best-effort; failures don't affect main query
- No pre-fetching beyond immediate neighbors (1 day before/after)

### Chunk data granularity
- 1-day chunks aligned with TimescaleDB chunk boundaries
- All bodies in a single chunk (not separate chunks per body)
- Full resolution data (1-minute for Moon, etc.) — no downsampling in Phase 2

### Swiss Ephemeris integration
- Use existing `swiss-eph` crate (already in Cargo.toml)
- Generate all planetary positions for the chunk date
- Calculate aspects during generation (not stored separately)
- Generate lunar conditions (Moon phase, VoC status)

### Error handling
- Database errors: Log warning, fall back to Swiss Ephemeris generation
- Swiss Ephemeris errors: Return error to caller (fail the chunk load)
- Cache errors: Non-fatal, log and continue
- Background pre-fetch errors: Silent (don't affect main flow)

### Data persistence
- Save generated chunks to database immediately after generation
- Use batch inserts for performance (not individual row inserts)
- Database write failures don't fail the chunk load (data is still in cache)

### Claude's Discretion
- Exact LRU cache implementation (can use `lru` crate or custom)
- Chunk data structure layout (Vec vs HashMap for planet storage)
- SQL query structure for batch inserts
- Background task spawning mechanism (tokio::spawn vs task queue)
- Exact error types and logging levels

</decisions>

<specifics>
## Specific Ideas

- Implementation plan already has detailed Rust code for ChunkManager, ChunkKey, ChunkData
- Target: 30MB memory for 30-day cache (~1MB per day)
- Use compact data structures where possible (u32 for timestamps, i32 for millidegrees)
- Background pre-fetching pattern from implementation plan: `tokio::spawn(async move { ... })`
- Cache lookup order: 1) LRU cache, 2) Database, 3) Swiss Ephemeris generation

</specifics>

<deferred>
## Deferred Ideas

- Multi-resolution storage (1min Moon, 5min inner planets, etc.) — Phase 4
- Aspect filtering to reduce storage — Phase 3 (when query functions are built)
- Continuous aggregates for lower resolutions — Phase 4
- Cache warming/pre-loading on startup — Phase 4
- Distributed caching (Redis, etc.) — future enhancement
- Chunk compression for memory savings — future enhancement

</deferred>

---

*Phase: 02-data-loading*
*Context gathered: 2026-02-25*
