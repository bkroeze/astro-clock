# Phase 1: Database Schema - Context

**Gathered:** 2026-02-24
**Status:** Ready for planning

<domain>
## Phase Boundary

Create optimized TimescaleDB schema for time-series astrological data. This includes hypertables for planet positions, aspects, lunar conditions, and house cusps with normalized location data. Indexes for common query patterns. Aspect summaries for fast aspect counting.

</domain>

<decisions>
## Implementation Decisions

### Multi-resolution storage
- Defer continuous aggregates to Phase 4
- Phase 1 creates only base hypertables with 1-day chunks
- Multi-resolution optimization (1min Moon, 5min inner planets, etc.) belongs in performance phase

### Aspect filtering
- aspect_summaries table stores ONLY "interesting" aspects
- Filter by body pairs (Sun, Moon, planets — not asteroids)
- Filter by orb threshold (configurable, default 10°)
- Target: 99% reduction in aspect storage vs storing all pairs

### Migration strategy
- Use sqlx migrate for Rust-native migrations
- Compile-time query checking with sqlx
- Migration files in standard sqlx location
- Justfile command to run migrations

### Table structure
- planet_positions: 1-minute resolution for all bodies initially
- aspects: Pre-calculated aspect data (filtered)
- lunar_conditions: Moon phases, void-of-course periods
- house_cusps: Normalized with locations table (location_id foreign key)
- aspect_summaries: Pre-aggregated counts per body per time
- locations: Normalized location data (lat/lon → location_id)

### Index strategy
- Composite indexes for common query patterns
- Partial indexes for specific bodies (e.g., Moon queries)
- Covering indexes for VoC checks
- Index on (time, body_id) for all position tables

### Claude's Discretion
- Exact column types (SMALLINT vs INTEGER for body IDs)
- Specific index names and ordering
- Whether to use table partitioning beyond TimescaleDB chunks
- Exact SQL for aspect summary aggregation logic

</decisions>

<specifics>
## Specific Ideas

- Implementation plan already has detailed SQL examples for aspect_summaries, indexes, and house_cusps refactoring
- Target performance: Wedding query <100ms for 60-day ranges
- Storage target: <50GB/year with filtering
- No existing data in database — clean slate for schema design
- Database connection string in .env as PG_URL

</specifics>

<deferred>
## Deferred Ideas

- Continuous aggregates for multi-resolution storage — Phase 4
- Aspect filtering implementation details — Phase 3 (Query System)
- Chunk manager and LRU cache — Phase 2 (Data Loading)
- Specialized query functions — Phase 3 (Query System)
- Performance benchmarking — Phase 4 (Performance)

</deferred>

---

*Phase: 01-database-schema*
*Context gathered: 2026-02-24*
