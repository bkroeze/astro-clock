# Database Migrations

This directory contains SQL migrations for the Astro Clock database.

## Migration Naming Convention

Migrations follow the **sqlx** naming convention:

```
{version}_{description}.sql
```

Where:
- `version`: Sequential number (001, 002, 003, ...)
- `description`: Underscore-separated description of the migration

## Current Migrations

| File | Description |
|------|-------------|
| `001_create_hypertables.sql` | Creates TimescaleDB hypertables for time-series data |
| `002_create_indexes.sql` | Creates composite and partial indexes for query optimization |
| `003_create_locations.sql` | Creates normalized locations table |
| `004_create_aspect_summaries.sql` | Creates aspect summaries table for fast queries |

## Running Migrations

Using Just:

```bash
# Set up database and run all migrations
just db-setup

# Run pending migrations
just migrate

# Check migration status
just migrate-info

# Revert last migration (use with caution)
just migrate-revert

# Create a new migration
just migrate-create migration_name
```

Using sqlx-cli directly:

```bash
# Run migrations
sqlx migrate run --database-url "$PG_URL"

# Check status
sqlx migrate info --database-url "$PG_URL"
```

## Requirements

- PostgreSQL 14+ with TimescaleDB extension
- sqlx-cli installed: `cargo install sqlx-cli`
- `PG_URL` environment variable set

## Migration Discovery

sqlx looks for migrations in the `./migrations` directory by default.
Our naming convention (NNN_description.sql) is fully compatible with sqlx.
