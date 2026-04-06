# T01: 06-data-loading 01

**Slice:** S06 — **Milestone:** M001

## Description

Create LoadJobHandler — a JobHandler trait implementation that orchestrates day-level incremental planetary data loading.

Purpose: Bridge the job system (Phase 5) with the existing data generation infrastructure (ChunkGenerator) to enable resume-capable data loading.

Output: Handler module with LoadJobHandler that integrates ChunkGenerator, LoadedDaysRepository, and produces structured job results.

## Must-Haves

- [ ] LoadJobHandler implements JobHandler trait for JobType::Load
- [ ] Handler skips already-loaded days using LoadedDaysRepository::get_missing_dates
- [ ] Handler generates data via ChunkGenerator and saves to database
- [ ] Handler marks loaded days in tracking table via LoadedDaysRepository::mark_day_loaded
- [ ] Handler returns structured JSON result with loaded/skipped/failed date counts

## Files

- `src/jobs/handlers/mod.rs`
- `src/jobs/handlers/load.rs`
- `src/jobs/mod.rs`
