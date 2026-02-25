# Requirements: Astro Clock

**Defined:** 2026-02-24
**Core Value:** Generate accurate, visually appealing astrological charts from any date/time/location with minimal configuration

## v1 Requirements

### Chart Generation (Existing)

- [x] **CHART-01**: User can generate chart for current time with default location
- [x] **CHART-02**: User can specify custom date, time, and location
- [x] **CHART-03**: Chart renders as PNG image with zodiac wheel
- [x] **CHART-04**: Chart renders as WebP image
- [x] **CHART-05**: Chart outputs as Markdown table with positions
- [x] **CHART-06**: User can select from 11 house systems
- [x] **CHART-07**: Chart displays planetary aspects (conjunctions, oppositions, etc.)

### HTTP Server (Existing)

- [x] **SRV-01**: Server mode provides HTTP endpoint for chart generation
- [x] **SRV-02**: Endpoint accepts lat/lon/time query parameters
- [x] **SRV-03**: Endpoint returns PNG image directly

### Database Schema (Phase 1)

- [x] **DB-01**: Create aspect_summaries table for pre-aggregated aspect counts
- [x] **DB-02**: Add composite indexes for common query patterns
- [x] **DB-03**: Refactor house_cusps with normalized locations table
- [x] **DB-04**: Create planet_positions hypertable with TimescaleDB
- [x] **DB-05**: Create aspects hypertable for aspect data
- [x] **DB-06**: Create lunar_conditions hypertable for Moon data

### Data Loading (Phase 2)

- [x] **LOAD-01**: Implement ChunkManager with LRU cache
- [x] **LOAD-02**: Load data from database when available
- [x] **LOAD-03**: Generate data from Swiss Ephemeris when not in database
- [x] **LOAD-04**: Save generated data to database for future queries
- [x] **LOAD-05**: Background pre-fetching of adjacent chunks

### Query System (Phase 3)

- [ ] **QUERY-01**: Find optimal wedding dates (Venus aspects, Moon sign, no VoC)
- [ ] **QUERY-02**: Find void-of-course Moon periods
- [ ] **QUERY-03**: Find planetary retrograde periods
- [ ] **QUERY-04**: Find exact aspects within date range
- [ ] **QUERY-05**: Query completes in <100ms for 60-day ranges (cached)

### Performance (Phase 4)

- [ ] **PERF-01**: Multi-resolution storage (different intervals per planet)
- [ ] **PERF-02**: Memory usage <30MB for 30-day cache
- [ ] **PERF-03**: Database storage <50GB/year
- [ ] **PERF-04**: Wedding query 51× faster than baseline (2.3s → 45ms)

## v2 Requirements

### Advanced Queries

- **ADV-01**: Find grand trine configurations
- **ADV-02**: Find T-square and grand cross patterns
- **ADV-03**: Calculate transits relative to natal chart
- **ADV-04**: Planetary ingress detection (sign changes)

### Export Features

- **EXP-01**: Export chart data as JSON
- **EXP-02**: Export chart data as CSV
- **EXP-03**: Batch export for date ranges

## Out of Scope

| Feature | Reason |
|---------|--------|
| Natal chart interpretation | Not a divination tool, focus on calculation |
| Real-time updates | Batch/offline processing only |
| Mobile app | CLI and web API only |
| User accounts | Stateless operation |
| Non-Earth locations | Geocentric only |
| Sidereal zodiac | Tropical zodiac only for v1 |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CHART-01 | Existing | Complete |
| CHART-02 | Existing | Complete |
| CHART-03 | Existing | Complete |
| CHART-04 | Existing | Complete |
| CHART-05 | Existing | Complete |
| CHART-06 | Existing | Complete |
| CHART-07 | Existing | Complete |
| SRV-01 | Existing | Complete |
| SRV-02 | Existing | Complete |
| SRV-03 | Existing | Complete |
| DB-01 | Phase 1 | Complete |
| DB-02 | Phase 1 | Complete |
| DB-03 | Phase 1 | Complete |
| DB-04 | Phase 1 | Complete |
| DB-05 | Phase 1 | Complete |
| DB-06 | Phase 1 | Complete |
| LOAD-01 | Phase 2 | Complete |
| LOAD-02 | Phase 2 | Complete |
| LOAD-03 | Phase 2 | Complete |
| LOAD-04 | Phase 2 | Complete |
| LOAD-05 | Phase 2 | Complete |
| QUERY-01 | Phase 3 | Pending |
| QUERY-02 | Phase 3 | Pending |
| QUERY-03 | Phase 3 | Pending |
| QUERY-04 | Phase 3 | Pending |
| QUERY-05 | Phase 3 | Pending |
| PERF-01 | Phase 4 | Pending |
| PERF-02 | Phase 4 | Pending |
| PERF-03 | Phase 4 | Pending |
| PERF-04 | Phase 4 | Pending |

**Coverage:**
- v1 requirements: 22 total
- Mapped to phases: 22
- Unmapped: 0 ✓

---
*Requirements defined: 2026-02-24*
*Last updated: 2026-02-25 after completing Plan 01-02*
