# S07: Named Queries

**Goal:** Create QueryJobHandler and QueryTemplateRegistry to enable named query execution as jobs with automatic data loading and structured JSON results.
**Demo:** Create QueryJobHandler and QueryTemplateRegistry to enable named query execution as jobs with automatic data loading and structured JSON results.

## Must-Haves


## Tasks

- [x] **T01: 07-named-queries 01** `est:11min`
  - Create QueryJobHandler and QueryTemplateRegistry to enable named query execution as jobs with automatic data loading and structured JSON results.

Purpose: Provide the foundation for wrapping wedding, project, and travel queries in the job system, enabling both sync and async execution with automatic data pre-loading.
Output: src/jobs/handlers/query.rs (QueryJobHandler), src/jobs/registry.rs (QueryTemplateRegistry), updated src/jobs/mod.rs
- [x] **T02: 07-named-queries 02** `est:9min`
  - Implement project and travel query functions with astrological criteria for finding favorable dates.

Purpose: Complete the set of named queries (wedding already exists) so users can find auspicious dates for starting projects and planning travel.
Output: src/queries/project.rs, src/queries/travel.rs, updated src/queries/types.rs and mod.rs
- [x] **T03: 07-named-queries 03** `est:7min`
  - Create HTTP API endpoints for named queries with sync/async execution modes and structured JSON responses.

Purpose: Enable HTTP clients to execute wedding, project, and travel queries via REST API with both blocking and non-blocking modes.
Output: src/server/routes/queries.rs, updated routes mod.rs and server mod.rs

## Files Likely Touched

- `src/jobs/handlers/query.rs`
- `src/jobs/registry.rs`
- `src/jobs/mod.rs`
- `src/queries/project.rs`
- `src/queries/travel.rs`
- `src/queries/types.rs`
- `src/queries/mod.rs`
- `src/jobs/registry.rs`
- `src/server/routes/queries.rs`
- `src/server/routes/mod.rs`
- `src/server/mod.rs`
- `src/jobs/handlers/query.rs`
