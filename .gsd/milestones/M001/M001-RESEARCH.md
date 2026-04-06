# Research Summary: Astro Clock v1.1 Job System

**Project:** Astro Clock v1.1 — Job System & Named Queries  
**Researched:** 2026-03-01  
**Research Status:** ✅ Complete — Ready for Requirements Definition

---

## Executive Summary

The Astro Clock v1.1 job system represents a **minimal, pragmatic extension** to the existing v1.0 architecture. Rather than adopting heavyweight external job queue libraries (pgmq, apalis, fang), the recommended approach leverages the existing **PostgreSQL + TimescaleDB + sqlx + tokio** stack to build a lightweight, in-database job queue. This custom approach requires only one new crate (`strum` for enum utilities) and approximately 200 lines of Rust + SQL code.

The job system adds **horizontal orchestration capabilities** to existing vertical features: it coordinates data loading via ChunkManager, wraps existing query functions (wedding, VOC, retrograde) in job lifecycle management, and supports both synchronous (CLI) and asynchronous (HTTP API) execution modes. Key architectural decisions include: using `FOR UPDATE SKIP LOCKED` for race-free job claiming, maintaining dual connection pools to protect query performance, and implementing transactionally-staged job drains to prevent race conditions.

Critical risks center on **connection pool exhaustion** (solved by dual pools), **cache coherency** (solved by explicit invalidation), and **async/sync mode boundaries** (solved by spawn_blocking + channel bridge pattern). The research confidence is HIGH across all areas, with well-documented patterns from the Rust async ecosystem and clear integration points with the existing codebase.

---

## Key Findings

### Stack Decisions

**Add These:**

| Technology | Version | Purpose | Rationale |
|------------|---------|---------|-----------|
| **strum** | 0.28 | Enum utilities | Derive `Display`, `AsRefStr`, `EnumIter` for job states with zero runtime overhead |

**Leverage Existing:**

| Technology | Purpose | Pattern |
|------------|---------|---------|
| **sqlx** | Job queue storage | Custom `jobs` table with PostgreSQL ENUM for status |
| **tokio::task** | Background execution | `tokio::spawn` for async, `spawn_blocking` for Swiss Ephemeris FFI |
| **serde_json** | Payload/result serialization | Store as JSONB for flexibility and queryability |
| **uuid** | Job identifiers | UUID v4 for distributed safety |

**Explicitly Avoid:**

| Technology | Why Excluded |
|------------|--------------|
| **pgmq extension** | Adds deployment complexity; custom table is simpler |
| **apalis/apalis-postgres** | Overkill — worker pools, scheduled jobs, middleware not needed |
| **fang** | Typetag complexity, migration conflicts, RC stability issues |
| **Redis/RabbitMQ** | Massive overkill for single-node job processing |
| **Separate job server binary** | Unnecessary — run in-process with tokio tasks |

### Key Features

**Table Stakes (Must Have):**

| Feature | Description | Complexity |
|---------|-------------|------------|
| Job lifecycle states | `pending → in_process → (complete \| failed)` | LOW |
| Unique job IDs | UUID v4 for distributed safety | LOW |
| Payload/result storage | JSONB columns for flexible job data | LOW |
| Sync execution mode | Blocking execution for CLI use cases | LOW |
| Async execution mode | Background execution with job-id polling | MEDIUM |
| Job status endpoint | GET /api/v1/jobs/{id} for polling | LOW |
| Job list endpoint | Paginated list with filters | LOW |
| Named query structure | Template registry for wedding/project/travel | MEDIUM |
| Date range tracking | `loaded_days` table for incremental loading | MEDIUM |
| Error reporting | Structured error codes + human messages | LOW |
| CLI job commands | `job status`, `job list` with table output | LOW |

**Differentiators (Competitive Advantage):**

| Feature | Value | Complexity |
|---------|-------|------------|
| **Intelligent incremental loading** | Only compute missing days — saves 90%+ time | MEDIUM |
| **Pre-flight data check** | Named queries auto-load missing data before executing | MEDIUM |
| **Parallel day loading** | Load multiple days concurrently (~4-8× speedup) | MEDIUM |
| **Atomic multi-day transactions** | All days succeed or fail together | MEDIUM |
| **CLI timeout warning** | Warn if sync job might exceed 10s | LOW |

**Anti-Features (NOT Building):**

| Feature | Why Deferred |
|---------|--------------|
| Real-time progress streaming | WebSocket/SSE complexity not justified |
| Job cancellation | Cooperative cancellation complex with FFI; "restart is safe" |
| Automatic job retry | Masking failures leads to confusion; manual retry preferred |
| Query result caching | Database IS the cache; another layer adds invalidation complexity |
| Job prioritization | FIFO is sufficient for user-initiated short jobs |
| Job scheduling/cron | Document external cron + CLI; keep app stateless |
| Distributed job workers | Single-node with tokio tasks is sufficient |

### Architecture Highlights

**New Components (to create):**

```
src/jobs/
├── mod.rs           # Public exports
├── types.rs         # Job, JobType, JobStatus, JobError
├── repository.rs    # JobRepository, LoadedDaysRepository
├── executor.rs      # JobExecutor — core orchestration
├── handlers/        # Job-type-specific handlers
│   ├── mod.rs
│   ├── load.rs      # LoadJobHandler
│   └── query.rs     # QueryJobHandler
└── registry.rs      # QueryTemplateRegistry
```

**Modified Components:**

| Component | Change |
|-----------|--------|
| `src/server/mod.rs` | Add `/api/v1/jobs/*` and `/api/v1/query/*` endpoints |
| `src/cli/app.rs` | Add `load`, `query`, `job` subcommands |
| Database | New migrations: `008_create_jobs.sql`, `009_create_loaded_days.sql` |

**Integration Points:**

1. **ChunkManager Integration:** Job executor calls `chunk_manager.get_chunk(date)` for missing days; cache invalidation after loading
2. **Query Function Integration:** Named queries wrap existing functions without modification
3. **Dual Connection Pools:** Separate pools for queries (3 conn, fast) and jobs (5 conn, can tolerate slow)
4. **State Machine:** Enforce valid transitions at database level with CHECK constraint

### Critical Pitfalls & Prevention

| Pitfall | Risk Level | Prevention Strategy | Phase |
|---------|------------|---------------------|-------|
| **Transaction-Job Race Condition** | CRITICAL | Use staged_jobs table within transaction; separate enqueuer process | Phase 5 |
| **Connection Pool Exhaustion** | CRITICAL | Dual pools: query_pool (3 conn) + job_pool (5 conn) | Phase 5 |
| **Async/Sync Mode Deadlocks** | CRITICAL | Use `spawn_blocking` + oneshot channel bridge; no async inside sync | Phase 5 |
| **Cache Coherency Violations** | HIGH | Invalidate cache entries after job completion; `invalidate_date_range()` | Phase 6 |
| **Job State Machine Inconsistency** | HIGH | Heartbeat pattern; timeout detection; idempotent jobs | Phase 5 |
| **Hypertable Lock Contention** | MEDIUM | Advisory locks (`pg_advisory_lock`) per day; batch inserts | Phase 6 |
| **Memory Pressure from Large Results** | MEDIUM | Result pagination; external storage for results >100KB | Phase 7 |

**Key Prevention Code Patterns:**

```rust
// Job claiming with SKIP LOCKED (race-free)
UPDATE jobs 
SET status = 'in_process', started_at = NOW()
WHERE id = (
    SELECT id FROM jobs 
    WHERE status = 'pending' 
    ORDER BY created_at 
    FOR UPDATE SKIP LOCKED
    LIMIT 1
)
RETURNING *

// Spawn blocking bridge for CPU-intensive work
let (tx, rx) = tokio::sync::oneshot::channel();
tokio::task::spawn_blocking(move || {
    let result = cpu_intensive_work(job);  // NO async calls here
    let _ = tx.send(result);
});
let result = rx.await?;
```

---

## Implications for Roadmap

### Suggested Phase Structure

Based on dependencies and build order analysis:

**Phase 5: Job System Infrastructure** (Foundation)
- **Rationale:** All subsequent phases depend on this foundation
- **Delivers:** Job tables, state machine, executor core, repository pattern
- **Features:** JOB-01 through JOB-06 (job infrastructure, sync/async modes)
- **Pitfalls to avoid:** Transaction-job race condition, connection pool exhaustion, async/sync deadlocks
- **Research needed:** NONE — patterns are well-documented

**Phase 6: Data Loading Jobs**
- **Rationale:** Named queries depend on data being available; load capability enables query pre-flight
- **Delivers:** Load job handler, CLI `load` command, API `/api/v1/load` endpoint
- **Features:** LOAD-06 through LOAD-10 (day-level incremental loading)
- **Pitfalls to avoid:** Cache coherency violations, hypertable lock contention
- **Research needed:** NONE — builds on existing ChunkManager patterns

**Phase 7: Named Query System**
- **Rationale:** Core v1.1 value proposition; depends on data loading from Phase 6
- **Delivers:** Query template registry, wedding/project/travel named queries, pre-flight data check
- **Features:** QUERY-06 through QUERY-11 (named queries, intelligent data loading)
- **Pitfalls to avoid:** Memory pressure from large results
- **Research needed:** NONE — wraps existing query functions

**Phase 8: CLI & API Integration**
- **Rationale:** Final integration layer; all previous phases must be complete
- **Delivers:** Complete CLI commands, HTTP endpoints, result handling
- **Features:** CLI-01 through CLI-05, API-01 through API-06, RESULT-01 through RESULT-05
- **Pitfalls to avoid:** Long-running sync jobs causing HTTP timeouts
- **Research needed:** NONE — follows existing server/CLI patterns

### Research Flags

| Phase | Research Needed? | Rationale |
|-------|------------------|-----------|
| Phase 5 | NO | Patterns are standard (sqlx + tokio) |
| Phase 6 | NO | Builds on existing ChunkManager |
| Phase 7 | NO | Wraps existing query functions |
| Phase 8 | NO | Follows existing axum/clap patterns |

**Conclusion:** All phases use established patterns. No additional research required before implementation.

---

## Confidence Assessment

| Area | Confidence | Assessment |
|------|------------|------------|
| **Technology Stack** | HIGH | Custom table + sqlx is idiomatic for this use case; minimal additions reduce risk |
| **Feature Completeness** | HIGH | Well-defined scope with clear MVP; anti-features explicitly excluded |
| **Architecture** | HIGH | Clean integration points with existing codebase; patterns are established |
| **Pitfall Mitigation** | HIGH | All critical pitfalls have documented prevention strategies |
| **Overall** | **HIGH** | Ready to proceed with requirements definition |

### Sources Quality

| Source Type | Quality | Notes |
|-------------|---------|-------|
| Official documentation (sqlx, tokio, PostgreSQL) | HIGH | Authoritative references for core patterns |
| Established patterns (Brandur's job drain) | HIGH | Production-tested solutions |
| Existing codebase analysis | HIGH | Direct knowledge of integration points |
| Requirements document | HIGH | Clear v1.1 specification |

### Gaps to Address

No significant gaps identified. The following are conscious design decisions, not gaps:

1. **No WebSocket/SSE for progress:** Polling is sufficient for v1.1
2. **No distributed workers:** Single-node is sufficient
3. **No automatic retry:** Manual retry is preferred for transparency

---

## Open Questions

None. Research is complete and all architectural decisions have been validated against:
- Existing codebase structure
- v1.1 requirements specification
- Rust async ecosystem best practices
- PostgreSQL/TimescaleDB operational patterns

---

## Sources

### Stack Research
- [sqlx PostgreSQL docs](https://docs.rs/sqlx/latest/sqlx/postgres/index.html)
- [pgmq GitHub](https://github.com/pgmq/pgmq)
- [apalis docs](https://docs.rs/apalis/latest/apalis/)
- [fang docs](https://docs.rs/fang/latest/fang/)
- [PostgreSQL SKIP LOCKED docs](https://www.postgresql.org/docs/current/sql-select.html#SQL-FOR-UPDATE-SHARE)
- [strum docs](https://docs.rs/strum/latest/strum/)
- [tokio task docs](https://docs.rs/tokio/latest/tokio/task/index.html)

### Feature Research
- Tokio Task Documentation: https://docs.rs/tokio/latest/tokio/task/
- SQLx Documentation: https://docs.rs/sqlx/latest/sqlx/
- Existing Astro Clock Codebase (v1.0 patterns)
- REQUIREMENTS.md v1.1 specifications
- PROJECT.md architecture decisions

### Architecture Research
- Astro Clock v1.0 codebase analysis
- Existing migrations in `migrations/` directory
- `.planning/REQUIREMENTS.md` v1.1 specification
- `.planning/PROJECT.md` project context

### Pitfalls Research
- [Transactionally Staged Job Drains in Postgres](https://brandur.org/job-drain) — Brandur Leach
- [SQLx Pool Documentation](https://docs.rs/sqlx/latest/sqlx/struct.Pool.html)
- [Tokio Mutex Documentation](https://docs.rs/tokio/latest/tokio/sync/struct.Mutex.html)
- [Async: What is Blocking?](https://ryhl.io/blog/async-what-is-blocking/) — Alice Ryhl
- [TimescaleDB Hypertable Documentation](https://docs.timescale.com/use-timescale/latest/hypertables/about-hypertables/)
- Astro Clock codebase analysis (chunk_manager.rs, pool.rs, server/mod.rs)

---

*Synthesized from STACK.md, FEATURES.md, ARCHITECTURE.md, PITFALLS.md*  
*Synthesis date: 2026-03-01*  
*Confidence: HIGH*

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

# Feature Landscape: Job System & Named Queries

**Domain:** Job orchestration for astrological data loading and electoral query execution
**Researched:** 2026-03-01
**Confidence:** HIGH

## Research Context

Building v1.1 of Astro Clock, which adds job-based orchestration for:
- Day-level incremental data loading
- Named query templates (wedding, project start, travel)
- Async/sync execution modes
- Job status tracking and polling

Existing infrastructure: TimescaleDB with 4 hypertables, chunk-based LRU cache, query layer with <100ms performance for wedding queries.

---

## Table Stakes (Users Expect These)

Features users assume exist in any job system. Missing these = product feels broken.

| Feature | Why Expected | Complexity | Dependencies | Notes |
|---------|--------------|------------|--------------|-------|
| **Job lifecycle states** | Core concept of job queuing | LOW | JOB-01, JOB-03 | pending → in-process → (complete \| failed). Must persist state transitions atomically. |
| **Unique job IDs** | Required for tracking and referencing jobs | LOW | JOB-04 | UUID v4 recommended for distributed safety. Sequential IDs also acceptable for single-node. |
| **Payload storage** | Jobs need input parameters | LOW | JOB-05 | Store as JSONB in PostgreSQL. Schema: `{start_date, days, query_type, ...}` |
| **Result storage** | Jobs must return query results | LOW | JOB-05 | Store as JSONB. Failed jobs store error details instead. |
| **Sync execution mode** | CLI users expect blocking calls | LOW | JOB-06, RESULT-01 | Execute job inline, return result directly. HTTP timeout risk for long jobs (>30s). |
| **Async execution mode** | API users expect non-blocking | MEDIUM | JOB-06, RESULT-02 | Spawn background task, return job-id immediately. Required for long-running loads. |
| **Job status endpoint** | Polling is standard pattern | LOW | API-04, RESULT-03 | GET /api/v1/jobs/{id} → `{status, created_at, completed_at, result? \| error?}` |
| **Job list endpoint** | Users need to discover jobs | LOW | API-05 | GET /api/v1/jobs → paginated list with filters (status, date range) |
| **Named query structure** | Templates need definition format | MEDIUM | QUERY-06 to QUERY-08 | Each query: name, criteria builder, result type, required data ranges |
| **Date range tracking** | Incremental loading requires knowing what's loaded | MEDIUM | JOB-02, LOAD-10 | loaded_days table: date, loaded_at. Enables resume capability. |
| **Error reporting** | Failed jobs need clear error messages | LOW | RESULT-04 | Include error code (enum) and human-readable message |
| **CLI job commands** | Command-line interface expected | LOW | CLI-04, CLI-05 | `job status <id>`, `job list` with table output |

---

## Differentiators (Competitive Advantage)

Features that set this implementation apart. Not required, but valuable for v1.1.

| Feature | Value Proposition | Complexity | Dependencies | Notes |
|---------|-------------------|------------|--------------|-------|
| **Intelligent incremental loading** | Only compute missing days, saving 90%+ time on subsequent calls | MEDIUM | LOAD-08, LOAD-10 | Check loaded_days before generating. Load gaps, not full range. |
| **Pre-flight data check** | Named queries auto-load missing data before executing | MEDIUM | QUERY-10 | Query determines required date range, triggers load job if gaps exist. Seamless UX. |
| **Parallel day loading** | Load multiple days concurrently | MEDIUM | LOAD-09 | Use `tokio::task::JoinSet` to load N days in parallel. Speedup ~4-8×. |
| **Job result streaming** | Large result sets need pagination | MEDIUM | RESULT-05 | Cursor-based pagination for query results >1000 rows. |
| **Query result metadata** | Execution time, rows examined, cache hit status | LOW | Existing QueryResult<T> | Already implemented in v1.0, just persist in job result. |
| **Atomic multi-day transactions** | All days in range succeed or fail together | MEDIUM | sqlx::Transaction | Wrap day-level inserts in transaction per day or per batch. |
| **CLI sync timeout warning** | Warn users if sync job might timeout | LOW | CLI | Calculate estimated time based on days to load, suggest --async if >10s. |

---

## Anti-Features (Explicitly NOT Building)

Features that seem good but create unnecessary complexity for v1.1.

| Feature | Why Requested | Why Problematic | What to Do Instead |
|---------|---------------|-----------------|-------------------|
| **Real-time progress streaming** | Users want to see % complete | Adds WebSocket/SSE complexity, not needed for astronomy tool | Use polling with status endpoint. Most jobs complete in <30s anyway. |
| **Job cancellation** | Users might want to abort long jobs | Requires cooperative cancellation throughout Swiss Ephemeris bindings | Document "restart is safe" due to incremental loading. Cancel via process kill if truly needed. |
| **Automatic job retry** | Failed jobs should retry automatically | Masking failures leads to confusion, wastes compute on persistent errors | Manual retry via CLI/API. User decides if failure was transient. |
| **Query result caching** | Speed up repeated queries | Database IS the cache. Adding another layer adds invalidation complexity | Rely on TimescaleDB query performance (<100ms). Re-execute query on each request. |
| **Job prioritization** | Some jobs more important than others | Complex queue management, starvation issues | FIFO is sufficient. Most jobs are user-initiated and short. |
| **Job scheduling/cron** | Load data automatically on schedule | Adds daemon complexity, scheduler dependency | Document cron + CLI for external scheduling. Keep app stateless. |
| **Distributed job workers** | Scale across multiple machines | Massive complexity (Redis, message queues, worker coordination) | Single-node async with tokio tasks is sufficient. Scale via bigger instance. |

---

## Feature Dependencies

```
[Job Infrastructure]
    ├──requires──> [Database Schema] (jobs table, loaded_days table)
    ├──requires──> [Tokio Runtime] (already present)
    └──enables──> [Data Loading], [Named Queries]

[Data Loading]
    ├──requires──> [Job Infrastructure]
    ├──requires──> [Swiss Ephemeris Integration] (existing)
    ├──requires──> [Multi-resolution Storage] (existing)
    ├──enhances──> [Named Queries] (pre-flight data check)
    └──conflicts──> [Real-time Streaming] (loading is batch, not stream)

[Named Queries]
    ├──requires──> [Job Infrastructure]
    ├──requires──> [Query Layer] (existing wedding/voc/retrograde)
    ├──requires──> [Data Loading] (for auto-load capability)
    └──enhances──> [CLI], [HTTP API]

[CLI Interface]
    ├──requires──> [Job Infrastructure]
    ├──requires──> [Named Queries]
    └──conflicts──> [Long-running Sync] (HTTP timeout risk)

[HTTP API]
    ├──requires──> [Job Infrastructure]
    ├──requires──> [Named Queries]
    ├──requires──> [Axum Server] (existing)
    └──enforces──> [Async Mode] (for long operations)
```

### Dependency Notes

- **Named Queries requires Data Loading:** For the intelligent pre-flight data check (QUERY-10), the named query system must be able to trigger data loading if required dates aren't present.

- **CLI conflicts with Long-running Sync:** Synchronous CLI calls that take >30s risk HTTP timeouts when proxied. Recommend async mode for loads >7 days.

- **Job Infrastructure enables everything:** The jobs table and state machine must be built first. All other features depend on it.

---

## Complexity Analysis

### LOW Complexity (1-2 days each)

These are mostly CRUD operations with straightforward database schemas:

| Feature | Implementation |
|---------|----------------|
| Job lifecycle states | Enum + CHECK constraint in PostgreSQL |
| Unique job IDs | `uuid::Uuid::new_v4()` or `SERIAL` |
| Payload/result storage | `JSONB` columns in jobs table |
| Job status endpoint | Single SQL query by ID |
| Job list endpoint | SQL query with LIMIT/OFFSET |
| Error reporting | Structured error type with code + message |
| CLI job commands | clap subcommands + table output |

### MEDIUM Complexity (3-5 days each)

These require coordination, async patterns, or algorithmic logic:

| Feature | Implementation |
|---------|----------------|
| Async execution mode | `tokio::task::spawn()` + background execution |
| Intelligent incremental loading | Date range diff algorithm against loaded_days |
| Pre-flight data check | Query planner that checks coverage before execution |
| Parallel day loading | `JoinSet` with semaphore-limited concurrency |
| Named query structure | Trait-based query registry with factory pattern |
| Atomic multi-day transactions | sqlx::Transaction per day or batch |

### HIGH Complexity (defer to v2+)

Not needed for v1.1:

| Feature | Why Deferred |
|---------|--------------|
| Real-time progress streaming | WebSocket/SSE overhead not justified |
| Job cancellation | Cooperative cancellation complex with FFI |
| Distributed workers | Single-node sufficient |
| Configurable query templates | Requires query DSL, validation, security review |

---

## MVP Definition for v1.1

### Launch With (v1.1 Core)

Minimum viable job system:

- [x] **Job infrastructure** (JOB-01 to JOB-06) — Foundation for everything
- [x] **Day-level incremental loading** (LOAD-06 to LOAD-10) — Core data pipeline
- [x] **Wedding named query** (QUERY-06, QUERY-09 to QUERY-11) — Proves template pattern
- [x] **Sync/async modes** (RESULT-01, RESULT-02) — Required for both CLI and API
- [x] **Job status/polling** (CLI-04, CLI-05, API-04, API-05) — Standard job patterns
- [x] **Intelligent data loading** (QUERY-10, LOAD-08) — Key differentiator

### Add After Validation (v1.1.x)

Features to add once core is working:

- [ ] **Project named query** (QUERY-07) — Copy wedding pattern with different criteria
- [ ] **Travel named query** (QUERY-08) — Copy wedding pattern with different criteria
- [ ] **Parallel day loading** — Optimize loading performance
- [ ] **CLI progress indicators** — Show loading progress for UX

### Future Consideration (v2+)

Features to defer until v1.1 proves value:

- [ ] **Configurable query templates** (CONFIG-01, CONFIG-02) — JSON/YAML query definitions
- [ ] **Advanced queries** (ADV-01 to ADV-04) — Grand trines, T-squares, transits
- [ ] **Job result streaming** — Pagination for massive result sets
- [ ] **Export features** (EXP-01 to EXP-03) — JSON/CSV export of results

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority | Phase |
|---------|------------|---------------------|----------|-------|
| Job infrastructure | HIGH | MEDIUM | P1 | Phase 5 |
| Day-level incremental loading | HIGH | MEDIUM | P1 | Phase 6 |
| Wedding named query | HIGH | LOW | P1 | Phase 7 |
| Intelligent data loading | HIGH | MEDIUM | P1 | Phase 7 |
| Sync/async modes | HIGH | LOW | P1 | Phase 5 |
| Job status/polling | HIGH | LOW | P1 | Phase 5-8 |
| Project named query | MEDIUM | LOW | P2 | Phase 7 |
| Travel named query | MEDIUM | LOW | P2 | Phase 7 |
| Parallel day loading | MEDIUM | MEDIUM | P2 | Phase 6 |
| CLI progress indicators | LOW | LOW | P3 | Phase 8 |
| Configurable templates | MEDIUM | HIGH | P3 | v2 |
| Advanced queries | MEDIUM | MEDIUM | P3 | v2 |

**Priority Key:**
- P1: Must have for v1.1 launch
- P2: Should have, add when core is stable
- P3: Nice to have, future consideration

---

## Integration with Existing Features

### Dependencies on v1.0 Infrastructure

| Existing Feature | How v1.1 Uses It |
|------------------|------------------|
| **TimescaleDB hypertables** | Job results query planet_positions, aspects, lunar_conditions. Loading populates these tables. |
| **Chunk-based LRU cache** | Named queries can leverage cache for hot date ranges. Job loading bypasses cache (direct DB insert). |
| **Query layer (wedding.rs, etc.)** | Named queries wrap these functions with job lifecycle management. |
| **DatabasePool** | Jobs acquire connections from existing pool. Need to ensure pool size (5) is sufficient for parallel loading. |
| **Axum server** | New endpoints added to existing Router. Follow existing handler patterns. |
| **CLI app.rs** | New subcommands added to Commands enum. Follow existing command structure. |

### Performance Constraints

| Constraint | v1.1 Impact |
|------------|-------------|
| **<100ms query time** | Named queries must maintain performance. Pre-flight data check adds overhead; async mode recommended for large ranges. |
| **~30MB memory limit** | Job results for large date ranges may exceed this. Implement cursor-based pagination if needed. |
| **<50GB/year storage** | Day-level loading granularity + aspect filtering keeps storage bounded. |

---

## Data Flow Diagram

```
User Request
    │
    ├──► CLI sync ──► Execute job inline ──► Return result
    │
    ├──► CLI async ──► Create job ──► Spawn background task
    │                       │
    ├──► API sync ──► Execute job inline ──► Return result
    │                       │
    └──► API async ──► Create job ──► Return job-id
                            │
                            ▼
                    [Job State: pending]
                            │
                            ▼
                    [Worker picks up job]
                            │
                            ▼
                    [Job State: in-process]
                            │
            ┌───────────────┼───────────────┐
            │               │               │
            ▼               ▼               ▼
    [Check loaded_days] [Load missing] [Execute query]
            │               │               │
            └───────────────┴───────────────┘
                            │
                            ▼
                    [Job State: complete]
                            │
            ┌───────────────┴───────────────┐
            │                               │
            ▼                               ▼
    [Store result JSON]           [On failure: store error]
            │                               │
            ▼                               ▼
    [User polls /jobs/{id}] ◄──── [Job State: failed]
            │
            ▼
    [Return result to user]
```

---

## Risk Assessment

| Feature | Risk | Mitigation |
|---------|------|------------|
| **Async job execution** | Panic in background task crashes process | Use `catch_unwind` or structured error handling. Log failures. |
| **Parallel day loading** | Database connection pool exhaustion | Limit concurrency with semaphore (max 3 concurrent days). |
| **Long-running sync jobs** | HTTP gateway timeout | Document 30s limit. Recommend async for >7 days. |
| **Job result size** | JSONB size limits (~1GB) | Paginate large results. Warn if result >10MB. |
| **Swiss Ephemeris FFI** | Not async-safe, blocks runtime | Use `spawn_blocking` for ephemeris calculations. |

---

## Sources

- **Tokio Task Documentation:** https://docs.rs/tokio/latest/tokio/task/ — Task spawning, JoinHandle, cancellation patterns
- **SQLx Documentation:** https://docs.rs/sqlx/latest/sqlx/ — Async PostgreSQL, transactions, JSONB support
- **Existing Astro Clock Codebase:** v1.0 implementation patterns (wedding queries, chunk loading, server structure)
- **Requirements Document:** REQUIREMENTS.md v1.1 specifications
- **Project Context:** PROJECT.md architecture decisions and constraints

---

*Feature research for: Astro Clock v1.1 Job System*
*Researched: 2026-03-01*
*Confidence: HIGH — based on established patterns in Rust async ecosystem and existing codebase*

# Pitfalls Research: Job System Implementation

**Domain:** Rust job queue with PostgreSQL/TimescaleDB, async/sync execution modes
**Researched:** 2026-03-01
**Confidence:** HIGH (based on official documentation, established patterns, and codebase analysis)

## Critical Pitfalls

### Pitfall 1: Transaction-Job Race Condition

**What goes wrong:**
When a job is created inside a database transaction, the job processor may start executing before the transaction commits. The job fails because the data it depends on isn't visible yet (transaction isolation). Even worse, if the transaction rolls back, the job remains in the queue but can never succeed.

**Why it happens:**
The job queue (whether in-memory or external like Redis) operates outside the database transaction. When `queue_job()` is called, the job becomes visible immediately, but the data changes are still uncommitted.

```rust
// DANGEROUS PATTERN
db.transaction(|tx| async move {
    insert_planet_data(&tx, data).await?;  // Data not committed yet
    job_queue.submit(LoadJob { date }).await?;  // Job visible immediately!
    update_loaded_days(&tx, date).await?;
    Ok(())
}).await?;
// Job may run here, before commit completes
```

**How to avoid:**
Use a **transactionally-staged job drain** pattern:
1. Insert jobs into a `staged_jobs` table within the same transaction as your data changes
2. A separate enqueuer process polls `staged_jobs` and moves jobs to the active queue only after the transaction commits
3. Jobs are invisible until committed, and rolled back automatically with the transaction

```rust
// SAFE PATTERN
db.transaction(|tx| async move {
    insert_planet_data(&tx, data).await?;
    // Job staged in same transaction - invisible until commit
    stage_job(&tx, LoadJob { date }).await?;
    update_loaded_days(&tx, date).await?;
    Ok(())
}).await?;
// Enqueuer polls staged_jobs and moves to active queue
```

**Warning signs:**
- "Record not found" errors in job logs for data that should exist
- Jobs failing with FK constraint violations on data that was "just inserted"
- Intermittent job failures that succeed on retry

**Phase to address:** Phase 5 (Job Infrastructure) - Design the job staging table and enqueuer logic from the start

---

### Pitfall 2: Connection Pool Exhaustion from Job Workers

**What goes wrong:**
Job workers consume database connections from the same pool as query handlers. Under load, long-running jobs starve the query handlers, causing the <100ms query target to be missed or requests to timeout.

**Why it happens:**
The Astro Clock uses a pool of 5 connections (`max_connections(5)` in `pool.rs`). If job workers take 3 connections for bulk loading, only 2 remain for queries. TimescaleDB hypertable operations can be particularly connection-hungry during chunk creation.

**How to avoid:**
1. **Separate connection pools**: Use a dedicated pool for job workers with its own limits
2. **Connection limits per pool**:
   - Query pool: 3 connections (priority for <100ms queries)
   - Job pool: 5 connections (can tolerate slower operations)
3. **Monitor pool metrics**: Track `pool.size()` and `pool.num_idle()` to detect exhaustion
4. **Use `Pool::try_acquire()` for queries** to fail fast rather than wait indefinitely

```rust
// Two pools with different purposes
pub struct DatabasePools {
    pub query_pool: Pool<Postgres>,   // Fast queries, small
    pub job_pool: Pool<Postgres>,     // Background jobs, larger
}
```

**Warning signs:**
- Query latency spikes correlate with job execution
- "Pool timed out" errors in query handlers
- `pool.num_idle()` consistently near zero

**Phase to address:** Phase 5 (Job Infrastructure) - Set up dual pool architecture during initial job system design

---

### Pitfall 3: Async/Sync Mode Deadlocks

**What goes wrong:**
When mixing sync and async job execution, blocking operations in async contexts cause deadlocks or thread pool exhaustion. The Tokio runtime can deadlock if all worker threads block waiting for sync job results.

**Why it happens:**
Tokio's default multi-threaded runtime spawns one worker thread per CPU core (typically 8). If you spawn sync jobs that block threads, and those sync jobs internally try to use async operations (even indirectly), you can exhaust all workers.

```rust
// DANGEROUS - Can deadlock
#[tokio::main]
async fn main() {
    let job = spawn_sync_job(data);  // Blocks a thread
    let result = job.await;  // May never complete if threads exhausted
}

// Inside sync job (running on spawn_blocking thread)
fn sync_job(data) {
    // If this tries to use tokio::spawn or any async op...
    tokio::spawn(async { ... });  // DEADLOCK: no threads available
}
```

**How to avoid:**
1. **Use `tokio::task::spawn_blocking`** for CPU-intensive work, but don't call async code from within
2. **Bridge pattern**: Use channels to communicate between sync and async boundaries
3. **Separate runtimes** (if needed): Create a dedicated Tokio runtime for jobs if they need async
4. **Clear API boundaries**:
   - Async jobs: Use `tokio::spawn` for I/O-bound work
   - Sync jobs: Use `tokio::task::spawn_blocking` for CPU-bound work, return results via channels

```rust
// SAFE - Bridge pattern
pub async fn execute_sync(job: Job) -> Result<JobResult> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    
    tokio::task::spawn_blocking(move || {
        // Pure synchronous work here - NO async calls
        let result = cpu_intensive_work(job);
        let _ = tx.send(result);  // Send result back
    });
    
    rx.await.map_err(|_| Error::JobFailed)?
}
```

**Warning signs:**
- Jobs that "hang" indefinitely
- Timeouts on sync job operations
- Runtime metrics showing all threads blocked

**Phase to address:** Phase 5 (Job Infrastructure) - Design the execution abstraction layer

---

### Pitfall 4: Cache Coherency Violations During Data Loading

**What goes wrong:**
When a job loads new data into the database, the existing LRU cache becomes stale. Subsequent queries may return incomplete or incorrect results because they're served from cache before the job's data is visible.

**Why it happens:**
The `ChunkManager` maintains an LRU cache (`LruCache<ChunkKey, Arc<ChunkData>>`) that checks the database on miss. If a job loads data for day X while the cache has an entry for day X (from before the load), queries return stale data until the cache entry expires.

**How to avoid:**
1. **Cache invalidation on job completion**: After a loading job completes, invalidate affected cache entries
2. **Versioned cache keys**: Include a data version/timestamp in the cache key
3. **Loaded days tracking**: Query the `loaded_days` table before cache lookup to detect new data
4. **Explicit cache warming**: Load new data into cache immediately after database insert

```rust
// After job completes
impl ChunkManager {
    pub async fn invalidate_date_range(&self, start: NaiveDate, end: NaiveDate) {
        let mut cache = self.cache.write().await;
        let mut current = start;
        while current <= end {
            let key = ChunkKey::new(current);
            cache.pop(&key);  // Remove if present
            current += Duration::days(1);
        }
    }
}
```

**Warning signs:**
- Query results missing recently-loaded data
- Cache hit rate drops after data loading jobs
- Inconsistent results between successive queries

**Phase to address:** Phase 6 (Data Loading) - Integrate cache invalidation with job completion

---

### Pitfall 5: Job State Machine Inconsistency

**What goes wrong:**
Jobs get "stuck" in intermediate states (e.g., `in_process` forever) due to crashes or errors. The system can't distinguish between "job is still running" and "job crashed without updating status."

**Why it happens:**
Without a heartbeat mechanism or timeout, a job that starts (`in_process`) but crashes before completion appears to be running indefinitely. Users can't tell if they should wait or retry.

**How to avoid:**
1. **Heartbeat pattern**: Jobs update `last_heartbeat` timestamp periodically while running
2. **Timeout detection**: A background task marks jobs as `failed` if heartbeat is stale (>5 minutes)
3. **Idempotency**: Design jobs to be safely retryable even if partially complete
4. **State transitions**: Enforce valid transitions (pending→in_process→complete|failed) at database level

```rust
// State machine enforcement with CHECK constraint
sqlx::query(r#"
    ALTER TABLE jobs ADD CONSTRAINT valid_state_transition
    CHECK (status IN ('pending', 'in_process', 'complete', 'failed'));
"#).execute(&pool).await?;

// Heartbeat update during long jobs
async fn update_heartbeat(job_id: Uuid, pool: &Pool) -> Result<()> {
    sqlx::query("UPDATE jobs SET last_heartbeat = NOW() WHERE id = $1")
        .bind(job_id)
        .execute(pool)
        .await?;
    Ok(())
}
```

**Warning signs:**
- Jobs stuck in `in_process` for hours
- Users unable to determine if job is actually running
- Duplicate job submissions because status is unclear

**Phase to address:** Phase 5 (Job Infrastructure) - Design state machine and heartbeat mechanism upfront

---

### Pitfall 6: Hypertable Lock Contention

**What goes wrong:**
Concurrent jobs inserting into the same hypertable chunk cause lock contention and performance degradation. TimescaleDB creates row-level locks, but chunk management operations (creating new chunks) require heavier locks.

**Why it happens:**
TimescaleDB partitions data into chunks by time. When inserting data, if a chunk doesn't exist, it's created with an exclusive lock. Multiple jobs inserting into the same time range compete for these locks.

**How to avoid:**
1. **Day-level locking**: Use advisory locks (`pg_advisory_lock`) to serialize jobs targeting the same day
2. **Pre-create chunks**: Generate chunks in advance to avoid creation during inserts
3. **Batch inserts**: Collect data in memory and insert in larger batches (reduces lock frequency)
4. **Stagger job start times**: Add jitter to prevent thundering herd

```rust
// Advisory lock for day-level serialization
async fn load_day_with_lock(date: NaiveDate, pool: &Pool) -> Result<()> {
    let date_hash = date_hash(date);  // Convert date to i64
    
    // Acquire advisory lock for this specific day
    sqlx::query("SELECT pg_advisory_lock($1)")
        .bind(date_hash)
        .fetch_optional(pool)
        .await?;
    
    // ... perform loading ...
    
    // Release lock
    sqlx::query("SELECT pg_advisory_unlock($1)")
        .bind(date_hash)
        .fetch_optional(pool)
        .await?;
    
    Ok(())
}
```

**Warning signs:**
- Insert performance degrades with concurrent jobs
- High lock wait times in PostgreSQL logs
- Jobs timing out during bulk inserts

**Phase to address:** Phase 6 (Data Loading) - Implement day-level locking strategy

---

### Pitfall 7: Memory Pressure from Large Job Results

**What goes wrong:**
Jobs that query large date ranges return massive result sets that exhaust memory. The JSON result storage in the jobs table grows unbounded, and fetching job status becomes slow.

**Why it happens:**
The job result field stores JSON data. A 60-day wedding query could return thousands of results. Storing this in the jobs table bloats the table and slows down job listing queries.

**How to avoid:**
1. **Result pagination**: Store large results in a separate table, paginated
2. **Result limits**: Cap result size; store "result available" flag with external storage
3. **Streaming results**: For very large queries, don't store results at all - require re-query
4. **Compression**: Compress large JSON results before storage

```rust
// Store large results separately
pub struct JobResult {
    pub job_id: Uuid,
    pub result_type: ResultType,  // Inline, External, Stream
    pub inline_data: Option<JsonValue>,  // Small results only
    pub external_ref: Option<String>,  // Reference to external storage
}

// Separate table for large results
sqlx::query(r#"
    CREATE TABLE job_results (
        job_id UUID PRIMARY KEY REFERENCES jobs(id) ON DELETE CASCADE,
        result_data JSONB,
        created_at TIMESTAMPTZ DEFAULT NOW()
    );
"#).execute(&pool).await?;
```

**Warning signs:**
- Job listing queries slow down over time
- Memory usage spikes when fetching job status
- Database table bloat in the jobs table

**Phase to address:** Phase 7 (Named Queries) - Design result storage with size limits

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Single connection pool | Simpler code, less config | Query starvation under load | Only in MVP with <2 concurrent jobs |
| No job staging table | Faster to implement, fewer tables | Transaction race conditions, lost jobs | Never in production |
| Synchronous-only jobs | Simpler mental model | Blocks HTTP handlers, poor UX | CLI-only mode only |
| No cache invalidation | Less code to write | Stale data, incorrect query results | Never when data changes |
| Store all results in jobs table | Simple result retrieval | Table bloat, performance degradation | Only if result size <10KB guaranteed |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| ChunkManager | Jobs bypass cache, queries get stale data | Invalidate cache entries after job completes |
| DatabasePool | One pool for everything | Separate pools for queries vs jobs |
| TimescaleDB | Bulk insert without chunk consideration | Use day-level advisory locks |
| Tokio Runtime | spawn_blocking without bridge | Use channels to return results to async context |
| SQLx | Holding connections across `.await` points | Acquire connection, do work, release quickly |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Unbounded job queue | Memory growth, OOM crashes | Limit queue size, use backpressure | >100 pending jobs |
| Long-running sync jobs | HTTP timeouts, unresponsive API | Set job timeout, use async for long tasks | Jobs >30 seconds |
| Missing query/job isolation | Query latency spikes | Separate connection pools | >3 concurrent jobs |
| No chunk-level locking | Insert deadlocks | Advisory locks per day | >2 jobs same day |
| Large result storage | Slow job listing, table bloat | External result storage | Results >100KB |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| SQL injection in named queries | Data breach, unauthorized access | Use parameterized queries only, validate query parameters |
| Job ID enumeration | Information disclosure about other users' jobs | Use UUIDs for job IDs, validate ownership |
| Unbounded date ranges | DoS via resource exhaustion | Enforce max date range (e.g., 365 days), rate limit |
| Job result exposure | Sensitive data leakage | Filter results based on requester permissions |

## "Looks Done But Isn't" Checklist

- [ ] **Job creation**: Often missing transaction wrapping - verify jobs are created atomically with data changes
- [ ] **Sync mode**: Often missing timeout handling - verify sync jobs have configurable timeouts
- [ ] **Cache invalidation**: Often forgotten - verify cache is cleared when jobs modify data
- [ ] **Error handling**: Often incomplete - verify all error paths update job status to `failed`
- [ ] **Connection cleanup**: Often leaky - verify connections are released even on panics
- [ ] **Heartbeat**: Often absent - verify long jobs update heartbeat to prevent timeout
- [ ] **Result limits**: Often unbounded - verify large results are paginated or rejected

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Stuck jobs (in_process forever) | LOW | Update status to `failed` manually; implement heartbeat to prevent recurrence |
| Cache corruption | MEDIUM | Clear cache entirely; implement proper invalidation |
| Connection pool exhaustion | LOW | Restart application; implement separate pools |
| Duplicate job execution | LOW | Implement idempotency keys; deduplicate on retry |
| Lost jobs (not in queue) | HIGH | Restore from backup; implement staging table pattern |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Transaction-Job Race Condition | Phase 5 | Test with concurrent job creation and verify no "not found" errors |
| Connection Pool Exhaustion | Phase 5 | Load test with 5+ concurrent jobs, verify query latency <100ms |
| Async/Sync Mode Deadlocks | Phase 5 | Test sync job spawning 20+ concurrent tasks |
| Cache Coherency Violations | Phase 6 | Load data via job, immediately query same dates, verify results include new data |
| Job State Machine Inconsistency | Phase 5 | Kill job process mid-execution, verify heartbeat timeout marks job failed |
| Hypertable Lock Contention | Phase 6 | Run 3 jobs for same date range concurrently, verify completion without deadlock |
| Memory Pressure from Large Results | Phase 7 | Create job with 365-day query, verify memory stable, verify result storage strategy |

## Sources

- [Transactionally Staged Job Drains in Postgres](https://brandur.org/job-drain) - Brandur Leach's seminal article on solving the transaction-job race condition
- [SQLx Pool Documentation](https://docs.rs/sqlx/latest/sqlx/struct.Pool.html) - Official SQLx connection pool behavior and configuration
- [Tokio Mutex Documentation](https://docs.rs/tokio/latest/tokio/sync/struct.Mutex.html) - When to use async vs sync mutexes
- [Async: What is Blocking?](https://ryhl.io/blog/async-what-is-blocking/) - Alice Ryhl's guide to blocking in async Rust
- [TimescaleDB Hypertable Documentation](https://docs.timescale.com/use-timescale/latest/hypertables/about-hypertables/) - Chunk behavior and partitioning
- Astro Clock codebase analysis (chunk_manager.rs, pool.rs, server/mod.rs) - Current architecture and constraints

---
*Pitfalls research for: Astro Clock v1.1 Job System*
*Researched: 2026-03-01*