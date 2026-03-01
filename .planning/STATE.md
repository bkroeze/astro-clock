# Project State: Astro Clock v1.1

**Status:** v1.1 Planning — Roadmap created
**Last Updated:** 2026-03-01

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
**Plan:** TBD (awaiting `/gsd-plan-phase 5`)
**Status:** Ready to begin planning

---

## v1.1 Phase Status

| Phase | Status | Requirements | Progress | Dependencies |
|-------|--------|--------------|----------|--------------|
| 5 — Job Infrastructure | 📋 Planned | 8 | 0% | Phase 4 (complete) |
| 6 — Data Loading | 📋 Planned | 7 | 0% | Phase 5 |
| 7 — Named Queries | 📋 Planned | 7 | 0% | Phase 6 |
| 8 — CLI & API Integration | 📋 Planned | 11 | 0% | Phase 7 |

---

## Progress Bar

```
v1.1 Progress: [░░░░░░░░░░░░░░░░░░░░] 0% (0/28 requirements)

Phase 5: [░░░░░░░░░░░░░░░░░░░░] 0%
Phase 6: [░░░░░░░░░░░░░░░░░░░░] 0%
Phase 7: [░░░░░░░░░░░░░░░░░░░░] 0%
Phase 8: [░░░░░░░░░░░░░░░░░░░░] 0%
```

---

## Decisions Log

### v1.1 Decisions (Pending)

None yet — planning phase.

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

1. Run `/gsd-plan-phase 5` to create detailed plan for Job Infrastructure
2. Review and approve Phase 5 plan
3. Execute Phase 5 plans
4. Repeat for Phases 6, 7, 8

---

## Session Continuity

**For Next Session:**
- ROADMAP.md has complete phase structure for v1.1
- REQUIREMENTS.md has updated traceability
- Ready to start planning Phase 5 (Job Infrastructure)
- Key context: Custom job queue using sqlx + PostgreSQL, dual connection pools, spawn_blocking pattern

---

*State tracking started: 2026-02-24*
*v1.0 completed: 2026-03-01*
*v1.1 roadmap created: 2026-03-01*
