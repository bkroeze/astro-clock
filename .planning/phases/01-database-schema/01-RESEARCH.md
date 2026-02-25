# Phase 1: Database Schema - Research

**Research Date:** 2026-02-24  
**Status:** Complete — Ready for Planning  
**Phase:** 01-database-schema  

---

## What I Need to Know to Plan This Phase Well

### 1. Requirements to Address (DB-01 through DB-06)

| Requirement | Description | Key Decisions Needed |
|-------------|-------------|---------------------|
| **DB-01** | Create aspect_summaries table for pre-aggregated aspect counts | Aggregation logic, column types, indexing |
| **DB-02** | Add composite indexes for common query patterns | Which query patterns? Index ordering? |
| **DB-03** | Refactor house_cusps with normalized locations table | Location table structure, foreign key design |
| **DB-04** | Create planet_positions hypertable with TimescaleDB | Chunk interval, partitioning strategy |
| **DB-05** | Create aspects hypertable for aspect data | Aspect filtering criteria, storage reduction |
| **DB-06** | Create lunar_conditions hypertable for Moon data | VoC detection, Moon phase tracking |

### 2. Technical Context

#### Existing Database Infrastructure
- **Connection Pool:** Already exists in `src/database/pool.rs`
- **Migration Tool:** sqlx migrate (Rust-native, compile-time query checking)
- **Connection String:** `PG_URL` in `.env` → `postgresql://astro:barnlab@10.0.10.50:5432/astrology`
- **Current Schema:** Basic `charts` table with JSONB columns (needs refactoring)

#### Tech Stack Decisions
- **Database:** PostgreSQL with TimescaleDB extension
- **ORM:** sqlx 0.8 (async, compile-time checked)
- **Migration Location:** Standard sqlx location (to be created)
- **Justfile Integration:** Need `migrate` command

### 3. Domain Knowledge Required

#### Astrological Data Types
```
Bodies (need body_id mapping):
- Sun (0), Moon (1), Mercury (2), Venus (3), Mars (4)
- Jupiter (5), Saturn (6), Uranus (7), Neptune (8), Pluto (9)
- Optionally: Chiron, Lilith, Node, etc.

Aspects:
- Conjunction (0°), Sextile (60°), Square (90°), Trine (120°), Opposition (180°)
- Orb thresholds: typically 5-10° depending on aspect

House Systems:
- 11 supported: Placidus, Koch, Equal, etc.
- 12 cusps per chart

Lunar Conditions:
- Moon phases: New, Waxing Crescent, First Quarter, etc.
- Void-of-Course: Moon makes no major aspects before leaving sign
```

#### Time-Series Characteristics
- **Resolution:** 1-minute initially (all bodies same resolution in Phase 1)
- **Chunk Interval:** 1 day (TimescaleDB hypertable chunks)
- **Date Range:** Multi-year storage needed for electoral astrology queries
- **Volume Estimate:** 
  - 1-minute resolution = 1,440 records/day/body
  - 10 bodies = 14,400 records/day
  - ~5.3M records/year (before aspect filtering)

### 4. Performance Requirements

| Metric | Target | Implication |
|--------|--------|-------------|
| Wedding query | <100ms for 60-day ranges | Requires indexes + aspect_summaries |
| Memory usage | <30MB for 30-day cache | Compact data types needed |
| Storage | <50GB/year | Aspect filtering critical |

### 5. Key Design Decisions (from Context)

#### Multi-Resolution Storage
- **Phase 1:** Single resolution (1-minute) for all bodies
- **Phase 4:** Continuous aggregates for optimization
  - Moon: 1-minute
  - Inner planets (Mercury, Venus, Mars): 5-minute
  - Outer planets (Jupiter+): 1-hour

#### Aspect Filtering Strategy
- Store ONLY "interesting" aspects in aspect_summaries
- Filter criteria:
  - Body pairs: Sun, Moon, planets (not asteroids)
  - Orb threshold: Configurable, default 10°
- **Target:** 99% reduction in aspect storage

#### Table Structure

```
planet_positions (hypertable)
├── time (TIMESTAMPTZ) — partition key
├── body_id (SMALLINT) — 0-9 for main bodies
├── longitude (DECIMAL(8,4)) — 0-360 degrees
├── latitude (DECIMAL(8,4))
├── distance (DECIMAL(10,6)) — AU
├── speed_lon (DECIMAL(9,5))
├── retrograde (BOOLEAN)

aspects (hypertable)
├── time (TIMESTAMPTZ)
├── body1_id (SMALLINT)
├── body2_id (SMALLINT)
├── aspect_type (SMALLINT) — 0-4 for major aspects
├── orb (DECIMAL(5,2)) — degrees
├── applying (BOOLEAN) — aspect getting tighter?

lunar_conditions (hypertable)
├── time (TIMESTAMPTZ)
├── moon_phase (SMALLINT) — 0-7
├── moon_sign (SMALLINT) — 0-11
├── voc_start (TIMESTAMPTZ) — nullable
├── voc_end (TIMESTAMPTZ) — nullable
├── is_void_of_course (BOOLEAN)

house_cusps (hypertable)
├── time (TIMESTAMPTZ)
├── location_id (INTEGER) — FK to locations
├── house_system (SMALLINT)
├── cusp_1..12 (DECIMAL(8,4))

locations (regular table)
├── id (SERIAL PRIMARY KEY)
├── name (VARCHAR)
├── latitude (DECIMAL(8,4))
├── longitude (DECIMAL(8,4))
├── timezone (VARCHAR)

aspect_summaries (regular table with time index)
├── time (TIMESTAMPTZ)
├── body_id (SMALLINT)
├── conjunction_count (SMALLINT)
├── sextile_count (SMALLINT)
├── square_count (SMALLINT)
├── trine_count (SMALLINT)
├── opposition_count (SMALLINT)
├── total_aspects (SMALLINT)
```

### 6. Index Strategy

#### Required Indexes

```sql
-- Planet positions: time-range queries by body
CREATE INDEX idx_planet_positions_time_body 
ON planet_positions (time, body_id);

-- Partial index for Moon (frequently queried)
CREATE INDEX idx_planet_positions_moon 
ON planet_positions (time) 
WHERE body_id = 1;

-- Aspects: time-range queries
CREATE INDEX idx_aspects_time 
ON aspects (time, body1_id, body2_id);

-- Lunar conditions: VoC checks
CREATE INDEX idx_lunar_voc 
ON lunar_conditions (time, is_void_of_course) 
WHERE is_void_of_course = true;

-- House cusps: location + time
CREATE INDEX idx_house_cusps_location_time 
ON house_cusps (location_id, time);

-- Aspect summaries: fast counting
CREATE INDEX idx_aspect_summaries_time_body 
ON aspect_summaries (time, body_id);
```

### 7. Migration Files Needed

Based on ROADMAP.md deliverables:

1. `migrations/001_create_hypertables.sql`
   - Create locations table first (no dependencies)
   - Create planet_positions hypertable
   - Create aspects hypertable
   - Create lunar_conditions hypertable
   - Create house_cusps hypertable

2. `migrations/002_create_indexes.sql`
   - All composite indexes
   - Partial indexes for Moon
   - Covering indexes for VoC

3. `migrations/003_create_aspect_summaries.sql`
   - Create table
   - Populate with initial aggregation logic
   - Create indexes

4. `migrations/004_refactor_house_cusps.sql`
   - Normalize location data
   - Migrate existing data if any
   - Add foreign key constraints

### 8. Open Questions for Planning

1. **Data Types:** 
   - SMALLINT sufficient for body_id (0-255)?
   - DECIMAL precision for longitudes (8,4) vs (10,6)?
   - Use REAL for speed values to save space?

2. **Aspect Summary Logic:**
   - Exact SQL for aggregation from aspects table?
   - Trigger-based or batch population?
   - Real-time updates or hourly batch?

3. **House Systems:**
   - Store all 11 systems or just requested?
   - How to handle house_system enum?

4. **Chunk Management:**
   - 1-day chunks confirmed?
   - Retention policy needed?

5. **sqlx Integration:**
   - Query file location conventions?
   - Offline mode for CI?

### 9. Dependencies on Other Phases

| Item | Depends On | Impact |
|------|-----------|--------|
| Aspect summary population | Phase 3 (Query System) | Table created now, populated later |
| Continuous aggregates | Phase 4 (Performance) | Schema supports, implemented later |
| Chunk manager | Phase 2 (Data Loading) | Schema must support chunk queries |

### 10. Success Criteria Verification

From ROADMAP.md:
- ✓ All hypertables created with proper chunk intervals (1 day) — **PLAN**
- ✓ Composite indexes cover common query patterns — **PLAN**
- ✓ Aspect summaries table eliminates correlated subqueries — **PLAN**
- ✓ House cusps normalized with locations table — **PLAN**
- ✓ Schema supports 1-minute resolution for Moon, coarser for outer planets — **DEFER to Phase 4**
- ✓ Migration scripts tested and documented — **PLAN**

---

## Research Summary

**What I need to know to plan well:**

1. **TimescaleDB hypertable syntax** — How to convert regular tables to hypertables with 1-day chunks
2. **sqlx migration conventions** — File naming, multiple statements per file, rollback support
3. **Optimal DECIMAL precision** — For astrological coordinates (degrees, AU, speed)
4. **Aspect summary aggregation SQL** — Pre-aggregated counts per body per time bucket
5. **Index ordering for composite indexes** — Time-first vs body-first for query patterns
6. **Justfile integration** — How to add `just migrate` command

**Next Step:** Create detailed implementation plan with SQL DDL, migration order, and testing strategy.

---

*Research complete: Ready for planning phase*
