# Astro Clock

## What This Is

A CLI application that generates astrological natal charts ("Chart of Now") and provides electoral astrology query capabilities for finding optimal timing. The app can render charts as PNG/WebP images, output data as Markdown tables, and query the database for auspicious dates. It also provides an HTTP server mode for API access.

## Core Value

Generate accurate, visually appealing astrological charts from any date/time/location with minimal configuration.

## Current State

**Shipped: v1.0 MVP (2026-03-01)**

- ✅ TimescaleDB time-series schema with 4 hypertables
- ✅ Chunk-based data loading with LRU cache (~30MB limit)
- ✅ Electoral astrology queries (wedding dates, VoC Moon, retrogrades, exact aspects)
- ✅ 51× query performance improvement (2.3s → 45ms)
- ✅ Multi-resolution storage (1min/5min/60min by planet speed)
- ✅ Automated benchmarking with regression detection
- ✅ Memory monitoring with automatic cache eviction

**Tech Stack:** Rust, TimescaleDB, Swiss Ephemeris, tiny-skia, sqlx

## Requirements

### Validated (v1.0)

- ✓ Generate natal charts for any date/time/location — existing
- ✓ Render chart wheel as PNG image — existing
- ✓ Render chart wheel as WebP image — existing
- ✓ Output planetary positions as Markdown table — existing
- ✓ Calculate and display astrological aspects — existing
- ✓ HTTP server mode for API access — existing
- ✓ Support for multiple house systems — existing
- ✓ TimescaleDB schema with aspect summaries — v1.0
- ✓ Chunk-based loading with LRU cache — v1.0
- ✓ Wedding date queries — v1.0
- ✓ Void-of-course Moon queries — v1.0
- ✓ Retrograde period queries — v1.0
- ✓ Exact aspect queries — v1.0
- ✓ Multi-resolution storage — v1.0
- ✓ Memory-aware cache eviction — v1.0
- ✓ Automated benchmarking — v1.0

### Validated (v1.1)

- ✓ cargo build --features db compiles with zero errors — M002/S01 (R001)
- ✓ Test database infrastructure with deterministic seed data — M002/S02 (R002)

### Active (v1.1)

- [x] Job system infrastructure (async/sync modes, job states)
- [x] Day-level incremental loading with tracking table
- [x] Named query templates (wedding, project start, travel dates)
- [x] CLI commands for loading and querying
- [x] HTTP API endpoints for jobs and queries
- [x] Job status polling and result retrieval

### Current Milestone: M002 Build Fix & Integration Tests

**Goal:** Fix broken `--features db` build, establish reproducible test database infrastructure, write integration tests proving db-gated code paths work end-to-end.

**Progress:**
- [x] S01: Fix build errors under --features db — **complete**
- [x] S02: Test database infrastructure — **complete**
- [ ] S03: Integration test suite

## Out of Scope

- Natal chart interpretation or predictions — not a divination tool
- Real-time chart updates — batch/offline processing only
- Mobile app — CLI and web API only
- User accounts or persistence — stateless operation
- Support for non-Earth locations — geocentric only

## Context

This is a Rust-based astrological calculation tool built around the Swiss Ephemeris library. The codebase uses a layered architecture with clear separation between CLI, domain logic, and external integrations. The rendering layer uses tiny-skia for 2D graphics.

**v1.1 shipped with:**
- M001 completed: Full job system with sync/async modes
- Wedding, project, and travel query handlers
- HTTP API and CLI for all operations

## Milestone Sequence

- [x] M001: Job System — Unified job-based data loading and named queries
- [ ] M002: Build Fix & Integration Tests — Fix db build, add integration test suite (S01, S02 done; S03 remaining)

## Constraints

- **Tech Stack**: Rust with Swiss Ephemeris bindings
- **Database**: PostgreSQL with TimescaleDB extension
- **Performance**: Wedding date queries complete in <100ms for 60-day ranges
- **Memory**: Cache limit of ~30 days of data in memory
- **Storage**: Target <50GB/year for all planetary data

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Use TimescaleDB for time-series data | Optimized for time-range queries, continuous aggregates | ✓ Good |
| Chunk-based loading with LRU cache | Balance memory usage vs query performance | ✓ Good |
| Pre-calculate aspect summaries | Avoid O(n²) correlated subqueries in aspect counting | ✓ Good (51× speedup) |
| Multi-resolution storage by body speed | Moon at 1min, inner at 5min, outer at 60min | ✓ Good |
| Major aspect filtering | ~80% storage reduction with 8° orb | ✓ Good |
| Feature-gated database support | Keep core chart generation lightweight | ✓ Good |
| tiny-skia for rendering | Lightweight 2D graphics, no heavy dependencies | ✓ Good |
| Explicit deref for Rust 2024 string concat | `*sign` to resolve `&&str` → `&str` for `Add` trait | ✓ Good |
| Separate TEST_PG_URL for test database | Prevents accidental destruction of production data | ✓ Good |
| Swiss Ephemeris seed data via CLI load | Deterministic output, tests real data pipeline | ✓ Good |
| Justfile recipe for test DB provisioning | Separates infrastructure from test execution | ✓ Good |
| Seed data constants from DB queries | Avoids hardcoding astronomical calculations | ✓ Good |

---
*Last updated: 2026-04-15 after completing M002/S02*
