---
id: S01
parent: M002
milestone: M002
provides:
  - ["Clean compilation under --features db", "verify-full Justfile gate for CI", "All 197 tests passing with --all-features"]
requires:
  []
affects:
  - ["S02", "S03"]
key_files:
  - ["src/server/mod.rs", "src/svg_renderer.rs", "Justfile"]
key_decisions:
  - ["Used explicit deref *sign rather than format!() to keep existing string concatenation style consistent", "Excluded clippy from verify-full gate due to ~40 pre-existing warnings outside this slice's scope"]
patterns_established:
  - ["verify-full Justfile recipe as single-gate CI smoke test for db feature compilation"]
observability_surfaces:
  - none
drill_down_paths:
  - [".gsd/milestones/M002/slices/S01/tasks/T01-SUMMARY.md", ".gsd/milestones/M002/slices/S01/tasks/T02-SUMMARY.md", ".gsd/milestones/M002/slices/S01/tasks/T03-SUMMARY.md", ".gsd/milestones/M002/slices/S01/tasks/T04-SUMMARY.md"]
duration: ""
verification_result: passed
completed_at: 2026-04-15T22:23:06.349Z
blocker_discovered: false
---

# S01: Fix build errors under --features db

**Fixed missing axum::routing::post import and Rust 2024 edition &&str deref coercion error, enabling cargo build --features db and cargo test --all-features to pass clean with 197 tests and zero errors.**

## What Happened

This slice resolved two compilation blockers that prevented building with the `db` feature enabled:

**T01 — Missing `post` import:** `src/server/mod.rs` imported only `get` from `axum::routing`, but the `#[cfg(feature = "db")]` method `build_app_with_db` used `post()` in 5 route registrations. Fixed by expanding the import to `use axum::routing::{get, post}`. This eliminated 5× E0425 errors.

**T02 — Rust 2024 string concatenation:** In `src/svg_renderer.rs`, iterating over a `&[&str; 12]` array yields `&&str` items, which the Rust 2024 edition no longer auto-derefs for the `String + &str` `Add` impl. Fixed with a single character change: `+ sign` → `+ *sign` on line 212. A second suspected site (line 353) was already correct because `if let Some(&glyph_name)` destructures to `&str` directly.

**T03 — Full verification:** `cargo test --all-features` passes all 197 tests (156 unit + 9 API query + 6 CLI job + 6 CLI query + 13 SVG output + 7 binary/doc) with zero failures and zero compilation errors.

**T04 — Smoke test gate:** Added `verify-full` recipe to Justfile chaining `build-db` and `test-all`. Clippy was excluded because ~40 pre-existing warnings in files outside this slice's scope would make the gate always fail.

## Verification

- `just verify-full` exits 0: runs `cargo build --features db` (compiles clean) then `cargo test --all-features` (197 tests pass, zero failures)
- `cargo build --features db` — zero E0425 or E0277 errors
- All task-specific verification commands confirmed passing in task summaries

## Requirements Advanced

None.

## Requirements Validated

- R001 — cargo build --features db compiles with zero errors; cargo test --all-features passes all 197 tests with zero failures; verified via just verify-full gate

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

["Clippy with -D warnings fails on ~40 pre-existing issues across renderer.rs, config.rs, database/chunk.rs, queries/aspects.rs, etc. Cannot be added to verify-full until those are addressed.", "Two dead_code warnings remain (format_svg, ChartDataQuery.time) — pre-existing, not introduced by this slice."]

## Follow-ups

["Future milestone should address the ~40 clippy warnings and add lint to verify-full gate"]

## Files Created/Modified

None.
