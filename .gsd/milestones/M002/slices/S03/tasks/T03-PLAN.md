---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T03: Full verification and requirements validation

Run the full test suite: cargo test --all-features plus the integration tests against the seeded database. Verify QUERY-07, QUERY-08, QUERY-10, QUERY-11 are satisfied. Update REQUIREMENTS.md to mark them validated. Run just verify-full as the final gate.

## Inputs

- `tests/api_integration.rs`
- `tests/cli_integration.rs`

## Expected Output

- `All tests passing`
- `QUERY-07, QUERY-08, QUERY-10, QUERY-11 marked validated`

## Verification

just verify-full exits 0 and just test-integration exits 0
