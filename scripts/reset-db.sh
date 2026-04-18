#!/usr/bin/env bash
set -euo pipefail

usage() {
    echo "Usage: $0 <database_url> [--seed]"
    echo ""
    echo "  database_url   PostgreSQL connection URL"
    echo "  --seed         Load 60 days of seed data after migrations (default: off)"
    exit 1
}

if [ $# -lt 1 ]; then
    usage
fi

DB_URL="$1"
SEED=false

if [ "${2:-}" = "--seed" ]; then
    SEED=true
fi

if [ -z "${DB_URL}" ]; then
    echo "Error: database URL is empty"
    usage
fi

DB_NAME=$(echo "${DB_URL}" | sed 's|.*/||')
MAINTENANCE_URL=$(echo "${DB_URL}" | sed "s|/${DB_NAME}|/postgres|")

echo "=== Reset Database ==="
echo "  Database: ${DB_NAME}"

echo "Dropping existing database..."
psql "${MAINTENANCE_URL}" -c "DROP DATABASE IF EXISTS \"${DB_NAME}\";" 2>/dev/null || true

echo "Creating database..."
psql "${MAINTENANCE_URL}" -c "CREATE DATABASE \"${DB_NAME}\";"

echo "Running migrations 1-5..."
sqlx migrate run --database-url "${DB_URL}" || true

echo "Applying migration 6 (continuous aggregates)..."
psql "${DB_URL}" -f migrations/006_create_continuous_aggregates.sql 2>&1 || true

echo "Applying migrations 7-9..."
for m in migrations/00[7-9]*.sql; do
    echo "  Applying $(basename "${m}")..."
    psql "${DB_URL}" -f "${m}" 2>&1 || true
done

if [ "${SEED}" = true ]; then
    echo "Loading 60 days of seed data..."
    DATABASE_URL="${DB_URL}" cargo run --bin astro-clock --features db -- load --start 2025-01-01 --days 60 --sync
    echo "  Seed data range: 2025-01-01 to 2025-03-01 (60 days)"
fi

echo ""
echo "=== Database reset complete ==="
