# T01: 07-named-queries 01

**Slice:** S07 — **Milestone:** M001

## Description

Create QueryJobHandler and QueryTemplateRegistry to enable named query execution as jobs with automatic data loading and structured JSON results.

Purpose: Provide the foundation for wrapping wedding, project, and travel queries in the job system, enabling both sync and async execution with automatic data pre-loading.
Output: src/jobs/handlers/query.rs (QueryJobHandler), src/jobs/registry.rs (QueryTemplateRegistry), updated src/jobs/mod.rs

## Must-Haves

- [ ] QueryJobHandler implements JobHandler trait for JobType::Query
- [ ] QueryTemplateRegistry maps "wedding", "project", "travel" names to query functions
- [ ] Query jobs automatically load missing data before executing
- [ ] Query jobs support both sync and async execution modes via JobExecutor
- [ ] Query results are returned as structured JSON in job.result field

## Files

- `src/jobs/handlers/query.rs`
- `src/jobs/registry.rs`
- `src/jobs/mod.rs`
