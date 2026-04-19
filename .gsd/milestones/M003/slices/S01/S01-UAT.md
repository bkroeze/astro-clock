# S01: Enhanced list filters — UAT

**Milestone:** M003
**Written:** 2026-04-19T19:06:18.786Z

# S01: Enhanced list filters — UAT

**Milestone:** M003
**Written:** 2026-04-19

## UAT Type

- UAT mode: artifact-driven
- Why this mode is sufficient: The slice is purely a query-parameter filtering enhancement with comprehensive unit tests covering all parsing and validation logic. No server startup or database is needed to verify the filter contract.

## Preconditions

- Rust toolchain installed
- Project compiles with `cargo build --features db`

## Smoke Test

Run `cargo test --features db --lib -- server::routes::jobs` and confirm all 36 tests pass.

## Test Cases

### 1. Multi-value status filter parsing

1. Call `parse_comma_separated::<JobStatus>("complete,failed")`
2. **Expected:** Returns `Ok(vec![JobStatus::Complete, JobStatus::Failed])`

### 2. Multi-value job_type filter parsing

1. Call `parse_comma_separated::<JobType>("load,query")`
2. **Expected:** Returns `Ok(vec![JobType::Load, JobType::Query])`

### 3. Invalid status value rejected

1. Call `parse_comma_separated::<JobStatus>("complete,invalid_status")`
2. **Expected:** Returns `Err` mentioning "invalid_status" in the error message

### 4. Invalid job_type value rejected

1. Call `parse_comma_separated::<JobType>("load,nonexistent")`
2. **Expected:** Returns `Err` mentioning "nonexistent" in the error message

### 5. Date parsing — RFC3339

1. Call `parse_date_param("2025-01-15T10:30:00Z")`
2. **Expected:** Returns `Some(DateTime)` representing 2025-01-15T10:30:00 UTC

### 6. Date parsing — YYYY-MM-DD fallback

1. Call `parse_date_param("2025-03-01")`
2. **Expected:** Returns `Some(DateTime)` representing 2025-03-01T00:00:00 UTC

### 7. Date parsing — RFC3339 with timezone offset

1. Call `parse_date_param("2025-01-15T10:30:00+05:00")`
2. **Expected:** Returns `Some(DateTime)` correctly converted to UTC

### 8. Date parsing — invalid format rejected

1. Call `parse_date_param("not-a-date")`
2. **Expected:** Returns `None`

### 9. Combined filters

1. Construct `ListJobsRequest` with `status=Some("complete,failed")`, `job_type=Some("load")`, `created_after=Some("2025-01-01")`, `created_before=Some("2025-03-01")`
2. Parse into `JobListFilters`
3. **Expected:** Filters contain 2 statuses, 1 job_type, and both date bounds set

### 10. Empty filters — backward compatibility

1. Construct `ListJobsRequest` with all filter fields as `None`
2. Parse into `JobListFilters`
3. **Expected:** Empty status vec, empty job_type vec, no date bounds — equivalent to "return all jobs"

### 11. Single status — backward compatibility

1. Construct `ListJobsRequest` with `status=Some("complete")`, no other filters
2. Parse into `JobListFilters`
3. **Expected:** Filters contain exactly `[JobStatus::Complete]`

### 12. Whitespace handling in CSV values

1. Call `parse_comma_separated::<JobStatus>("complete, failed")`
2. **Expected:** Returns `Ok(vec![JobStatus::Complete, JobStatus::Failed])` — whitespace trimmed

## Edge Cases

- Empty string CSV: `parse_comma_separated::<JobStatus>("")` returns `Ok(vec![])` (no filter applied)
- Response shape unchanged: `ListJobsResponse` still contains `jobs`, `total`, `limit`, `offset` — no new fields added
- CLI list command still works: `handle_job_list` constructs `JobListFilters` from single status value
