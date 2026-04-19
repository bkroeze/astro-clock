# Requirements

This file is the explicit capability and coverage contract for the project.

Use it to track what is actively in scope, what has been validated by completed work, what is intentionally deferred, and what is explicitly out of scope.

Guidelines:
- Keep requirements capability-oriented, not a giant feature wishlist.
- Requirements should be atomic, testable, and stated in plain language.
- Every **Active** requirement should be mapped to a slice, deferred, blocked with reason, or moved out of scope.
- Each requirement should have one accountable primary owner and may have supporting slices.
- Research may suggest requirements, but research does not silently make them binding.
- Validation means the requirement was actually proven by completed work and verification, not just discussed.

## Active

### R004 — Multi-value status and job_type filters on job list
- Class: core-capability
- Status: active
- Description: GET /api/v1/jobs accepts comma-separated lists for `status` (pending,in_process,complete,failed) and `job_type` (load,query) query parameters, returning only matching jobs.
- Why it matters: Users need to filter jobs by multiple statuses and types in a single request — e.g., "show me all failed and pending load jobs."
- Source: user
- Primary owning slice: M003/S01
- Supporting slices: none
- Validation: mapped
- Notes: Replaces the current single-value status filter. Must validate individual values against known enums and return 400 for invalid entries.

### R005 — Date range filters on job list
- Class: core-capability
- Status: active
- Description: GET /api/v1/jobs accepts `created_after` and `created_before` query parameters (ISO 8601 timestamps or YYYY-MM-DD dates) to filter by job creation time.
- Why it matters: Time-bounded queries are essential for operational use — "show me jobs from last week" or "jobs created since deployment."
- Source: user
- Primary owning slice: M003/S01
- Supporting slices: none
- Validation: mapped
- Notes: Both parameters are optional. Either or both may be provided.

### R006 — Cursor-based pagination with stable anchors
- Class: core-capability
- Status: active
- Description: GET /api/v1/jobs uses cursor-based pagination anchored on (created_at, id) instead of offset/limit. The `count` parameter (default 20, max 100) controls page size. Cursors are opaque base64 tokens encoding the boundary row's timestamp and ID.
- Why it matters: Offset-based pagination produces shifting results when new jobs are inserted between page fetches. Stable cursors guarantee consistent page boundaries.
- Source: user
- Primary owning slice: M003/S02
- Supporting slices: M003/S01
- Validation: mapped
- Notes: First request uses filter params + count. Response includes next/prev URLs. Subsequent requests can use either cursors or fresh filter params.

### R007 — Auto-generated next/prev URLs in list response
- Class: core-capability
- Status: active
- Description: The job list response payload includes `next` and `prev` URL strings that clients can follow directly. These URLs encode the cursor and all active filter parameters, preserving the query context across pages.
- Why it matters: Clients shouldn't need to understand cursor encoding — they just follow the URL. Filter state is preserved automatically.
- Source: user
- Primary owning slice: M003/S02
- Supporting slices: M003/S01
- Validation: mapped
- Notes: `prev` is null on the first page. `next` is null when there are no more results.

### R008 — DELETE /api/v1/jobs/:id with in_process guard
- Class: core-capability
- Status: active
- Description: DELETE /api/v1/jobs/:id removes a job. Returns 204 No Content on success, 404 if job not found, 409 Conflict if job is in_process. Standard REST semantics.
- Why it matters: Job cleanup is essential for operational hygiene. Guarding in_process prevents orphaning a worker that's actively executing the job.
- Source: user
- Primary owning slice: M003/S03
- Supporting slices: none
- Validation: mapped
- Notes: Deleting a pending job that gets claimed between the status check and the DELETE is acceptable — this is a known race window.

### R009 — Integration tests for enhanced job endpoints
- Class: quality-attribute
- Status: active
- Description: Integration tests covering: multi-value filters, date range filters, cursor pagination forward/backward, next/prev URL generation, DELETE success/404/409, edge cases. Tests follow M02 patterns.
- Why it matters: The M02 test suite proved the value of comprehensive integration tests. New endpoints need the same coverage.
- Source: inferred
- Primary owning slice: M003/S03
- Supporting slices: M003/S01, M003/S02
- Validation: mapped
- Notes: Existing integration tests must continue passing — no regressions.

## Validated

### R001 — cargo build --features db compiles with zero errors
- Class: quality-attribute
- Status: validated
- Description: cargo build --features db compiles with zero errors.
- Why it matters: The db feature gate hides a significant chunk of the server, jobs, and query code from normal builds.
- Source: user
- Primary owning slice: M002/S01
- Supporting slices: none
- Validation: validated
- Notes: Verified via just verify-full. All 197 tests pass with --all-features.

### R002 — Justfile test-db-setup recipe
- Class: operability
- Status: validated
- Description: Justfile recipe (test-db-setup) that idempotently drops, recreates, migrates, and seeds a test database using TEST_PG_URL.
- Why it matters: Integration tests need a reproducible known-state database.
- Source: user
- Primary owning slice: M002/S02
- Supporting slices: none
- Validation: validated
- Notes: Seed data covers 60 days with deterministic values.

### R003 — Integration tests for API routes and CLI commands
- Class: quality-attribute
- Status: validated
- Description: 35 integration tests (21 API + 14 CLI) run against seeded test database.
- Why it matters: Real integration tests prove db-gated code paths work end-to-end.
- Source: user
- Primary owning slice: M002/S03
- Supporting slices: none
- Validation: validated
- Notes: Covers load, query, job lifecycle, input validation.

## Deferred

### R010 — Authentication for job endpoints
- Class: compliance/security
- Status: deferred
- Description: Authentication and authorization for all job management endpoints.
- Why it matters: Production deployments need access control.
- Source: user
- Primary owning slice: none
- Supporting slices: none
- Validation: unmapped
- Notes: Explicitly deferred to a future milestone per user decision.

### R011 — Bulk delete endpoint
- Class: admin/support
- Status: deferred
- Description: DELETE /api/v1/jobs with status/date filters for batch cleanup.
- Why it matters: Operational cleanup of old jobs in bulk is more efficient than single-job deletes.
- Source: user
- Primary owning slice: none
- Supporting slices: none
- Validation: unmapped
- Notes: Scoped out of M003. Natural follow-up.

## Out of Scope

### R012 — Job retry/resubmit endpoint
- Class: core-capability
- Status: out-of-scope
- Description: POST endpoint to retry a failed job or resubmit a completed job.
- Why it matters: Not requested. Prevents scope creep.
- Source: inferred
- Primary owning slice: none
- Supporting slices: none
- Validation: n/a
- Notes: May be useful later.

## Traceability

| ID | Class | Status | Primary owner | Supporting | Proof |
|---|---|---|---|---|---|
| R004 | core-capability | active | M003/S01 | none | mapped |
| R005 | core-capability | active | M003/S01 | none | mapped |
| R006 | core-capability | active | M003/S02 | M003/S01 | mapped |
| R007 | core-capability | active | M003/S02 | M003/S01 | mapped |
| R008 | core-capability | active | M003/S03 | none | mapped |
| R009 | quality-attribute | active | M003/S03 | M003/S01, M003/S02 | mapped |
| R001 | quality-attribute | validated | M002/S01 | none | validated |
| R002 | operability | validated | M002/S02 | none | validated |
| R003 | quality-attribute | validated | M002/S03 | none | validated |
| R010 | compliance/security | deferred | none | none | unmapped |
| R011 | admin/support | deferred | none | none | unmapped |
| R012 | core-capability | out-of-scope | none | none | n/a |

## Coverage Summary

- Active requirements: 6
- Mapped to slices: 6
- Validated: 3
- Unmapped active requirements: 0
