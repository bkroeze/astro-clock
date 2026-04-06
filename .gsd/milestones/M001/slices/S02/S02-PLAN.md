# S02: Data Loading

**Goal:** Create compact chunk data structures and add LRU cache dependency.
**Demo:** Create compact chunk data structures and add LRU cache dependency.

## Must-Haves


## Tasks

- [x] **T01: 02-data-loading 01** `est:2 min`
  - Create compact chunk data structures and add LRU cache dependency.

Purpose: Establish the foundation for memory-efficient data loading with ~4.5× size reduction through compact representations.
Output: src/database/chunk.rs with ChunkKey, ChunkData, and CompactPlanetPosition structs; Cargo.toml updated with lru dependency.
- [x] **T02: 02-data-loading 02** `est:7min`
  - Implement ChunkManager with LRU cache and database loading capability.

Purpose: Provide the core data loading infrastructure with cache-first lookup and database fallback.
Output: ChunkManager struct with thread-safe LRU cache, database loading, and cache population on miss.
- [x] **T03: 02-data-loading 03** `est:5min`
  - Implement Swiss Ephemeris chunk generation and database persistence.

Purpose: Generate astrological data on-demand when not in database and persist for future queries.
Output: ChunkGenerator with Swiss Ephemeris integration, aspect calculation, lunar conditions, and batch database persistence.
- [x] **T04: 02-data-loading 04** `est:4min`
  - Implement background pre-fetching of adjacent chunks.

Purpose: Improve query performance by loading neighboring chunks in advance while user processes current data.
Output: Pre-fetching functionality in ChunkManager that loads ±1 day chunks in background after main chunk load.

## Files Likely Touched

- `Cargo.toml`
- `src/database/chunk.rs`
- `src/database/mod.rs`
- `src/database/chunk_manager.rs`
- `src/database/mod.rs`
- `src/database/chunk_generator.rs`
- `src/database/mod.rs`
- `src/database/chunk_manager.rs`
