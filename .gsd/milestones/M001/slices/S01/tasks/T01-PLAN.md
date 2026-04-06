# T01: 01-database-schema 01

**Slice:** S01 — **Milestone:** M001

## Description

Create TimescaleDB hypertables and supporting tables for astrological time-series data.

Purpose: Establish the foundation for efficient storage and querying of planet positions, aspects, lunar conditions, and house cusps with 1-minute resolution.
Output: Four migration files creating all necessary tables with proper TimescaleDB configuration.

## Must-Haves

- [ ] All hypertables created with proper chunk intervals (1 day)
- [ ] Locations table normalizes lat/lon data
- [ ] Aspect summaries table exists for pre-aggregated data
- [ ] Migration scripts are valid SQL and follow TimescaleDB conventions

## Files

- `migrations/001_create_hypertables.sql`
- `migrations/002_create_indexes.sql`
- `migrations/003_create_locations.sql`
- `migrations/004_create_aspect_summaries.sql`
