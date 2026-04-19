---
id: T01
parent: S02
milestone: M003
key_files:
  - Cargo.toml
  - src/jobs/repository.rs
  - src/server/routes/jobs.rs
  - src/cli/app.rs
  - migrations/010_create_jobs_cursor_index.sql
key_decisions:
  - Used base64url-no-pad (URL_SAFE_NO_PAD) for cursor encoding — produces clean URLs without padding characters
  - Cursor errors include specific failure reason (encoding vs data) per slice verification requirement for 400 responses with diagnostic detail
  - Kept count_jobs() intact for CLI diagnostic use while handler will stop using total in response (T02)
duration: 
verification_result: passed
completed_at: 2026-04-19T19:18:29.994Z
blocker_discovered: false
---

# T01: Replace offset/limit pagination with cursor-based (created_at, id) tuple pagination in list_jobs

**Replace offset/limit pagination with cursor-based (created_at, id) tuple pagination in list_jobs**

## What Happened

Implemented cursor-based pagination for the jobs listing endpoint, replacing the previous offset/limit approach with stable (created_at, id) tuple cursors.

Key changes:
1. Added `base64 = "0.22"` to Cargo.toml for base64url-no-pad cursor encoding.
2. Defined `JobCursor` struct with `encode()`/`decode()` methods that serialize (created_at, id) tuples as opaque base64url-no-pad JSON strings. Decode errors distinguish between encoding failures ("Invalid cursor encoding") and data failures ("Invalid cursor data") for diagnostic clarity.
3. Defined `CursorDirection` enum (Forward, Backward) to control pagination direction.
4. Rewrote `list_jobs()` signature from `(filters, limit, offset)` to `(filters, count, cursor, direction)`. The SQL now uses tuple comparison `(created_at, id) < (cursor_at, cursor_id)` for forward pagination and `>` for backward, fetching count+1 rows to detect page existence. Backward pages fetch in ASC order and reverse results to maintain DESC order for callers.
5. Updated CLI caller to use new signature with `None` cursor and `CursorDirection::Forward` for simple first-page listing, with count_jobs still used for total display.
6. Updated HTTP handler caller similarly — full cursor URL generation will be in T02.
7. Created migration `010_create_jobs_cursor_index.sql` with composite index on `(created_at DESC, id DESC)` for efficient cursor-based page traversal.
8. Added 4 unit tests: roundtrip encode/decode, invalid base64, invalid JSON, no-padding verification — all pass.

## Verification

All verification checks pass:
- `cargo check --features db` compiles cleanly (no new warnings from changes)
- `cargo test --features db --lib -- test_cursor` — 4/4 tests pass:
  - test_cursor_encode_decode_roundtrip ✅
  - test_cursor_decode_invalid_base64 ✅
  - test_cursor_decode_invalid_json ✅
  - test_cursor_encode_no_padding ✅
- CLI and handler callers updated to use new signature and compile successfully
- count_jobs() preserved for CLI diagnostics as planned

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check --features db` | 0 | ✅ pass | 1010ms |
| 2 | `cargo test --features db --lib -- test_cursor` | 0 | ✅ pass (4/4 tests) | 130ms |

## Deviations

Minor: Task plan verification command `cargo test --features db --lib -- jobs::repository::tests::cursor` matches 0 tests due to Rust test filter requiring substring match on full test name. The actual test names are `test_cursor_*`, so the correct filter is `-- test_cursor`. All 4 tests pass with the corrected filter.

## Known Issues

None.

## Files Created/Modified

- `Cargo.toml`
- `src/jobs/repository.rs`
- `src/server/routes/jobs.rs`
- `src/cli/app.rs`
- `migrations/010_create_jobs_cursor_index.sql`
