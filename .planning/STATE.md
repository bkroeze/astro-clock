# Project State: Astro Clock v1.1

**Status:** Phase 7 in progress
**Last Updated:** 2026-03-01T23:52:54Z

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
**Phase:** 7 — Named Queries
**Plan:** 02 (completed)
**Status:** Plan 01 complete — QueryJobHandler and QueryTemplateRegistry — 1/3 plans complete in Phase 7

---

## v1.1 Phase Status

| Phase | Status | Requirements | Progress | Dependencies |
|-------|--------|--------------|----------|--------------|
| 5 — Job Infrastructure | ✅ Complete | 8 | 100% (8/8) | Phase 4 (complete) |
| 6 — Data Loading | ✅ Complete | 7 | 100% (7/7) | Phase 5 |
| 7 — Named Queries | 🔄 In Progress | 7 | 14% (1/7) | Phase 6 |
| 8 — CLI & API Integration | 📋 Planned | 11 | 0% | Phase 7 |

---

## Progress Bar

```
v1.1 Progress: [█████████████░░░░░░░] 50% (14/28 requirements)

Phase 5: [████████████████████] 100% (8/8)
Phase 6: [████████████████████] 100% (7/7)
Phase 7: [█████░░░░░░░░░░░░░░░] 14% (1/7)
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
11. **Pool storage strategy** (06-01) — Store Pool<Postgres> in handler, create DatabasePool on demand for compatibility with both repository types
12. **Sequential day processing** (06-01) — Process days sequentially within job for Swiss Ephemeris thread safety
13. **Partial success handling** (06-01) — Mark job Complete if >=1 day loaded, Failed only if 0 progress
14. **Feature-gated handlers** (06-01) — #[cfg(feature = "db")] on handler modules for clean compilation
15. **Runtime-per-async-block pattern** (06-02) — Create tokio runtime for each async block in sync CLI context
16. **Structured result display** (06-02) — Deserialize LoadJobResult JSON for formatted terminal output
17. **impl IntoResponse for handlers** (06-03) — Cleaner handler signatures than Result<impl IntoResponse, StatusCode>
18. **Validate in handler before executor** (06-03) — Fast failure for bad input with clear 400 responses
19. **AppState with accessor methods** (06-03) — Private fields with executor() and get_pool() accessors
20. **Clone pool and payload before async blocks** (07-02) — Resolve lifetime issues in QueryTemplateRegistry closures by cloning DatabasePool and payload fields before moving into async blocks
21. **Mercury direct criteria for project/travel** (07-02) — Both queries filter for Mercury direct periods for clear planning and smooth travel
22. **Favorable sign selection by purpose** (07-02) — Aries for projects (initiation), Gemini for travel (movement), both exclude Scorpio/Capricorn

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
├── registry.rs      # QueryTemplateRegistry
└── QueryJobHandler with auto data loading

src/queries/
├── wedding.rs       # Wedding date queries (Venus aspects)
├── project.rs       # Project start queries (Mercury direct)
├── travel.rs        # Travel date queries (Moon/Mercury)
└── types.rs         # Criteria structs, candidate types
```

**Database Migrations:**
- `008_create_jobs.sql` — Jobs table with state tracking
- `009_create_loaded_days.sql` — Date range tracking

**Critical Patterns:**
- Dual connection pools: query_pool (3 conn) + job_pool (5 conn)
- `FOR UPDATE SKIP LOCKED` for race-free job claiming
- `spawn_blocking` + oneshot channel for CPU-intensive work
- Transactionally-staged job drains to prevent race conditions
- **Handler orchestration pattern**: Handler coordinates repositories/generators, implements JobHandler trait
- **Structured job results**: Type-safe JSON results using serde-serializable structs

### Performance Constraints

- Wedding date queries: <100ms for 60-day ranges
- Memory: Cache limit of ~30 days of data in memory
- Storage: Target <50GB/year for all planetary data

---

## Blockers

None — ready to begin Phase 5 planning.

---

## Next Steps

1. ✅ Phase 6 complete: Data Loading (LoadJobHandler)
2. 🔄 Phase 7 in progress: Named Queries — 07-02 complete, ready for next plan
3. ⏳ Phase 8: CLI & API Integration — waiting for Phase 7 completion

---

## Session Continuity

**Last Session:** 2026-03-01 — Completed 07-02: Project and Travel Queries
**Stopped At:** Completed 07-02-PLAN.md

**For Next Session:**
- Phase 7 in progress: 07-02 complete
- Project and Travel query functions implemented with Mercury direct + favorable Moon criteria
- QueryTemplateRegistry updated with actual query functions (no more placeholders)
- All 3 query templates (wedding, project, travel) execute real queries
- 115 tests passing
- Ready for Phase 8: CLI & API Integration
- Key context: QueryTemplateRegistry pattern, QueryJobHandler with auto data loading

---

*State tracking started: 2026-02-24*
*v1.0 completed: 2026-03-01*
*v1.1 roadmap created: 2026-03-01*
