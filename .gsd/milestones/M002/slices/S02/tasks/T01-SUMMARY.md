---
id: T01
parent: S02
milestone: M002
key_files:
  - Justfile
  - .env.example
key_decisions:
  - (none)
duration: 
verification_result: mixed
completed_at: 2026-04-15T22:26:14.686Z
blocker_discovered: false
---

# T01: Add just test-db-setup and test-integration recipes plus TEST_PG_URL to .env.example

**Add just test-db-setup and test-integration recipes plus TEST_PG_URL to .env.example**

## What Happened

Added two new Justfile recipes and a test database URL to .env.example.

**test-db-setup recipe:** Drops and recreates the test database using TEST_PG_URL, runs all migrations via sqlx-cli, then loads 60 days of seed data (2025-01-01 through 2025-03-02) via the CLI `load --sync` command. Uses a maintenance URL (connecting to the `postgres` DB) for drop/create operations. URL parsing extracts the database name and constructs the maintenance URL by replacing the DB name with `postgres`.

**test-integration recipe:** Validates TEST_PG_URL is set, then runs `cargo test --features db -- --ignored --test-threads=1` to execute integration tests that are marked with `#[ignore]`.

**URL parsing fix:** Initially used `sed` with `$` anchor which had escaping issues inside the just script context. Switched to pipe-delimited sed with path-based replacement (`s|/${DB_NAME}|/postgres|`) for robustness.

**Verification:** Ran `just test-db-setup` against a real PostgreSQL instance — successfully dropped, created the test database, and ran migrations through migration 5. Migration 6 failed due to a pre-existing TimescaleDB continuous aggregate bug (`cannot cast type boolean to smallint` in the `last()` aggregate) — this is unrelated to the recipe changes.

## Verification

Verified both recipes parse correctly with `just --list` and `just --dump`. Ran `just test-db-setup` end-to-end against a real database — it successfully dropped/recreated the test DB and applied migrations 1-5. The migration 6 failure is a pre-existing schema bug, not a recipe issue. The `test-integration` recipe was verified via `just --dry-run test-integration` showing correct command generation.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `just --list 2>&1 | grep -E 'test-db-setup|test-integration'` | 0 | ✅ pass | 500ms |
| 2 | `just --dump 2>&1 | grep -A2 'test-db-setup'` | 0 | ✅ pass | 300ms |
| 3 | `source .env && export TEST_PG_URL=postgresql://astro:barnlab@10.0.10.50:5432/astrology_test && just test-db-setup` | 1 | ⚠️ partial — migrations 1-5 applied, migration 6 pre-existing bug | 15000ms |
| 4 | `source .env && export TEST_PG_URL=postgresql://astro:barnlab@10.0.10.50:5432/astrology_test && just --dry-run test-integration` | 0 | ✅ pass | 400ms |

## Deviations

Fixed URL parsing approach in test-db-setup: replaced `sed` with `$` anchor (which had escaping issues inside just script context) with pipe-delimited sed using path-based replacement for robustness.

## Known Issues

Migration 006 (continuous aggregates) fails with "cannot cast type boolean to smallint" — pre-existing bug in the TimescaleDB aggregate definition, not related to this task. This blocks full test-db-setup completion until the migration is fixed.

## Files Created/Modified

- `Justfile`
- `.env.example`
