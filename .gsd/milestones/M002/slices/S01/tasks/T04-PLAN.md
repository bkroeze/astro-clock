---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T04: Add justfile verify-full recipe as compile smoke test

Add `just build-db` and `just test-all` as verification steps in the Justfile to ensure these gates are always checked. Currently these recipes exist but there's no documented expectation that they should pass. Add a `just verify-full` recipe that runs build-db + test-all + clippy as a single gate.

## Inputs

- `Justfile`

## Expected Output

- `Updated Justfile with verify-full recipe`

## Verification

just verify-full exits 0
