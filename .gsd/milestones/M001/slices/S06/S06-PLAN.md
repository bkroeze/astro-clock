# S06: Data Loading

**Goal:** Create LoadJobHandler — a JobHandler trait implementation that orchestrates day-level incremental planetary data loading.
**Demo:** Create LoadJobHandler — a JobHandler trait implementation that orchestrates day-level incremental planetary data loading.

## Must-Haves


## Tasks

- [x] **T01: 06-data-loading 01** `est:5min`
  - Create LoadJobHandler — a JobHandler trait implementation that orchestrates day-level incremental planetary data loading.

Purpose: Bridge the job system (Phase 5) with the existing data generation infrastructure (ChunkGenerator) to enable resume-capable data loading.

Output: Handler module with LoadJobHandler that integrates ChunkGenerator, LoadedDaysRepository, and produces structured job results.
- [x] **T02: 06-data-loading 02** `est:3min`
  - Add `astro-clock load` CLI command for synchronous and asynchronous data loading.

Purpose: Enable users to load planetary data for date ranges via command line, with choice of blocking (sync) or background (async) execution.

Output: Extended CLI with load subcommand supporting --start, --days, and --sync flags.
- [x] **T03: 06-data-loading 03** `est:4min`
  - Add HTTP API endpoints for data loading and job status retrieval.

Purpose: Enable programmatic access to data loading via REST API with both synchronous and asynchronous execution modes, plus job status polling.

Output: Extended server with POST /api/v1/load and GET /api/v1/jobs/{job-id} endpoints.

## Files Likely Touched

- `src/jobs/handlers/mod.rs`
- `src/jobs/handlers/load.rs`
- `src/jobs/mod.rs`
- `src/cli/app.rs`
- `src/server/mod.rs`
- `src/server/routes/mod.rs`
- `src/server/routes/jobs.rs`
- `src/server/state.rs`
