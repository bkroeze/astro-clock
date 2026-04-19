---
estimated_steps: 19
estimated_files: 4
skills_used: []
---

# T01: Implement DELETE /api/v1/jobs/:id endpoint

Add a delete_job method to JobRepository and a delete_job_handler to the routes, wired into the existing /api/v1/jobs/:id route with axum's .delete() modifier.

## Design

**Two-step approach (fetch then delete):** Use the existing `get_job()` to check existence and status, then perform `DELETE FROM jobs WHERE id = $1` if the job is not in_process. This avoids adding new error variants — the handler maps get_job results directly to HTTP status codes.

**Repository method:** `delete_job(&self, id: Uuid) -> JobResult<bool>` — executes `DELETE FROM jobs WHERE id = $1`, returns true if a row was deleted, false if not found. The handler calls get_job first to check status, then delete_job to perform the deletion.

**Handler logic:**
1. Call `repository.get_job(job_id)`
2. If None → return 404 with `ErrorResponse { error: "not_found", message: "Job {id} not found" }`
3. If Some(job) and `job.status == "in_process"` → return 409 with `ErrorResponse { error: "conflict", message: "Cannot delete job {id} in in_process status" }`
4. Otherwise → call `repository.delete_job(job_id)`, return 204 No Content

**Route wiring:** In `src/server/mod.rs`, change the `/api/v1/jobs/:id` route from `.route("/api/v1/jobs/:id", get(routes::get_job_handler))` to include `.delete(routes::delete_job_handler)`. Also update `tests/api_integration.rs` `build_app()` to include the DELETE route.

**No new error variants needed** — handler uses inline status checks and the existing ErrorResponse type.

## Constraints
- All new code behind `#[cfg(feature = "db")]`
- Import `delete` in `use axum::routing::{get, post}` → `use axum::routing::{get, post, delete}`
- 204 No Content returns `StatusCode::NO_CONTENT` with empty body
- Handler signature: `pub async fn delete_job_handler(State(state): State<AppState>, Path(job_id): Path<Uuid>) -> impl IntoResponse`
- Export delete_job_handler from routes module (add to `src/server/routes/mod.rs` if it exists, or ensure pub visibility)

## Route file structure
Check if `src/server/routes/mod.rs` exists — if routes are in `src/server/routes/jobs.rs`, the handler goes there. If there's a mod.rs re-exporting, update it too.

## Inputs

- ``src/jobs/repository.rs` — existing get_job method to reuse for existence check`
- ``src/server/routes/jobs.rs` — existing handler patterns (get_job_handler, ErrorResponse) to follow`
- ``src/server/mod.rs` — route table to add DELETE method to /api/v1/jobs/:id`
- ``src/jobs/error.rs` — existing JobError::NotFound variant for error mapping`

## Expected Output

- ``src/jobs/repository.rs` — new delete_job method`
- ``src/server/routes/jobs.rs` — new delete_job_handler function`
- ``src/server/mod.rs` — DELETE route wired on /api/v1/jobs/:id`

## Verification

cargo check --features db && cargo test --features db --lib
