# Requirements: Astro Clock v1.1

**Defined:** 2026-03-01
**Core Value:** Generate accurate, visually appealing astrological charts from any date/time/location with minimal configuration

## v1.1 Requirements (Job System)

### Job Infrastructure

- [x] **JOB-01**: Create jobs table for tracking job state (pending, in-process, complete, failed)
- [x] **JOB-02**: Create loaded_days tracking table for incremental data loading
- [x] **JOB-03**: Job states: pending → in-process → (complete | failed)
- [x] **JOB-04**: Jobs have unique job-id (UUID or sequential)
- [x] **JOB-05**: Jobs store payload (parameters), result (JSON), error details on failure
- [x] **JOB-06**: Support both synchronous and asynchronous execution modes

### Data Loading

- [x] **LOAD-06**: CLI command to load date range: `astro-clock load --start YYYY-MM-DD --days N [--sync]`
- [x] **LOAD-07**: API endpoint: `POST /api/v1/load` with `{start_date, days, sync?}`
- [x] **LOAD-08**: Day-level incremental loading — skip already-loaded days
- [x] **LOAD-09**: Loading jobs populate planet_positions, aspects, lunar_conditions tables
- [x] **LOAD-10**: Track loaded days in tracking table for resume capability

### Named Queries

- [x] **QUERY-06**: Named query "wedding" — find auspicious wedding dates
- [ ] **QUERY-07**: Named query "project" — find good dates to start projects
- [ ] **QUERY-08**: Named query "travel" — find favorable travel dates
- [x] **QUERY-09**: Named queries accept date range parameters (start_date, days)
- [ ] **QUERY-10**: Named queries intelligently load missing data before executing
- [ ] **QUERY-11**: Named queries support sync/async execution modes

### CLI Interface

- [x] **CLI-01**: Command `astro-clock query wedding --start YYYY-MM-DD --days N [--sync]`
- [x] **CLI-02**: Command `astro-clock query project --start YYYY-MM-DD --days N [--sync]`
- [x] **CLI-03**: Command `astro-clock query travel --start YYYY-MM-DD --days N [--sync]`
- [x] **CLI-04**: Command `astro-clock job status <job-id>` — get job status and results
- [x] **CLI-05**: Command `astro-clock job list` — list recent jobs with statuses

### HTTP API

- [ ] **API-01**: Endpoint `POST /api/v1/query/wedding` with `{start_date, days, sync?}`
- [ ] **API-02**: Endpoint `POST /api/v1/query/project` with `{start_date, days, sync?}`
- [ ] **API-03**: Endpoint `POST /api/v1/query/travel` with `{start_date, days, sync?}`
- [ ] **API-04**: Endpoint `GET /api/v1/jobs/{job-id}` — get job status and results
- [ ] **API-05**: Endpoint `GET /api/v1/jobs` — list recent jobs
- [ ] **API-06**: Job result response includes: `status`, `created_at`, `completed_at`, `result` (JSON) or `error` (object)

### Job Results

- [x] **RESULT-01**: Synchronous jobs block until complete, return result directly
- [x] **RESULT-02**: Asynchronous jobs return immediately with job-id
- [x] **RESULT-03**: Job status endpoint returns: `{job_id, status, payload, result?, error?, created_at, updated_at}`
- [x] **RESULT-04**: Failed jobs include error code and message in error field
- [x] **RESULT-05**: Complete jobs include query results in result field (JSON)

## v2 Requirements (Future)

### Advanced Queries

- **ADV-01**: Find grand trine configurations
- **ADV-02**: Find T-square and grand cross patterns
- **ADV-03**: Calculate transits relative to natal chart
- **ADV-04**: Planetary ingress detection (sign changes)

### Export Features

- **EXP-01**: Export chart data as JSON
- **EXP-02**: Export chart data as CSV
- **EXP-03**: Batch export for date ranges

### Configurability

- **CONFIG-01**: Named queries defined in config files (JSON/YAML)
- **CONFIG-02**: Custom query templates without code changes

## Out of Scope

| Feature | Reason |
|---------|--------|
| Real-time job progress streaming | Polling sufficient for v1.1 |
| Job cancellation | Add if needed based on usage |
| Job retry logic | Manual retry via re-submit for now |
| Query result caching | Database is already the cache |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| JOB-01 | Phase 5 | Planned |
| JOB-02 | Phase 5 | Planned |
| JOB-03 | Phase 5 | Planned |
| JOB-04 | Phase 5 | Planned |
| JOB-05 | Phase 5 | Planned |
| JOB-06 | Phase 5 | Planned |
| LOAD-06 | Phase 6 | Planned |
| LOAD-07 | Phase 6 | Planned |
| LOAD-08 | Phase 6 | Planned |
| LOAD-09 | Phase 6 | Planned |
| LOAD-10 | Phase 6 | Planned |
| QUERY-06 | Phase 7 | Planned |
| QUERY-07 | Phase 7 | Planned |
| QUERY-08 | Phase 7 | Planned |
| QUERY-09 | Phase 7 | Planned |
| QUERY-10 | Phase 7 | Planned |
| QUERY-11 | Phase 7 | Planned |
| CLI-01 | Phase 8 | Planned |
| CLI-02 | Phase 8 | Planned |
| CLI-03 | Phase 8 | Planned |
| CLI-04 | Phase 8 | Complete |
| CLI-05 | Phase 8 | Complete |
| API-01 | Phase 8 | Planned |
| API-02 | Phase 8 | Planned |
| API-03 | Phase 8 | Planned |
| API-04 | Phase 8 | Planned |
| API-05 | Phase 8 | Planned |
| API-06 | Phase 8 | Planned |
| RESULT-01 | Phase 5 | Planned |
| RESULT-02 | Phase 5 | Planned |
| RESULT-03 | Phase 6 | Planned |
| RESULT-04 | Phase 6 | Planned |
| RESULT-05 | Phase 7 | Planned |

**Coverage Validation:**
- v1.1 requirements: 28 unique requirements
- Mapped to phases: 28 ✓
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-01*
*Last updated: 2026-03-01 after roadmap creation*
