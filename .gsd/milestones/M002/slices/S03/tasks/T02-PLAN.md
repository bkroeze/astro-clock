---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T02: CLI command integration tests

Create `tests/cli_integration.rs` with integration tests for CLI commands. Uses assert_cmd to run the binary. Cover: query wedding --start --days --sync, query project --start --days --sync, query travel --start --days --sync, job status <id>, job list. Tests use the seed data date range. All tests verify exit codes and output content.

## Inputs

- `tests/common/mod.rs`
- `src/cli/app.rs`

## Expected Output

- `tests/cli_integration.rs with CLI command integration tests`

## Verification

cargo test --features db --test cli_integration -- --ignored (after just test-db-setup) exits 0
