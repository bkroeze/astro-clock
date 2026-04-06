# Requirements

## Active

### QUERY-07 — Named query "project" — find good dates to start projects

- Status: active
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Named query "project" — find good dates to start projects

### QUERY-08 — Named query "travel" — find favorable travel dates

- Status: active
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Named query "travel" — find favorable travel dates

### QUERY-10 — Named queries intelligently load missing data before executing

- Status: active
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Named queries intelligently load missing data before executing

### QUERY-11 — Named queries support sync/async execution modes

- Status: active
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Named queries support sync/async execution modes

## Validated

### JOB-01 — Create jobs table for tracking job state (pending, in-process, complete, failed)

- Status: validated
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Create jobs table for tracking job state (pending, in-process, complete, failed)

### JOB-02 — Create loaded_days tracking table for incremental data loading

- Status: validated
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Create loaded_days tracking table for incremental data loading

### JOB-03 — Job states: pending → in-process → (complete | failed)

- Status: validated
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Job states: pending → in-process → (complete | failed)

### JOB-04 — Jobs have unique job-id (UUID or sequential)

- Status: validated
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Jobs have unique job-id (UUID or sequential)

### JOB-05 — Jobs store payload (parameters), result (JSON), error details on failure

- Status: validated
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Jobs store payload (parameters), result (JSON), error details on failure

### JOB-06 — Support both synchronous and asynchronous execution modes

- Status: validated
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Support both synchronous and asynchronous execution modes

### LOAD-06 — CLI command to load date range: `astro-clock load --start YYYY-MM-DD --days N [--sync]`

- Status: validated
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

CLI command to load date range: `astro-clock load --start YYYY-MM-DD --days N [--sync]`

### LOAD-07 — API endpoint: `POST /api/v1/load` with `{start_date, days, sync?}`

- Status: validated
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

API endpoint: `POST /api/v1/load` with `{start_date, days, sync?}`

### LOAD-08 — Day-level incremental loading — skip already-loaded days

- Status: validated
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Day-level incremental loading — skip already-loaded days

### LOAD-09 — Loading jobs populate planet_positions, aspects, lunar_conditions tables

- Status: validated
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Loading jobs populate planet_positions, aspects, lunar_conditions tables

### LOAD-10 — Track loaded days in tracking table for resume capability

- Status: validated
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Track loaded days in tracking table for resume capability

### QUERY-06 — Named query "wedding" — find auspicious wedding dates

- Status: validated
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Named query "wedding" — find auspicious wedding dates

### QUERY-09 — Named queries accept date range parameters (start_date, days)

- Status: validated
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Named queries accept date range parameters (start_date, days)

### CLI-01 — Command `astro-clock query wedding --start YYYY-MM-DD --days N [--sync]`

- Status: validated
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Command `astro-clock query wedding --start YYYY-MM-DD --days N [--sync]`

### CLI-02 — Command `astro-clock query project --start YYYY-MM-DD --days N [--sync]`

- Status: validated
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Command `astro-clock query project --start YYYY-MM-DD --days N [--sync]`

### CLI-03 — Command `astro-clock query travel --start YYYY-MM-DD --days N [--sync]`

- Status: validated
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Command `astro-clock query travel --start YYYY-MM-DD --days N [--sync]`

### CLI-04 — Command `astro-clock job status <job-id>` — get job status and results

- Status: validated
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Command `astro-clock job status <job-id>` — get job status and results

### CLI-05 — Command `astro-clock job list` — list recent jobs with statuses

- Status: validated
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Command `astro-clock job list` — list recent jobs with statuses

### API-01 — Endpoint `POST /api/v1/query/wedding` with `{start_date, days, sync?}`

- Status: validated
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Endpoint `POST /api/v1/query/wedding` with `{start_date, days, sync?}`

### API-02 — Endpoint `POST /api/v1/query/project` with `{start_date, days, sync?}`

- Status: validated
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Endpoint `POST /api/v1/query/project` with `{start_date, days, sync?}`

### API-03 — Endpoint `POST /api/v1/query/travel` with `{start_date, days, sync?}`

- Status: validated
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Endpoint `POST /api/v1/query/travel` with `{start_date, days, sync?}`

### API-04 — Endpoint `GET /api/v1/jobs/{job-id}` — get job status and results

- Status: validated
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Endpoint `GET /api/v1/jobs/{job-id}` — get job status and results

### API-05 — Endpoint `GET /api/v1/jobs` — list recent jobs

- Status: validated
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Endpoint `GET /api/v1/jobs` — list recent jobs

### API-06 — Job result response includes: `status`, `created_at`, `completed_at`, `result` (JSON) or `error` (object)

- Status: validated
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Job result response includes: `status`, `created_at`, `completed_at`, `result` (JSON) or `error` (object)

### RESULT-01 — Synchronous jobs block until complete, return result directly

- Status: validated
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Synchronous jobs block until complete, return result directly

### RESULT-02 — Asynchronous jobs return immediately with job-id

- Status: validated
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Asynchronous jobs return immediately with job-id

### RESULT-03 — Job status endpoint returns: `{job_id, status, payload, result?, error?, created_at, updated_at}`

- Status: validated
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Job status endpoint returns: `{job_id, status, payload, result?, error?, created_at, updated_at}`

### RESULT-04 — Failed jobs include error code and message in error field

- Status: validated
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Failed jobs include error code and message in error field

### RESULT-05 — Complete jobs include query results in result field (JSON)

- Status: validated
- Class: core-capability
- Source: inferred
- Primary Slice: none yet

Complete jobs include query results in result field (JSON)

## Deferred

## Out of Scope
