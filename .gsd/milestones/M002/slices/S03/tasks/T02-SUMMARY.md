---
id: T02
parent: S03
milestone: M002
key_files:
  - tests/cli_integration.rs
  - src/cli/app.rs
key_decisions:
  - Fixed pool-timed-out bug: CLI job/query commands now use single tokio Runtime for pool creation and all async operations (matching the load command pattern)
  - Used manual UUID finder instead of regex to avoid adding dev-dependency
  - Removed #[tokio::test] attribute since tests spawn CLI subprocesses, not async operations
duration: 
verification_result: passed
completed_at: 2026-04-15T23:27:10.635Z
blocker_discovered: false
---

# T02: Created CLI integration tests covering query wedding/project/travel, job status/list, load sync/async, input validation, and job status roundtrip; also fixed pool-timed-out bug in CLI job/query commands

**Created CLI integration tests covering query wedding/project/travel, job status/list, load sync/async, input validation, and job status roundtrip; also fixed pool-timed-out bug in CLI job/query commands**

## What Happened

Created `tests/cli_integration.rs` with 14 integration tests using `assert_cmd` to exercise the CLI binary against the test database. The tests cover:

**Query commands (sync and async):** wedding, project, and travel queries with `--start`, `--days`, and `--sync` flags. Sync tests verify exit code 0 and output contains completion/results markers. Async tests verify job ID in output.

**Job commands:** `job list` (default, with status filter, with invalid status), `job status` (nonexistent UUID, invalid format), and a full roundtrip test that lists complete jobs, extracts a UUID from output, and retrieves its status.

**Load commands:** sync load, async load, invalid date, and days=0 validation.

**Bug discovered and fixed:** During testing, discovered that `handle_job_status`, `handle_job_list`, and `handle_query_command` in `src/cli/app.rs` each created a database pool inside one `tokio::Runtime` but then tried to use the pool's connections from a *different* `Runtime` (via `Runtime::new()?.block_on()`). This caused "pool timed out while waiting for an open connection" because the connections were bound to the dropped runtime. Fixed all three by using a single shared `Runtime` for both pool creation and subsequent async operations, matching the pattern already used by the `load` command.

**UUID extraction:** Implemented a manual UUID finder (no regex dependency) to parse full UUIDs from the job list's table output for the roundtrip test.

## Verification

All 14 CLI integration tests pass:
- cli_job_list_invalid_status_fails ✅
- cli_job_list_shows_jobs ✅
- cli_job_list_with_status_filter ✅
- cli_job_status_invalid_uuid_fails ✅
- cli_job_status_nonexistent_reports_not_found ✅
- cli_job_status_roundtrip ✅
- cli_load_async_returns_job_id ✅
- cli_load_days_zero_fails ✅
- cli_load_invalid_date_fails ✅
- cli_load_sync_succeeds ✅
- cli_query_project_sync_succeeds ✅
- cli_query_travel_sync_succeeds ✅
- cli_query_wedding_async_returns_job_id ✅
- cli_query_wedding_sync_succeeds ✅

Also verified existing API integration tests still pass (21/21).

Command: `cargo test --features db --test cli_integration -- --ignored --test-threads=1` → exit 0

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --features db --test cli_integration -- --ignored --test-threads=1` | 0 | ✅ pass | 1360ms |
| 2 | `cargo test --features db --test api_integration -- --ignored --test-threads=1` | 0 | ✅ pass | 3590ms |

## Deviations

Also fixed a pre-existing bug in src/cli/app.rs where job status, job list, and query commands created database pools in one Runtime and used them in another, causing pool timeouts. This was discovered during test development when all CLI tests that touch the database were failing.

## Known Issues

None.

## Files Created/Modified

- `tests/cli_integration.rs`
- `src/cli/app.rs`
