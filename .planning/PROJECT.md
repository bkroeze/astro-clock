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

### Current Milestone: v1.1 Job System

**Goal:** Build unified job-based system for data loading and named queries with async/sync execution modes.

**Target features:**
- Job orchestration for DB population (day-level incremental loading)
- Named query templates (starting with "wedding" query)
- Synchronous and asynchronous execution modes
- Job status tracking and result polling
- CLI commands and HTTP API endpoints
- Tracking table for loaded date ranges

## Active (v1.1)

- [ ] Job system infrastructure (async/sync modes, job states)
- [ ] Day-level incremental loading with tracking table
- [ ] Named query templates (wedding, project start, travel dates)
- [ ] CLI commands for loading and querying
- [ ] HTTP API endpoints for jobs and queries
- [ ] Job status polling and result retrieval

### Out of Scope

- Natal chart interpretation or predictions — not a divination tool
- Real-time chart updates — batch/offline processing only
- Mobile app — CLI and web API only
- User accounts or persistence — stateless operation
- Support for non-Earth locations — geocentric only

## Context

This is a Rust-based astrological calculation tool built around the Swiss Ephemeris library. The codebase uses a layered architecture with clear separation between CLI, domain logic, and external integrations. The rendering layer uses tiny-skia for 2D graphics.

**v1.0 shipped with:**
- 97 commits over 5 days
- 4 phases, 16 plans, all complete
- 22/22 v1 requirements delivered
- 51× query performance improvement
- ~80% storage reduction through aspect filtering

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

---
*Last updated: 2026-03-01 after starting v1.1 milestone planning*
