# T03: 02-data-loading 03

**Slice:** S02 — **Milestone:** M001

## Description

Implement Swiss Ephemeris chunk generation and database persistence.

Purpose: Generate astrological data on-demand when not in database and persist for future queries.
Output: ChunkGenerator with Swiss Ephemeris integration, aspect calculation, lunar conditions, and batch database persistence.

## Must-Haves

- [ ] Swiss Ephemeris generates data when not in database
- [ ] Generated data is persisted to database immediately
- [ ] Batch inserts used for performance (not individual rows)
- [ ] Database write failures don't fail the chunk load

## Files

- `src/database/chunk_generator.rs`
- `src/database/mod.rs`
