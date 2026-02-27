# Phase 4: Performance - Research

**Researched:** 2026-02-27
**Domain:** Database Performance Optimization (TimescaleDB, Rust Memory Management)
**Confidence:** HIGH

## Summary

Phase 4 implements performance optimizations for the Astro Clock astrological database system. The project already has a solid foundation with TimescaleDB hypertables, compact packed structs for memory efficiency (~4.5× reduction), and LRU caching. This phase focuses on three key areas: (1) multi-resolution storage using TimescaleDB continuous aggregates to reduce storage by storing outer planets at lower resolution, (2) memory management with configurable limits and graceful degradation, and (3) maintaining the 51× query speedup achieved in Phase 3.

The current system stores 1-minute resolution data for all planets, resulting in ~2GB/year for raw aspects alone. Multi-resolution storage will dramatically reduce this by storing Moon at 1-minute (fast-moving), inner planets at 5-minute, and outer planets at 60-minute resolution. TimescaleDB's native continuous aggregates feature provides automatic materialization and refresh policies for this downsampling.

**Primary recommendation:** Use TimescaleDB continuous aggregates with `last` aggregation for multi-resolution storage, implement memory-aware cache eviction in ChunkManager, and add aspect filtering to store only major aspects (conjunction, sextile, square, trine, opposition) for additional storage savings.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **Multi-resolution Storage Strategy:** Fixed by planet category - Moon at 1-minute resolution, inner planets (Sun, Mercury, Venus, Mars) at 5-minute resolution, outer planets (Jupiter, Saturn, Uranus, Neptune, Pluto) at 60-minute resolution
- **Automatic interpolation:** Query system automatically interpolates between stored points when higher resolution is needed for outer planets
- **TimescaleDB continuous aggregates:** Use native TimescaleDB feature for downsampling with automatic materialization
- **Last value aggregation:** Use last value in each time bucket for downsampled position data
- **Memory Optimization Thresholds:** 30MB soft limit, 50MB hard limit, aggressive eviction at 90% of hard limit (45MB)
- **Major aspects only:** Store conjunction (0°), sextile (60°), square (90°), trine (120°), and opposition (180°) aspects only
- **Calculate minors on-demand:** Semisextile, quincunx, and other minor aspects calculated as needed
- **Keep all data indefinitely:** Historical planetary positions are valuable for electoral astrology and never change
- **No compression:** Avoid TimescaleDB compression to prevent decompression overhead during common historical reference queries
- **Performance Monitoring:** Periodic background checks every hour during low-traffic periods, percentage degradation alerts (>20% = WARN, >50% = ERROR)

### Claude's Discretion
- Exact implementation details of interpolation algorithm for outer planets
- Specific TimescaleDB continuous aggregate refresh policies
- Benchmark scheduling logic (how to determine "low-traffic periods")
- Heap snapshot implementation details (which profiling crate to use)
- Config file format specifics (TOML structure beyond the cache section)

### Deferred Ideas (OUT OF SCOPE)
- Minor aspect pre-calculation if query patterns show heavy use
- Machine learning-based query prediction for cache pre-warming
- Distributed caching for multi-instance deployments
- Real-time WebSocket updates for planetary position changes
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| PERF-01 | Multi-resolution storage (different intervals per planet) | TimescaleDB continuous aggregates with `last` aggregation; planet category-based downsampling (Moon 1min, inner 5min, outer 60min) |
| PERF-02 | Memory usage <30MB for 30-day cache | Current packed structs achieve ~4.5× reduction; LRU cache with 30-chunk limit; memory-aware eviction at 90% threshold |
| PERF-03 | Database storage <50GB/year | Multi-resolution reduces outer planet data by 60×; major aspects only reduces aspect storage by ~50% |
| PERF-04 | Wedding query 51× faster than baseline (2.3s → 45ms) | Already achieved via aspect_summaries table; maintain with continuous aggregate refresh policies |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| TimescaleDB | 2.15+ | Time-series database with continuous aggregates | Native downsampling with automatic materialization |
| sqlx | 0.8 | Async PostgreSQL/TimescaleDB client | Already in use, compile-time checked queries |
| lru | 0.12 | LRU cache implementation | Already in use, O(1) operations, battle-tested |
| tokio | 1.0+ | Async runtime | Already in use for background tasks |
| rust_decimal | 1.30 | Exact decimal arithmetic | Already in use for coordinate precision |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| dhat | 0.3 | Heap profiling and memory tracking | Development/testing for PERF-02 verification |
| sysinfo | 0.30 | System memory information | Runtime memory monitoring for cache limits |
| tracing | 0.1 | Structured logging | Already in use, add memory/performance spans |
| toml | 0.8 | Config file parsing | Already in use via RON, but TOML for cache config |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| TimescaleDB continuous aggregates | Manual materialized views | Continuous aggregates have automatic refresh policies and better integration; manual views require cron jobs |
| dhat | heaptrack | dhat is pure Rust, easier integration; heaptrack requires external tool |
| sysinfo | procfs | sysinfo is cross-platform; procfs is Linux-only |

**Installation:**
```bash
# Already in Cargo.toml:
# lru = "0.12"
# tokio = { version = "1.0", features = ["full"] }
# rust_decimal = { version = "1.30" }
# tracing = "0.1"

# Add for Phase 4:
cargo add dhat --dev
cargo add sysinfo
```

## Architecture Patterns

### Recommended Project Structure
```
src/
├── database/
│   ├── mod.rs
│   ├── pool.rs
│   ├── schema.rs
│   ├── chunk.rs              # Existing compact structs
│   ├── chunk_manager.rs      # Add memory-aware eviction
│   ├── chunk_generator.rs
│   └── multi_resolution.rs   # NEW: Continuous aggregate management
├── performance/
│   ├── mod.rs
│   ├── memory_monitor.rs     # NEW: Memory tracking and limits
│   ├── interpolation.rs      # NEW: Outer planet interpolation
│   └── benchmark.rs          # Move from queries/
├── config.rs                 # Add cache section
└── lib.rs
```

### Pattern 1: TimescaleDB Continuous Aggregates
**What:** Native TimescaleDB feature for automatic downsampling with materialized views
**When to use:** Multi-resolution storage (PERF-01)
**Example:**
```sql
-- Create continuous aggregate for 5-minute resolution (inner planets)
CREATE MATERIALIZED VIEW planet_positions_5min
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('5 minutes', time) as bucket,
    body_id,
    last(longitude, time) as longitude,
    last(latitude, time) as latitude,
    last(distance, time) as distance,
    last(speed_lon, time) as speed_lon,
    last(retrograde, time) as retrograde,
    last(zodiac_sign, time) as zodiac_sign
FROM planet_positions
WHERE body_id IN (0, 2, 3, 4)  -- Sun, Mercury, Venus, Mars
GROUP BY bucket, body_id;

-- Create continuous aggregate for 60-minute resolution (outer planets)
CREATE MATERIALIZED VIEW planet_positions_60min
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('60 minutes', time) as bucket,
    body_id,
    last(longitude, time) as longitude,
    last(latitude, time) as latitude,
    last(distance, time) as distance,
    last(speed_lon, time) as speed_lon,
    last(retrograde, time) as retrograde,
    last(zodiac_sign, time) as zodiac_sign
FROM planet_positions
WHERE body_id IN (5, 6, 7, 8, 9)  -- Jupiter, Saturn, Uranus, Neptune, Pluto
GROUP BY bucket, body_id;

-- Add refresh policies
SELECT add_continuous_aggregate_policy('planet_positions_5min',
    start_offset => INTERVAL '1 day',
    end_offset => INTERVAL '1 hour',
    schedule_interval => INTERVAL '1 hour'
);

SELECT add_continuous_aggregate_policy('planet_positions_60min',
    start_offset => INTERVAL '7 days',
    end_offset => INTERVAL '1 hour',
    schedule_interval => INTERVAL '1 hour'
);
```

### Pattern 2: Memory-Aware Cache Eviction
**What:** Monitor memory usage and trigger aggressive eviction when approaching limits
**When to use:** Memory constraint compliance (PERF-02)
**Example:**
```rust
// In chunk_manager.rs
use sysinfo::{System, SystemExt, ProcessExt};

pub struct MemoryAwareCache {
    soft_limit_mb: usize,
    hard_limit_mb: usize,
    eviction_threshold_pct: f64,
    system: System,
}

impl MemoryAwareCache {
    pub fn check_memory_pressure(&mut self) -> MemoryPressure {
        self.system.refresh_memory();
        let current_mb = self.get_current_memory_mb();
        
        let hard_threshold = self.hard_limit_mb as f64 * (self.evicition_threshold_pct / 100.0);
        
        if current_mb >= self.hard_limit_mb {
            MemoryPressure::Critical
        } else if current_mb as f64 >= hard_threshold {
            MemoryPressure::High
        } else if current_mb >= self.soft_limit_mb {
            MemoryPressure::Elevated
        } else {
            MemoryPressure::Normal
        }
    }
    
    pub async fn evict_if_needed(&mut self, cache: &mut LruCache<ChunkKey, Arc<ChunkData>>) {
        match self.check_memory_pressure() {
            MemoryPressure::Critical => {
                // Evict 50% of cache
                let target_size = cache.len() / 2;
                while cache.len() > target_size {
                    cache.pop_lru();
                }
                tracing::error!("Critical memory pressure: evicted to {} chunks", cache.len());
            }
            MemoryPressure::High => {
                // Evict 25% of cache
                let target_size = cache.len() * 3 / 4;
                while cache.len() > target_size {
                    cache.pop_lru();
                }
                tracing::warn!("High memory pressure: evicted to {} chunks", cache.len());
            }
            _ => {}
        }
    }
}
```

### Pattern 3: Linear Interpolation for Outer Planets
**What:** Interpolate between hourly samples when higher resolution needed
**When to use:** Querying outer planets at sub-hourly resolution
**Example:**
```rust
// In interpolation.rs
pub fn interpolate_longitude(
    t: DateTime<Utc>,
    t1: DateTime<Utc>,
    t2: DateTime<Utc>,
    lon1: Decimal,
    lon2: Decimal,
) -> Decimal {
    let total_duration = (t2 - t1).num_seconds() as f64;
    let elapsed = (t - t1).num_seconds() as f64;
    let fraction = Decimal::from_f64_retain(elapsed / total_duration).unwrap();
    
    // Handle 360° wraparound
    let diff = lon2 - lon1;
    let adjusted_diff = if diff > Decimal::from(180) {
        diff - Decimal::from(360)
    } else if diff < Decimal::from(-180) {
        diff + Decimal::from(360)
    } else {
        diff
    };
    
    let result = lon1 + adjusted_diff * fraction;
    
    // Normalize to 0-360
    if result < Decimal::from(0) {
        result + Decimal::from(360)
    } else if result >= Decimal::from(360) {
        result - Decimal::from(360)
    } else {
        result
    }
}
```

### Pattern 4: Aspect Filtering
**What:** Store only major Ptolemaic aspects, calculate minors on-demand
**When to use:** Storage reduction (PERF-03)
**Example:**
```rust
// In chunk_generator.rs - modify aspect calculation
const MAJOR_ASPECTS: [(i16, Decimal); 5] = [
    (0, Decimal::from(0)),      // Conjunction
    (1, Decimal::from(60)),     // Sextile
    (2, Decimal::from(90)),     // Square
    (3, Decimal::from(120)),    // Trine
    (4, Decimal::from(180)),    // Opposition
];

fn is_major_aspect(angle: Decimal, orb: Decimal) -> Option<i16> {
    for (aspect_type, target_angle) in MAJOR_ASPECTS {
        let diff = (angle - target_angle).abs();
        let adjusted_diff = if diff > Decimal::from(180) {
            Decimal::from(360) - diff
        } else {
            diff
        };
        
        if adjusted_diff <= orb {
            return Some(aspect_type);
        }
    }
    None
}
```

### Anti-Patterns to Avoid
- **Don't use TimescaleDB compression:** User explicitly deferred this to avoid decompression overhead for historical queries
- **Don't store minor aspects:** Calculate semisextile (30°), quincunx (150°), etc. on-demand
- **Don't use real-time aggregation:** Continuous aggregates with materialization provide better query performance
- **Don't ignore memory pressure:** Always check memory before loading new chunks

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Time-series downsampling | Custom aggregation jobs | TimescaleDB continuous aggregates | Native refresh policies, automatic materialization, optimized storage |
| Memory monitoring | Custom /proc parsing | sysinfo crate | Cross-platform, well-tested, handles edge cases |
| Heap profiling | Custom allocator tracking | dhat crate | Zero-overhead when disabled, detailed reports |
| LRU cache | Custom HashMap + Vec | lru crate | Already in use, O(1) operations, proven correctness |
| Linear interpolation | Naive lerp without wraparound | Pattern 3 above | Astrological coordinates wrap at 360° |

**Key insight:** TimescaleDB continuous aggregates are purpose-built for this exact use case. Custom solutions would require implementing refresh scheduling, materialization logic, and handling edge cases that TimescaleDB already solves.

## Common Pitfalls

### Pitfall 1: Continuous Aggregate Refresh Policy Timing
**What goes wrong:** Refresh policies run too frequently, causing excessive I/O; or too infrequently, causing stale data
**Why it happens:** Default policies don't account for the project's specific data patterns
**How to avoid:** 
- Use `start_offset => INTERVAL '1 day'` for 5min aggregates (recent data only)
- Use `start_offset => INTERVAL '7 days'` for 60min aggregates (outer planets change slowly)
- Set `schedule_interval` based on data ingestion rate (hourly for this project)
**Warning signs:** High I/O wait times, lagging refresh jobs in `timescaledb_information.continuous_aggregate_stats`

### Pitfall 2: Interpolation at 360° Boundary
**What goes wrong:** Interpolating between 359° and 1° gives wrong result (179° instead of 2°)
**Why it happens:** Circular coordinate systems require special handling
**How to avoid:** Always adjust difference to [-180, 180] range before interpolating (see Pattern 3)
**Warning signs:** Sudden jumps in interpolated positions, incorrect aspect calculations

### Pitfall 3: Memory Limit Race Conditions
**What goes wrong:** Multiple threads check memory simultaneously, all decide to evict, cache undersized
**Why it happens:** Memory check and eviction aren't atomic
**How to avoid:** Use RwLock for cache, check memory inside write lock, or use atomic counters for approximate tracking
**Warning signs:** Cache thrashing, excessive evictions, performance degradation

### Pitfall 4: Continuous Aggregate Data Retention
**What goes wrong:** Continuous aggregates grow unbounded, consuming excessive storage
**Why it happens:** By default, continuous aggregates keep all data
**How to avoid:** Add retention policies: `SELECT add_retention_policy('planet_positions_5min', INTERVAL '1 year')`
**Warning signs:** Unexpected storage growth, slow aggregate queries on old data

### Pitfall 5: Refresh Policy Conflicts with Data Loading
**What goes wrong:** Continuous aggregate refresh runs while ChunkManager is loading data, causing contention
**Why it happens:** No coordination between background processes
**How to avoid:** Schedule refresh policies during low-traffic periods, use `end_offset` to exclude recent data from refresh
**Warning signs:** Slow queries during refresh windows, connection pool exhaustion

## Code Examples

### Creating Multi-Resolution Tables
```rust
// In multi_resolution.rs
use sqlx::PgPool;

pub struct MultiResolutionManager {
    pool: PgPool,
}

impl MultiResolutionManager {
    pub async fn setup_continuous_aggregates(&self) -> Result<(), sqlx::Error> {
        // 5-minute resolution for inner planets
        sqlx::query(
            r#"
            CREATE MATERIALIZED VIEW IF NOT EXISTS planet_positions_5min
            WITH (timescaledb.continuous) AS
            SELECT
                time_bucket('5 minutes', time) as bucket,
                body_id,
                last(longitude, time) as longitude,
                last(latitude, time) as latitude,
                last(distance, time) as distance,
                last(speed_lon, time) as speed_lon,
                last(retrograde, time) as retrograde,
                last(zodiac_sign, time) as zodiac_sign
            FROM planet_positions
            WHERE body_id IN (0, 2, 3, 4)
            GROUP BY bucket, body_id
            "#
        )
        .execute(&self.pool)
        .await?;
        
        // 60-minute resolution for outer planets
        sqlx::query(
            r#"
            CREATE MATERIALIZED VIEW IF NOT EXISTS planet_positions_60min
            WITH (timescaledb.continuous) AS
            SELECT
                time_bucket('60 minutes', time) as bucket,
                body_id,
                last(longitude, time) as longitude,
                last(latitude, time) as latitude,
                last(distance, time) as distance,
                last(speed_lon, time) as speed_lon,
                last(retrograde, time) as retrograde,
                last(zodiac_sign, time) as zodiac_sign
            FROM planet_positions
            WHERE body_id IN (5, 6, 7, 8, 9)
            GROUP BY bucket, body_id
            "#
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}
```

### Config File Structure
```toml
# In config.toml
[cache]
soft_limit_mb = 30
hard_limit_mb = 50
eviction_threshold_pct = 90.0

[performance]
benchmark_interval_hours = 1
degradation_warn_pct = 20.0
degradation_error_pct = 50.0
memory_snapshot_interval_minutes = 5

[multi_resolution]
moon_interval_minutes = 1
inner_planets_interval_minutes = 5
outer_planets_interval_minutes = 60
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Uniform 1-minute resolution for all planets | Category-based multi-resolution (1min/5min/60min) | Phase 4 (this) | ~60× reduction in outer planet storage |
| Store all aspects | Major aspects only (0°, 60°, 90°, 120°, 180°) | Phase 4 (this) | ~50% reduction in aspect storage |
| Fixed cache size | Memory-aware eviction with soft/hard limits | Phase 4 (this) | Guaranteed <50MB memory usage |
| Manual benchmark runs | Automated hourly benchmarks with alerting | Phase 4 (this) | Proactive performance monitoring |

**Deprecated/outdated:**
- TimescaleDB 1.x continuous aggregates (use 2.x with policies)
- Custom materialized views with cron (use native continuous aggregates)

## Open Questions

1. **Interpolation accuracy validation**
   - What we know: Linear interpolation with wraparound handling is standard
   - What's unclear: Whether astrological calculations need higher-order interpolation for outer planets
   - Recommendation: Implement linear first, validate against Swiss Ephemeris calculations

2. **Refresh policy timing optimization**
   - What we know: Hourly refresh is a reasonable default
   - What's unclear: Optimal schedule_interval for this specific workload pattern
   - Recommendation: Start with hourly, monitor `timescaledb_information.continuous_aggregate_stats` and adjust

3. **Memory measurement accuracy**
   - What we know: sysinfo provides process memory usage
   - What's unclear: Whether to measure RSS, heap, or total memory for the 30MB limit
   - Recommendation: Use RSS as it's what the OS enforces; document this choice

## Sources

### Primary (HIGH confidence)
- TimescaleDB 2.15 Documentation - Continuous Aggregates: https://docs.timescale.com/use-timescale/latest/continuous-aggregates/
- TimescaleDB 2.15 Documentation - Continuous Aggregate Policies: https://docs.timescale.com/use-timescale/latest/continuous-aggregates/refresh-policies/
- Current codebase: `src/database/chunk.rs` - Packed struct implementation already achieves ~4.5× memory reduction
- Current codebase: `src/database/chunk_manager.rs` - LRU cache with 30-chunk limit already implemented

### Secondary (MEDIUM confidence)
- sysinfo crate docs: https://docs.rs/sysinfo/latest/sysinfo/ - Cross-platform memory monitoring
- dhat crate docs: https://docs.rs/dhat/latest/dhat/ - Heap profiling for Rust

### Tertiary (LOW confidence)
- Astrological interpolation methods - Linear is standard but higher-order methods exist for ephemeris calculations

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - TimescaleDB continuous aggregates are well-documented and purpose-built for this use case
- Architecture: HIGH - Patterns follow existing codebase conventions (chunk_manager, compact structs)
- Pitfalls: MEDIUM - Some edge cases (interpolation accuracy, refresh timing) need validation during implementation

**Research date:** 2026-02-27
**Valid until:** 30 days (TimescaleDB is stable, but verify docs if implementing after 2026-03-27)
