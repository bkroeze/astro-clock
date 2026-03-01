# Project State: Astro Clock v1.1

**Status:** v1.1 Planning — Roadmap created
**Last Updated:** 2026-03-01T15:41:25Z

---

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-03-01)

**Core value:** Generate accurate, visually appealing astrological charts from any date/time/location with minimal configuration
**Current focus:** v1.1 Job System — Build unified job-based system for data loading and named queries

---

## Milestone Status

| Milestone | Status | Phases | Date |
|-----------|--------|--------|------|
| v1.0 MVP | ✅ Shipped | 4 | 2026-03-01 |
| v1.1 Job System | 📋 Planned | 4 (Phases 5-8) | — |

---

## Current Position

**Milestone:** v1.1 Job System
**Phase:** 5 — Job Infrastructure
**Plan:** 02 (awaiting execution)
**Status:** Plan 01 complete — 2 of 3 plans remaining

---

## v1.1 Phase Status

| Phase | Status | Requirements | Progress | Dependencies |
|-------|--------|--------------|----------|--------------|
| 5 — Job Infrastructure | 🚧 In Progress | 8 | 25% (2/8) | Phase 4 (complete) |
| 6 — Data Loading | 📋 Planned | 7 | 0% | Phase 5 |
| 7 — Named Queries | 📋 Planned | 7 | 0% | Phase 6 |
| 8 — CLI & API Integration | 📋 Planned | 11 | 0% | Phase 7 |

---

## Progress Bar

```
v1.1 Progress: [░░░░░░░░░░░░░░░░░░░░] 7% (2/28 requirements)

Phase 5: [████░░░░░░░░░░░░░░░░] 25% (2/8)
Phase 6: [░░░░░░░░░░░░░░░░░░░░] 0%
Phase 7: [░░░░░░░░░░░░░░░░░░░░] 0%
Phase 8: [░░░░░░░░░░░░░░░░░░░░] 0%
```

---

## Decisions Log

### v1.1 Decisions

1. **CHECK constraints for status validation** (05-01) — Data integrity at database level prevents invalid status values
2. **JSONB for flexible job parameters** (05-01) — Payload, result, error use JSONB to accommodate different job types without schema changes
3. **coverage_minutes range (0-1440)** (05-01) — Validates that a day cannot have more than 1440 minutes of data
4. **ON DELETE SET NULL for traceability** (05-01) — When jobs are deleted, loaded_days entries retain dates but lose traceability

### Key v1.0 Decisions (Carried Forward)

1. **TimescaleDB for time-series** — Optimized for astrological time-range queries
2. **Chunk-based LRU cache** — Balance memory vs performance
3. **Aspect summaries table** — 51× query speedup
4. **Multi-resolution storage** — By planet movement speed
5. **Major aspect filtering** — ~80% storage reduction
6. **Custom job queue** — PostgreSQL-based (not external library)

---

## Accumulated Context

### v1.1 Architecture (from Research)

**New Components:**
```
src/jobs/
├── mod.rs           # Public exports
├── types.rs         # Job, JobType, JobStatus, JobError
├── repository.rs    # JobRepository, LoadedDaysRepository
├── executor.rs      # JobExecutor — core orchestration
├── handlers/        # Job-type-specific handlers
│   ├── mod.rs
│   ├── load.rs      # LoadJobHandler
│   └── query.rs     # QueryJobHandler
└── registry.rs      # QueryTemplateRegistry
```

**Database Migrations:**
- `008_create_jobs.sql` — Jobs table with state tracking
- `009_create_loaded_days.sql` — Date range tracking

**Critical Patterns:**
- Dual connection pools: query_pool (3 conn) + job_pool (5 conn)
- `FOR UPDATE SKIP LOCKED` for race-free job claiming
- `spawn_blocking` + oneshot channel for CPU-intensive work
- Transactionally-staged job drains to prevent race conditions

### Performance Constraints

- Wedding date queries: <100ms for 60-day ranges
- Memory: Cache limit of ~30 days of data in memory
- Storage: Target <50GB/year for all planetary data

---

## Blockers

None — ready to begin Phase 5 planning.

---

## Next Steps

1. Execute 05-02: Job types and repository implementation
2. Execute 05-03: Job executor and worker infrastructure
3. Move to Phase 6: Data Loading
4. Continue with Phases 7, 8

---

## Session Continuity

**Last Session:** 2026-03-01 — Completed 05-01: Job System Database Schema
**Stopped At:** Completed 05-01-PLAN.md

**For Next Session:**
- Database schema created (migrations 008, 009)
- strum dependencies available for JobStatus enum derives
- Ready for 05-02: Job types and repository implementation
- Key context: Custom job queue using sqlx + PostgreSQL, dual connection pools, spawn_blocking pattern

---

*State tracking started: 2026-02-24*
*v1.0 completed: 2026-03-01*
*v1.1 roadmap created: 2026-03-01*
