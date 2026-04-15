# S03: Integration test suite

**Goal:** Write integration tests that prove the db-gated code paths work end-to-end: API routes, CLI commands, job lifecycle, query execution
**Demo:** just test-integration runs full suite of API and CLI integration tests against seeded DB, all pass

## Must-Haves

- API route tests: POST /api/v1/load creates a load job, returns job-id\nAPI route tests: POST /api/v1/query/wedding returns wedding candidates\nAPI route tests: POST /api/v1/query/project returns project candidates\nAPI route tests: POST /api/v1/query/travel returns travel candidates\nAPI route tests: GET /api/v1/jobs/:id returns job status\nAPI route tests: GET /api/v1/jobs lists jobs with pagination\nAPI validation tests: bad date, bad days, unknown query return 400\nCLI tests: query wedding/project/travel --sync returns results\nCLI tests: job status shows job details\nQUERY-07, QUERY-08, QUERY-10, QUERY-11 validated

## Proof Level

- This slice proves: integration

## Integration Closure

Terminal slice — milestone complete when integration tests pass

## Verification

- Integration test suite serves as living documentation of expected behavior

## Tasks

- [x] **T01: API route integration tests** `est:90 min`
  Create `tests/api_integration.rs` with integration tests for all API routes. Tests spin up the server against the test database using tower::ServiceExt. Cover: POST /api/v1/load (sync and async modes), POST /api/v1/query/wedding, POST /api/v1/query/project, POST /api/v1/query/travel, GET /api/v1/jobs/:id, GET /api/v1/jobs. Validation tests: invalid date format, days out of range, unknown query name. All tests gated behind `#[cfg(feature = "db")]` and require TEST_PG_URL.
  - Files: `tests/api_integration.rs`
  - Verify: cargo test --features db --test api_integration -- --ignored (after just test-db-setup) exits 0

- [x] **T02: CLI command integration tests** `est:60 min`
  Create `tests/cli_integration.rs` with integration tests for CLI commands. Uses assert_cmd to run the binary. Cover: query wedding --start --days --sync, query project --start --days --sync, query travel --start --days --sync, job status <id>, job list. Tests use the seed data date range. All tests verify exit codes and output content.
  - Files: `tests/cli_integration.rs`
  - Verify: cargo test --features db --test cli_integration -- --ignored (after just test-db-setup) exits 0

- [ ] **T03: Full verification and requirements validation** `est:20 min`
  Run the full test suite: cargo test --all-features plus the integration tests against the seeded database. Verify QUERY-07, QUERY-08, QUERY-10, QUERY-11 are satisfied. Update REQUIREMENTS.md to mark them validated. Run just verify-full as the final gate.
  - Files: `.gsd/REQUIREMENTS.md`
  - Verify: just verify-full exits 0 and just test-integration exits 0

## Files Likely Touched

- tests/api_integration.rs
- tests/cli_integration.rs
- .gsd/REQUIREMENTS.md
