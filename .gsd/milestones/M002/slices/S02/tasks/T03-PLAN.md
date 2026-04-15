---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T03: Load seed data and discover deterministic values

Run just test-db-setup to load 60 days of seed data. Query the database to discover and document the deterministic values: how many retrograde periods exist, which bodies are retrograde, what Moon signs appear, VoC periods present, aspect summary counts. Record these as constants in the test helper module so integration tests can assert against them.

## Inputs

- `tests/common/mod.rs`

## Expected Output

- `Seed data constants in tests/common/mod.rs populated with actual values from the loaded database`

## Verification

Queries against the test database return the documented seed data values
