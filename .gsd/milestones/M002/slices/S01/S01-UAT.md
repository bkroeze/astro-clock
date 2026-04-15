# S01: Fix build errors under --features db — UAT

**Milestone:** M002
**Written:** 2026-04-15T22:23:06.349Z

# S01: Fix build errors under --features db — UAT

**Milestone:** M002
**Written:** 2026-04-15

## UAT Type

- UAT mode: artifact-driven
- Why this mode is sufficient: This slice is purely a compile-time fix. There are no runtime behaviors to manually test — the proof is that the code compiles and tests pass.

## Preconditions

- Rust toolchain installed (edition 2024 compatible)
- Swiss Ephemeris data files present in `data/` directory
- No running services required

## Smoke Test

Run `just verify-full` and confirm it exits 0.

## Test Cases

### 1. Build with db feature compiles

1. Run `cargo build --features db`
2. **Expected:** Build completes with exit code 0. No E0425 or E0277 errors. Only benign dead_code warnings for `format_svg` and `ChartDataQuery.time`.

### 2. Full test suite passes with all features

1. Run `cargo test --all-features`
2. **Expected:** All 197 tests pass across all test targets (unit, integration, doc). Zero failures.

### 3. verify-full gate works

1. Run `just verify-full`
2. **Expected:** Exits 0. Runs `cargo build --features db` then `cargo test --all-features` sequentially.

### 4. Specific fix verification — post import

1. Run `cargo build --features db 2>&1 | grep 'E0425' | wc -l`
2. **Expected:** Output is `0` (no "cannot find function" errors)

### 5. Specific fix verification — string deref

1. Run `cargo test --all-features 2>&1 | grep 'E0277' | wc -l`
2. **Expected:** Output is `0` (no trait bound mismatch errors)

## Edge Cases

### Clippy with -D warnings

1. Run `cargo clippy --all-features -- -D warnings`
2. **Expected:** Fails with ~40 pre-existing warnings in files outside this slice's scope. This is a known limitation documented in T04's deviation. NOT a regression from this slice.

## Failure Signals

- E0425 errors in `src/server/mod.rs` — post import missing again
- E0277 errors in `src/svg_renderer.rs` — string deref regression
- Test failures under `--all-features` — new compilation or logic errors in db-gated code

## Not Proven By This UAT

- Runtime correctness of db-gated server routes (requires running database)
- Job execution or query handler behavior (requires database connection)
- These are deferred to S02/S03 integration tests

## Notes for Tester

- The two dead_code warnings (`format_svg` in output_handler.rs, `ChartDataQuery.time` in server/mod.rs) are pre-existing and not related to this slice's changes.
- The deprecated `cargo_bin` warning in assert_cmd tests is also pre-existing.
- `just verify-full` does NOT include `lint` (clippy) because of pre-existing warnings. See T04 deviation.
