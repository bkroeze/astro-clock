# Phase 8: CLI & API Integration - Research

**Researched:** 2026-03-02
**Domain:** Rust CLI tooling (clap), HTTP API (axum), job result display
**Confidence:** HIGH

## Summary

Phase 8 integrates the complete job system (established in Phases 5-7) with user-facing CLI commands and HTTP endpoints. The infrastructure already exists: job types, repository, executor, and handlers are fully implemented. This phase focuses on exposing functionality through:

1. **CLI extensions**: New `query` subcommands (wedding, project, travel) and `job` subcommands (status, list) to complement existing `load` command
2. **API endpoint additions**: Dedicated POST endpoints per query type and GET /api/v1/jobs listing endpoint
3. **Result formatting**: Human-readable CLI output for job results (structured tables, color coding)

The codebase already follows established patterns: clap for CLI parsing, axum for HTTP routing, and JSON serialization via serde. Implementation leverages existing AppState, JobExecutor, and repository patterns from Phases 6-7.

**Primary recommendation:** Follow the existing `Load` command pattern from Phase 6—create dedicated CLI subcommands that validate input, create appropriate payloads, and format results. For API, mirror the existing `POST /api/v1/query/:query_name` pattern but add dedicated routes for each query type and implement pagination for job listing.

<user_constraints>
## User Constraints (from CONTEXT.md)

No CONTEXT.md exists for Phase 8—this is the initial research phase.

### Phase 8 Requirements (from REQUIREMENTS.md)
- CLI-01: Command `astro-clock query wedding --start YYYY-MM-DD --days N [--sync]`
- CLI-02: Command `astro-clock query project --start YYYY-MM-DD --days N [--sync]`
- CLI-03: Command `astro-clock query travel --start YYYY-MM-DD --days N [--sync]`
- CLI-04: Command `astro-clock job status <job-id>` — get job status and results
- CLI-05: Command `astro-clock job list` — list recent jobs with statuses
- API-01: Endpoint `POST /api/v1/query/wedding` with `{start_date, days, sync?}`
- API-02: Endpoint `POST /api/v1/query/project` with `{start_date, days, sync?}`
- API-03: Endpoint `POST /api/v1/query/travel` with `{start_date, days, sync?}`
- API-04: Endpoint `GET /api/v1/jobs/{job-id}` — get job status and results
- API-05: Endpoint `GET /api/v1/jobs` — list recent jobs
- API-06: Job result response includes: `status`, `created_at`, `completed_at`, `result` (JSON) or `error` (object)
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CLI-01 | `astro-clock query wedding` command | Use existing `Load` command pattern (src/cli/app.rs:326-447), mirror sync/async execution, deserialize QueryResult for display |
| CLI-02 | `astro-clock query project` command | Same pattern as CLI-01, different query name parameter |
| CLI-03 | `astro-clock query travel` command | Same pattern as CLI-01, different query name parameter |
| CLI-04 | `astro-clock job status <job-id>` | Query JobRepository::get_job() via runtime-per-async-block pattern (STATE.md decision 15) |
| CLI-05 | `astro-clock job list` command | Use JobRepository::list_jobs() with pagination (limit/offset), format as table |
| API-01 | POST /api/v1/query/wedding endpoint | Clone existing query_handler pattern (src/server/routes/queries.rs:61-152), hardcode query_name |
| API-02 | POST /api/v1/query/project endpoint | Same pattern as API-01, hardcode "project" |
| API-03 | POST /api/v1/query/travel endpoint | Same pattern as API-01, hardcode "travel" |
| API-04 | GET /api/v1/jobs/{job-id} | Already implemented (src/server/routes/jobs.rs:165-192) |
| API-05 | GET /api/v1/jobs (list) | Add new handler using JobRepository::list_jobs() (src/jobs/repository.rs:141-163) |
| API-06 | Job result response format | Use existing build_job_response() (src/server/routes/jobs.rs:195-223), already includes all fields |
</phase_requirements>

## Standard Stack

### Core (Already in Project)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| clap | 4.x | CLI argument parsing | Derive macros for declarative commands, validation |
| axum | 0.7.x | HTTP framework | Type-safe routing, middleware, IntoResponse |
| serde/serde_json | 1.x | Serialization | Standard Rust ecosystem choice |
| uuid | 1.x | Job IDs | Already used for job identifiers |
| chrono | 0.4.x | Date/time handling | NaiveDate parsing for YYYY-MM-DD validation |
| tokio | 1.x | Async runtime | spawn_blocking for sync CLI context |
| sqlx | 0.7.x | Database access | Repository pattern already established |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tabled | 0.14+ | CLI table formatting | For `job list` output (human-readable tables) |
| colored | 2.x | Terminal colors | Status coloring (green=complete, red=failed, yellow=pending) |

**Installation:**
```bash
# Already installed (from existing Cargo.toml)
# Add only if new tabled/colored desired:
cargo add tabled colored
```

## Architecture Patterns

### Pattern 1: CLI Command Structure (from Load command)
**What:** Subcommand with validation → runtime creation → handler execution → result formatting
**When to use:** All new CLI commands that interact with database/jobs
**Example:** (from src/cli/app.rs:326-447)
```rust
Commands::Load { start, days, sync } => {
    // 1. Validate input
    let _start_date = chrono::NaiveDate::parse_from_str(start, "%Y-%m-%d")?;
    if *days < 1 || *days > 365 { /* error */ }
    
    // 2. Create runtime and execute
    let db_pool = tokio::runtime::Runtime::new()?.block_on(async {
        DatabasePool::connect(&db_url).await
    })?;
    
    // 3. Create repositories and executor
    let job_repo = JobRepository::new(pool.clone());
    let executor = JobExecutor::new(job_repo, handlers, "cli-worker".to_string());
    
    // 4. Build payload and execute
    let payload = json!({"start_date": start, "days": days});
    
    if *sync {
        let result = executor.execute_sync(JobType::Query, payload).await;
        // Format and display result
    } else {
        let job_id = executor.execute_async(JobType::Query, payload).await;
        // Display job ID for polling
    }
}
```

### Pattern 2: API Handler Pattern
**What:** Extract state → validate input → call executor → return structured response
**When to use:** All new HTTP endpoints
**Example:** (from src/server/routes/queries.rs:61-152)
```rust
pub async fn query_handler(
    State(state): State<AppState>,
    Path(query_name): Path<String>,
    Json(request): Json<QueryRequest>,
) -> impl IntoResponse {
    // 1. Validate
    if !valid_queries.contains(&query_name.as_str()) { /* 400 error */ }
    
    // 2. Build payload
    let payload = json!({"query_name": query_name, ...});
    
    // 3. Execute
    if is_sync {
        match state.executor().execute_sync(JobType::Query, payload).await {
            Ok(job) => (StatusCode::OK, Json(response)).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, ...).into_response(),
        }
    } else {
        match state.executor().execute_async(JobType::Query, payload).await {
            Ok(job_id) => (StatusCode::ACCEPTED, Json(response)).into_response(),
            ...
        }
    }
}
```

### Pattern 3: Dedicated Query Endpoints
**What:** Hardcoded query_name in route handler instead of path parameter
**When to use:** API-01, API-02, API-03 (dedicated endpoints vs generic)
**Example:**
```rust
// Instead of: .route("/api/v1/query/:query_name", post(query_handler))
// Use dedicated routes:
.route("/api/v1/query/wedding", post(wedding_query_handler))
.route("/api/v1/query/project", post(project_query_handler))
.route("/api/v1/query/travel", post(travel_query_handler))

// Handler simply hardcodes the query name:
pub async fn wedding_query_handler(...) -> impl IntoResponse {
    let payload = json!({"query_name": "wedding", ...});
    // ... rest same as generic handler
}
```

### Pattern 4: Job Listing with Pagination
**What:** Optional status filter + limit/offset for paginated results
**When to use:** API-05 (GET /api/v1/jobs)
**Example:** (from src/jobs/repository.rs:141-163)
```rust
// Repository method already exists:
pub async fn list_jobs(
    &self,
    status: Option<JobStatus>,
    limit: i64,
    offset: i64,
) -> JobResult<Vec<Job>>

// Request/Response types:
#[derive(Debug, Deserialize)]
pub struct ListJobsRequest {
    pub status: Option<String>,  // "pending", "in_process", "complete", "failed"
    pub limit: Option<i64>,      // default: 20
    pub offset: Option<i64>,     // default: 0
}

#[derive(Debug, Serialize)]
pub struct ListJobsResponse {
    pub jobs: Vec<JobResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
```

### Pattern 5: CLI Result Formatting
**What:** Deserialize job.result JSON into structured struct, display as human-readable table
**When to use:** CLI-01, CLI-02, CLI-03, CLI-05 (any command showing job results)
**Example:** (from src/cli/app.rs:393-412)
```rust
if let Some(ref result) = job.result {
    if let Ok(load_result) = serde_json::from_value::<LoadJobResult>(result.clone()) {
        println!("\n✓ Load completed successfully");
        println!("  Dates loaded: {}", load_result.dates_loaded);
        println!("  Dates skipped: {}", load_result.dates_skipped);
        // ... etc
    }
}
```

### Anti-Patterns to Avoid
- **Don't block the main thread**: Always use `spawn_blocking` or `Runtime::new()?.block_on()` for async operations in sync CLI context
- **Don't leak database details**: Repository already abstracts SQLx—keep CLI handlers focused on user interaction
- **Don't hardcode pagination limits in multiple places**: Use constants or config
- **Don't parse UUIDs manually**: Use `Uuid::parse_str()` which provides clear error messages

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| CLI table output | Custom formatting | `tabled` crate | Handles alignment, wrapping, headers automatically |
| Terminal colors | ANSI escape codes | `colored` crate | Cross-platform, handles NO_COLOR env var |
| Date validation | Regex parsing | `chrono::NaiveDate::parse_from_str` | Correct leap year, month/day validation |
| JSON pretty printing | Custom serializers | `serde_json::to_string_pretty` | Standard, handles escaping |
| Pagination math | Manual offset calculations | Repository LIMIT/OFFSET | Database-optimized |

**Key insight:** The project already has patterns for all these—follow the Load command and existing API handlers rather than inventing new approaches.

## Common Pitfalls

### Pitfall 1: Runtime-per-async-block Pattern Mismatch
**What goes wrong:** Creating a new tokio runtime for each async operation in sync CLI context causes hangs or deadlocks
**Why it happens:** Attempting to call `.await` in sync functions without proper runtime setup
**How to avoid:** Follow STATE.md decision 15: "Runtime-per-async-block pattern—Create tokio runtime for each async block in sync CLI context"
```rust
// CORRECT:
let result = tokio::runtime::Runtime::new()?.block_on(async {
    executor.execute_sync(JobType::Query, payload).await
});

// WRONG (would cause compile error anyway):
let result = executor.execute_sync(JobType::Query, payload).await;
```

### Pitfall 2: Feature Flag Confusion
**What goes wrong:** CLI commands fail to compile or run when `db` feature is disabled
**Why it happens:** Database-dependent code not wrapped in `#[cfg(feature = "db")]`
**How to avoid:** Follow existing pattern (src/cli/app.rs:343-447):
```rust
#[cfg(feature = "db")]
{
    // Database code here
}

#[cfg(not(feature = "db"))]
{
    Err(crate::errors::Error::Config(
        "Database support not enabled...".to_string()
    ))
}
```

### Pitfall 3: UUID Parsing Errors
**What goes wrong:** `job status <invalid-uuid>` produces confusing error messages
**Why it happens:** Not validating UUID format before database query
**How to avoid:** Parse UUID first, return clear 400/CLI error if invalid:
```rust
let job_id = match Uuid::parse_str(&job_id_str) {
    Ok(id) => id,
    Err(_) => return Err(Error::Config(format!("Invalid job ID: {}", job_id_str))),
};
```

### Pitfall 4: Query Result Display
**What goes wrong:** Query results display as raw JSON instead of human-readable format
**Why it happens:** Not deserializing result into appropriate struct before display
**How to avoid:** Define result structs (e.g., `WeddingQueryResult`) and deserialize:
```rust
#[derive(Deserialize)]
struct WeddingQueryResult {
    candidates: Vec<WeddingCandidate>,
}

// Then:
if let Ok(result) = serde_json::from_value::<WeddingQueryResult>(job.result.clone()) {
    // Format as table
}
```

### Pitfall 5: Pagination Default Values
**What goes wrong:** API requests without limit/offset cause unbounded queries
**Why it happens:** Not providing defaults for optional parameters
**How to avoid:** Use `unwrap_or()` with sensible defaults:
```rust
let limit = request.limit.unwrap_or(20).min(100);  // Cap at 100
let offset = request.offset.unwrap_or(0);
```

## Code Examples

### CLI Query Command Pattern
```rust
/// Query subcommand for wedding, project, travel
#[derive(Subcommand, Debug)]
pub enum QueryCommands {
    /// Find auspicious wedding dates
    Wedding {
        #[arg(long, value_name = "DATE")]
        start: String,
        #[arg(long, value_name = "N")]
        days: i64,
        #[arg(long)]
        sync: bool,
    },
    // ... Project, Travel similar
}

// In match &self.cli.command:
Commands::Query(query_cmd) => {
    let (query_name, start, days, sync) = match query_cmd {
        QueryCommands::Wedding { start, days, sync } => ("wedding", start, days, sync),
        // ... etc
    };
    
    // Validate, create runtime, execute with query_name in payload
}
```

### Job List CLI Output
```rust
// Using tabled crate for formatted output
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct JobListRow {
    #[tabled(rename = "Job ID")]
    job_id: String,
    #[tabled(rename = "Type")]
    job_type: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Created")]
    created_at: String,
}

// Query jobs, map to rows, print table
let rows: Vec<JobListRow> = jobs.into_iter().map(|j| JobListRow {
    job_id: j.id.to_string(),
    job_type: j.job_type,
    status: j.status,
    created_at: j.created_at.format("%Y-%m-%d %H:%M").to_string(),
}).collect();

println!("{}", Table::new(rows));
```

### API Job Listing Handler
```rust
#[derive(Debug, Deserialize)]
pub struct ListJobsRequest {
    pub status: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default = "default_offset")]
    pub offset: i64,
}

fn default_limit() -> i64 { 20 }
fn default_offset() -> i64 { 0 }

pub async fn list_jobs_handler(
    State(state): State<AppState>,
    Query(params): Query<ListJobsRequest>,
) -> impl IntoResponse {
    let repository = JobRepository::new(state.get_pool());
    
    // Parse status if provided
    let status_filter = params.status.and_then(|s| match s.as_str() {
        "pending" => Some(JobStatus::Pending),
        "in_process" => Some(JobStatus::InProcess),
        "complete" => Some(JobStatus::Complete),
        "failed" => Some(JobStatus::Failed),
        _ => None,
    });
    
    match repository.list_jobs(status_filter, params.limit, params.offset).await {
        Ok(jobs) => {
            let responses: Vec<JobResponse> = jobs.into_iter()
                .map(build_job_response)
                .collect();
            (StatusCode::OK, Json(json!({"jobs": responses}))).into_response()
        }
        Err(e) => { /* error response */ }
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Separate CLI/API result structs | Unified JobResponse | Phase 6 | Single source of truth for job serialization |
| Path-param query names | Dedicated endpoints per query | Phase 8 | Better API documentation, type safety |
| Manual table formatting | `tabled` crate | Phase 8 (proposed) | Less code, better alignment |
| Raw JSON CLI output | Structured deserialization | Phase 6-7 | Human-readable results |

**Deprecated/outdated:**
- None—all patterns from previous phases remain current

## Open Questions

1. **Should we use tabled/colored crates or keep it simple?**
   - What we know: Project currently has minimal CLI output formatting
   - What's unclear: Whether adding dependencies for table formatting is warranted
   - Recommendation: Start with simple `println!` formatting (as in Load command), upgrade to tabled if needed

2. **What should `job list` display by default?**
   - What we know: Repository has list_jobs() with status filter, limit, offset
   - What's unclear: Default limit (20?), which fields to show, sorting order
   - Recommendation: Default to last 20 jobs, sorted by created_at DESC, show: ID (shortened), type, status, created_at

3. **Should we implement job result formatting per query type?**
   - What we know: Each query type has different result structure
   - What's unclear: Whether generic JSON display is sufficient or type-specific tables needed
   - Recommendation: Phase 8 focuses on infrastructure—use generic JSON pretty-print; Phase 9 could add type-specific formatters

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (built-in) |
| Config file | None—standard Rust test setup |
| Quick run command | `cargo test --lib` |
| Full suite command | `cargo test --all-features` |
| Estimated runtime | ~30 seconds |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CLI-01 | Wedding query execution | integration | `cargo test --test cli_query_tests` | ❌ Wave 0 gap |
| CLI-02 | Project query execution | integration | `cargo test --test cli_query_tests` | ❌ Wave 0 gap |
| CLI-03 | Travel query execution | integration | `cargo test --test cli_query_tests` | ❌ Wave 0 gap |
| CLI-04 | Job status display | integration | `cargo test --test cli_job_tests` | ❌ Wave 0 gap |
| CLI-05 | Job list display | integration | `cargo test --test cli_job_tests` | ❌ Wave 0 gap |
| API-01 | POST /api/v1/query/wedding | integration | `cargo test --test api_query_tests` | ❌ Wave 0 gap |
| API-02 | POST /api/v1/query/project | integration | `cargo test --test api_query_tests` | ❌ Wave 0 gap |
| API-03 | POST /api/v1/query/travel | integration | `cargo test --test api_query_tests` | ❌ Wave 0 gap |
| API-04 | GET /api/v1/jobs/{job-id} | unit | `cargo test routes::jobs` | ✅ exists |
| API-05 | GET /api/v1/jobs (list) | unit | `cargo test routes::jobs` | ❌ Wave 0 gap |
| API-06 | Response format | unit | `cargo test routes::jobs::tests` | ✅ exists |

### Nyquist Sampling Rate
- **Minimum sample interval:** After every committed task → run: `cargo check --all-features`
- **Full suite trigger:** Before merging final task of any plan wave
- **Phase-complete gate:** Full suite green before `/gsd-verify-work` runs
- **Estimated feedback latency per task:** ~15 seconds

### Wave 0 Gaps (must be created before implementation)
- [ ] `tests/cli_query_tests.rs` — covers CLI-01, CLI-02, CLI-03
- [ ] `tests/cli_job_tests.rs` — covers CLI-04, CLI-05
- [ ] `tests/api_query_tests.rs` — covers API-01, API-02, API-03
- [ ] Add list_jobs_handler test to `src/server/routes/jobs.rs` tests — covers API-05

## Sources

### Primary (HIGH confidence)
- `src/cli/app.rs` — Existing CLI implementation patterns
- `src/server/routes/jobs.rs` — Job API handlers and response types
- `src/server/routes/queries.rs` — Query handler pattern
- `src/server/mod.rs` — Router setup and route registration
- `src/jobs/repository.rs` — JobRepository::list_jobs() already implemented
- `.planning/STATE.md` — Decisions 15, 16, 17, 18, 19, 20, 23, 24, 25

### Secondary (MEDIUM confidence)
- `src/jobs/handlers/query.rs` — QueryJobHandler implementation details
- `src/queries/` — Wedding, project, travel query implementations

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — Using existing project dependencies
- Architecture: HIGH — Clear patterns from Phases 6-7
- Pitfalls: MEDIUM — Some uncertainty around edge cases in pagination

**Research date:** 2026-03-02
**Valid until:** 2026-04-02 (30 days—stable Rust ecosystem)
