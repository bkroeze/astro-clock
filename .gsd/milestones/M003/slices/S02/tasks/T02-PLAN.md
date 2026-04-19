---
estimated_steps: 77
estimated_files: 1
skills_used: []
---

# T02: Change handler request/response shape and build next/prev URLs

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

## Inputs

- ``src/jobs/repository.rs` — JobCursor, CursorDirection, updated list_jobs signature from T01`
- ``src/server/routes/jobs.rs` — current ListJobsRequest/Response and list_jobs_handler`

## Expected Output

- ``src/server/routes/jobs.rs` — updated request/response types, rewritten handler with URL building, new unit tests for cursor parsing and URL generation`

## Verification

cargo test --features db --lib -- server::routes::jobs
