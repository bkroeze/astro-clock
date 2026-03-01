# Project State: Astro Clock v1.1

**Status:** v1.1 Planning — Roadmap created
**Last Updated:** 2026-03-01T15:46:32Z

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
**Plan:** 03 (completed)
**Status:** Plan 03 complete — All 3 plans complete — Phase 5 complete

---

## v1.1 Phase Status

| Phase | Status | Requirements | Progress | Dependencies |
|-------|--------|--------------|----------|--------------|
| 5 — Job Infrastructure | ✅ Complete | 8 | 100% (8/8) | Phase 4 (complete) |
| 6 — Data Loading | 📋 Planned | 7 | 0% | Phase 5 |
| 7 — Named Queries | 📋 Planned | 7 | 0% | Phase 6 |
| 8 — CLI & API Integration | 📋 Planned | 11 | 0% | Phase 7 |

---

## Progress Bar

```
v1.1 Progress: [████████░░░░░░░░░░░░] 29% (8/28 requirements)

Phase 5: [████████████████████] 100% (8/8)
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
5. **String-Enum Bridge Pattern** (05-02) — DB uses strings for job_type/status, domain uses enums with parse methods for sqlx compatibility
6. **FOR UPDATE SKIP LOCKED** (05-02) — PostgreSQL row-level locking pattern ensures race-free job claiming across multiple workers
7. **State machine validation in code** (05-02) — JobStatus::can_transition_to validates state transitions before database updates
8. **spawn_blocking for CPU-intensive work** (05-03) — Swiss Ephemeris FFI calls run in spawn_blocking to avoid blocking async runtime
9. **Dual execution modes** (05-03) — execute_sync for CLI (blocking), execute_async for API (background with job-id)
10. **Handler registry pattern** (05-03) — Arc<dyn JobHandler> registered by JobType in HashMap for pluggable handlers

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

1. Move to Phase 6: Data Loading (LoadJobHandler)
2. Continue with Phase 7: Named Queries
3. Continue with Phase 8: CLI & API Integration

---

## Session Continuity

**Last Session:** 2026-03-01 — Completed 05-03: Job Executor
**Stopped At:** Completed 05-03-PLAN.md

**For Next Session:**
- JobExecutor with sync/async execution modes complete
- JobHandler trait defined for pluggable handlers
- spawn_blocking + oneshot pattern for CPU-intensive work
- Ready for Phase 6: Data Loading (LoadJobHandler)
- Key context: Handler registration, dual execution modes, background job processing

---

*State tracking started: 2026-02-24*
*v1.0 completed: 2026-03-01*
*v1.1 roadmap created: 2026-03-01*
