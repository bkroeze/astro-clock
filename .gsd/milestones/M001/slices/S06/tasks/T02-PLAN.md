# T02: 06-data-loading 02

**Slice:** S06 — **Milestone:** M001

## Description

Add `astro-clock load` CLI command for synchronous and asynchronous data loading.

Purpose: Enable users to load planetary data for date ranges via command line, with choice of blocking (sync) or background (async) execution.

Output: Extended CLI with load subcommand supporting --start, --days, and --sync flags.

## Must-Haves

- [ ] CLI has 'load' subcommand with --start, --days, --sync arguments
- [ ] CLI validates date format (YYYY-MM-DD) and days range (1-365)
- [ ] Sync mode blocks until job completes and displays result
- [ ] Async mode returns job-id immediately for polling
- [ ] CLI creates JobExecutor with LoadJobHandler registered

## Files

- `src/cli/app.rs`
