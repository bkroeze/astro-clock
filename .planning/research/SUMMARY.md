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
