# T01: 04-performance 01

**Slice:** S04 — **Milestone:** M001

## Description

Implement multi-resolution storage using TimescaleDB continuous aggregates to reduce database storage by storing outer planets at lower resolution (60-minute) while maintaining 1-minute resolution for Moon and 5-minute for inner planets.

Purpose: Reduce annual storage from ~2GB/year to <50GB/year target by downsampling slowly-moving outer planets (PERF-01, PERF-03)
Output: Database migration for continuous aggregates, Rust module for multi-resolution data loading

## Must-Haves

- [ ] TimescaleDB continuous aggregates exist for 5-minute and 60-minute resolutions
- [ ] Planet positions are downsampled by category (Moon 1min, inner 5min, outer 60min)
- [ ] Continuous aggregates have automatic refresh policies configured
- [ ] Query system can load data from appropriate resolution based on planet category

## Files

- `migrations/006_create_continuous_aggregates.sql`
- `src/database/multi_resolution.rs`
- `src/database/mod.rs`
