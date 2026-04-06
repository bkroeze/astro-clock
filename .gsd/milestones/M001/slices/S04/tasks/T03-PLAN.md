# T03: 04-performance 03

**Slice:** S04 — **Milestone:** M001

## Description

Implement interpolation for outer planet queries and filter aspects to store only major aspects (conjunction, sextile, square, trine, opposition) to meet the <50GB/year storage target.

Purpose: Reduce storage by ~80% through aspect filtering while maintaining accuracy via interpolation for outer planets queried at higher resolution (PERF-03)
Output: Interpolation module, aspect filtering in chunk generator

## Must-Haves

- [ ] Linear interpolation handles 360° wraparound correctly for outer planets
- [ ] Only major aspects (conjunction, sextile, square, trine, opposition) are stored
- [ ] Minor aspects are calculated on-demand when needed
- [ ] Storage stays under 50GB/year target with multi-resolution + filtering

## Files

- `src/performance/interpolation.rs`
- `src/aspects/chunk_generator.rs`
- `src/aspects/mod.rs`
