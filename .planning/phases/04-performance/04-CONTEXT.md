# Phase 4: Performance - Context

**Gathered:** 2026-02-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Optimize storage, memory, and query performance for the astrological database and query system. This phase implements multi-resolution storage, memory management, storage reduction techniques, and performance monitoring to meet targets: <30MB memory for 30-day cache, <50GB/year storage, and maintain 51× query speedup.

</domain>

<decisions>
## Implementation Decisions

### Multi-resolution Storage Strategy
- **Fixed by planet category:** Moon at 1-minute resolution, inner planets (Sun, Mercury, Venus, Mars) at 5-minute resolution, outer planets (Jupiter, Saturn, Uranus, Neptune, Pluto) at 60-minute resolution
- **Automatic interpolation:** Query system automatically interpolates between stored points when higher resolution is needed for outer planets
- **TimescaleDB continuous aggregates:** Use native TimescaleDB feature for downsampling with automatic materialization
- **Last value aggregation:** Use last value in each time bucket for downsampled position data

### Memory Optimization Thresholds
- **Soft target with graceful degradation:** 30MB soft limit, 50MB hard limit
- **Percentage-based trigger:** Aggressive eviction kicks in at 90% of hard limit (45MB)
- **Config file section:** `[cache]` with `soft_limit_mb`, `hard_limit_mb`, and `eviction_threshold_pct` settings
- **Log warnings:** INFO log when exceeding soft limit, WARN at >90% of hard limit, ERROR when hard limit hit with forced evictions

### Storage Reduction Techniques
- **Major aspects only:** Store conjunction (0°), sextile (60°), square (90°), trine (120°), and opposition (180°) aspects only
- **Calculate minors on-demand:** Semisextile, quincunx, and other minor aspects calculated as needed
- **Keep all data indefinitely:** Historical planetary positions are valuable for electoral astrology and never change
- **No compression:** Avoid TimescaleDB compression to prevent decompression overhead during common historical reference queries
- **No additional filtering:** Multi-resolution storage provides sufficient storage savings

### Performance Monitoring Approach
- **Periodic background checks:** Run benchmarks every hour during low-traffic periods
- **Percentage degradation alerts:** WARN when >20% slower than baseline, ERROR when >50% slower
- **Baseline establishment:** First successful benchmark run establishes the baseline for comparison
- **Database table storage:** Store results in `benchmark_results` table for trend analysis
- **Periodic heap snapshots:** Capture heap size every 5 minutes during benchmarks
- **Memory growth alerts:** Alert if memory grows >10% week-over-week

### Claude's Discretion
- Exact implementation details of interpolation algorithm for outer planets
- Specific TimescaleDB continuous aggregate refresh policies
- Benchmark scheduling logic (how to determine "low-traffic periods")
- Heap snapshot implementation details (which profiling crate to use)
- Config file format specifics (TOML structure beyond the cache section)

</decisions>

<specifics>
## Specific Ideas

- Historical reference searches are common in electoral astrology, so query performance on old data is as important as recent data
- Outer planets move slowly (Jupiter ~30°/year), so hourly resolution with interpolation is astrologically sufficient
- The 51× speedup achieved in Phase 3 should be maintained or improved, not degraded by new features
- Memory pressure should be handled gracefully without failing queries

</specifics>

<deferred>
## Deferred Ideas

- Minor aspect pre-calculation if query patterns show heavy use
- Machine learning-based query prediction for cache pre-warming
- Distributed caching for multi-instance deployments
- Real-time WebSocket updates for planetary position changes

</deferred>

---

*Phase: 04-performance*
*Context gathered: 2026-02-27*
