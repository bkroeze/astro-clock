# Architecture Research: Job System Integration

**Domain:** Astro Clock — Astrological calculation tool with time-series database
**Researched:** 2026-03-01
**Confidence:** HIGH (based on existing codebase analysis)

## Executive Summary

The Astro Clock v1.1 job system requires integrating background job orchestration into an existing layered Rust architecture with TimescaleDB, ChunkManager LRU cache, and HTTP server mode. The job system must:

1. **Leverage existing infrastructure**: ChunkManager for data loading, existing query functions for named queries
2. **Add minimal new components**: Job executor, job tables, query template registry
3. **Maintain clean boundaries**: Job system is a new horizontal layer that orchestrates existing vertical features
4. **Support both sync/async**: CLI uses sync mode, HTTP API supports both

## Current Architecture Overview

### Existing System Structure

```
┌─────────────────────────────────────────────────────────────────┐
│                          CLI LAYER                               │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │  Chart   │  │ Aspects  │  │  Serve   │  │ (new)    │        │
│  │ Command  │  │ Command  │  │ Command  │  │  Load/   │        │
│  └──────────┘  └──────────┘  └──────────┘  │  Query   │        │
│                                            └──────────┘        │
├─────────────────────────────────────────────────────────────────┤
│                        HTTP SERVER LAYER                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐     │
│  │ /health     │  │  /chart     │  │ (new) /api/v1/jobs  │     │
│  │  (existing) │  │  (existing) │  │ (new) /api/v1/query │     │
│  └─────────────┘  └─────────────┘  └─────────────────────┘     │
├─────────────────────────────────────────────────────────────────┤
│                        DOMAIN LAYER                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌─────────────────┐ │
│  │  Chart   │  │ Aspects  │  │ Ephemeris│  │   (new) Job     │ │
│  │  Logic   │  │ Analysis │  │  (swiss) │  │   Executor      │ │
│  └──────────┘  └──────────┘  └──────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                      DATABASE LAYER                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │ ChunkManager │  │   Queries    │  │   (new) Job Store    │  │
│  │  (LRU cache) │  │(wedding/voc) │  │  (jobs, loaded_days) │  │
│  └──────────────┘  └──────────────┘  └──────────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│                     TIMESCALEDB SCHEMA                           │
│  ┌─────────┐ ┌─────────┐ ┌─────────────┐ ┌──────────────────┐  │
│  │ planet_ │ │ aspects │ │ lunar_cond  │ │ (new) jobs,      │  │
│  │positions│ │         │ │             │ │ (new) loaded_days│  │
│  └─────────┘ └─────────┘ └─────────────┘ └──────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### Existing Key Components

| Component | Responsibility | Integration Point for Jobs |
|-----------|----------------|---------------------------|
| **ChunkManager** | LRU cache for day-level astrological data; 3-tier lookup (cache→DB→generate) | Job executor uses this to load data on-demand; jobs populate cache after generation |
| **ChunkGenerator** | Generates 1-day chunks from Swiss Ephemeris; saves to DB in background | Job executor calls this for incremental loading; batch generation for date ranges |
| **Query Functions** (`wedding`, `voc`, `retrograde`, `aspects`) | Specialized electoral astrology queries against TimescaleDB | Named query templates wrap these functions; job executor runs them in background |
| **DatabasePool** | sqlx connection pool (max 5 connections) | Job executor uses same pool; job tables live in same database |
| **HTTPServer (axum)** | Serves chart images and health checks | New endpoints for job submission and status polling |
| **CLI (clap)** | Chart, aspects, and serve commands | New load/query/job subcommands |

## Job System Architecture

### New Component: Job Executor

The Job Executor is the central orchestration component. It manages job lifecycle, coordinates with ChunkManager for data loading, and executes named queries.

```
┌─────────────────────────────────────────────────────────────────┐
│                      JOB EXECUTOR                                │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │   Job Queue  │  │   Job State  │  │  Query Template      │  │
│  │   (in-DB)    │  │   Machine    │  │  Registry            │  │
│  │              │  │              │  │                      │  │
│  │ - pending    │  │ pending →    │  │ - "wedding" →        │  │
│  │ - in-process │  │ in-process → │  │   find_wedding_dates │  │
│  │ - complete   │  │ complete/    │  │ - "project" →        │  │
│  │ - failed     │  │ failed       │  │   find_project_dates │  │
│  └──────────────┘  └──────────────┘  │ - "travel" →         │  │
│                                      │   find_travel_dates  │  │
│                                      └──────────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│                     EXECUTION STRATEGIES                         │
│  ┌─────────────────┐  ┌─────────────────────────────────────┐  │
│  │  SYNC MODE      │  │  ASYNC MODE                         │  │
│  │                 │  │                                     │  │
│  │ 1. Create job   │  │ 1. Create job → return job_id       │  │
│  │ 2. Execute      │  │ 2. Spawn background task            │  │
│  │ 3. Return       │  │ 3. Return immediately               │  │
│  │    result       │  │ 4. Client polls GET /jobs/{id}      │  │
│  └─────────────────┘  └─────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### Job State Machine

```
                    ┌─────────┐
                    │ PENDING │
                    └────┬────┘
                         │ claim_job()
                         ↓
                  ┌──────────────┐
                  │  IN-PROCESS  │
                  └──────┬───────┘
                         │
           ┌─────────────┼─────────────┐
           │             │             │
           ↓             │             ↓
     ┌─────────┐         │       ┌─────────┐
     │COMPLETE │         │       │ FAILED  │
     └────┬────┘         │       └────┬────┘
          │              │            │
          │ on success   │            │ on error
          ↓              │            ↓
   Store result    ┌─────┴─────┐ Store error
   in job.result   │  Timeout  │ in job.error
                   │ (retry?)  │
                   └───────────┘
```

**State Transitions:**
- `PENDING` → `IN-PROCESS`: Worker claims job via atomic UPDATE
- `IN-PROCESS` → `COMPLETE`: Job succeeds, result stored as JSON
- `IN-PROCESS` → `FAILED`: Job fails, error details stored
- **No retry in v1.1**: Manual re-submission required

### Database Schema Additions

```sql
-- Job tracking table
CREATE TABLE jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_type VARCHAR(50) NOT NULL, -- 'load', 'wedding', 'project', 'travel'
    status VARCHAR(20) NOT NULL CHECK (status IN ('pending', 'in-process', 'complete', 'failed')),
    payload JSONB NOT NULL, -- Query parameters, date ranges, etc.
    result JSONB, -- Query results for complete jobs
    error JSONB, -- Error details for failed jobs
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

-- Index for efficient job claiming and status queries
CREATE INDEX idx_jobs_status_created ON jobs(status, created_at);
CREATE INDEX idx_jobs_type_status ON jobs(job_type, status);

-- Loaded days tracking for incremental loading
CREATE TABLE loaded_days (
    date DATE PRIMARY KEY,
    loaded_at TIMESTAMPTZ DEFAULT NOW(),
    record_count INTEGER, -- Total records loaded for this day
    source VARCHAR(20) -- 'swiss_eph', 'import', etc.
);

-- Index for date range queries
CREATE INDEX idx_loaded_days_date ON loaded_days(date);
```

### Named Query Template System

Named queries are **templates** that map to existing query functions with predefined criteria.

```rust
/// Query template definition
pub struct QueryTemplate {
    pub name: String,
    pub description: String,
    pub default_criteria: QueryCriteria,
    pub execute_fn: Arc<dyn Fn(&DatabasePool, &QueryCriteria) -> Pin<Box<dyn Future<Output = Result<QueryResult, QueryError>> + Send>> + Send + Sync>,
}

/// Registry of available named queries
pub struct QueryTemplateRegistry {
    templates: HashMap<String, QueryTemplate>,
}

impl QueryTemplateRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            templates: HashMap::new(),
        };
        
        // Register built-in templates
        registry.register(QueryTemplate {
            name: "wedding".to_string(),
            description: "Find auspicious wedding dates".to_string(),
            default_criteria: QueryCriteria::default_wedding(),
            execute_fn: Arc::new(|pool, criteria| {
                Box::pin(async move {
                    let wedding_criteria = WeddingCriteria::from(criteria);
                    find_wedding_dates(pool, &wedding_criteria).await
                })
            }),
        });
        
        // Similar for "project" and "travel"
        
        registry
    }
}
```

## Integration Points with Existing Code

### 1. ChunkManager Integration

**Pattern: Orchestrated Loading**

The Job Executor uses ChunkManager for intelligent data loading:

```rust
impl JobExecutor {
    /// Ensure data is loaded for a date range before querying
    async fn ensure_data_loaded(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<(), JobError> {
        // Check loaded_days table for missing dates
        let missing_days = self.find_missing_days(start_date, end_date).await?;
        
        for date in missing_days {
            // Use ChunkManager to load/generate each day
            // This handles cache, DB fallback, and Swiss Ephemeris generation
            let _chunk = self.chunk_manager.get_chunk(date).await?;
            
            // Track that we've loaded this day
            self.mark_day_loaded(date).await?;
        }
        
        Ok(())
    }
}
```

**Key Integration:**
- Job executor calls `chunk_manager.get_chunk(date)` for each missing day
- ChunkManager handles the 3-tier lookup (cache → DB → generate)
- Generated chunks are automatically saved to DB in background (existing behavior)
- `loaded_days` table tracks which days have been persisted

### 2. Query Function Integration

**Pattern: Template Wrapper**

Named queries wrap existing query functions without modifying them:

```rust
// Existing function (unchanged)
pub async fn find_wedding_dates(
    pool: &DatabasePool,
    criteria: &WeddingCriteria,
) -> Result<QueryResult<WeddingCandidate>, QueryError> {
    // ... existing implementation
}

// New template wrapper (in job executor)
async fn execute_wedding_query(
    pool: &DatabasePool,
    params: &QueryParams,
) -> Result<serde_json::Value, JobError> {
    // Convert generic params to specific criteria
    let criteria = WeddingCriteria::try_from(params)?;
    
    // Call existing function
    let result = find_wedding_dates(pool, &criteria).await?;
    
    // Serialize to JSON for job storage
    Ok(serde_json::to_value(result)?)
}
```

### 3. HTTP Server Integration

**New Endpoints:**

```rust
// In src/server/mod.rs or src/server/jobs.rs

async fn create_job_handler(
    State(state): State<AppState>,
    Json(request): Json<CreateJobRequest>,
) -> Result<Json<JobResponse>, StatusCode> {
    let job_id = state.job_executor.submit_job(request).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(JobResponse {
        job_id,
        status: "pending".to_string(),
    }))
}

async fn get_job_handler(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<JobStatus>, StatusCode> {
    let job = state.job_executor.get_job(job_id).await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    Ok(Json(job.into()))
}

async fn list_jobs_handler(
    State(state): State<AppState>,
    Query(params): Query<ListJobsParams>,
) -> Result<Json<Vec<JobSummary>>, StatusCode> {
    let jobs = state.job_executor.list_jobs(params).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(jobs))
}
```

**Route Registration:**
```rust
let app = axum::Router::new()
    .route("/health", get(health_handler))
    .route("/chart", get(chart_handler))
    // New job endpoints
    .route("/api/v1/jobs", post(create_job_handler))
    .route("/api/v1/jobs", get(list_jobs_handler))
    .route("/api/v1/jobs/:id", get(get_job_handler))
    // New query endpoints
    .route("/api/v1/query/wedding", post(query_wedding_handler))
    .route("/api/v1/query/project", post(query_project_handler))
    .route("/api/v1/query/travel", post(query_travel_handler))
    .route("/api/v1/load", post(load_data_handler))
    .with_state(app_state);
```

### 4. CLI Integration

**New Commands:**

```rust
#[derive(Subcommand, Debug)]
pub enum Commands {
    // ... existing commands (Chart, Aspects, Serve)
    
    /// Load astrological data into database
    Load {
        #[arg(long)]
        start: String, // YYYY-MM-DD
        
        #[arg(long)]
        days: u32,
        
        /// Run synchronously (wait for completion)
        #[arg(long)]
        sync: bool,
    },
    
    /// Run named queries
    Query {
        #[command(subcommand)]
        query_type: QueryType,
    },
    
    /// Job management
    Job {
        #[command(subcommand)]
        job_command: JobCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum QueryType {
    Wedding {
        #[arg(long)]
        start: String,
        #[arg(long)]
        days: u32,
        #[arg(long)]
        sync: bool,
    },
    Project { /* ... */ },
    Travel { /* ... */ },
}

#[derive(Subcommand, Debug)]
pub enum JobCommand {
    Status { job_id: String },
    List { 
        #[arg(long, default_value = "10")]
        limit: usize 
    },
}
```

## Data Flow: Job Creation to Result

### Scenario 1: Synchronous Query (CLI)

```
User: astro-clock query wedding --start 2024-06-01 --days 30 --sync

┌─────────┐     ┌─────────────┐     ┌─────────────────┐     ┌─────────────┐
│   CLI   │────▶│ JobExecutor │────▶│ ensure_data_    │────▶│ ChunkManager│
│         │     │             │     │ loaded()        │     │             │
└─────────┘     └─────────────┘     └─────────────────┘     └──────┬──────┘
                                                                   │
                              ┌────────────────────────────────────┘
                              │ For each missing day:
                              │ 1. Check loaded_days table
                              │ 2. Call chunk_manager.get_chunk()
                              │ 3. Mark day loaded
                              ▼
                       ┌──────────────┐
                       │  Swiss Ephem │ (if not in DB)
                       │  Generation  │
                       └──────────────┘
                              │
                              ▼
┌─────────┐     ┌─────────────┐     ┌─────────────────┐     ┌─────────────┐
│   CLI   │◀────│ JobExecutor │◀────│ find_wedding_   │◀────│  TimescaleDB│
│ (print) │     │ (blocking)  │     │ dates()         │     │ (query)     │
└─────────┘     └─────────────┘     └─────────────────┘     └─────────────┘
```

### Scenario 2: Asynchronous Query (HTTP API)

```
Client: POST /api/v1/query/wedding {start_date, days}

┌─────────┐     ┌─────────────┐     ┌─────────────────┐     ┌─────────────┐
│  HTTP   │────▶│ JobExecutor │────▶│ INSERT INTO     │────▶│  jobs table │
│  API    │     │ .submit_job() │     │ jobs (...)      │     │ (pending)   │
└─────────┘     └─────────────┘     └─────────────────┘     └─────────────┘
      │
      │ Return immediately: {job_id, status: "pending"}
      ▼

[Background Task Spawned]
      │
      ▼
┌─────────────┐     ┌─────────────────┐     ┌─────────────┐
│ JobExecutor │────▶│ ensure_data_    │────▶│ ChunkManager│
│ (worker)    │     │ loaded()        │     │             │
└─────────────┘     └─────────────────┘     └─────────────┘
      │
      ▼
┌─────────────┐     ┌─────────────────┐
│  UPDATE     │────▶│ jobs table      │
│  jobs SET   │     │ status=complete │
│  status=    │     │ result=JSON     │
│  complete   │     │                 │
└─────────────┘     └─────────────────┘

[Client Polling]
      │
      ▼
GET /api/v1/jobs/{job_id}
      │
      ▼
Return: {status: "complete", result: {...}}
```

## Suggested Build Order (Dependencies)

### Phase 5: Job System Infrastructure

**Order based on dependencies:**

1. **Database Schema (JOB-01, JOB-02)**
   - Create `jobs` table migration
   - Create `loaded_days` table migration
   - *No dependencies; foundational*

2. **Job Types and Models (JOB-03, JOB-04, JOB-05)**
   - Define `Job` struct with states
   - Define `JobType` enum (Load, Wedding, Project, Travel)
   - Define `JobStatus` enum
   - *Depends on: schema*

3. **Job Repository (Data Access)**
   - `JobRepository` struct with methods:
     - `create_job()`
     - `get_job()`
     - `claim_job()` (atomic status update)
     - `complete_job()`
     - `fail_job()`
   - *Depends on: models, DatabasePool*

4. **Loaded Days Repository**
   - `LoadedDaysRepository` with methods:
     - `find_missing_days()`
     - `mark_day_loaded()`
   - *Depends on: schema, DatabasePool*

5. **Job Executor Core (JOB-06)**
   - `JobExecutor` struct
   - State machine implementation
   - `submit_job()` method
   - `execute_sync()` method
   - *Depends on: JobRepository, LoadedDaysRepository*

### Phase 6: Data Loading Jobs

1. **Load Job Implementation (LOAD-06, LOAD-07, LOAD-08, LOAD-09, LOAD-10)**
   - `LoadJobHandler` that:
     - Iterates through date range
     - Calls ChunkManager for each day
     - Updates loaded_days tracking
   - CLI `load` command
   - API `POST /api/v1/load` endpoint
   - *Depends on: JobExecutor, ChunkManager*

### Phase 7: Named Query Jobs

1. **Query Template Registry (QUERY-06, QUERY-07, QUERY-08)**
   - `QueryTemplateRegistry` struct
   - Registration of "wedding", "project", "travel" templates
   - Mapping to existing query functions
   - *Depends on: existing query functions, JobExecutor*

2. **Query Job Implementation (QUERY-09, QUERY-10, QUERY-11)**
   - `QueryJobHandler` that:
     - Parses parameters from job payload
     - Ensures data is loaded (via ChunkManager)
     - Executes appropriate query template
     - Stores results
   - *Depends on: QueryTemplateRegistry, Load Job*

### Phase 8: CLI and API Integration

1. **CLI Commands (CLI-01 through CLI-05)**
   - `query wedding` command
   - `query project` command
   - `query travel` command
   - `job status` command
   - `job list` command
   - *Depends on: all previous phases*

2. **HTTP API Endpoints (API-01 through API-06)**
   - `POST /api/v1/query/*` endpoints
   - `GET /api/v1/jobs` endpoints
   - *Depends on: all previous phases*

3. **Result Handling (RESULT-01 through RESULT-05)**
   - Response formatting
   - Error serialization
   - *Depends on: API endpoints*

## Architecture Patterns

### Pattern 1: Repository Pattern for Data Access

**What:** Isolate database operations in repository structs
**Why:** Clean separation, easier testing, transaction management

```rust
pub struct JobRepository {
    pool: DatabasePool,
}

impl JobRepository {
    pub async fn create_job(&self, job_type: JobType, payload: JsonValue) -> Result<Uuid, sqlx::Error> {
        sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO jobs (job_type, status, payload) VALUES ($1, 'pending', $2) RETURNING id"
        )
        .bind(job_type.as_str())
        .bind(payload)
        .fetch_one(self.pool.pool())
        .await
    }
    
    pub async fn claim_job(&self) -> Result<Option<Job>, sqlx::Error> {
        // Atomic: UPDATE ... WHERE status = 'pending' RETURNING *
    }
}
```

### Pattern 2: Handler Pattern for Job Types

**What:** Each job type has a handler that implements a trait
**Why:** Extensibility for future job types without modifying core executor

```rust
#[async_trait]
pub trait JobHandler {
    async fn execute(&self, payload: &JsonValue) -> Result<JsonValue, JobError>;
}

pub struct LoadJobHandler {
    chunk_manager: ChunkManager,
}

#[async_trait]
impl JobHandler for LoadJobHandler {
    async fn execute(&self, payload: &JsonValue) -> Result<JsonValue, JobError> {
        // Load data implementation
    }
}
```

### Pattern 3: Registry Pattern for Query Templates

**What:** Central registry maps query names to execution functions
**Why:** Decouples query definition from execution; enables dynamic registration later

```rust
pub struct QueryTemplateRegistry {
    templates: HashMap<String, Box<dyn QueryTemplate>>,
}

impl QueryTemplateRegistry {
    pub fn register<T: QueryTemplate + 'static>(&mut self, template: T) {
        self.templates.insert(template.name(), Box::new(template));
    }
    
    pub fn get(&self, name: &str) -> Option<&dyn QueryTemplate> {
        self.templates.get(name).map(|b| b.as_ref())
    }
}
```

### Pattern 4: Type-State Pattern for Job Lifecycle

**What:** Use Rust's type system to enforce valid state transitions
**Why:** Compile-time guarantees against invalid state changes

```rust
// Instead of stringly-typed status
pub struct PendingJob { /* ... */ }
pub struct InProcessJob { /* ... */ }
pub struct CompleteJob { result: JsonValue }
pub struct FailedJob { error: JobError }

// State transitions return new types
impl PendingJob {
    pub fn start(self) -> InProcessJob { /* ... */ }
}

impl InProcessJob {
    pub fn complete(self, result: JsonValue) -> CompleteJob { /* ... */ }
    pub fn fail(self, error: JobError) -> FailedJob { /* ... */ }
}
```

*Note: For simplicity in v1.1, we may use status strings in the database with runtime validation rather than full type-state pattern.*

## Anti-Patterns to Avoid

### Anti-Pattern 1: Direct ChunkManager Access from CLI

**What people might do:** Call ChunkManager directly from CLI commands
**Why it's wrong:** Bypasses job tracking, can't support async mode, inconsistent API
**Do this instead:** Always route through JobExecutor; CLI is just a thin wrapper

### Anti-Pattern 2: Modifying Existing Query Functions

**What people might do:** Add job-awareness to `find_wedding_dates()`
**Why it's wrong:** Pollutes pure query logic with job concerns; harder to test
**Do this instead:** Keep query functions pure; wrap them in job handlers

### Anti-Pattern 3: Separate Job Database

**What people might do:** Use a separate SQLite/Redis for jobs
**Why it's wrong:** Adds operational complexity; transactions across databases are hard
**Do this instead:** Keep jobs in existing TimescaleDB; use transactions for consistency

### Anti-Pattern 4: Polling Database in Tight Loop

**What people might do:** `while job.status != "complete" { check_db() }`
**Why it's wrong:** Wastes resources; can overwhelm database
**Do this instead:** Exponential backoff polling in clients; consider LISTEN/NOTIFY for future

## New vs Modified Components

| Component | Status | Rationale |
|-----------|--------|-----------|
| **New:** `src/jobs/` module | Create | New horizontal layer for job orchestration |
| **New:** `src/jobs/repository.rs` | Create | Job and loaded_days data access |
| **New:** `src/jobs/executor.rs` | Create | Core job execution logic |
| **New:** `src/jobs/handlers/` | Create | Job-type-specific handlers (load, query) |
| **New:** `src/jobs/registry.rs` | Create | Query template registry |
| **New:** `src/jobs/types.rs` | Create | Job types, states, errors |
| **Modified:** `src/server/mod.rs` | Extend | Add job and query endpoints |
| **Modified:** `src/cli/app.rs` | Extend | Add load/query/job commands |
| **New:** Migrations `008_create_jobs.sql` | Create | Jobs table schema |
| **New:** Migrations `009_create_loaded_days.sql` | Create | Loaded days tracking |
| **Unchanged:** `src/database/chunk_manager.rs` | Keep | Use as-is; no modifications needed |
| **Unchanged:** `src/queries/wedding.rs` | Keep | Use as-is; wrap, don't modify |
| **Unchanged:** `src/queries/voc.rs` | Keep | Use as-is |
| **Unchanged:** `src/queries/retrograde.rs` | Keep | Use as-is |
| **Unchanged:** `src/database/chunk_generator.rs` | Keep | Use as-is |

## Scalability Considerations

| Concern | v1.1 (Current) | Future (>10K jobs/day) |
|---------|---------------|------------------------|
| **Job Queue** | In-DB with polling | Consider Redis/RabbitMQ |
| **Workers** | Single-threaded Tokio | Worker pool with job claiming |
| **Result Storage** | JSONB in PostgreSQL | Separate result store; pagination |
| **Job Retention** | Keep all jobs | TTL/Archive old jobs |
| **Progress Tracking** | Status only | Percent complete, step details |

## Sources

- Astro Clock v1.0 codebase analysis (2026-03-01)
- Existing migrations in `migrations/` directory
- `.planning/REQUIREMENTS.md` v1.1 specification
- `.planning/PROJECT.md` project context

---
*Architecture research for: Astro Clock v1.1 Job System*
*Researched: 2026-03-01*
