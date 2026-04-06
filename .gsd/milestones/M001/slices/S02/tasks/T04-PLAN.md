# T04: 02-data-loading 04

**Slice:** S02 — **Milestone:** M001

## Description

Implement background pre-fetching of adjacent chunks.

Purpose: Improve query performance by loading neighboring chunks in advance while user processes current data.
Output: Pre-fetching functionality in ChunkManager that loads ±1 day chunks in background after main chunk load.

## Must-Haves

- [ ] Adjacent chunks (±1 day) are pre-fetched after main chunk load
- [ ] Pre-fetching happens in background via tokio::spawn
- [ ] Pre-fetch failures are silent (don't affect main query)
- [ ] Only immediate neighbors are pre-fetched (not recursive)

## Files

- `src/database/chunk_manager.rs`
