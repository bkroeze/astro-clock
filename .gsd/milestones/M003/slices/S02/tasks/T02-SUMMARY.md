---
id: T02
parent: S02
milestone: M003
key_files:
  - src/server/routes/jobs.rs
key_decisions:
  - Built URL query strings manually instead of adding a URL crate dependency — cursor encoding already produces URL-safe characters via base64url-no-pad, so no percent-encoding is needed for filter values (they pass through as-is from query params)
duration: 
verification_result: passed
completed_at: 2026-04-19T19:23:24.073Z
blocker_discovered: false
---

# T02: Rewrite jobs list handler from offset/limit to cursor-based pagination with next/prev URLs

**Rewrite jobs list handler from offset/limit to cursor-based pagination with next/prev URLs**

## What Happened

Replaced the offset/limit pagination in the jobs list handler with cursor-based pagination using opaque (created_at, id) cursors. Key changes:

1. **ListJobsRequest**: Removed `limit`/`offset` fields, added `count` (default 20, max 100) and `cursor` (optional opaque string). Removed `default_limit`/`default_offset` functions, added `default_count`.

2. **ListJobsResponse**: Removed `total`/`limit`/`offset` fields, added `next: Option<String>` and `prev: Option<String>` containing full URLs.

3. **build_page_url helper**: Constructs pagination URLs that preserve all active filter parameters (status, job_type, created_after, created_before) alongside count and cursor. Since cursor encoding uses base64url-no-pad, no additional percent-encoding is needed.

4. **Handler rewrite**: Added `OriginalUri` extractor for base path. Handler now decodes cursor (returns 400 with descriptive error on failure), calls repository with count+1 rows to detect next-page existence, and builds next/prev URLs from boundary rows. Removed the `count_jobs` call entirely — total is no longer in the response.

5. **Tests**: Updated 7 existing tests to use new request/response shape. Added 7 new handler-layer tests: count+cursor deserialization, cursor decode error handling, build_page_url with/without filters, partial filter preservation, first-page-no-prev verification, and cursor URL validity check. All 199 project tests pass.

## Verification

All 43 handler tests pass including 7 new cursor-specific tests. Full project suite (199 tests) passes with zero failures. `cargo check --features db` compiles clean (only pre-existing warnings).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check --features db` | 0 | ✅ pass | 1260ms |
| 2 | `cargo test --features db --lib -- server::routes::jobs` | 0 | ✅ pass (43/43 tests) | 6700ms |
| 3 | `cargo test --features db --lib` | 0 | ✅ pass (199/199 tests) | 260ms |

## Deviations

None. All must-haves met: request uses count/cursor, response has next/prev, handler builds full URLs with filters, invalid cursor returns 400 with descriptive error, OriginalUri used for base path, count_jobs removed, all tests updated, new cursor tests added.

## Known Issues

None.

## Files Created/Modified

- `src/server/routes/jobs.rs`
