---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: Add just test-db-setup and test-integration recipes

Add `test-db-setup` and `test-integration` recipes to Justfile. test-db-setup: drops and recreates the test database using TEST_PG_URL, runs migrations via sqlx, loads 60 days of seed data via the CLI load command. test-integration: runs cargo test with the integration tests enabled. Also update .env.example to include TEST_PG_URL.

## Inputs

- `Justfile`
- `.env.example`

## Expected Output

- `Updated Justfile with test-db-setup and test-integration recipes`
- `Updated .env.example with TEST_PG_URL`

## Verification

just test-db-setup exits 0 (requires TEST_PG_URL set and TimescaleDB available)
