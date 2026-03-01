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
