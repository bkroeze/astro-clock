# Technology Stack: Job System Infrastructure

**Project:** Astro Clock v1.1  
**Domain:** Job queue, async execution, job state management  
**Researched:** 2026-03-01  
**Confidence:** HIGH

## Executive Summary

The job system requires **minimal stack additions** to the existing Astro Clock infrastructure. Given the project's existing use of **PostgreSQL + TimescaleDB + sqlx + tokio**, the most pragmatic approach is to **build a custom lightweight job queue** using SQLx directly rather than adopting a heavyweight job queue library. This avoids external dependencies, leverages existing expertise, and keeps the stack minimal.

**Key Decision:** Custom job queue table + state machine enum over external libraries (pgmq, apalis, fang).

## Existing Stack (v1.0)

| Technology | Version | Purpose | Status |
|------------|---------|---------|--------|
| Rust | 2024 edition | Core language | ✅ Existing |
| tokio | 1.x | Async runtime | ✅ Existing |
| PostgreSQL | 14-16 | Database | ✅ Existing |
| TimescaleDB | 2.x | Time-series extension | ✅ Existing |
| sqlx | 0.8 | Async SQL toolkit | ✅ Existing |
| axum | 0.7 | HTTP server | ✅ Existing |
| tower | 0.5 | Middleware/services | ✅ Existing (dev) |
| clap | 4.x | CLI parsing | ✅ Existing |
| serde | 1.x | Serialization | ✅ Existing |
| serde_json | 1.x | JSON handling | ✅ Existing |

## Recommended Stack Additions

### 1. Job State Management

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **strum** | 0.28 | Enum utilities | Derive `Display`, `AsRefStr`, iteration for job states |

**Rationale:**
- Job state machine needs: `Pending` → `InProgress` → `Complete` | `Failed`
- strum provides `#[derive(Display, AsRefStr, EnumIter, EnumString)]` for seamless state↔string conversion
- Essential for database storage (state as TEXT) and API responses
- Zero runtime overhead, compile-time derive macros only

**Installation:**
```toml
[dependencies]
strum = { version = "0.28", features = ["derive"] }
```

### 2. JSON Payload Storage

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **serde_json** | 1.0 | Job payload serialization | Already in stack — use for job parameters/results |

**Rationale:**
- Already in Cargo.toml (existing dependency)
- Store job payloads as `JSONB` in PostgreSQL for flexibility
- Different job types (LoadDay, RunQuery) can have different payload shapes
- Queryable: PostgreSQL JSON operators allow filtering by payload fields

**Pattern:**
```rust
#[derive(Serialize, Deserialize)]
struct LoadDayPayload {
    date: NaiveDate,
    bodies: Vec<Body>,
}

#[derive(Serialize, Deserialize)]
struct QueryPayload {
    query_name: String,
    params: QueryParams,
}
```

### 3. Async Task Execution

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **tokio::task** | 1.x | Spawn background tasks | Already in stack — use `tokio::spawn` + `JoinHandle` |
| **tokio::sync** | 1.x | Job coordination | `Mutex`, `RwLock`, `mpsc` for job status updates |

**Rationale:**
- No additional dependencies needed
- `tokio::spawn` for fire-and-forget async jobs
- `JoinSet` for managing multiple concurrent jobs
- `spawn_blocking` for CPU-intensive Swiss Ephemeris calculations

**Patterns:**
```rust
// Async job execution
let handle = tokio::spawn(async move {
    run_job(job_id, pool).await
});

// For CPU-intensive work (Swiss Ephemeris)
let result = tokio::task::spawn_blocking(move || {
    calculate_positions(date, bodies)
}).await?;
```

### 4. Job Queue Schema

**No crate needed** — implement as PostgreSQL table using sqlx migrations.

**Schema Design:**
```sql
CREATE TYPE job_status AS ENUM ('pending', 'in_progress', 'complete', 'failed');

CREATE TABLE jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_type TEXT NOT NULL,              -- 'load_day', 'run_query'
    status job_status NOT NULL DEFAULT 'pending',
    payload JSONB NOT NULL,              -- Job parameters
    result JSONB,                        -- Job output (null until complete)
    error TEXT,                          -- Error message if failed
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,              -- When picked up by worker
    completed_at TIMESTAMPTZ,            -- When finished
    retries INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3
);

-- Index for efficient job polling
CREATE INDEX idx_jobs_status_created 
    ON jobs(status, created_at) 
    WHERE status = 'pending';

-- Track loaded date ranges (for data loading jobs)
CREATE TABLE loaded_days (
    date DATE PRIMARY KEY,
    loaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    bodies_loaded TEXT[] NOT NULL,       -- Which bodies were calculated
    resolution_minutes INTEGER NOT NULL  -- 1, 5, or 60
);
```

**Why custom table over pgmq/apalis/fang:**
1. **Simpler** — No new crate dependency or extension to install
2. **Flexible** — Full control over schema (payload, result, job_type)
3. **Transparent** — Direct SQL visibility for debugging
4. **TimescaleDB-native** — Works with existing hypertable patterns
5. **Minimal** — Only need ~200 lines of Rust + SQL

## Alternatives Considered

### pgmq (v0.32)

**What:** PostgreSQL-native message queue (SQS-like)

**Why NOT chosen:**
- Requires PostgreSQL extension (`CREATE EXTENSION pgmq`) — adds deployment complexity
- API designed for message queue patterns, not job state tracking
- Less flexible for complex job results/tracking needs
- Adds 8+ transitive dependencies

**When to use:** If we needed cross-service messaging or exactly-once delivery guarantees with external consumers.

### apalis + apalis-postgres (v0.7 / v1.0-rc)

**What:** Full-featured background job framework with PostgreSQL backend

**Why NOT chosen:**
- **Overkill for our needs** — Designed for high-throughput, distributed job processing
- Heavy dependency tree (tower, ulid, metrics, etc.)
- Opinionated worker pool model doesn't match our simpler CLI/server hybrid
- Still in RC (v1.0 not stable)

**When to use:** If we needed scheduled jobs (cron), retries with backoff, or multiple worker processes.

### fang (v0.10 / v0.11-rc)

**What:** Background processing with PostgreSQL/SQLite/MySQL backends

**Why NOT chosen:**
- Requires `typetag` for trait object serialization — adds complexity
- Migration system conflicts with sqlx's migrate
- Opinionated task trait (`Runnable`/`AsyncRunnable`) doesn't fit our state polling model
- RC version (v0.11) not stable

**When to use:** If we needed CRON scheduling or single-purpose worker pools.

## Architecture Patterns

### Job Lifecycle

```
┌─────────┐    create     ┌─────────┐    pick_up     ┌─────────┐
│  Start  │──────────────▶│ Pending │───────────────▶│ InProgress│
└─────────┘               └─────────┘                └────┬────┘
                                                          │
                    ┌─────────────────────────────────────┼──────┐
                    │                                     │      │
                    ▼                                     ▼      │
               ┌─────────┐                           ┌─────────┐ │
               │ Failed  │◀──────────────────────────│Complete │◀┘
               └────┬────┘   (retry limit reached)   └─────────┘
                    │
                    │ retry
                    └──────────────────────────────────────┐
                                                             │
                    ┌────────────────────────────────────────┘
                    ▼
               ┌─────────┐
               │ Pending │ (retries + 1)
               └─────────┘
```

### Execution Modes

**Synchronous (CLI use case):**
```rust
pub async fn run_job_sync(job_id: Uuid, pool: &PgPool) -> Result<JobResult> {
    execute_job(job_id, pool).await
}
```

**Asynchronous (API use case):**
```rust
pub async fn spawn_job_async(job_id: Uuid, pool: PgPool) -> Result<JoinHandle<()>> {
    let handle = tokio::spawn(async move {
        if let Err(e) = execute_job(job_id, &pool).await {
            tracing::error!("Job {} failed: {}", job_id, e);
        }
    });
    Ok(handle)
}
```

### Job Types

| Job Type | Payload | Result | Description |
|----------|---------|--------|-------------|
| `load_day` | `{ date, bodies, resolution }` | `{ rows_inserted }` | Calculate and store ephemeris data for one day |
| `run_query` | `{ query_name, params }` | `{ matches: Vec<Match> }` | Execute named query template |

## Implementation Notes

### Job Polling (Worker)

```rust
pub async fn poll_next_job(pool: &PgPool) -> Result<Option<Job>> {
    sqlx::query_as::<_, Job>(
        r#"
        UPDATE jobs 
        SET status = 'in_progress', started_at = NOW()
        WHERE id = (
            SELECT id FROM jobs 
            WHERE status = 'pending' 
            ORDER BY created_at 
            FOR UPDATE SKIP LOCKED
            LIMIT 1
        )
        RETURNING *
        "#
    )
    .fetch_optional(pool)
    .await
}
```

**Key features:**
- `FOR UPDATE SKIP LOCKED` — Prevents race conditions between multiple workers
- Single query atomically claims job
- Returns immediately if no pending jobs

### State Machine Transitions

```rust
#[derive(Debug, Clone, Copy, Display, EnumString, sqlx::Type)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "job_status", rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    InProgress,
    Complete,
    Failed,
}

impl Job {
    pub fn can_transition_to(&self, new_status: JobStatus) -> bool {
        match (self.status, new_status) {
            (JobStatus::Pending, JobStatus::InProgress) => true,
            (JobStatus::InProgress, JobStatus::Complete) => true,
            (JobStatus::InProgress, JobStatus::Failed) => true,
            (JobStatus::Failed, JobStatus::Pending) => self.retries < self.max_retries,
            _ => false,
        }
    }
}
```

### API Integration

```rust
// Job status endpoint
async fn get_job_status(
    Path(job_id): Path<Uuid>,
    State(pool): State<PgPool>,
) -> Result<Json<JobResponse>, StatusCode> {
    let job = Job::fetch(job_id, &pool).await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    Ok(Json(JobResponse {
        id: job.id,
        status: job.status.to_string(),
        result: job.result,
        error: job.error,
        created_at: job.created_at,
        completed_at: job.completed_at,
    }))
}
```

## Installation Summary

```toml
[dependencies]
# Existing — no changes needed
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "macros", "chrono", "uuid"], optional = true }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }

# New addition
strum = { version = "0.28", features = ["derive"] }

# May need to add uuid feature to sqlx
uuid = { version = "1.0", features = ["serde", "v4"] }
```

## What NOT to Add

| Technology | Why Excluded |
|------------|--------------|
| **pgmq extension** | Adds deployment complexity (extension install) for minimal benefit |
| **apalis/apalis-postgres** | Too heavyweight — worker pools, scheduled jobs, middleware overkill |
| **fang** | Typetag complexity, migration conflicts, RC stability |
| **Redis** | Not needed — PostgreSQL is already our data store |
| **RabbitMQ/AMQP** | Massive overkill for single-node job processing |
| **Separate job server binary** | Unnecessary complexity — run in-process with tokio tasks |

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack choice | HIGH | Custom table + sqlx is idiomatic for this use case |
| Version stability | HIGH | All recommended crates are stable (v1.0+) |
| Integration risk | LOW | Builds on existing sqlx/tokio patterns |
| Performance | HIGH | `FOR UPDATE SKIP LOCKED` is production-tested |
| Maintainability | HIGH | Simple SQL schema, no hidden magic |

## Sources

- [sqlx PostgreSQL docs](https://docs.rs/sqlx/latest/sqlx/postgres/index.html) — Connection pooling, query macros
- [pgmq GitHub](https://github.com/pgmq/pgmq) — Reviewed but rejected for complexity
- [apalis docs](https://docs.rs/apalis/latest/apalis/) — Reviewed but overkill
- [fang docs](https://docs.rs/fang/latest/fang/) — Reviewed, typetag complexity
- [PostgreSQL SKIP LOCKED docs](https://www.postgresql.org/docs/current/sql-select.html#SQL-FOR-UPDATE-SHARE) — Concurrency control
- [strum docs](https://docs.rs/strum/latest/strum/) — Enum utilities
- [tokio task docs](https://docs.rs/tokio/latest/tokio/task/index.html) — Background task execution

---

**Recommendation:** Proceed with custom job table approach. It's the minimal, idiomatic Rust solution that leverages existing infrastructure.
