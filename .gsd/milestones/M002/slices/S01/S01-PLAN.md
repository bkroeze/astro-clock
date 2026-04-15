# S01: Fix build errors under --features db

**Goal:** Fix all compilation errors that prevent building and testing with the db feature enabled
**Demo:** cargo build --features db and cargo test --all-features both pass clean with zero errors

## Must-Haves

- cargo build --features db exits 0 with no errors\ncargo test --all-features exits 0 with all tests passing\ncargo clippy --all-features -- -D warnings passes for db-gated code

## Proof Level

- This slice proves: contract

## Integration Closure

Clean compile — downstream slices depend on a compilable binary with db feature

## Verification

- No runtime observability changes — this is a compile-time fix

## Tasks

- [x] **T01: Add missing `post` import to server/mod.rs** `est:5 min`
  Add `use axum::routing::post;` import to `src/server/mod.rs`. Currently only `get` is imported on line 2, but 5 route registrations inside the `#[cfg(feature = "db")]` method use `post()`.
  - Files: `src/server/mod.rs`
  - Verify: cargo build --features db 2>&1 | grep 'E0425' | wc -l returns 0

- [x] **T02: Fix Rust 2024 string concatenation in svg_renderer.rs** `est:15 min`
  Fix string concatenation on line 212 (`String + &&str`) and line 353 (`String + &String`) in `src/svg_renderer.rs`. Rust 2024 edition tightened deref coercion — `+` operator on String requires `&str`, not `&&str` or `&String`. Use `format!()` or explicit derefs.
  - Files: `src/svg_renderer.rs`
  - Verify: cargo test --all-features 2>&1 | grep 'E0277' | wc -l returns 0

- [ ] **T03: Verify cargo test --all-features passes clean** `est:10 min`
  Run full test suite with all features enabled. Verify zero errors, zero test failures. Check for any additional Rust 2024 edition regressions or warnings in db-gated code that weren't caught by the first two fixes.
  - Verify: cargo test --all-features exits 0

- [ ] **T04: Add justfile verify-full recipe as compile smoke test** `est:10 min`
  Add `just build-db` and `just test-all` as verification steps in the Justfile to ensure these gates are always checked. Currently these recipes exist but there's no documented expectation that they should pass. Add a `just verify-full` recipe that runs build-db + test-all + clippy as a single gate.
  - Files: `Justfile`
  - Verify: just verify-full exits 0

## Files Likely Touched

- src/server/mod.rs
- src/svg_renderer.rs
- Justfile
