# S04: Performance

**Goal:** Implement multi-resolution storage using TimescaleDB continuous aggregates to reduce database storage by storing outer planets at lower resolution (60-minute) while maintaining 1-minute resolution for Moon and 5-minute for inner planets.
**Demo:** Implement multi-resolution storage using TimescaleDB continuous aggregates to reduce database storage by storing outer planets at lower resolution (60-minute) while maintaining 1-minute resolution for Moon and 5-minute for inner planets.

## Must-Haves


## Tasks

- [x] **T01: 04-performance 01** `est:3min`
  - Implement multi-resolution storage using TimescaleDB continuous aggregates to reduce database storage by storing outer planets at lower resolution (60-minute) while maintaining 1-minute resolution for Moon and 5-minute for inner planets.

Purpose: Reduce annual storage from ~2GB/year to <50GB/year target by downsampling slowly-moving outer planets (PERF-01, PERF-03)
Output: Database migration for continuous aggregates, Rust module for multi-resolution data loading
- [x] **T02: 04-performance 02** `est:3min`
  - Create memory monitoring infrastructure to track process memory usage and detect pressure levels for cache management.

Purpose: Provide memory monitoring capabilities that ChunkManager can use for eviction decisions (PERF-02)
Output: Memory monitoring module with pressure detection
- [x] **T03: 04-performance 03** `est:3min`
  - Implement interpolation for outer planet queries and filter aspects to store only major aspects (conjunction, sextile, square, trine, opposition) to meet the <50GB/year storage target.

Purpose: Reduce storage by ~80% through aspect filtering while maintaining accuracy via interpolation for outer planets queried at higher resolution (PERF-03)
Output: Interpolation module, aspect filtering in chunk generator
- [x] **T04: 04-performance 04** `est:2min`
  - Implement automated performance monitoring with scheduled benchmarks, regression detection, and database storage of results to ensure the 51× query speedup is maintained over time.

Purpose: Proactively detect performance regressions and maintain the 51× speedup target (PERF-04)
Output: Benchmark database table, automated benchmark runner with alerting
- [x] **T05: 04-performance 05** `est:2 min`
  - Integrate memory monitoring into ChunkManager to enable automatic cache eviction based on memory pressure levels.

Purpose: Complete PERF-02 by connecting memory monitoring to cache management for automatic eviction (PERF-02)
Output: Updated ChunkManager with memory-aware eviction
- [x] **T06: 04-performance 06** `est:2min`
  - Integrate the existing `is_major_aspect()` function into `calculate_aspects()` to filter minor aspects before storage, achieving the planned ~80% storage reduction.

Purpose: The aspect filtering infrastructure exists but is not connected. This gap closure connects the filtering logic to achieve the storage optimization goal.
Output: Modified chunk_generator.rs with integrated aspect filtering and no dead code warnings.

## Files Likely Touched

- `migrations/006_create_continuous_aggregates.sql`
- `src/database/multi_resolution.rs`
- `src/database/mod.rs`
- `src/performance/memory_monitor.rs`
- `src/performance/mod.rs`
- `Cargo.toml`
- `src/performance/interpolation.rs`
- `src/aspects/chunk_generator.rs`
- `src/aspects/mod.rs`
- `src/performance/benchmark.rs`
- `src/performance/mod.rs`
- `migrations/007_create_benchmark_results.sql`
- `src/database/chunk_manager.rs`
- `src/lib.rs`
- `src/database/chunk_generator.rs`
