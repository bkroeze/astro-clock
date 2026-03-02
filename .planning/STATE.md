# Project State: Astro Clock v1.1

**Status:** Phase 7 complete, ready for Phase 8
**Last Updated:** 2026-03-02T00:03:59Z

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
**Plan:** 03 (completed)
**Status:** 3/3 plans complete — Query API endpoints with sync/async execution — Phase 7 complete

---

## v1.1 Phase Status

| Phase | Status | Requirements | Progress | Dependencies |
|-------|--------|--------------|----------|--------------|
| 5 — Job Infrastructure | ✅ Complete | 8 | 100% (8/8) | Phase 4 (complete) |
| 6 — Data Loading | ✅ Complete | 7 | 100% (7/7) | Phase 5 |
| 7 — Named Queries | ✅ Complete | 7 | 100% (7/7) | Phase 6 |
| 8 — CLI & API Integration | 📋 Planned | 11 | 0% | Phase 7 |

---

## Progress Bar

```
v1.1 Progress: [█████████████████░░░] 68% (19/28 requirements)

Phase 5: [████████████████████] 100% (8/8)
Phase 6: [████████████████████] 100% (7/7)
Phase 7: [████████████████████] 100% (7/7)
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
23. **202 ACCEPTED for async mode** (07-03) — HTTP 202 indicates job acceptance for async processing, 200 OK for sync completion
24. **Query name whitelist validation** (07-03) — Explicit whitelist (wedding, project, travel) prevents injection attacks
25. **Consistent route handler patterns** (07-03) — queries.rs mirrors jobs.rs structure for maintainability

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
│   └── query.rs     # QueryJobHandler with integration tests
├── registry.rs      # QueryTemplateRegistry
└── QueryJobHandler with auto data loading

src/queries/
├── wedding.rs       # Wedding date queries (Venus aspects)
├── project.rs       # Project start queries (Mercury direct)
├── travel.rs        # Travel date queries (Moon/Mercury)
└── types.rs         # Criteria structs, candidate types

src/server/routes/
├── mod.rs           # Route module exports
├── jobs.rs          # POST /api/v1/load, GET /api/v1/jobs/:id
└── queries.rs       # POST /api/v1/query/:query_name (new)
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
2. ✅ Phase 7 complete: Named Queries — all 3 plans complete
3. ⏳ Phase 8: CLI & API Integration — ready to begin

---

## Session Continuity

**Last Session:** 2026-03-02 — Completed 07-03: Query API Endpoints
**Stopped At:** Completed 07-03-PLAN.md

**For Next Session:**
- Phase 7 complete: All 3 plans done
- Query API endpoints implemented with POST /api/v1/query/:query_name
- Sync and async execution modes both available
- Input validation for query names, dates, and day ranges
- All 127 tests passing (5 new integration tests)
- Ready for Phase 8: CLI & API Integration
- Key context: HTTP query endpoints, request/response types, server route integration

---

*State tracking started: 2026-02-24*
*v1.0 completed: 2026-03-01*
*v1.1 roadmap created: 2026-03-01*
