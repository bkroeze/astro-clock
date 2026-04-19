---
id: T03
parent: S02
milestone: M003
key_files:
  - src/cli/app.rs
  - src/server/routes/jobs.rs
key_decisions:
  - CLI uses count (min 1, max 100, default 20) instead of limit/offset — matches the API's count parameter; no --offset flag since cursors are opaque and only the API supports cursor-based pagination beyond the first page
duration: 
verification_result: passed
completed_at: 2026-04-19T19:26:06.596Z
blocker_discovered: false
---

# T03: Update CLI from limit/offset to count parameter and add 7 comprehensive cursor pagination tests

**Update CLI from limit/offset to count parameter and add 7 comprehensive cursor pagination tests**

## What Happened

Updated the CLI `JobCommands::List` variant to replace `limit`/`offset` arguments with a single `count` parameter (default 20, max 100). Rewrote `handle_job_list` to use cursor-based pagination via `list_jobs(filters, count, None, CursorDirection::Forward)`, removing the `count_jobs` total-count query and the old "showing X of Y" display format. The CLI now shows "cursor-based pagination" in the output and hints at the API for further pages.

Added 7 new cursor-specific tests in `src/server/routes/jobs.rs`:
1. `test_cursor_roundtrip_various_timestamps` — encodes/decodes cursors with past (2020), future (2099), leap day (Feb 29), and Unix epoch timestamps plus max UUID values
2. `test_build_page_url_preserves_status_filter` — verifies multi-value status filter (complete,failed) appears in URL
3. `test_build_page_url_preserves_job_type_filter` — verifies job_type=load appears in URL
4. `test_build_page_url_preserves_date_filters` — verifies created_after and created_before appear in URL
5. `test_build_page_url_minimal` — verifies URL has exactly 2 params (count, cursor) when no filters active
6. `test_empty_results_no_next_prev` — verifies both next and prev are null when 0 results
7. `test_single_page_no_next` — verifies next is null when results fit in one page

Full test suite grew from 199 to 206 tests, all passing.

## Verification

All 206 tests pass (199 existing + 7 new cursor-specific tests). CLI tests (9/9) pass with updated count parameter. `cargo check --features db` compiles clean with only pre-existing warnings.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check --features db` | 0 | ✅ pass | 1310ms |
| 2 | `cargo test --features db --lib -- cli::app` | 0 | ✅ pass (9/9 tests) | 120ms |
| 3 | `cargo test --features db --lib` | 0 | ✅ pass (206/206 tests) | 370ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/cli/app.rs`
- `src/server/routes/jobs.rs`
