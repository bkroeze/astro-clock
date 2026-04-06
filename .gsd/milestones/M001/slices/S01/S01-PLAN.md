# S01: Database Schema

**Goal:** Create TimescaleDB hypertables and supporting tables for astrological time-series data.
**Demo:** Create TimescaleDB hypertables and supporting tables for astrological time-series data.

## Must-Haves


## Tasks

- [x] **T01: 01-database-schema 01** `est:2min`
  - Create TimescaleDB hypertables and supporting tables for astrological time-series data.

Purpose: Establish the foundation for efficient storage and querying of planet positions, aspects, lunar conditions, and house cusps with 1-minute resolution.
Output: Four migration files creating all necessary tables with proper TimescaleDB configuration.
- [x] **T02: 01-database-schema 02** `est:1 min`
  - Set up sqlx migration tooling and Justfile integration for database management.

Purpose: Enable developers to run migrations easily and ensure sqlx can discover migration files.
Output: Updated Justfile with migrate command, working migration discovery.
- [x] **T03: 01-database-schema 03** `est:5min`
  - Create schema verification script and update Rust schema module to reflect the new database structure.

Purpose: Enable verification that migrations applied correctly and provide Rust types for database operations.
Output: SQL verification script and updated Rust schema module with table structs.

## Files Likely Touched

- `migrations/001_create_hypertables.sql`
- `migrations/002_create_indexes.sql`
- `migrations/003_create_locations.sql`
- `migrations/004_create_aspect_summaries.sql`
- `Justfile`
- `Cargo.toml`
- `.env.example`
- `migrations/verify_schema.sql`
- `src/database/schema.rs`
