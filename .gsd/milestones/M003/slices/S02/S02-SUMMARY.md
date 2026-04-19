---
id: S02
parent: M003
milestone: M003
provides:
  - ["JobCursor struct with encode()/decode() for opaque base64url-no-pad cursors", "CursorDirection enum (Forward, Backward) for bidirectional pagination", "list_jobs() rewritten with cursor-based SQL using (created_at, id) tuple comparison", "build_page_url helper that constructs next/prev URLs preserving all filter parameters", "ListJobsRequest with count (default 20, max 100) and cursor fields", "ListJobsResponse with next/prev URL strings instead of total/limit/offset", "Composite index migration 010 on (created_at DESC, id DESC) for efficient cursor traversal"]
requires:
  - slice: S01
    provides: JobListFilters struct, parse_comma_separated and parse_date_param helpers from src/server/routes/jobs.rs
affects:
  []
key_files:
  - ["src/jobs/repository.rs", "src/server/routes/jobs.rs", "src/cli/app.rs", "migrations/010_create_jobs_cursor_index.sql", "Cargo.toml"]
key_decisions:
  - ["Used base64url-no-pad (URL_SAFE_NO_PAD) for cursor encoding — clean URLs without padding characters", "Built URL query strings manually without additional URL crate — cursor encoding already produces URL-safe characters", "Preserved count_jobs() for CLI diagnostics while removing total from API response", "CLI supports first-page listing only via count — cursor navigation is API-only since cursors are opaque URLs"]
patterns_established:
  - ["Cursor-based pagination using PostgreSQL tuple comparison (created_at, id) < ($1, $2) for stable page boundaries", "count+1 row fetch pattern for page existence detection without COUNT query", "Backward pagination via ASC order fetch + Vec reverse to maintain DESC presentation order", "Opaque base64url-no-pad JSON cursors that clients treat as tokens", "Pagination URL construction preserving all active filter parameters across pages"]
observability_surfaces:
  - ["Invalid cursor returns 400 with specific error message (encoding vs data failure) for diagnostic clarity"]
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-19T19:28:18.446Z
blocker_discovered: false
---

# S02: Cursor-based pagination with next/prev URLs

**Replaced offset/limit pagination with cursor-based pagination using (created_at, id) tuple cursors, opaque base64 encoding, and next/prev URLs that preserve all filter parameters.**

## What Happened

## Overview

Replaced the offset/limit pagination on GET /api/v1/jobs with stable cursor-based pagination anchored on (created_at, id) tuples. The response shape changed from `{jobs, total, limit, offset}` to `{jobs, next, prev}` where next/prev are full URLs encoding the cursor and all active filter parameters.

## Task-by-Task

**T01: Cursor types, SQL rewrite, and migration** — Added `base64 = "0.22"` crate. Created `JobCursor` struct with encode/decode using base64url-no-pad JSON, and `CursorDirection` enum (Forward/Backward). Rewrote `list_jobs()` from `(filters, limit, offset)` to `(filters, count, cursor, direction)` using PostgreSQL tuple comparison `(created_at, id) < ($cursor_at, $cursor_id)` for forward pagination and `>` with ASC+reverse for backward. Fetches count+1 rows to detect page existence. Created composite index migration `010_create_jobs_cursor_index.sql` on `(created_at DESC, id DESC)`. Added 4 cursor unit tests.

**T02: Handler request/response shape and URL construction** — Replaced `ListJobsRequest` fields (limit/offset → count/cursor). Replaced `ListJobsResponse` fields (total/limit/offset → next/prev). Added `build_page_url` helper that preserves all active filters (status, job_type, created_after, created_before) in next/prev URLs. Handler uses `OriginalUri` extractor for base path, decodes cursors with 400 errors on invalid encoding, and removed the `count_jobs` call. Updated 7 existing tests and added 7 new handler-layer cursor tests.

**T03: CLI update and comprehensive tests** — Updated CLI `job list` to use `--count` instead of `--limit`/`--offset`. Added cursor roundtrip tests with various timestamps, URL filter preservation tests (status, job_type, date, partial, minimal), empty results test, and single-page test. Full suite passes at 206 tests.

## Key Design Decisions

- **base64url-no-pad encoding** — produces clean URLs without `=` padding characters
- **Manual URL construction** — no additional URL crate needed since cursor encoding already produces URL-safe characters
- **count_jobs() preserved** — still useful for CLI diagnostics even though handler no longer includes total in response
- **CLI first-page only** — CLI uses count with no cursor; cursor navigation is API-only since cursors are opaque URLs

## Verification

All verification checks pass:

1. **Full test suite**: `cargo test --features db --lib` — 206 tests pass, 0 failures
2. **Cursor unit tests**: `cargo test --features db --lib -- test_cursor` — 7/7 pass (4 repository + 3 handler)
3. **Handler tests**: `cargo test --features db --lib -- server::routes::jobs` — 50/50 pass
4. **CLI tests**: `cargo test --features db --lib -- cli::app` — 9/9 pass
5. **Compilation**: `cargo check --features db` — clean (only pre-existing warnings)

All must-haves verified:
- First page returns next URL and null prev ✅
- Following next URL returns next page with both next and prev URLs ✅
- Cursor encoding is opaque base64 JSON (created_at, id) ✅
- Filter parameters preserved in next/prev URLs ✅
- Backward pagination works correctly ✅
- count defaults to 20, max 100, min 1 ✅
- Invalid cursor returns 400 with descriptive error ✅
- CLI uses count instead of limit/offset ✅

## Requirements Advanced

- R006 — Cursor-based pagination implemented with (created_at, id) tuple cursors, opaque base64 encoding, count parameter (default 20, max 100). SQL uses tuple comparison for stable page boundaries.
- R007 — Response includes next and prev URL strings encoding cursor and all active filter parameters. build_page_url helper preserves status, job_type, created_after, created_before in pagination URLs.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

None.

## Follow-ups

["S03 integration tests will exercise full cursor pagination flow against real database including forward/backward navigation and filter preservation"]

## Files Created/Modified

- `Cargo.toml` — Added base64 = 0.22 dependency for cursor encoding
- `src/jobs/repository.rs` — Added JobCursor, CursorDirection types; rewrote list_jobs() with cursor-based SQL using tuple comparison and count+1 detection
- `src/server/routes/jobs.rs` — Replaced limit/offset with count/cursor in request; replaced total/limit/offset with next/prev URLs in response; added build_page_url helper and OriginalUri extractor; removed count_jobs call from handler
- `src/cli/app.rs` — Replaced --limit/--offset with --count in job list command; updated handler call to new list_jobs signature
- `migrations/010_create_jobs_cursor_index.sql` — Composite index on (created_at DESC, id DESC) for efficient cursor-based page traversal
