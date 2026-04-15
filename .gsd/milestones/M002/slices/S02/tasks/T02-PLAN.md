---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T02: Create test helper module with seed data constants

Create `tests/common/mod.rs` as a shared test helper module. Provides: database connection helper (reads TEST_PG_URL, creates pool), seed data constants (date range, known planet positions, expected query results), test fixture functions (verify seed data loaded, get specific position assertions). This module is shared across integration test files.

## Inputs

- `migrations/`
- `src/queries/types.rs`

## Expected Output

- `tests/common/mod.rs with connection helper, seed data constants, and fixture functions`

## Verification

cargo test --features db --test api_integration 2>&1 | grep 'error' | wc -l returns 0 (compiles, may fail at runtime without DB)
