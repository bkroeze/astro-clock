# T03: 07-named-queries 03

**Slice:** S07 — **Milestone:** M001

## Description

Create HTTP API endpoints for named queries with sync/async execution modes and structured JSON responses.

Purpose: Enable HTTP clients to execute wedding, project, and travel queries via REST API with both blocking and non-blocking modes.
Output: src/server/routes/queries.rs, updated routes mod.rs and server mod.rs

## Must-Haves

- [ ] POST /api/v1/query/{wedding,project,travel} endpoint exists
- [ ] Query endpoint accepts start_date, days, and optional sync flag
- [ ] Sync mode blocks until complete and returns results directly
- [ ] Async mode returns job-id immediately for polling
- [ ] Invalid query names return 400 Bad Request
- [ ] Query results are serialized as JSON in response
- [ ] Wedding query end-to-end integration test verifies full job execution path

## Files

- `src/server/routes/queries.rs`
- `src/server/routes/mod.rs`
- `src/server/mod.rs`
- `src/jobs/handlers/query.rs`
