# M002: Build Fix & Integration Test Suite

**Gathered:** 2026-04-15
**Status:** Ready for planning

## Project Description

The astro-clock project has a significant gap: `cargo build --features db` fails to compile, meaning the entire database-dependent code path (server routes, job handlers, query execution) is untested at build time. The existing integration test files are hollow shells with commented-out tests. This milestone fixes the build, establishes a reproducible test database, and writes real integration tests.

## Why This Milestone

M001 shipped the job system and query handlers, but the `db` feature gate means `cargo test` (without features) never exercises that code. Two distinct build errors prevent compilation with `--features db`: a missing `axum::routing::post` import and Rust 2024 edition string concatenation regressions. No CI gate exists to catch these.

## User-Visible Outcome

### When this milestone is complete, the user can:

- Run `just build-db` and get a clean compile
- Run `just test-all` (or `cargo test --all-features`) and see all tests pass
- Run `just test-db-setup` to create a seeded test database
- Run integration tests against that database and see meaningful assertions pass

### Entry point / environment

- Entry point: Justfile recipes (`just build-db`, `just test-all`, `just test-db-setup`, `just test-integration`)
- Environment: Local development with PostgreSQL/TimescaleDB
- Live dependencies involved: PostgreSQL via TEST_PG_URL, Swiss Ephemeris data files

## Completion Class

- Contract complete means: `cargo build --features db` compiles, `cargo test --all-features` passes, all unit tests in db-gated modules pass
- Integration complete means: API routes and CLI commands work end-to-end against a real seeded database
- Operational complete means: `just test-db-setup` idempotently provisions the test database from scratch

## Final Integrated Acceptance

To call this milestone complete, we must prove:

- `cargo build --features db` exits 0 with no errors
- `cargo test --all-features` exits 0 with no test failures
- A justfile recipe creates a seeded test DB from an empty database
- Integration tests query the seeded DB and assert correct results for wedding, project, and travel queries
- Job lifecycle (create → execute → poll status) works through both API and CLI

## Architectural Decisions

### Test database via TEST_PG_URL

**Decision:** Use a separate `TEST_PG_URL` environment variable for test database connection, distinct from `PG_URL`/`DATABASE_URL` used in production.

**Rationale:** Prevents accidental destruction of production data. The user explicitly requested this separation.

**Alternatives Considered:**
- Testcontainers — heavier dependency, Docker requirement
- In-memory SQLite — schema uses TimescaleDB-specific features (hypertables)
- Shared PG_URL with schema prefix — error-prone, risk of data collision

### Seed data via ephemeris loading functions

**Decision:** Use the existing CLI load command to populate 60 days of deterministic ephemeris data rather than hand-crafted SQL fixtures.

**Rationale:** The Swiss Ephemeris produces deterministic output for given inputs. Using the real loading path tests the actual data pipeline. Fixture SQL would be brittle and wouldn't catch regressions in the loading code.

**Alternatives Considered:**
- Hand-crafted SQL INSERT statements — brittle, doesn't test the loading pipeline
- Saved database dump — opaque, hard to regenerate if schema changes

### 60-day seed range

**Decision:** Load 60 days of data for integration testing.

**Rationale:** Covers at least one Mercury retrograde period, multiple Moon sign transitions, VoC windows, and enough variety for all three query types to return meaningful results. Compact enough for fast loading.

**Alternatives Considered:**
- 30 days — may miss a retrograde window
- 365 days — unnecessarily slow for test setup

## Error Handling Strategy

Build errors are compile-time failures with clear diagnostics. Test failures use standard Rust assertions with descriptive messages. Database setup failures surface through the justfile recipe exit code.

## Risks and Unknowns

- **Date range selection** — Need to pick a 60-day window that includes a Mercury retrograde. If we pick wrong, travel/project queries may return empty results because Mercury is always direct.
- **TimescaleDB availability** — Schema uses `create_hypertable`. Standard PostgreSQL won't work. Tests need TimescaleDB or the setup recipe needs to handle the extension gracefully.
- **Rust 2024 edition regressions** — The string concatenation issue in `svg_renderer.rs` may not be the only one. Fixing the two known instances may reveal more.
- **Test isolation** — Integration tests running against a shared database need to be idempotent and not interfere with each other.

## Existing Codebase / Prior Art

- `src/server/mod.rs` — Missing `post` import (line 2, only imports `get`)
- `src/svg_renderer.rs` — Lines 212 and 353 have `String + &&str` and `String + &String` that fail under Rust 2024 edition with `--features db`
- `tests/api_query_tests.rs` — Hollow shell with commented-out reqwest tests
- `tests/cli_query_tests.rs` — Only tests `--help` output and validation errors, not actual query execution
- `tests/cli_job_tests.rs` — Same pattern, no real database interaction
- `src/jobs/handlers/query.rs` — Query job handler with auto-loading logic, needs integration testing
- `src/server/routes/queries.rs` — API route handlers with validation logic, needs integration testing
- `Justfile` — Already has `build-db`, `test-all`, `migrate` recipes

## Relevant Requirements

- R001 — Build passes with db feature
- R002 — Test database infrastructure
- R003 — Integration test suite
- QUERY-07 — Named query "project" (validation)
- QUERY-08 — Named query "travel" (validation)
- QUERY-10 — Auto-loading missing data (validation)
- QUERY-11 — Sync/async execution modes (validation)

## Scope

### In Scope

- Fix all build errors under `--features db` and `--all-features`
- Add `post` import to `server/mod.rs`
- Fix string concatenation in `svg_renderer.rs` for Rust 2024 edition
- Create `just test-db-setup` recipe
- Create test helper module for database provisioning
- Write integration tests for API routes (load, queries, jobs)
- Write integration tests for CLI commands (load, query, job)
- Validate QUERY-07, QUERY-08, QUERY-10, QUERY-11 through integration tests

### Out of Scope / Non-Goals

- CI pipeline setup
- Performance benchmarking
- Load testing
- Testcontainers or Docker orchestration
- Refactoring the existing code structure

## Technical Constraints

- Rust 2024 edition with stricter deref coercion rules
- TimescaleDB required for hypertable creation
- Swiss Ephemeris data files must be present in `data/` directory
- `TEST_PG_URL` environment variable must be set for integration tests
- Integration tests are gated behind `#[ignore]` or a feature flag so they don't fail without a database

## Testing Requirements

- All existing tests must continue to pass
- New unit tests: none expected (existing unit tests cover serialization/validation)
- New integration tests: API routes, CLI commands, job lifecycle, query execution
- Integration tests assert against deterministic seed data values

## Acceptance Criteria

- S01: `cargo build --features db` exits 0, `cargo test --all-features` exits 0
- S02: `just test-db-setup` creates a seeded test DB from an empty database, idempotent on re-run
- S03: Integration tests for all API routes and CLI commands pass against seeded test DB

## Open Questions

- Specific 60-day date range to use — need to pick one that includes a Mercury retrograde period for meaningful travel/project query results
