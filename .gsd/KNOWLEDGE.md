# Project Knowledge

## Database & CLI

- **CLI reads `DATABASE_URL` env var, not `PG_URL`.** The load/query/job commands in `src/cli/app.rs` use `std::env::var("DATABASE_URL")`. The `.env` file uses `PG_URL` for other tooling (sqlx-cli, psql). The Justfile's `test-db-setup` recipe must set `DATABASE_URL=$TEST_PG_URL` when running the load command.

- **TimescaleDB hypertables don't support `ON CONFLICT`.** The `INSERT ... ON CONFLICT DO NOTHING` syntax fails on hypertable chunks with "no unique or exclusion constraint matching the ON CONFLICT specification". Use plain `INSERT` for hypertables since the test DB is always freshly created.

- **Tokio runtime must span pool creation and query execution.** Creating a sqlx `PgPool` inside one `Runtime::new().block_on()` and using it inside a different one causes "pool timed out" errors. The CLI load command was refactored to use a single runtime for both pool creation and job execution.

- **`and_hms_opt(0, minute, 0)` bug.** The chunk_generator.rs had `and_hms_opt(0, minute, 0)` where `minute` ranges 0..1439. For minutes >= 60, this tries to set the "minutes" field to 60+, which fails. Fixed to `and_hms_opt(minute / 60, minute % 60, 0)` in all three save methods (positions, aspects, lunar conditions).

- **Migration 006 continuous aggregate bug.** The `last(retrograde::smallint, time)::boolean` cast fails in TimescaleDB continuous aggregates. Fixed by using `last(CASE WHEN retrograde THEN 1::smallint ELSE 0::smallint END, time) AS retrograde_smallint`.

## Seed Data (2025-01-01 to 2025-03-02)

- 60 days loaded (Jan 29 failed due to moon_phase_angle constraint — acceptable ~1.6% loss)
- 864,000 position records (10 bodies × 86,400 minutes)
- 1,465,067 aspects total
- 86,400 lunar conditions (1 per minute)
- Retrograde bodies: Venus (1,402 min), Mars (76,441 min), Jupiter (48,102 min), Uranus (41,304 min)
- 68 distinct VoC periods, 48,294 VoC minutes
- Moon transits all 12 signs with 27 sign changes
