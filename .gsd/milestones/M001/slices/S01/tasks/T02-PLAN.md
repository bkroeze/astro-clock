# T02: 01-database-schema 02

**Slice:** S01 — **Milestone:** M001

## Description

Set up sqlx migration tooling and Justfile integration for database management.

Purpose: Enable developers to run migrations easily and ensure sqlx can discover migration files.
Output: Updated Justfile with migrate command, working migration discovery.

## Must-Haves

- [ ] Justfile has migrate command to run sqlx migrations
- [ ] Database connection can be verified via CLI
- [ ] Migration files are discoverable by sqlx

## Files

- `Justfile`
- `Cargo.toml`
- `.env.example`
