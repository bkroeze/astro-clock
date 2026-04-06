# T05: 04-performance 05

**Slice:** S04 — **Milestone:** M001

## Description

Integrate memory monitoring into ChunkManager to enable automatic cache eviction based on memory pressure levels.

Purpose: Complete PERF-02 by connecting memory monitoring to cache management for automatic eviction (PERF-02)
Output: Updated ChunkManager with memory-aware eviction

## Must-Haves

- [ ] ChunkManager triggers aggressive eviction at 90% of hard limit (45MB)
- [ ] Memory pressure is logged at appropriate levels (INFO/WARN/ERROR)
- [ ] Cache eviction happens gracefully without failing queries
- [ ] Memory stats are accessible for monitoring

## Files

- `src/database/chunk_manager.rs`
- `src/lib.rs`
