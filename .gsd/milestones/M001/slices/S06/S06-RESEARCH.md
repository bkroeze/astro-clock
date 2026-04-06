# Phase 6: Data Loading - Research

**Researched:** 2026-03-01  
**Domain:** Rust job handlers, incremental data loading, CLI/API integration  
**Confidence:** HIGH

## Summary

Phase 6 builds on the job infrastructure from Phase 5 to implement day-level incremental data loading with intelligent resume capability. The core task is implementing `LoadJobHandler` — a `JobHandler` trait implementation that generates planetary data for date ranges using the existing `ChunkGenerator`, while tracking progress in the `loaded_days` table.

**Key insight:** The project already has mature data generation (ChunkGenerator) and job execution (JobExecutor) infrastructure. Phase 6 is primarily about **integration** — wiring these systems together through the `JobHandler` trait and exposing them via CLI/API.

**Primary recommendation:** Implement `LoadJobHandler` as a thin orchestration layer that delegates to `ChunkGenerator` for actual data generation, uses `LoadedDaysRepository` for date tracking, and produces structured job results compatible with the job status endpoints.

## Standard Stack

### Core (Already Present)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `tokio` | 1.0 | Async runtime | Already used throughout codebase |
| `sqlx` | 0.8 | Database access | Existing pattern in repositories |
| `serde` / `serde_json` | 1.0 | Serialization | Job payload/results use JSONB |
| `chrono` | 0.4 | Date/time handling | NaiveDate for date ranges |
| `uuid` | 1.0 | Job identifiers | Jobs table uses UUID PK |
| `thiserror` | 1.0 | Error handling | Established pattern in jobs module |
| `clap` | 4.0 | CLI parsing | Existing CLI uses derive macros |
| `axum` | 0.7 | HTTP API | Existing server module |
| `async-trait` | 0.1 | Trait async methods | Already used for `JobHandler` |
| `tracing` | 0.1 | Logging | Used throughout codebase |

### Project-Specific Components (Reused)
| Component | Purpose | Location |
|-----------|---------|----------|
| `ChunkGenerator` | Swiss Ephemeris data generation | `src/database/chunk_generator.rs` |
| `LoadedDaysRepository` | Date range tracking | `src/jobs/repository.rs` |
| `JobExecutor` | Job lifecycle management | `src/jobs/executor.rs` |
| `JobHandler` trait | Pluggable job handlers | `src/jobs/executor.rs` |
| `JobType::Load` | Job type identifier | `src/jobs/types.rs` |

## Architecture Patterns

### Recommended Project Structure

Phase 6 adds handlers for job types:

```
src/
├── jobs/
│   ├── mod.rs              # Existing - add handler exports
│   ├── types.rs            # Existing - JobType::Load already defined
│   ├── repository.rs       # Existing - LoadedDaysRepository ready
│   ├── executor.rs         # Existing - JobExecutor ready
│   ├── error.rs            # Existing - JobError types
│   └── handlers/           # NEW: Job-specific handlers
│       ├── mod.rs          # Handler module exports
│       └── load.rs         # LoadJobHandler implementation
├── cli/
│   └── app.rs              # Add 'load' subcommand
├── server/
│   └── mod.rs              # Add POST /api/v1/load endpoint
```

### Pattern 1: JobHandler Implementation

**What:** Implement the `JobHandler` trait for data loading operations.

**When to use:** When creating a new job type that the `JobExecutor` can process.

**Example (from `src/jobs/executor.rs`):**
```rust
#[async_trait]
pub trait JobHandler: Send + Sync {
    fn job_type(&self) -> JobType;
    async fn execute(&self, job: &Job) -> JobResult<JsonValue>;
}
```

**LoadJobHandler structure:**
```rust
pub struct LoadJobHandler {
    pool: Pool<Postgres>,
    loaded_days_repo: LoadedDaysRepository,
    chunk_generator: ChunkGenerator,
}

#[async_trait]
impl JobHandler for LoadJobHandler {
    fn job_type(&self) -> JobType {
        JobType::Load
    }

    async fn execute(&self, job: &Job) -> JobResult<JsonValue> {
        // 1. Parse payload (start_date, days)
        // 2. Use LoadedDaysRepository::get_missing_dates to find gaps
        // 3. For each missing date:
        //    - Generate chunk via ChunkGenerator::generate_chunk
        //    - Save to database via ChunkGenerator::save_chunk_to_db
        //    - Mark as loaded via LoadedDaysRepository::mark_day_loaded
        // 4. Return result JSON with stats
    }
}
```

### Pattern 2: Job Payload Structure

**What:** JSON payload format for load jobs.

**When to use:** Creating jobs via CLI or API.

**Standard payload:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct LoadJobPayload {
    pub start_date: String,  // "YYYY-MM-DD"
    pub days: i64,           // Number of days to load
}
```

**Standard result:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct LoadJobResult {
    pub dates_requested: Vec<String>,
    pub dates_loaded: Vec<String>,
    pub dates_skipped: Vec<String>,
    pub total_positions: usize,
    pub total_aspects: usize,
    pub total_lunar_conditions: usize,
}
```

### Pattern 3: Sync/Async Dual Mode Execution

**What:** Execute jobs in blocking (CLI) or background (API) mode.

**When to use:** CLI uses `execute_sync`, API uses `execute_async`.

**From Phase 5 implementation:**
```rust
// CLI: blocks until complete, returns result directly
let job = executor.execute_sync(JobType::Load, payload).await?;

// API: returns job-id immediately, processes in background
let job_id = executor.execute_async(JobType::Load, payload).await?;
```

### Pattern 4: Incremental Loading with Gap Detection

**What:** Skip already-loaded days, only process missing dates.

**From `src/jobs/repository.rs`:**
```rust
impl LoadedDaysRepository {
    /// Returns dates in range that are NOT yet loaded
    pub async fn get_missing_dates(
        &self,
        start: NaiveDate,
        days: i64,
    ) -> JobResult<Vec<NaiveDate>> {
        // Uses PostgreSQL generate_series for efficient gap detection
    }

    /// Mark a date as successfully loaded
    pub async fn mark_day_loaded(
        &self,
        date: NaiveDate,
        coverage_minutes: i16,
        job_id: Option<Uuid>,
    ) -> JobResult<()> {
        // UPSERT into loaded_days table
    }
}
```

### Pattern 5: CLI Subcommand Structure

**What:** Add `load` subcommand using clap derive macros.

**Following existing pattern in `src/cli/app.rs`:**
```rust
#[derive(Subcommand, Debug)]
pub enum Commands {
    // ... existing commands

    /// Load planetary data for date range
    Load {
        /// Start date (YYYY-MM-DD)
        #[arg(long, value_name = "DATE")]
        start: String,

        /// Number of days to load
        #[arg(long, value_name = "N")]
        days: i64,

        /// Wait for completion (synchronous mode)
        #[arg(long)]
        sync: bool,
    },
}
```

### Pattern 6: API Endpoint Structure

**What:** Add axum route handler for load endpoint.

**Following existing server pattern:**
```rust
async fn load_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoadJobPayload>,
) -> Result<impl IntoResponse, StatusCode> {
    let payload_json = serde_json::to_value(payload)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    if payload.sync.unwrap_or(false) {
        // Synchronous: block and return result
        let job = state.executor.execute_sync(JobType::Load, payload_json).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Json(job))
    } else {
        // Asynchronous: return job-id immediately
        let job_id = state.executor.execute_async(JobType::Load, payload_json).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Json(json!({ "job_id": job_id, "status": "pending" })))
    }
}
```

### Anti-Patterns to Avoid

- **Don't bypass the job system:** Always use `JobExecutor` for data loading, don't call `ChunkGenerator` directly from CLI/API handlers. This ensures consistent state tracking and error handling.

- **Don't reimplement gap detection:** Use `LoadedDaysRepository::get_missing_dates` instead of querying individual dates in a loop.

- **Don't ignore transaction boundaries:** Each day's data should be saved in a single transaction (already handled by `ChunkGenerator::save_chunk_to_db`).

- **Don't fail entire job on single day failure:** Track per-day success/failure and report partial results.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Swiss Ephemeris calculations | Custom ephemeris | `ChunkGenerator` | Complex astronomical math, already tested |
| Date range iteration | Manual date math | `LoadedDaysRepository::get_missing_dates` | Uses PostgreSQL generate_series, handles timezones |
| Day tracking | Custom tracking table | `loaded_days` table + `LoadedDaysRepository` | Already has migrations, FK to jobs |
| Job execution | Custom thread pool | `JobExecutor` with spawn_blocking | Handles CPU-intensive work, state machine |
| JSON serialization | Manual JSON construction | `serde_json` | Type-safe, tested, used throughout |
| CLI parsing | Manual arg parsing | `clap` derive macros | Existing pattern in codebase |
| HTTP routing | Custom server | `axum` | Existing pattern in codebase |

**Key insight:** All infrastructure exists. This phase is about **composition**, not **construction**.

## Common Pitfalls

### Pitfall 1: Date Boundary Errors

**What goes wrong:** Off-by-one errors when calculating date ranges, timezone issues when converting dates.

**Why it happens:** `NaiveDate` doesn't include timezone info, but database stores `TIMESTAMPTZ`.

**How to avoid:**
- Always use `NaiveDate` for date parameters (YYYY-MM-DD without time)
- Convert to `DateTime<Utc>` at midnight UTC for database queries
- Use `chrono::Duration::days(1)` for date arithmetic

**Warning signs:** Tests pass locally but fail in CI (different timezone), or loaded dates are off by one day.

### Pitfall 2: Memory Exhaustion on Large Ranges

**What goes wrong:** Loading 365 days at once exhausts memory with `spawn_blocking`.

**Why it happens:** Each day generates ~14,400 position records (10 bodies × 1,440 minutes).

**How to avoid:**
- Process days in batches (e.g., 7 days at a time)
- Stream results rather than collecting all in memory
- Use the existing `ChunkGenerator` which already handles single-day generation

**Warning signs:** OOM kills during testing, high memory usage in logs.

### Pitfall 3: Job State Inconsistency

**What goes wrong:** Job marked as `Complete` but not all days were loaded due to partial failure.

**Why it happens:** Error handling doesn't distinguish between "some days failed" and "all days failed".

**How to avoid:**
- Track per-day success/failure in job result
- Only mark `Complete` if all requested days were processed
- Mark `Failed` only if no progress could be made
- Include partial results in the result JSON

**Warning signs:** loaded_days table shows gaps despite job status being Complete.

### Pitfall 4: Swiss Ephemeris Thread Safety

**What goes wrong:** Panic or incorrect results when multiple threads use Swiss Ephemeris simultaneously.

**Why it happens:** Swiss Ephemeris C library may have global state.

**How to avoid:**
- Use `spawn_blocking` (already used in `JobExecutor::execute_sync`)
- Generate one day at a time, sequentially within a job
- Let `JobExecutor` handle the concurrency control

**Warning signs:** Intermittent test failures, incorrect planetary positions.

### Pitfall 5: CLI/API Divergence

**What goes wrong:** CLI works but API fails (or vice versa) due to different payload handling.

**Why it happens:** CLI parses arguments differently than API deserializes JSON.

**How to avoid:**
- Use the same `LoadJobPayload` struct for both
- Centralize validation in the handler, not in CLI/API layers
- Share error handling logic

**Warning signs:** Same parameters work via CLI but fail via API.

## Code Examples

### LoadJobHandler Implementation

```rust
// src/jobs/handlers/load.rs
use async_trait::async_trait;
use chrono::NaiveDate;
use serde_json::Value as JsonValue;
use sqlx::Pool;
use sqlx::Postgres;
use uuid::Uuid;

use crate::database::chunk_generator::ChunkGenerator;
use crate::jobs::error::{JobError, JobResult};
use crate::jobs::executor::JobHandler;
use crate::jobs::repository::LoadedDaysRepository;
use crate::jobs::types::{Job, JobType};

#[derive(Debug, Clone)]
pub struct LoadJobHandler {
    loaded_days_repo: LoadedDaysRepository,
    chunk_generator: ChunkGenerator,
}

impl LoadJobHandler {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self {
            loaded_days_repo: LoadedDaysRepository::new(pool.clone()),
            chunk_generator: ChunkGenerator::new(pool.into()),
        }
    }
}

#[async_trait]
impl JobHandler for LoadJobHandler {
    fn job_type(&self) -> JobType {
        JobType::Load
    }

    async fn execute(&self, job: &Job) -> JobResult<JsonValue> {
        // Parse payload
        let payload = job.payload.as_ref()
            .ok_or_else(|| JobError::Other("Missing payload".to_string()))?;
        
        let start_date_str = payload.get("start_date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JobError::Other("Missing start_date".to_string()))?;
        
        let days = payload.get("days")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| JobError::Other("Missing days".to_string()))?;

        let start_date = NaiveDate::parse_from_str(start_date_str, "%Y-%m-%d")
            .map_err(|e| JobError::Other(format!("Invalid date format: {}", e)))?;

        // Find missing dates
        let missing_dates = self.loaded_days_repo
            .get_missing_dates(start_date, days)
            .await?;

        let mut loaded = Vec::new();
        let mut failed = Vec::new();
        let mut skipped = Vec::new();

        // Calculate which dates are already loaded (for reporting)
        let end_date = start_date + chrono::Duration::days(days - 1);
        let mut all_dates = Vec::new();
        let mut current = start_date;
        while current <= end_date {
            all_dates.push(current);
            current = current + chrono::Duration::days(1);
        }
        
        for date in &all_dates {
            if !missing_dates.contains(date) {
                skipped.push(date.to_string());
            }
        }

        // Load missing dates
        for date in missing_dates {
            match self.chunk_generator.generate_chunk(date).await {
                Ok(chunk) => {
                    let positions = chunk.planet_positions.len();
                    let aspects = chunk.aspects.len();
                    let lunar = chunk.lunar_conditions.len();

                    match self.chunk_generator.save_chunk_to_db(&chunk).await {
                        Ok(_) => {
                            self.loaded_days_repo
                                .mark_day_loaded(date, 1440, Some(job.id))
                                .await?;
                            loaded.push(serde_json::json!({
                                "date": date.to_string(),
                                "positions": positions,
                                "aspects": aspects,
                                "lunar_conditions": lunar,
                            }));
                        }
                        Err(e) => {
                            failed.push(serde_json::json!({
                                "date": date.to_string(),
                                "error": e.to_string(),
                            }));
                        }
                    }
                }
                Err(e) => {
                    failed.push(serde_json::json!({
                        "date": date.to_string(),
                        "error": e.to_string(),
                    }));
                }
            }
        }

        // Return result
        Ok(serde_json::json!({
            "start_date": start_date_str,
            "days_requested": days,
            "dates_loaded": loaded.len(),
            "dates_skipped": skipped.len(),
            "dates_failed": failed.len(),
            "loaded": loaded,
            "skipped": skipped,
            "failed": failed,
        }))
    }
}
```

### CLI Integration

```rust
// In src/cli/app.rs Commands enum
#[derive(Subcommand, Debug)]
pub enum Commands {
    // ... existing commands

    /// Load planetary data for a date range
    Load {
        /// Start date (YYYY-MM-DD)
        #[arg(long, value_name = "DATE")]
        start: String,

        /// Number of days to load
        #[arg(long, value_name = "N")]
        days: i64,

        /// Execute synchronously and wait for completion
        #[arg(long)]
        sync: bool,
    },
}

// In App::run() match statement
Commands::Load { start, days, sync } => {
    // Parse and validate date
    let _start_date = NaiveDate::parse_from_str(&start, "%Y-%m-%d")
        .map_err(|e| crate::errors::Error::Config(format!("Invalid date: {}", e)))?;

    if days <= 0 || days > 365 {
        return Err(crate::errors::Error::Config(
            "Days must be between 1 and 365".to_string()
        ));
    }

    let payload = serde_json::json!({
        "start_date": start,
        "days": days,
    });

    // Note: Actual implementation would create executor and run job
    // This requires database pool which is available with --features db
    println!("Loading data for {} days starting from {}", days, start);
    
    if *sync {
        println!("Synchronous mode: waiting for completion...");
        // let job = executor.execute_sync(JobType::Load, payload).await?;
        // println!("Job completed: {:?}", job);
    } else {
        println!("Asynchronous mode: job submitted");
        // let job_id = executor.execute_async(JobType::Load, payload).await?;
        // println!("Job ID: {}", job_id);
    }
    
    Ok(())
}
```

### API Integration

```rust
// In src/server/mod.rs or new routes module
#[derive(Debug, Deserialize)]
pub struct LoadRequest {
    pub start_date: String,
    pub days: i64,
    #[serde(default)]
    pub sync: bool,
}

async fn load_handler(
    State(state): State<AppState>,
    Json(req): Json<LoadRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Validate request
    if req.days <= 0 || req.days > 365 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let payload = serde_json::json!({
        "start_date": req.start_date,
        "days": req.days,
    });

    if req.sync {
        match state.executor.execute_sync(JobType::Load, payload).await {
            Ok(job) => Ok((StatusCode::OK, Json(json!({
                "job_id": job.id,
                "status": job.status,
                "result": job.result,
            })))),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        match state.executor.execute_async(JobType::Load, payload).await {
            Ok(job_id) => Ok((StatusCode::ACCEPTED, Json(json!({
                "job_id": job_id,
                "status": "pending",
                "poll_url": format!("/api/v1/jobs/{}", job_id),
            })))),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| On-demand generation | ChunkManager with LRU cache | Phase 2 (v1.0) | Query speedup via caching |
| Single-threaded | spawn_blocking for CPU work | Phase 5 (v1.1) | Non-blocking async runtime |
| Direct DB calls | Repository pattern | Phase 5 (v1.1) | Testable, mockable data access |
| Ad-hoc status tracking | Job table with state machine | Phase 5 (v1.1) | Reliable async operation tracking |
| Re-generate everything | Incremental loading with skip | Phase 6 (v1.1) | Resume capability, efficiency |

**Deprecated/outdated:**
- Direct Swiss Ephemeris calls from CLI handlers (use JobHandler instead)
- Manual date iteration with `String` (use `NaiveDate` and `Duration`)
- Single-transaction for multiple days (process days independently)

## Open Questions

1. **Batch size for large ranges:**
   - What we know: Swiss Ephemeris generation is CPU-intensive
   - What's unclear: Optimal batch size for memory vs. throughput
   - Recommendation: Start with sequential day-by-day processing (simplest), measure before optimizing

2. **Error handling granularity:**
   - What we know: Job can partially succeed (some days loaded, some failed)
   - What's unclear: Whether to mark job Complete (with warnings) or Failed
   - Recommendation: Mark Complete if ≥1 day loaded, include failed dates in result JSON; mark Failed only if 0 days loaded

3. **Progress tracking:**
   - What we know: Job executor doesn't support progress updates currently
   - What's unclear: Whether to add progress for long-running loads
   - Recommendation: Defer to v1.2 — current polling-based status is sufficient for v1.1

## Validation Architecture

> Skipped — `workflow.nyquist_validation` is false in `.planning/config.json`

## Sources

### Primary (HIGH confidence)
- `src/jobs/executor.rs` - JobHandler trait, JobExecutor implementation
- `src/jobs/types.rs` - Job, JobType, JobStatus definitions
- `src/jobs/repository.rs` - JobRepository, LoadedDaysRepository
- `src/database/chunk_generator.rs` - ChunkGenerator for data generation
- `migrations/008_create_jobs.sql` - Jobs table schema
- `migrations/009_create_loaded_days.sql` - Loaded days tracking schema

### Secondary (MEDIUM confidence)
- `src/cli/app.rs` - Existing CLI patterns with clap
- `src/server/mod.rs` - Existing axum patterns
- `src/database/chunk_manager.rs` - Usage patterns for ChunkGenerator

### Phase 5 Summaries (Implementation precedent)
- `05-01-SUMMARY.md` - Database schema decisions
- `05-02-SUMMARY.md` - Repository pattern, String-Enum bridge
- `05-03-SUMMARY.md` - JobExecutor dual modes, spawn_blocking pattern

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All libraries already in use
- Architecture: HIGH - Built on proven Phase 5 patterns
- Pitfalls: MEDIUM-HIGH - Based on existing codebase analysis and Rust async best practices

**Research date:** 2026-03-01  
**Valid until:** 2026-04-01 (30 days for stable patterns)

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| LOAD-06 | CLI command `astro-clock load --start YYYY-MM-DD --days N` | Pattern 5 (CLI), LoadJobHandler example |
| LOAD-07 | API endpoint `POST /api/v1/load` with `{start_date, days, sync?}` | Pattern 6 (API), handler example |
| LOAD-08 | Day-level incremental loading — skip already-loaded days | Pattern 4 (gap detection), get_missing_dates |
| LOAD-09 | Loading jobs populate planet_positions, aspects, lunar_conditions | ChunkGenerator::save_chunk_to_db pattern |
| LOAD-10 | Track loaded days in tracking table for resume capability | LoadedDaysRepository::mark_day_loaded |
| RESULT-03 | Job status endpoint returns full job details | Job struct already has all fields |
| RESULT-04 | Failed jobs include error code and message | Error JSON pattern in LoadJobHandler |
</content>