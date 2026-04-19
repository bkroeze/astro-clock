---
estimated_steps: 46
estimated_files: 2
skills_used: []
---

# T03: Update CLI call site, existing tests, and add comprehensive cursor tests

## Steps

1. **Update CLI `JobCommands::List` variant** in `src/cli/app.rs`:
   - Replace `limit` and `offset` args with `count`:
     ```rust
     List {
         #[arg(long, value_name = "STATUS")]
         status: Option<String>,
         /// Number of jobs to show (default: 20, max: 100)
         #[arg(long, value_name = "N")]
         count: Option<i64>,
     }
     ```

2. **Update `handle_job_list`** in `src/cli/app.rs`:
   - Change signature from `(status, limit, offset, config)` to `(status, count, config)`
   - Set default count: `let count = count.unwrap_or(20).min(100).max(1);`
   - Pass `CursorDirection::Forward` and `None` cursor to `list_jobs` (CLI only supports forward pagination, first page)
   - Remove the offset/"Use --offset X to see more results" hint since cursors are opaque
   - Adjust the output format — remove total/offset display, optionally note that cursor pagination is active

3. **Update `handle_job_command`** match arm to pass new args:
   ```rust
   JobCommands::List { status, count } => {
       self.handle_job_list(status.as_ref(), *count, config)
   }
   ```

4. **Run `cargo check --features db`** to find all compilation errors from the signature changes. Fix every call site.

5. **Add comprehensive cursor-specific tests** in `src/server/routes/jobs.rs` test module:
   - `test_cursor_roundtrip_various_timestamps` — encode/decode with past, future, edge-case timestamps
   - `test_build_page_url_preserves_status_filter` — URL contains `status=complete,failed`
   - `test_build_page_url_preserves_job_type_filter` — URL contains `job_type=load`
   - `test_build_page_url_preserves_date_filters` — URL contains created_after and created_before
   - `test_build_page_url_minimal` — URL contains only count and cursor (no filters)
   - `test_empty_results_no_next_prev` — when 0 results, both next and prev are None
   - `test_single_page_no_next` — when results fit in one page, next is None

6. **Run full test suite**: `cargo test --features db --lib` — all 188+ tests pass with 0 failures.

7. **Run `cargo test --features db --lib -- cli::app`** to verify CLI tests still pass.

## Must-Haves

- [ ] CLI uses count instead of limit/offset
- [ ] CLI calls updated list_jobs signature correctly
- [ ] All existing tests updated to new response shape
- [ ] Cursor URL construction tests with filter preservation
- [ ] Full test suite passes (cargo test --features db --lib)

## Verification

```bash
cargo test --features db --lib
```
Full test suite passes with 0 failures. `cargo test --features db --lib -- cli::app` passes for CLI-specific tests.

## Inputs

- ``src/jobs/repository.rs` — finalized list_jobs signature from T01`
- ``src/server/routes/jobs.rs` — finalized handler from T02`
- ``src/cli/app.rs` — current handle_job_list with limit/offset`

## Expected Output

- ``src/cli/app.rs` — updated handle_job_list with count parameter`
- ``src/server/routes/jobs.rs` — all tests passing with new response shape, new cursor-specific tests added`

## Verification

cargo test --features db --lib
