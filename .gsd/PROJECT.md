# Project

## What This Is

Astro Clock is a Rust astrology calculation engine with Swiss Ephemeris bindings. It computes planetary positions, aspects, lunar conditions, retrograde periods, and void-of-course periods. It stores computed data in TimescaleDB and exposes it through a REST API (axum) and CLI (clap). Query types include wedding date optimization, project timing, and travel planning — each scoring date ranges against astrological criteria.

## Core Value

Accurate ephemeris data, computed via Swiss Ephemeris, served through queryable API endpoints with deterministic results.

## Current State

- Swiss Ephemeris integration computes positions, aspects, lunar conditions for 10 bodies
- TimescaleDB storage with hypertables, continuous aggregates, multi-resolution data
- REST API: chart rendering, data loading, named queries (wedding/project/travel), job management
- CLI: load, query, job status/list commands
- Integration test suite: 35 tests (21 API + 14 CLI) against seeded test database
- SVG glyph system for zodiac/planet rendering
- Memory-monitored chunk cache with LRU eviction

## Architecture / Key Patterns

- **Language:** Rust 2024 edition
- **Web framework:** Axum 0.7 with State extractor, feature-gated behind `db`
- **Database:** PostgreSQL/TimescaleDB via sqlx 0.8, migrations in `migrations/`
- **CLI:** Clap 4 with derive macros, feature-gated behind `db`
- **Jobs:** Async job queue with `FOR UPDATE SKIP LOCKED` claiming, state machine (pending → in_process → complete/failed)
- **Testing:** Tower::ServiceExt for API tests, assert_cmd for CLI tests, `--ignored` flag for DB-gated integration tests
- **Error handling:** thiserror for domain errors, `(StatusCode, Json).into_response()` pattern for API errors
- **Query construction:** sqlx QueryBuilder for dynamic WHERE clauses

## Capability Contract

See `.gsd/REQUIREMENTS.md` for the explicit capability contract, requirement status, and coverage mapping.

## Milestone Sequence

- [x] M001: Migration — Database schema migration to TimescaleDB with hypertables, indexes, and continuous aggregates
- [x] M002: Build Fix & Integration Test Suite — Fix db-gated compilation errors, create seeded test database, write integration tests for all API routes and CLI commands
- [ ] M003: Job Maintenance & Enhanced List API — Enhanced list filters (multi-value status/job_type, date ranges), cursor-based pagination with stable anchors, DELETE endpoint with in_process guard
