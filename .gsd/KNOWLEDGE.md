# Project Knowledge

## Database & CLI

- **CLI reads `DATABASE_URL` env var, not `PG_URL`.** The load/query/job commands in `src/cli/app.rs` use `std::env::var("DATABASE_URL")`. The `.env` file uses `PG_URL` for other tooling (sqlx-cli, psql). The Justfile's `test-db-setup` recipe must set `DATABASE_URL=$TEST_PG_URL` when running the load command.

- **TimescaleDB hypertables don't support `ON CONFLICT`.** The `INSERT ... ON CONFLICT DO NOTHING` syntax fails on hypertable chunks with "no unique or exclusion constraint matching the ON CONFLICT specification". Use plain `INSERT` for hypertables since the test DB is always freshly created.

- **Tokio runtime must span pool creation and query execution.** Creating a sqlx `PgPool` inside one `Runtime::new().block_on()` and using it inside a different one causes "pool timed out" errors. The CLI load command was refactored to use a single runtime for both pool creation and job execution.

- **`and_hms_opt(0, minute, 0)` bug.** The chunk_generator.rs had `and_hms_opt(0, minute, 0)` where `minute` ranges 0..1439. For minutes >= 60, this tries to set the "minutes" field to 60+, which fails. Fixed to `and_hms_opt(minute / 60, minute % 60, 0)` in all three save methods (positions, aspects, lunar conditions).

- **Migration 006 continuous aggregate bug.** The `last(retrograde::smallint, time)::boolean` cast fails in TimescaleDB continuous aggregates. Fixed by using `last(CASE WHEN retrograde THEN 1::smallint ELSE 0::smallint END, time) AS retrograde_smallint`.
- **Migration 006 `IF NOT EXISTS` fails on re-creation.** TimescaleDB continuous aggregates are special views, not regular materialized views. `CREATE MATERIALIZED VIEW IF NOT EXISTS` tries `DROP MATERIALIZED VIEW` internally which fails with "is not a materialized view". The Justfile test-db-setup recipe works around this by applying migration 6 via psql with error suppression.
- **sqlx migrate run stops at first failure.** Migration 6's continuous aggregate issue blocks migrations 7-9 from being applied. The Justfile recipe applies migrations 1-5 via sqlx, then 6 via psql (tolerating errors), then 7-9 via psql individually.

## Seed Data (2025-01-01 to 2025-03-01, 60 days via --days 60)

- 60 days loaded (Jan 29 failed due to moon_phase_angle constraint — acceptable ~1.6% loss)
- 864,000 position records (10 bodies × 1,440 min/day × 60 days)
- 1,465,067 aspects total
- 86,400 lunar conditions (1 per minute)
- Retrograde bodies: Venus (1,402 min), Mars (76,441 min), Jupiter (48,102 min), Uranus (41,304 min)
- 68 distinct VoC periods, 48,294 VoC minutes
- Moon transits all 12 signs with 27 sign changes

## Testing Patterns

- **`build_app()` must be async in `#[tokio::test]`.** Using `Runtime::new().block_on()` inside a `#[tokio::test]` function panics with "Cannot start a runtime from within a runtime". Make the helper async and use `.await` directly.

- **Project/travel queries fail without derived tables.** The `aspect_summaries` and `retrograde_periods` tables are empty in the test database (seed data only populates raw tables). Project and travel queries return `status=failed` because their SQL depends on these derived tables. Wedding queries work because they use `planet_positions` and `lunar_conditions` directly.

- **CLI pool-timed-out bug affects job/query commands.** `handle_job_status`, `handle_job_list`, and `handle_query_command` in `src/cli/app.rs` each created a PgPool inside one Runtime then used it in another. Fixed by using a single shared Runtime for both pool creation and all async operations (same pattern as the load command).

- **Integration tests use `--ignored` flag.** All database-gated integration tests are marked `#[ignore]` with a note "requires TEST_PG_URL and TimescaleDB with seed data". Run with `cargo test --features db -- --ignored --test-threads=1` via `just test-integration`. Non-ignored tests (constant validation, unit tests) run in normal `cargo test`.

## API Filter Patterns

- **strum EnumString for FromStr on snake_case enums.** Adding `#[derive(strum_macros::EnumString)]` alongside `#[strum(serialize_all = "snake_case")]` gives a `FromStr` impl that matches the serialization format. Used for `JobStatus` and `JobType` comma-separated parsing in query parameters.

- **Generic `parse_comma_separated<T: FromStr>()` for CSV query params.** A single generic function handles parsing any comma-separated query parameter into a `Vec<T>`, collecting invalid values for error reporting. Eliminates duplication when multiple enum types need CSV parsing (see `src/server/routes/jobs.rs`).

- **sqlx QueryBuilder with separated `push_bind()` for IN clauses.** Use `separated.push_bind(value.as_ref())` to generate parameterized `$1, $2, ...` for `IN (...)` clauses. Requires `AsRefStr` derive on enums. Track `has_where` boolean to correctly chain `WHERE`/`AND` keywords across multiple optional conditions.

- **Date parsing: RFC3339 first, YYYY-MM-DD fallback.** `parse_date_param` tries `DateTime::parse_from_rfc3339()` first, then falls back to `NaiveDate::parse_from_str()` + midnight UTC. Gives API consumers maximum flexibility without ambiguity.

## Cursor Pagination Patterns

- **Cursor-based pagination uses (created_at, id) tuple comparison in SQL.** PostgreSQL supports row-wise tuple comparison: `WHERE (created_at, id) < ($1, $2)` leverages the composite index on `(created_at DESC, id DESC)` for efficient page boundary detection. This is more stable than offset/limit when rows are inserted between page fetches.

- **Fetch count+1 rows to detect next page existence.** The list_jobs query fetches one extra row beyond the requested count. If results.len() > count, a next page exists. This avoids a separate COUNT query and is the standard cursor pagination pattern.

- **Backward pagination fetches ASC then reverses.** For `prev` (backward) pagination, the query uses `WHERE (created_at, id) > cursor ORDER BY created_at ASC, id ASC LIMIT count+1` then reverses the results to maintain DESC order in the response. This ensures the client always sees jobs in newest-first order.

- **base64url-no-pad for cursor encoding.** Using `URL_SAFE_NO_PAD` produces cursors without `=` padding, keeping URLs clean. Cursors encode (created_at, id) as JSON then base64 — clients treat them as opaque strings.

- **build_page_url preserves all active filters.** The next/prev URL construction includes all non-None filter parameters (status, job_type, created_after, created_before) alongside count and cursor. This ensures paginating through a filtered result set stays filtered without client-side state.

- **CLI uses count (not cursor) for first-page listing.** The CLI's `job list` command passes `count` with no cursor, only supporting first-page display. Cursor-based navigation is API-only since cursors are opaque URLs unsuitable for CLI flags.
