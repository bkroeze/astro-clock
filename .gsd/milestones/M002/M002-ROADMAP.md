# M002: Build Fix & Integration Test Suite

## Vision
Fix the broken `--features db` build, establish reproducible test database infrastructure, and write integration tests that prove the db-gated code paths (server routes, job handlers, query execution) work end-to-end against a real database with deterministic seed data.

## Slice Overview
| ID | Slice | Risk | Depends | Done | After this |
|----|-------|------|---------|------|------------|
| S01 | S01 | low | — | ✅ | cargo build --features db and cargo test --all-features both pass clean with zero errors |
| S02 | S02 | medium | — | ⬜ | just test-db-setup creates a seeded test DB; a test binary can connect, query planet_positions, and assert known values |
| S03 | Integration test suite | medium | S02 | ⬜ | just test-integration runs full suite of API and CLI integration tests against seeded DB, all pass |
