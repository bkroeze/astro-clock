# Roadmap: Astro Clock

**Created:** 2026-02-24
**Last Updated:** 2026-03-01

---

## Milestones

- ✅ **v1.0 MVP** — Phases 1-4 (shipped 2026-03-01) — [Archive](milestones/v1.0-ROADMAP.md)
- 🚧 **v1.1 Job System** — Phases 5-8 (in planning)

---

## Phases

### v1.0 (Shipped)

- [x] **Phase 1: Database Schema** — TimescaleDB hypertables and migrations
- [x] **Phase 2: Data Loading** — Chunk-based loading with LRU cache
- [x] **Phase 3: Query System** — Electoral astrology queries
- [x] **Phase 4: Performance** — Optimization and benchmarking

### v1.1 (Planned)

- [ ] **Phase 5: Job Infrastructure** — Job tables, state machine, sync/async executor
- [ ] **Phase 6: Data Loading** — Day-level incremental loading with tracking
- [ ] **Phase 7: Named Queries** — Wedding, project, travel query templates
- [ ] **Phase 8: CLI & API Integration** — Commands, endpoints, and result handling

---

## Completed Work

<details>
<summary>✅ v1.0 MVP (Phases 1-4) — SHIPPED 2026-03-01</summary>

### Phase 1: Database Schema (3/3 plans)
- [x] 01-01: Create core hypertables and indexes — completed 2026-02-25
- [x] 01-02: Set up sqlx migration tooling — completed 2026-02-25
- [x] 01-03: Verify schema and update Rust types — completed 2026-02-25

### Phase 2: Data Loading (4/4 plans)
- [x] 02-01: Create compact chunk data structures — completed 2026-02-25
- [x] 02-02: Implement ChunkManager with LRU cache — completed 2026-02-25
- [x] 02-03: Implement Swiss Ephemeris generation — completed 2026-02-25
- [x] 02-04: Add background pre-fetching — completed 2026-02-25

### Phase 3: Query System (3/3 plans)
- [x] 03-01: Create query infrastructure — completed 2026-02-25
- [x] 03-02: Implement wedding and VoC queries — completed 2026-02-25
- [x] 03-03: Implement retrograde and aspect queries — completed 2026-02-25

### Phase 4: Performance (6/6 plans)
- [x] 04-01: Create TimescaleDB continuous aggregates — completed 2026-02-28
- [x] 04-02: Implement memory-aware cache eviction — completed 2026-03-01
- [x] 04-03: Add interpolation for outer planets — completed 2026-03-01
- [x] 04-04: Create automated benchmark runner — completed 2026-03-01
- [x] 04-05: Integrate memory monitoring with ChunkManager — completed 2026-03-01
- [x] 04-06: Integrate aspect filtering into calculate_aspects() — completed 2026-03-01

</details>

---

## Progress Summary

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Database Schema | v1.0 | 3/3 | ✅ Complete | 2026-02-25 |
| 2. Data Loading | v1.0 | 4/4 | ✅ Complete | 2026-02-25 |
| 3. Query System | v1.0 | 3/3 | ✅ Complete | 2026-02-25 |
| 4. Performance | v1.0 | 6/6 | ✅ Complete | 2026-03-01 |
| 5. Job Infrastructure | v1.1 | 0/TBD | 📋 Planned | — |
| 6. Data Loading | v1.1 | 0/TBD | 📋 Planned | — |
| 7. Named Queries | v1.1 | 0/TBD | 📋 Planned | — |
| 8. CLI & API Integration | v1.1 | 0/TBD | 📋 Planned | — |

---

## Phase Details

### Phase 5: Job Infrastructure
**Goal:** Establish foundational job system with state management and dual execution modes
**Depends on:** Phase 4 (v1.0 foundation)
**Requirements:** JOB-01, JOB-02, JOB-03, JOB-04, JOB-05, JOB-06, RESULT-01, RESULT-02
**Success Criteria** (what must be TRUE):
1. Jobs table exists with state tracking (pending → in-process → complete/failed)
2. Loaded days table tracks which dates have planetary data
3. Jobs have unique UUID identifiers stored in database
4. Jobs persist payload (JSON) and results (JSON) or error details
5. Synchronous execution blocks until job completes and returns result directly
6. Asynchronous execution returns job-id immediately for later polling
**Plans:** 3 plans
- [ ] 05-01-PLAN.md — Database schema (jobs table, loaded_days table, migrations)
- [ ] 05-02-PLAN.md — Job types and repository layer (Job struct, JobRepository, LoadedDaysRepository)
- [ ] 05-03-PLAN.md — Job executor (JobExecutor, sync/async modes, state machine)

### Phase 6: Data Loading
**Goal:** Enable day-level incremental loading with intelligent resume capability
**Depends on:** Phase 5 (job infrastructure)
**Requirements:** LOAD-06, LOAD-07, LOAD-08, LOAD-09, LOAD-10, RESULT-03, RESULT-04
**Success Criteria** (what must be TRUE):
1. CLI command `astro-clock load --start YYYY-MM-DD --days N` works synchronously
2. API endpoint `POST /api/v1/load` accepts start_date, days, and optional sync flag
3. Loading skips dates already present in loaded_days tracking table
4. Loading populates planet_positions, aspects, and lunar_conditions tables
5. Tracking table records loaded date ranges for resume capability
6. Failed loading jobs store error code and message for troubleshooting
**Plans:** TBD

### Phase 7: Named Queries
**Goal:** Deliver wedding, project, and travel query templates with automatic data loading
**Depends on:** Phase 6 (data loading)
**Requirements:** QUERY-06, QUERY-07, QUERY-08, QUERY-09, QUERY-10, QUERY-11, RESULT-05
**Success Criteria** (what must be TRUE):
1. Named query "wedding" finds auspicious wedding dates in date range
2. Named query "project" finds good dates to start projects
3. Named query "travel" finds favorable travel dates
4. Named queries accept start_date and days parameters
5. Named queries automatically load missing data before executing query
6. Named queries support both sync (block until complete) and async (return job-id) modes
7. Complete query jobs include results in JSON format
**Plans:** TBD

### Phase 8: CLI & API Integration
**Goal:** Expose complete job system through CLI commands and HTTP endpoints
**Depends on:** Phase 7 (named queries)
**Requirements:** CLI-01, CLI-02, CLI-03, CLI-04, CLI-05, API-01, API-02, API-03, API-04, API-05, API-06
**Success Criteria** (what must be TRUE):
1. CLI command `astro-clock query wedding --start YYYY-MM-DD --days N` returns results
2. CLI commands exist for project and travel queries with same interface
3. CLI command `astro-clock job status <job-id>` displays job status and results
4. CLI command `astro-clock job list` shows recent jobs with statuses
5. HTTP endpoint `POST /api/v1/query/{wedding,project,travel}` executes named queries
6. HTTP endpoint `GET /api/v1/jobs/{job-id}` returns job status with payload, result/error, timestamps
7. HTTP endpoint `GET /api/v1/jobs` lists recent jobs with pagination
8. Job result responses include status, created_at, completed_at, result (JSON) or error (object)
**Plans:** TBD

---

## Coverage

**v1.1 Requirements:** 28 total requirements

| Phase | Requirements | Count |
|-------|--------------|-------|
| Phase 5 | JOB-01, JOB-02, JOB-03, JOB-04, JOB-05, JOB-06, RESULT-01, RESULT-02 | 8 |
| Phase 6 | LOAD-06, LOAD-07, LOAD-08, LOAD-09, LOAD-10, RESULT-03, RESULT-04 | 7 |
| Phase 7 | QUERY-06, QUERY-07, QUERY-08, QUERY-09, QUERY-10, QUERY-11, RESULT-05 | 7 |
| Phase 8 | CLI-01, CLI-02, CLI-03, CLI-04, CLI-05, API-01, API-02, API-03, API-04, API-05, API-06 | 11 |
| **Total** | | **33** |

*Note: RESULT requirements appear in multiple phases as they span the job lifecycle*

**Coverage Validation:** ✓ All 28 unique v1.1 requirements mapped

---

*For detailed milestone information, see [milestones/v1.0-ROADMAP.md](milestones/v1.0-ROADMAP.md)*
