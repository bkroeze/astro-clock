---
id: T01
parent: S03
milestone: M003
key_files:
  - src/jobs/repository.rs
  - src/server/routes/jobs.rs
  - src/server/mod.rs
  - src/server/routes/mod.rs
key_decisions:
  - Two-step fetch-then-delete approach (get_job + delete_job) instead of a single conditional DELETE query — keeps business logic in the handler layer and reuses the existing get_job for the existence/status check
  - Race condition handling: if a job disappears between get and delete, returns 404 rather than erroring — idempotent and safe
  - No new JobError variants — the handler uses inline status checks and maps directly to HTTP status codes using the existing ErrorResponse type
duration: 
verification_result: passed
completed_at: 2026-04-19T19:34:31.615Z
blocker_discovered: false
---

# T01: Add DELETE /api/v1/jobs/:id endpoint with 204/404/409 responses

**Add DELETE /api/v1/jobs/:id endpoint with 204/404/409 responses**

## What Happened

Implemented the DELETE endpoint for job deletion following a two-step approach: fetch via `get_job` to check existence and status, then delete if the job is not in_process.

**Changes made:**

1. **`src/jobs/repository.rs`** — Added `delete_job(&self, id: Uuid) -> JobResult<bool>` method that executes `DELETE FROM jobs WHERE id = $1` and returns true if a row was deleted, false if not found. This simple repository method keeps the business logic in the handler layer.

2. **`src/server/routes/jobs.rs`** — Added `delete_job_handler` function with the following logic:
   - Calls `repository.get_job(job_id)` to check existence and status
   - Returns 404 with `ErrorResponse { error: "not_found", message: "Job {id} not found" }` if the job doesn't exist
   - Returns 409 with `ErrorResponse { error: "conflict", message: "Cannot delete job {id} in in_process status" }` if the job is actively being processed
   - Calls `repository.delete_job(job_id)` for deletable jobs, returns 204 No Content on success
   - Handles the race condition where a job disappears between get and delete (returns 404)
   - Returns 500 for database errors

3. **`src/server/mod.rs`** — Wired DELETE method onto the existing `/api/v1/jobs/:id` route using `.delete(routes::delete_job_handler)` method chaining.

4. **`src/server/routes/mod.rs`** — Added `delete_job_handler` to the re-exports for convenient router access.

No new error variants were needed — the handler uses inline status checks and the existing `ErrorResponse` type. The unused `delete` import from axum was cleaned up since axum uses method chaining on routes rather than explicit HTTP method imports.

## Verification

Compiled successfully with `cargo check --features db` — no new warnings. All 206 existing tests pass with `cargo test --features db --lib`. The handler follows the existing patterns from `get_job_handler` for consistency (same ErrorResponse shape, same status code mapping, same tracing on errors).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check --features db` | 0 | ✅ pass | 120ms |
| 2 | `cargo test --features db --lib` | 0 | ✅ pass | 3560ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/jobs/repository.rs`
- `src/server/routes/jobs.rs`
- `src/server/mod.rs`
- `src/server/routes/mod.rs`
