# T03: 06-data-loading 03

**Slice:** S06 — **Milestone:** M001

## Description

Add HTTP API endpoints for data loading and job status retrieval.

Purpose: Enable programmatic access to data loading via REST API with both synchronous and asynchronous execution modes, plus job status polling.

Output: Extended server with POST /api/v1/load and GET /api/v1/jobs/{job-id} endpoints.

## Must-Haves

- [ ] API has POST /api/v1/load endpoint accepting {start_date, days, sync?}
- [ ] API validates request parameters (date format, days range)
- [ ] Sync mode returns 200 with complete job result
- [ ] Async mode returns 202 with job-id and poll URL
- [ ] API has GET /api/v1/jobs/{job-id} endpoint returning full job details
- [ ] Job status response includes payload, result/error, timestamps
- [ ] Failed jobs include error code and message in error field

## Files

- `src/server/mod.rs`
- `src/server/routes/mod.rs`
- `src/server/routes/jobs.rs`
- `src/server/state.rs`
