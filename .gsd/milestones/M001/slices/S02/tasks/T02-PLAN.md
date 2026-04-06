# T02: 02-data-loading 02

**Slice:** S02 — **Milestone:** M001

## Description

Implement ChunkManager with LRU cache and database loading capability.

Purpose: Provide the core data loading infrastructure with cache-first lookup and database fallback.
Output: ChunkManager struct with thread-safe LRU cache, database loading, and cache population on miss.

## Must-Haves

- [ ] ChunkManager provides LRU cache with 30-chunk capacity
- [ ] Cache lookup order: cache → database (on miss)
- [ ] Database hits populate cache for future queries
- [ ] Thread-safe access via RwLock<LruCache<...>>

## Files

- `src/database/chunk_manager.rs`
- `src/database/mod.rs`
