---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T03: Verify cargo test --all-features passes clean

Run full test suite with all features enabled. Verify zero errors, zero test failures. Check for any additional Rust 2024 edition regressions or warnings in db-gated code that weren't caught by the first two fixes.

## Inputs

- `src/server/mod.rs`
- `src/svg_renderer.rs`

## Expected Output

- `Clean test output with all tests passing`

## Verification

cargo test --all-features exits 0
