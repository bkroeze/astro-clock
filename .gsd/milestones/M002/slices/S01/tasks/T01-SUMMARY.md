---
id: T01
parent: S01
milestone: M002
key_files:
  - src/server/mod.rs
key_decisions:
  - (none)
duration: 
verification_result: passed
completed_at: 2026-04-15T21:57:12.095Z
blocker_discovered: false
---

# T01: Add missing `post` import to server/mod.rs to fix E0425 compilation error under --features db

**Add missing `post` import to server/mod.rs to fix E0425 compilation error under --features db**

## What Happened

The file `src/server/mod.rs` imported `use axum::routing::get` on line 2 but the `build_app_with_db` method (guarded by `#[cfg(feature = "db")]`) used `post()` in 5 route registrations. This caused E0425 "cannot find function `post` in this scope" errors when building with `--features db`. Fixed by changing the import to `use axum::routing::{get, post}`. Build now compiles cleanly with zero errors (only pre-existing dead-code warnings).

## Verification

Ran `cargo build --features db` — exits successfully with 0 errors. Confirmed zero E0425 errors via `cargo build --features db 2>&1 | grep 'E0425' | wc -l` returning 0.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features db 2>&1 | grep 'E0425' | wc -l` | 0 | ✅ pass | 500ms |
| 2 | `cargo build --features db` | 0 | ✅ pass | 100ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/server/mod.rs`
