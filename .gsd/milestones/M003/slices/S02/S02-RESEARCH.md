# S02 — Research

**Date:** 2026-04-19

## Summary

S02 replaces the current offset/limit pagination with cursor-based pagination anchored on `(created_at, id)` tuples, and adds auto-generated `next`/`prev` URLs to the response. This is a moderately complex slice: the cursor encoding/decoding is straightforward (base64 JSON), but the SQL cursor comparison and the next/prev URL generation require careful implementation.

The current `list_jobs` uses `ORDER BY created_at DESC LIMIT $1 OFFSET $2` with `JobListFilters` for dynamic WHERE clauses. S02 must: (1) add a `count` parameter replacing `limit`/`offset`, (2) add cursor parameters (`after` for next, `before` for prev), (3) change the SQL to use tuple comparison `(created_at, id) < ($1, $2)` instead of OFFSET, (4) build `next`/`prev` URLs encoding cursor + filters, (5) change the response shape from `{jobs, total, limit, offset}` to `{jobs, next, prev}`.

## Recommendation

Replace `limit`/`offset` with `count` parameter and cursor parameters. Use PostgreSQL row tuple comparison `(created_at, id) < ($1, $2)` for forward pagination and `(created_at, id) > ($1, $2)` for backward pagination (both in DESC order context). Encode the cursor as `base64url(JSON({"created_at": "...", "id": "..."}))`. Build next/prev URLs by combining the base path `/api/v1/jobs` with query parameters for filters + cursor. The handler needs `axum::extract::OriginalUri` to get the request path for URL construction.

Build order: (1) cursor types + encode/decode in `repository.rs`, (2) SQL change in `list_jobs` to accept cursor instead of offset, (3) response shape change + URL building in handler, (4) unit tests for cursor encoding, URL generation, and request parsing.

## Implementation Landscape

### Key Files

- `src/jobs/repository.rs` — `JobListFilters`, `list_jobs()`, `count_jobs()`. Needs: `JobCursor` struct with encode/decode methods, `list_jobs()` signature change from `(filters, limit, offset)` to `(filters, count, cursor: Option<JobCursor>, direction: CursorDirection)`.
- `src/server/routes/jobs.rs` — `ListJobsRequest`, `ListJobsResponse`, `list_jobs_handler()`. Needs: replace `limit`/`offset` with `count`/`cursor` params, change response shape to include `next`/`prev` URLs, add `OriginalUri` extractor for URL building.
- `migrations/010_create_jobs_cursor_index.sql` (new) — Add composite index `(created_at DESC, id)` for efficient cursor pagination.
- `Cargo.toml` — Add `base64` dependency.
- `src/cli/app.rs` — Update CLI call site from `list_jobs(filters, limit, offset)` to new signature.
- `tests/api_integration.rs` — Update existing pagination tests to use `count` instead of `limit`/`offset`.

### Build Order

1. **Add `base64` crate to `Cargo.toml`** — unblocks all cursor work.
2. **Create cursor types in `repository.rs`** — `JobCursor` struct with `(DateTime<Utc>, Uuid)`, `encode()` → `String` (base64url JSON), `decode()` from `&str`. Add `CursorDirection` enum (`Forward`, `Backward`). Unit-testable in isolation.
3. **Add composite index migration** — `(created_at DESC, id)` index for cursor pagination queries. The existing `idx_jobs_created_at` index only has `created_at`; adding `id` as second column enables the row tuple comparison to be index-backed.
4. **Change `list_jobs()` SQL** — Replace `LIMIT/OFFSET` with cursor-based WHERE. For forward (next): `WHERE (created_at, id) < ($cursor_at, $cursor_id)` applied after filter clauses. For first page (no cursor): no tuple condition. For backward (prev): `WHERE (created_at, id) > ($cursor_at, $cursor_id)` with `ORDER BY created_at ASC, id ASC` then reverse results. Always `LIMIT count + 1` to detect if there's a next/prev page.
5. **Update `ListJobsRequest` and response** — Replace `limit`/`offset` with `count` (default 20, max 100) and `cursor` (optional string). Change `ListJobsResponse` from `{jobs, total, limit, offset}` to `{jobs, next: Option<String>, prev: Option<String>}`. Build URLs in handler.
6. **Update handler and CLI call site** — Handler extracts `OriginalUri` for base path, constructs URLs with filter params + cursor. CLI constructs `JobListFilters` + count.
7. **Unit tests** — Cursor encode/decode roundtrip, URL generation with various filter combos, backward compatibility (request without cursor = first page).

### Verification Approach

```bash
# Unit tests — cursor encoding
cargo test --features db --lib -- jobs::repository::tests::cursor

# Unit tests — handler (request parsing, URL generation)
cargo test --features db --lib -- server::routes::jobs

# Full unit suite
cargo test --features db --lib

# Integration tests (requires DB)
cargo test --features db --ignored -- list_jobs
```

Behavioral verification: `GET /api/v1/jobs?count=5` returns first page with `next` URL, `null` prev. Following `next` URL returns second page with both `next` and `prev`. Inserting jobs between page fetches doesn't shift results (cursor stability).

## Constraints

- **SQL tuple comparison with QueryBuilder:** `sqlx::QueryBuilder` supports `.push("(created_at, id) < (")` as raw SQL with `.push_bind()` for values. This is not a first-class QueryBuilder feature but works as a pushed SQL fragment. Must use parameterized binds — do not interpolate values into raw SQL.
- **Backward compatibility:** The response shape changes from `{jobs, total, limit, offset}` to `{jobs, next, prev}`. The `total`, `limit`, `offset` fields are removed. Clients must update. This is acceptable per milestone scope (no existing external consumers).
- **`count_jobs` may still be needed:** If we remove `total` from the response, we may not need `count_jobs()` at all. But `count` queries are useful for diagnostics. Consider keeping the method but not calling it in the handler.
- **ORDER BY must include `id` as tiebreaker:** `ORDER BY created_at DESC, id DESC` instead of just `ORDER BY created_at DESC`. This ensures stable ordering when multiple jobs share the same `created_at` timestamp.
- **All code behind `#[cfg(feature = "db")]`** — feature gate applies.

## Common Pitfalls

- **Cursor direction vs sort direction** — With `ORDER BY created_at DESC, id DESC`, "forward" (next page = older jobs) means `(created_at, id) < (cursor)`. "Backward" (prev page = newer jobs) means `(created_at, id) > (cursor)`, but results must be reversed to maintain DESC order in the response. Get this wrong and pages go the wrong direction or overlap.
- **FETCH count+1 rows** — Must request `count + 1` rows to detect whether a next/prev page exists. If you get `count + 1` rows back, trim to `count` and set the appropriate URL. If you get ≤ `count` rows, that's the last page in that direction.
- **First page cursor is null** — No cursor means "start from the beginning." `prev` should be `null` on the first page. Don't generate a cursor that points to nothing.
- **URL encoding of cursor** — Base64url encoding produces URL-safe characters (no `+`, `/`, `=`). Use `base64::engine::general_purpose::URL_SAFE_NO_PAD` to avoid padding `=` characters that would need percent-encoding in URLs.
- **Filter params in next/prev URLs** — The cursor encodes position but NOT filter state. The `next`/`prev` URLs must include the original filter params (`status`, `job_type`, `created_after`, `created_before`) so paginating through a filtered result set stays filtered.

## Open Risks

- **PostgreSQL row comparison with DESC index:** The `<(created_at, id) < ($1, $2)` comparison works with a `(created_at DESC, id DESC)` index because PostgreSQL's row comparison is lexicographic regardless of index direction. However, if performance is suboptimal, we may need to verify with `EXPLAIN ANALYZE` in integration tests. This is a minor risk — the query will be correct either way.
