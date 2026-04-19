# M003: Job Maintenance & Enhanced List API

**Gathered:** 2026-04-19
**Status:** Ready for planning

## Project Description

Enhanced job management API for the astro-clock server. The existing `GET /api/v1/jobs` endpoint has basic single-value status filtering and offset/limit pagination. This milestone adds multi-value filters, cursor-based pagination with stable anchors, and a DELETE endpoint.

## Why This Milestone

The current job list endpoint is minimal — single status filter, offset pagination that shifts on inserts, no way to delete jobs. Operational use requires filtering by multiple statuses and job types simultaneously, time-bounded queries, stable page boundaries, and cleanup capability.

## User-Visible Outcome

### When this milestone is complete, the user can:

- Filter jobs by multiple statuses (`status=complete,failed`) and job types (`job_type=load,query`) in a single request
- Filter jobs by creation date range (`created_after=2025-01-01&created_before=2025-03-01`)
- Navigate paginated job lists via `next`/`prev` URLs that remain stable across concurrent inserts
- Delete individual jobs via `DELETE /api/v1/jobs/:id`, with a 409 guard preventing deletion of in_process jobs

### Entry point / environment

- Entry point: REST API endpoints under `/api/v1/jobs`
- Environment: Local dev with seeded test database
- Live dependencies involved: PostgreSQL/TimescaleDB

## Completion Class

- Contract complete means: API endpoints accept and return the documented shapes; unit tests pass for request/response types
- Integration complete means: Endpoints work against the seeded test database with real job data; cursor pagination is stable; DELETE guards work correctly
- Operational complete means: None (no daemon/service lifecycle)

## Final Integrated Acceptance

To call this milestone complete, we must prove:

- `GET /api/v1/jobs?status=complete,failed&job_type=load&created_after=2025-01-01` returns filtered results from the real database
- Paginating through 50+ jobs via `next`/`prev` URLs produces stable, non-overlapping results even when new jobs are inserted between page fetches
- `DELETE /api/v1/jobs/:id` returns 204 for completed jobs, 404 for nonexistent, 409 for in_process
- Existing M002 integration tests continue passing without modification

## Architectural Decisions

### Cursor encoding format

**Decision:** Opaque base64-encoded cursor containing `(created_at, id)` tuple

**Rationale:** Base64 opacity allows the wire format to evolve (e.g., adding filter state) without breaking clients. The cursor encodes a `(DateTime<Utc>, Uuid)` pair as JSON then base64-encodes it. Clients treat it as an opaque token.

**Alternatives Considered:**
- Readable `timestamp|uuid` format — simpler to debug but locks the format, harder to extend with filter state

### Pagination model: filters + cursor

**Decision:** First request accepts filter parameters (`status`, `job_type`, `created_after`, `created_before`) and `count`. Response includes `next`/`prev` URLs with encoded cursors. Subsequent requests can use either cursors (from response URLs) or fresh filter params.

**Rationale:** Keeps the API explorable — clients can construct queries from scratch without needing a session. Cursors preserve filter state so `next`/`prev` don't lose the original query context.

**Alternatives Considered:**
- Cursor-only (all state in cursor) — less discoverable, harder to construct ad-hoc queries

### DELETE scope: single job, in_process guard

**Decision:** `DELETE /api/v1/jobs/:id` only. Returns 409 if job is in_process. No bulk delete.

**Rationale:** Single-job DELETE keeps scope tight. The 409 guard prevents orphaning a worker actively executing the job — an orphaned worker silently continuing against a deleted job would be a difficult bug to diagnose. Bulk delete is deferred.

**Alternatives Considered:**
- Allow deleting any status — simpler but risks orphaning workers
- Include bulk delete — adds scope, deferred to future milestone

## Error Handling Strategy

Follow existing patterns: `(StatusCode, Json(serde_json::json!(error))).into_response()` for all error responses.

- 400 Bad Request for invalid filter values (unrecognized status/job_type, malformed dates, invalid cursor)
- 404 Not Found for DELETE on nonexistent job
- 409 Conflict for DELETE on in_process job
- 500 Internal Server Error for database failures (existing pattern)

Error response shape matches existing `ErrorResponse { error: String, message: String }`.

## Risks and Unknowns

- Cursor pagination SQL row comparison — PostgreSQL supports `(created_at, id) < ($1, $2)` row comparison, but sqlx QueryBuilder doesn't have first-class support for tuple comparisons. Need to verify the SQL compiles and performs correctly with the composite index.

## Existing Codebase / Prior Art

- `src/server/routes/jobs.rs` — Current job handlers (`load_handler`, `get_job_handler`, `list_jobs_handler`), request/response types, `build_job_response` helper
- `src/jobs/repository.rs` — `JobRepository` with `list_jobs` (already uses QueryBuilder for dynamic WHERE), `count_jobs`, `get_job`
- `src/jobs/types.rs` — `Job`, `JobType`, `JobStatus` enums with string conversion helpers
- `src/jobs/error.rs` — `JobError` with `NotFound` variant
- `src/server/mod.rs` — Router setup with route definitions
- `tests/api_integration.rs` — Integration test patterns using tower::ServiceExt

## Relevant Requirements

- R004 — Multi-value status and job_type filters (primary: S01)
- R005 — Date range filters (primary: S01)
- R006 — Cursor-based pagination with stable anchors (primary: S02)
- R007 — Auto-generated next/prev URLs (primary: S02)
- R008 — DELETE with in_process guard (primary: S03)
- R009 — Integration tests for all new endpoints (primary: S03)

## Scope

### In Scope

- Enhanced GET /api/v1/jobs with multi-value status/job_type filters
- Date range filters (created_after, created_before)
- Cursor-based pagination replacing offset/limit
- Auto-generated next/prev URLs in response
- DELETE /api/v1/jobs/:id with 409 guard for in_process
- Integration tests for all new functionality
- Unit tests for new request/response types

### Out of Scope / Non-Goals

- Authentication/authorization (future milestone)
- Bulk delete endpoint (deferred)
- Job retry/resubmit endpoint
- CLI changes for new filters (CLI uses sync mode mostly)
- Changes to existing load/query/job-status endpoints

## Technical Constraints

- All new code behind `#[cfg(feature = "db")]` feature gate
- Must not break existing API consumers — the list endpoint's response shape changes (adds fields) but remains backward-compatible
- Database queries must use existing `idx_jobs_status_type` and `idx_jobs_created_at` indexes
- Cursor tokens must be URL-safe (base64url encoding)

## Integration Points

- PostgreSQL/TimescaleDB — existing jobs table, existing indexes
- Existing integration tests — must continue passing unchanged

## Testing Requirements

- Unit tests for: ListJobsRequest deserialization (multi-value filters, dates, cursor), cursor encode/decode, response serialization
- Integration tests for: filtered queries, cursor pagination forward/backward, next/prev URL generation, DELETE success/404/409, edge cases (empty results, single page, invalid cursor)
- Follow M02 patterns: tower::ServiceExt, `--ignored`, `just test-integration`

## Acceptance Criteria

### S01: Enhanced list filters
- `GET /api/v1/jobs?status=complete,failed` returns only matching jobs
- `GET /api/v1/jobs?job_type=load,query` returns only matching jobs
- `GET /api/v1/jobs?created_after=2025-01-01&created_before=2025-03-01` returns time-bounded results
- Invalid filter values return 400 with descriptive error

### S02: Cursor-based pagination
- `GET /api/v1/jobs?count=5` returns first page with `next` URL
- Following `next` URL returns next page with both `next` and `prev`
- Pages are stable — inserting new jobs doesn't shift page boundaries
- `prev` is null on first page; `next` is null on last page
- Cursor tokens are opaque base64 strings

### S03: DELETE + integration tests
- `DELETE /api/v1/jobs/:id` on completed job returns 204
- `DELETE /api/v1/jobs/:id` on nonexistent returns 404
- `DELETE /api/v1/jobs/:id` on in_process returns 409
- All existing M02 integration tests still pass
- New integration tests cover all new functionality

## Open Questions

- None — all decisions confirmed in discussion
