# Astro Clock

## What This Is

A CLI application that generates astrological natal charts ("Chart of Now") - visual representations of planetary positions at a specific moment in time. The app can render charts as PNG/WebP images or output data as Markdown tables. It also provides an HTTP server mode for API access to chart generation.

## Core Value

Generate accurate, visually appealing astrological charts from any date/time/location with minimal configuration.

## Requirements

### Validated

- ✓ Generate natal charts for any date/time/location — existing
- ✓ Render chart wheel as PNG image — existing
- ✓ Render chart wheel as WebP image — existing
- ✓ Output planetary positions as Markdown table — existing
- ✓ Calculate and display astrological aspects — existing
- ✓ HTTP server mode for API access — existing
- ✓ Support for multiple house systems (Placidus, Koch, Equal, etc.) — existing
- ✓ Configuration via RON config file — existing
- ✓ Swiss Ephemeris integration for accurate calculations — existing

### Active

- [ ] Electoral astrology query system for finding auspicious dates
- [ ] Database schema for time-series planetary data (TimescaleDB)
- [ ] Chunk-based data loading with LRU cache for performance
- [ ] Pre-calculated aspect summaries for fast queries
- [ ] Specialized query functions (wedding dates, void-of-course Moon, etc.)
- [ ] Multi-resolution data storage (different granularities per planet)
- [ ] Database connection pooling and async operations

### Out of Scope

- Natal chart interpretation or predictions — not a divination tool
- Real-time chart updates — batch/offline processing only
- Mobile app — CLI and web API only
- User accounts or persistence — stateless operation
- Support for non-Earth locations — geocentric only

## Context

This is a Rust-based astrological calculation tool built around the Swiss Ephemeris library. The codebase uses a layered architecture with clear separation between CLI, domain logic, and external integrations. The rendering layer uses tiny-skia for 2D graphics.

The project is expanding to support electoral astrology (finding optimal timing for events), which requires efficient querying of large date ranges. The implementation plan focuses on database optimizations, on-demand data loading, and query performance.

## Constraints

- **Tech Stack**: Rust with Swiss Ephemeris bindings — established
- **Database**: PostgreSQL with TimescaleDB extension — required for time-series
- **Performance**: Wedding date queries must complete in <100ms for 60-day ranges
- **Memory**: Cache limit of ~30 days of data in memory
- **Storage**: Target <50GB/year for all planetary data

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Use TimescaleDB for time-series data | Optimized for time-range queries, continuous aggregates | — Pending |
| Chunk-based loading with LRU cache | Balance memory usage vs query performance | — Pending |
| Pre-calculate aspect summaries | Avoid O(n²) correlated subqueries in aspect counting | — Pending |
| Feature-gated database support | Keep core chart generation lightweight | ✓ Good |
| tiny-skia for rendering | Lightweight 2D graphics, no heavy dependencies | ✓ Good |

---
*Last updated: 2026-02-24 after initialization*
