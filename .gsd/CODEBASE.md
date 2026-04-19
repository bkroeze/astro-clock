# Codebase Map

Generated: 2026-04-19T19:49:40Z | Files: 128 | Described: 0/128
<!-- gsd:codebase-meta {"generatedAt":"2026-04-19T19:49:40Z","fingerprint":"8be64c64ded046d81064392ffb8625b634cd0e96","fileCount":128,"truncated":false} -->

### (root)/
- `.emdash.json`
- `.env.example`
- `.gitignore`
- `.python-version`
- `AGENTS.md`
- `api.yaml`
- `Cargo.toml`
- `config.ron`
- `Justfile`
- `opencode.json`
- `pyproject.toml`
- `README.md`
- `tmp.json`

### .iteratr/data/
- `.iteratr/data/server.port`

### .iteratr/data/jetstream/$G/streams/iteratr_events/
- `.iteratr/data/jetstream/$G/streams/iteratr_events/meta.inf`
- `.iteratr/data/jetstream/$G/streams/iteratr_events/meta.sum`

### .iteratr/data/jetstream/$G/streams/iteratr_events/msgs/
- `.iteratr/data/jetstream/$G/streams/iteratr_events/msgs/1.blk`
- `.iteratr/data/jetstream/$G/streams/iteratr_events/msgs/index.db`

### .tickets/
- *(25 files: 25 .md)*

### data/
- `data/bruce.ron`
- `data/fonts.csv`

### docs/
- `docs/README.md`
- `docs/RUST_CLI_TOOLS_BEST_PRACTICES.md`
- `docs/swiss-eph.md`

### docs/plans/
- `docs/plans/2026-03-26-svg-glyphs-design.md`
- `docs/plans/2026-03-26-svg-glyphs-implementation.md`
- `docs/plans/ELECTORAL_PLAN.md`
- `docs/plans/implementation_plan.md`

### fonts/AstronomiconFonts_1.1/
- `fonts/AstronomiconFonts_1.1/OFL-FAQ.txt`
- `fonts/AstronomiconFonts_1.1/OFL-License.txt`

### migrations/
- `migrations/001_create_hypertables.sql`
- `migrations/002_create_indexes.sql`
- `migrations/003_create_locations.sql`
- `migrations/004_create_aspect_summaries.sql`
- `migrations/005_create_retrograde_periods.sql`
- `migrations/006_create_continuous_aggregates.sql`
- `migrations/007_create_benchmark_results.sql`
- `migrations/008_create_jobs.sql`
- `migrations/009_create_loaded_days.sql`
- `migrations/010_create_jobs_cursor_index.sql`
- `migrations/README.md`

### notebooks/
- `notebooks/explore.py`

### notebooks/__marimo__/session/
- `notebooks/__marimo__/session/explore.py.json`

### scripts/
- `scripts/reset-db.sh`
- `scripts/verify_schema.sql`

### src/
- `src/aspects.rs`
- `src/chart.rs`
- `src/config.rs`
- `src/errors.rs`
- `src/lib.rs`
- `src/logging.rs`
- `src/main.rs`
- `src/output_handler.rs`
- `src/renderer.rs`
- `src/svg_glyph_paths.rs`
- `src/svg_glyphs.rs`
- `src/svg_renderer.rs`
- `src/swiss_eph_impl.rs`

### src/bin/
- `src/bin/main.rs`

### src/cli/
- `src/cli/app.rs`
- `src/cli/mod.rs`

### src/database/
- `src/database/chunk_generator.rs`
- `src/database/chunk_manager.rs`
- `src/database/chunk.rs`
- `src/database/mod.rs`
- `src/database/multi_resolution.rs`
- `src/database/pool.rs`
- `src/database/schema.rs`

### src/ephemeris/
- `src/ephemeris/mod.rs`

### src/jobs/
- `src/jobs/error.rs`
- `src/jobs/executor.rs`
- `src/jobs/mod.rs`
- `src/jobs/registry.rs`
- `src/jobs/repository.rs`
- `src/jobs/types.rs`

### src/jobs/handlers/
- `src/jobs/handlers/load.rs`
- `src/jobs/handlers/mod.rs`
- `src/jobs/handlers/query.rs`

### src/performance/
- `src/performance/benchmark.rs`
- `src/performance/interpolation.rs`
- `src/performance/memory_monitor.rs`
- `src/performance/mod.rs`

### src/queries/
- `src/queries/aspects.rs`
- `src/queries/benchmark.rs`
- `src/queries/error.rs`
- `src/queries/mod.rs`
- `src/queries/project.rs`
- `src/queries/retrograde.rs`
- `src/queries/travel.rs`
- `src/queries/types.rs`
- `src/queries/voc.rs`
- `src/queries/wedding.rs`

### src/server/
- `src/server/mod.rs`
- `src/server/state.rs`

### src/server/routes/
- `src/server/routes/jobs.rs`
- `src/server/routes/mod.rs`
- `src/server/routes/queries.rs`

### tests/
- `tests/api_integration.rs`
- `tests/api_query_tests.rs`
- `tests/cli_integration.rs`
- `tests/cli_job_tests.rs`
- `tests/cli_query_tests.rs`
- `tests/svg_output_test.rs`

### tests/common/
- `tests/common/mod.rs`
