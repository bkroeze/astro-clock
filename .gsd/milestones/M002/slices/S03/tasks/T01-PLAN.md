---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: API route integration tests

Create `tests/api_integration.rs` with integration tests for all API routes. Tests spin up the server against the test database using tower::ServiceExt. Cover: POST /api/v1/load (sync and async modes), POST /api/v1/query/wedding, POST /api/v1/query/project, POST /api/v1/query/travel, GET /api/v1/jobs/:id, GET /api/v1/jobs. Validation tests: invalid date format, days out of range, unknown query name. All tests gated behind `#[cfg(feature = "db")]` and require TEST_PG_URL.

## Inputs

- `tests/common/mod.rs`
- `src/server/routes/`
- `src/server/mod.rs`

## Expected Output

- `tests/api_integration.rs with comprehensive API route tests`

## Verification

cargo test --features db --test api_integration -- --ignored (after just test-db-setup) exits 0
