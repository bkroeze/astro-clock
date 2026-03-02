# Project State: Astro Clock v1.1

**Status:** Milestone complete
**Last Updated:** 2026-03-02T22:46:00Z

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
**Phase:** 8 — CLI & API Integration
**Plan:** 04 (completed)
**Status:** 4/5 plans complete — Dedicated query API endpoints — Phase 8 in progress

---

## v1.1 Phase Status

| Phase | Status | Requirements | Progress | Dependencies |
|-------|--------|--------------|----------|--------------|
| 5 — Job Infrastructure | ✅ Complete | 8 | 100% (8/8) | Phase 4 (complete) |
| 6 — Data Loading | ✅ Complete | 7 | 100% (7/7) | Phase 5 |
| 7 — Named Queries | ✅ Complete | 7 | 100% (7/7) | Phase 6 |
| 8 — CLI & API Integration | 🚧 In Progress | 11 | 45% (5/11) | Phase 7 |

---

## Progress Bar

```
v1.1 Progress: [████████████████████] 86% (24/28 requirements)

Phase 5: [████████████████████] 100% (8/8)
Phase 6: [████████████████████] 100% (7/7)
Phase 7: [████████████████████] 100% (7/7)
Phase 8: [████████░░░░░░░░░░░░] 45% (5/11)
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
26. **Nested subcommand pattern** (08-01) — Use `#[command(subcommand)]` attribute for nested CLI commands like `query wedding`
27. **Query CLI consistency** (08-01) — Follow Load command pattern for query commands: same validation, execution modes, and result formatting
28. **Feature-gated CLI commands** (08-01) — All database-dependent CLI commands behind `#[cfg(feature = "db")]` for clean compilation
29. **Cap limit at 100** (08-02) — Prevent excessive database queries in job list command
30. **Truncate job ID display** (08-02) — Show first 8 chars with "..." suffix for table readability

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

**New CLI Components:**
```
src/cli/
├── app.rs           # Extended with QueryCommands, JobCommands, handlers
```

**New CLI Commands:**
- `astro-clock query wedding --start YYYY-MM-DD --days N [--sync]`
- `astro-clock query project --start YYYY-MM-DD --days N [--sync]`
- `astro-clock query travel --start YYYY-MM-DD --days N [--sync]`
- `astro-clock job status <job-id>` — View job details and results
- `astro-clock job list [--status STATUS] [--limit N] [--offset N]` — List recent jobs

**Critical Patterns:**
- Dual connection pools: query_pool (3 conn) + job_pool (5 conn)
- `FOR UPDATE SKIP LOCKED` for race-free job claiming
- `spawn_blocking` + oneshot channel for CPU-intensive work
- Transactionally-staged job drains to prevent race conditions
- **Handler orchestration pattern**: Handler coordinates repositories/generators, implements JobHandler trait
- **Structured job results**: Type-safe JSON results using serde-serializable structs
- **Nested subcommand pattern**: `#[command(subcommand)]` for CLI command hierarchies
- **CLI handler pattern**: Match on QueryCommands variants to extract parameters

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
3. 🚧 Phase 8: CLI & API Integration — 4/5 plans complete
   - ✅ 08-01: CLI Query Subcommands
   - ✅ 08-02: CLI Job Management
   - ✅ 08-03: API list jobs endpoint
   - ✅ 08-04: Dedicated query API endpoints
   - ⏳ 08-05: End-to-End Testing

---

## Session Continuity

**Last Session:** 2026-03-02 — Completed 08-04: Dedicated Query API Endpoints
**Stopped At:** Completed 08-04-PLAN.md

**For Next Session:**
- Phase 8 in progress: 4/5 plans complete
- Dedicated query endpoints implemented: POST /api/v1/query/{wedding,project,travel}
- All endpoints accept {start_date, days, sync?} JSON body
- Route ordering ensures dedicated endpoints take precedence over generic endpoint
- 10 unit tests passing for query handler request/response types
- Ready for 08-05: End-to-End Testing
- All 131 tests passing (2 new CLI help tests for job commands)
- Ready for 08-03: CLI Integration Tests
- Key context: JobCommands enum, handle_job_command dispatcher, job status/list handlers

---

*State tracking started: 2026-02-24*
*v1.0 completed: 2026-03-01*
*v1.1 roadmap created: 2026-03-01*
