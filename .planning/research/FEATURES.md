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
