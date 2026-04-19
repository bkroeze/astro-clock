---
estimated_steps: 75
estimated_files: 3
skills_used: []
---

# T01: Add cursor types, encode/decode, and change list_jobs SQL to cursor-based

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

## Inputs

- ``src/jobs/repository.rs` — current list_jobs with offset/limit and JobListFilters`
- ``Cargo.toml` — needs base64 dependency added`

## Expected Output

- ``Cargo.toml` — base64 dependency added`
- ``src/jobs/repository.rs` — JobCursor, CursorDirection types; list_jobs rewritten with cursor SQL; cursor encode/decode tests`
- ``migrations/010_create_jobs_cursor_index.sql` — composite index on (created_at DESC, id DESC)`

## Verification

cargo test --features db --lib -- jobs::repository::tests::cursor
