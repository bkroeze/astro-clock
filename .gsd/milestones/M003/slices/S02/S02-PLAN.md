# S02: Cursor-based pagination with next/prev URLs

**Goal:** Replace offset/limit pagination with cursor-based pagination on GET /api/v1/jobs. Cursors encode (created_at, id) tuples as opaque base64 tokens. The response shape changes from {jobs, total, limit, offset} to {jobs, next, prev} where next/prev are full URLs encoding the cursor and all active filter parameters.
**Demo:** GET /api/v1/jobs?count=5 returns page with next/prev URLs; following next returns the next stable page; inserting jobs between page fetches doesn't shift results

## Must-Haves

- GET /api/v1/jobs?count=5 returns first page with next URL and null prev
- Following next URL returns next page with both next and prev URLs
- Cursor encoding is opaque base64 JSON containing (created_at, id)
- Filter parameters are preserved in next/prev URLs
- Inserting jobs between page fetches does not shift results
- Backward pagination (following prev) returns the correct previous page
- count parameter defaults to 20, max 100, min 1
- All existing tests pass after response shape change

## Proof Level

- This slice proves: contract

## Integration Closure

Upstream surfaces consumed: JobListFilters from src/jobs/repository.rs (S01), parse_comma_separated and parse_date_param helpers from src/server/routes/jobs.rs (S01). New wiring: OriginalUri extractor in list_jobs_handler for URL construction. What remains: S03 integration tests will exercise the full cursor pagination flow against a real database.

## Verification

- Invalid cursor strings return 400 with descriptive error message including the decode failure reason. This is the primary diagnostic signal for pagination issues. The response shape no longer includes total — diagnostic count queries are available via CLI.

## Tasks

- [x] **T01: Add cursor types, encode/decode, and change list_jobs SQL to cursor-based** `est:1.5h`
  ## Steps

1. Add `base64` crate to `[dependencies]` in `Cargo.toml`:
   ```toml
   base64 = "0.22"
   ```

2. In `src/jobs/repository.rs`, add imports at the top:
   ```rust
   use base64::engine::general_purpose::URL_SAFE_NO_PAD;
   use base64::Engine;
   use serde::{Deserialize, Serialize};
   use uuid::Uuid;
   ```

3. Define `JobCursor` struct and `CursorDirection` enum above `JobRepository`:
   ```rust
   /// Opaque cursor encoding a (created_at, id) boundary row.
   /// Clients treat this as an opaque string — the encoding may change.
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct JobCursor {
       pub created_at: DateTime<Utc>,
       pub id: Uuid,
   }

   impl JobCursor {
       /// Encode cursor as base64url-no-pad JSON string
       pub fn encode(&self) -> String {
           let json = serde_json::to_string(self).expect("cursor serialization infallible");
           URL_SAFE_NO_PAD.encode(json)
       }

       /// Decode cursor from base64url-no-pad string
       pub fn decode(s: &str) -> Result<Self, String> {
           let json = URL_SAFE_NO_PAD.decode(s)
               .map_err(|e| format!("Invalid cursor encoding: {}", e))?;
           serde_json::from_slice(&json)
               .map_err(|e| format!("Invalid cursor data: {}", e))
       }
   }

   /// Direction of cursor pagination
   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   pub enum CursorDirection {
       /// Forward = older jobs (default direction, DESC order)
       Forward,
       /// Backward = newer jobs (reversed within page to maintain DESC order)
       Backward,
   }
   ```

4. Rewrite `list_jobs()` signature and SQL. Replace the current `(filters, limit, offset)` with `(filters, count, cursor, direction)`. The key SQL change:
   - **No cursor (first page):** `ORDER BY created_at DESC, id DESC LIMIT $count + 1`
   - **Forward (next page):** `WHERE (created_at, id) < ($cursor_at, $cursor_id) ORDER BY created_at DESC, id DESC LIMIT $count + 1`
   - **Backward (prev page):** `WHERE (created_at, id) > ($cursor_at, $cursor_id) ORDER BY created_at ASC, id ASC LIMIT $count + 1` then reverse results
   - The `+1` row detects whether another page exists
   - IMPORTANT: For backward pagination, apply cursor condition AFTER the filter WHERE clauses (same position as forward), but change the comparison from `<` to `>` and the ORDER BY from DESC to ASC. After fetching, reverse the results to maintain DESC order in the response.

5. Keep `count_jobs()` method as-is — it's still useful for CLI diagnostics.

6. Create `migrations/010_create_jobs_cursor_index.sql`:
   ```sql
   -- Composite index for cursor-based pagination using (created_at, id) tuple comparison
   CREATE INDEX idx_jobs_cursor_pagination ON jobs(created_at DESC, id DESC);
   ```

7. Add a `#[cfg(test)]` module inside `repository.rs` (or extend existing) with cursor encode/decode unit tests:
   - `test_cursor_encode_decode_roundtrip` — encode then decode, verify fields match
   - `test_cursor_decode_invalid_base64` — garbage string returns error
   - `test_cursor_decode_invalid_json` — valid base64 of non-JSON returns error
   - `test_cursor_encode_no_padding` — verify no `=` padding chars in encoded string

## Must-Haves

- [ ] JobCursor struct with encode()/decode() using base64url-no-pad JSON
- [ ] CursorDirection enum (Forward, Backward)
- [ ] list_jobs() rewritten with cursor-based WHERE clause using tuple comparison
- [ ] list_jobs() fetches count+1 rows to detect page existence
- [ ] Backward pagination reverses results to maintain DESC order
- [ ] Composite index migration file created
- [ ] base64 crate added to Cargo.toml
- [ ] Encode/decode roundtrip unit tests pass

## Verification

```bash
cargo test --features db --lib -- jobs::repository::tests::cursor
```
All cursor encode/decode tests pass. `cargo check --features db` compiles without errors.
  - Files: `Cargo.toml`, `src/jobs/repository.rs`, `migrations/010_create_jobs_cursor_index.sql`
  - Verify: cargo test --features db --lib -- jobs::repository::tests::cursor

- [x] **T02: Change handler request/response shape and build next/prev URLs** `est:1.5h`
  ## Steps

1. **Update `ListJobsRequest`** in `src/server/routes/jobs.rs`:
   - Remove `limit` and `offset` fields
   - Add `count` field: `#[serde(default = "default_count")] pub count: i64` where `default_count()` returns 20
   - Add `cursor` field: `#[serde(default)] pub cursor: Option<String>`
   - Remove `default_limit` and `default_offset` functions, add `default_count`

2. **Update `ListJobsResponse`**:
   - Remove `total`, `limit`, `offset` fields
   - Add `next: Option<String>` and `prev: Option<String>` fields

3. **Add `build_page_url` helper function**:
   ```rust
   fn build_page_url(
       base_path: &str,
       count: i64,
       cursor: &JobCursor,
       status: Option<&str>,
       job_type: Option<&str>,
       created_after: Option<&str>,
       created_before: Option<&str>,
   ) -> String {
       // Build URL with query params: count, cursor, + any active filters
       // Use form_urlencoding or manual construction
   }
   ```
   This function must include all active filter parameters in the URL so that paginating through a filtered result set stays filtered.

4. **Rewrite `list_jobs_handler`** signature to include `OriginalUri`:
   ```rust
   pub async fn list_jobs_handler(
       State(state): State<AppState>,
       OriginalUri(original_uri): OriginalUri,
       Query(params): Query<ListJobsRequest>,
   ) -> impl IntoResponse
   ```
   Add import: `use axum::extract::OriginalUri;`

5. **Rewrite handler body**:
   - Validate/cap count: `let count = params.count.max(1).min(100);`
   - Parse filters (status, job_type, created_after, created_before) exactly as before
   - Decode cursor if present: `let (cursor, direction) = match params.cursor { ... }`
     - No cursor → `(None, CursorDirection::Forward)`
     - Has cursor → `(Some(JobCursor::decode(&cursor)?), CursorDirection::Forward)`
     - On decode error → return 400 with descriptive message
   - Call `repository.list_jobs(filters, count, cursor, direction).await`
   - Determine has_next/has_prev:
     - If results.len() > count: trim to count, set has_next (forward) or has_prev (backward)
     - If cursor is Some and results.len() > 0: set has_prev (forward) or has_next (backward)
     - If no cursor (first page): prev is always null
   - Build next/prev URLs using `build_page_url` with the boundary row's cursor
   - Return `ListJobsResponse { jobs, next, prev }`

6. **IMPORTANT: Remove the `count_jobs` call** — the handler no longer needs total count. The response has no `total` field.

7. **Add URL construction import** — use `form_urlencoding` (from `url` crate) or manual `&param=value` construction. Check if `url` crate is available; if not, construct manually with proper percent-encoding. The `base64::URL_SAFE_NO_PAD` encoding already produces URL-safe characters.

8. **Update existing tests** that reference old `ListJobsResponse` shape:
   - `test_list_jobs_request_deserialization` — update to use `count` instead of `limit`/`offset`
   - `test_list_jobs_request_defaults` — verify `count` defaults to 20
   - `test_list_jobs_request_partial` — update fields
   - `test_list_jobs_request_all_new_fields` — update fields
   - `test_list_jobs_response_serialization` — verify `next`/`prev` fields instead of `total`/`limit`/`offset`
   - `test_list_jobs_response_includes_pagination` — verify `next`/`prev` fields

9. **Add new handler-layer tests**:
   - `test_list_jobs_request_with_count_and_cursor` — verify new fields deserialize
   - `test_cursor_decode_in_handler` — verify invalid cursor returns 400-style error
   - `test_build_page_url_with_filters` — verify URL includes filter params
   - `test_build_page_url_without_filters` — verify URL has only count and cursor
   - `test_first_page_no_prev` — verify prev is None when no cursor provided

## Must-Haves

- [ ] ListJobsRequest uses count (default 20, max 100) and cursor instead of limit/offset
- [ ] ListJobsResponse has next/prev Option<String> instead of total/limit/offset
- [ ] Handler builds full URLs for next/prev including filter params
- [ ] Invalid cursor string returns 400 with descriptive error
- [ ] OriginalUri extractor used for base path
- [ ] count_jobs call removed from handler
- [ ] All existing tests updated to new response shape
- [ ] New handler-layer cursor tests added

## Verification

```bash
cargo test --features db --lib -- server::routes::jobs
```
All handler tests pass including new cursor-specific tests.
  - Files: `src/server/routes/jobs.rs`
  - Verify: cargo test --features db --lib -- server::routes::jobs

- [x] **T03: Update CLI call site, existing tests, and add comprehensive cursor tests** `est:1h`
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
  - Files: `src/cli/app.rs`, `src/server/routes/jobs.rs`
  - Verify: cargo test --features db --lib

## Files Likely Touched

- Cargo.toml
- src/jobs/repository.rs
- migrations/010_create_jobs_cursor_index.sql
- src/server/routes/jobs.rs
- src/cli/app.rs
