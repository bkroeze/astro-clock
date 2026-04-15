---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: Add missing `post` import to server/mod.rs

Add `use axum::routing::post;` import to `src/server/mod.rs`. Currently only `get` is imported on line 2, but 5 route registrations inside the `#[cfg(feature = "db")]` method use `post()`.

## Inputs

- `src/server/mod.rs`

## Expected Output

- `src/server/mod.rs with post import added`

## Verification

cargo build --features db 2>&1 | grep 'E0425' | wc -l returns 0
