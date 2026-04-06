# T01: 02-data-loading 01

**Slice:** S02 — **Milestone:** M001

## Description

Create compact chunk data structures and add LRU cache dependency.

Purpose: Establish the foundation for memory-efficient data loading with ~4.5× size reduction through compact representations.
Output: src/database/chunk.rs with ChunkKey, ChunkData, and CompactPlanetPosition structs; Cargo.toml updated with lru dependency.

## Must-Haves

- [ ] Chunk data structure holds 1 day of data for all bodies
- [ ] Compact representation uses ~16 bytes per position (vs ~72 bytes)
- [ ] ChunkKey supports cache lookup by date
- [ ] Memory target of ~1MB per day is achievable

## Files

- `Cargo.toml`
- `src/database/chunk.rs`
- `src/database/mod.rs`
