# Phase 7: Named Queries - Research

**Researched:** 2026-03-01
**Domain:** Astro Clock Job System — Query Templates
**Confidence:** HIGH

---

## Summary

Phase 7 implements **named query templates** for wedding, project, and travel date queries. Building on the job infrastructure (Phase 5) and data loading (Phase 6), this phase wraps existing query functions from `src/queries/` in a `JobHandler` that:

1. **Automatically loads missing data** before executing queries (QUERY-10)
2. **Wraps existing query functions** without modification (QUERY-06, QUERY-07, QUERY-08)
3. **Supports both sync/async modes** via JobExecutor (QUERY-11)
4. **Returns structured JSON results** in job responses (RESULT-05)

**Primary recommendation:** Create a `QueryJobHandler` following the `LoadJobHandler` pattern, with a `QueryTemplateRegistry` that maps query names ("wedding", "project", "travel") to existing query functions. Implement project and travel query functions following the wedding query SQL pattern.

---

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| QUERY-06 | Named query "wedding" — find auspicious wedding dates | Use existing `find_wedding_dates()` in `src/queries/wedding.rs` |
| QUERY-07 | Named query "project" — find good dates to start projects | Create new `find_project_dates()` with Mercury-focused criteria |
| QUERY-08 | Named query "travel" — find favorable travel dates | Create new `find_travel_dates()` with Moon/Mercury criteria |
| QUERY-09 | Named queries accept date range parameters (start_date, days) | Payload parsing pattern from LoadJobHandler |
| QUERY-10 | Named queries intelligently load missing data before executing | Use LoadedDaysRepository + ChunkGenerator pre-flight check |
| QUERY-11 | Named queries support sync/async execution modes | JobExecutor.execute_sync() / execute_async() |
| RESULT-05 | Complete jobs include query results in result field (JSON) | Serialize QueryResult<T> to JSON |

---

## Standard Stack

### Core (Existing)
| Component | Location | Purpose |
|-----------|----------|---------|
| JobExecutor | `src/jobs/executor.rs` | Sync/async job execution |
| JobHandler trait | `src/jobs/executor.rs` | Handler interface |
| LoadJobHandler | `src/jobs/handlers/load.rs` | Pattern to follow |
| LoadedDaysRepository | `src/jobs/repository.rs` | Gap detection |
| ChunkGenerator | `src/database/chunk_generator.rs` | Data generation |

### New Components Required
| Component | Purpose | Pattern |
|-----------|---------|---------|
| `QueryJobHandler` | Execute named queries as jobs | Follow `LoadJobHandler` structure |
| `QueryTemplateRegistry` | Map names to query functions | HashMap<String, QueryTemplate> |
| `ProjectCriteria` / `TravelCriteria` | Query-specific parameters | Follow `WeddingCriteria` pattern in `src/queries/types.rs` |
| `find_project_dates()` | Project start query logic | SQL similar to wedding query |
| `find_travel_dates()` | Travel query logic | SQL with Moon sign + Mercury direct |

---

## Architecture Patterns

### Pattern 1: Handler Orchestration (from LoadJobHandler)

**What:** Handler coordinates repositories and generators, implements JobHandler trait

**Structure:**
```rust
#[derive(Debug, Clone)]
pub struct QueryJobHandler {
    loaded_days_repo: LoadedDaysRepository,
    pool: Pool<Postgres>,
}

#[async_trait]
impl JobHandler for QueryJobHandler {
    fn job_type(&self) -> JobType {
        JobType::Query
    }

    async fn execute(&self, job: &Job) -> JobResult<JsonValue> {
        // 1. Parse payload to extract query_name, start_date, days
        // 2. Check for missing dates via loaded_days_repo
        // 3. If missing dates, load them via ChunkGenerator
        // 4. Execute query via registry
        // 5. Return results as JSON
    }
}
```

### Pattern 2: Query Template Registry

**What:** Central registry maps query names to execution functions

**Why:** Decouples query definition from execution; enables dynamic registration later

```rust
pub struct QueryTemplate {
    pub name: String,
    pub description: String,
    pub execute_fn: Box<dyn Fn(&DatabasePool, &QueryParams) -> 
        Pin<Box<dyn Future<Output = Result<JsonValue, QueryError>> + Send>> + Send + Sync>,
}

pub struct QueryTemplateRegistry {
    templates: HashMap<String, QueryTemplate>,
}

impl QueryTemplateRegistry {
    pub fn new() -> Self {
        let mut registry = Self { templates: HashMap::new() };
        
        registry.register("wedding", |pool, params| {
            Box::pin(async move {
                let criteria = WeddingCriteria::from(params)?;
                let result = find_wedding_dates(pool, &criteria).await?;
                Ok(serde_json::to_value(result)?)
            })
        });
        
        // Similar for "project" and "travel"
        registry
    }
}
```

### Pattern 3: Pre-flight Data Loading (from QUERY-10)

**What:** Check loaded_days table before executing query; load missing data via ChunkGenerator

```rust
async fn ensure_data_loaded(&self, start_date: NaiveDate, days: i64) -> JobResult<()> {
    let missing_dates = self.loaded_days_repo
        .get_missing_dates(start_date, days)
        .await?;
    
    if !missing_dates.is_empty() {
        let chunk_gen = ChunkGenerator::new(DatabasePool::from_pool(self.pool.clone()));
        for date in missing_dates {
            let chunk = chunk_gen.generate_chunk(date).await?;
            chunk_gen.save_chunk_to_db(&chunk).await?;
            self.loaded_days_repo.mark_day_loaded(date, 1440, None).await?;
        }
    }
    Ok(())
}
```

### Pattern 4: Structured Job Results (from RESULT-05)

**What:** Return QueryResult<T> serialized to JSON for job.result field

```rust
pub async fn execute(&self, job: &Job) -> JobResult<JsonValue> {
    // ... execute query ...
    let query_result: QueryResult<WeddingCandidate> = find_wedding_dates(pool, &criteria).await?;
    
    // Serialize to JSON for job storage
    let job_result = QueryJobResult {
        query_name: "wedding".to_string(),
        start_date: params.start_date.clone(),
        days: params.days,
        total_candidates: query_result.data.len(),
        candidates: query_result.data,
        execution_time_ms: query_result.execution_time_ms,
    };
    
    serde_json::to_value(job_result).map_err(JobError::from)
}
```

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Job execution modes | Custom sync/async logic | `JobExecutor::execute_sync()` / `execute_async()` | Already handles state machine, error handling, JSON serialization |
| Data loading | Inline chunk generation | `ChunkGenerator` + `LoadedDaysRepository` | Existing retry logic, transaction handling |
| Query dispatch | Match statement on query name | `QueryTemplateRegistry` | Extensible, testable, type-safe |
| Result serialization | Manual JSON building | `serde_json::to_value()` | Handles all edge cases, type safety |

---

## Common Pitfalls

### Pitfall 1: Modifying Existing Query Functions

**What goes wrong:** Adding job-awareness to `find_wedding_dates()` pollutes pure query logic

**Why it happens:** Temptation to make queries "job-aware" for convenience

**How to avoid:** Keep query functions in `src/queries/` pure; wrap them in QueryJobHandler. Existing `find_wedding_dates()` signature stays unchanged.

### Pitfall 2: Skipping Data Pre-flight

**What goes wrong:** Query returns empty results because data isn't loaded for the date range

**Why it happens:** Forgetting QUERY-10 requirement to auto-load missing data

**How to avoid:** Always call `ensure_data_loaded()` before executing query in QueryJobHandler. Check LoadedDaysRepository first.

### Pitfall 3: Blocking on Data Loading in Async Mode

**What goes wrong:** Async job submission blocks until data loading completes

**Why it happens:** Calling synchronous data loading from async context without spawn_blocking

**How to avoid:** Use `spawn_blocking` for CPU-intensive work (ChunkGenerator), same pattern as LoadJobHandler. The JobExecutor already wraps handler execution in spawn_blocking.

### Pitfall 4: Inconsistent Query Result Format

**What goes wrong:** Different query types return incompatible JSON structures

**Why it happens:** Each query implements result serialization independently

**How to avoid:** Define standard `QueryJobResult` wrapper struct with common fields (query_name, start_date, days, total_results, execution_time_ms, results array).

---

## Code Examples

### Example 1: QueryJobHandler Payload Parsing

```rust
/// Payload for query jobs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryJobPayload {
    /// Query name: "wedding", "project", or "travel"
    pub query_name: String,
    /// Start date in YYYY-MM-DD format
    pub start_date: String,
    /// Number of days to query
    pub days: i64,
}

impl QueryJobHandler {
    fn parse_payload(&self, job: &Job) -> JobResult<QueryJobPayload> {
        let payload = job.payload.as_ref()
            .ok_or_else(|| JobError::Other("Missing job payload".to_string()))?;
        
        let query_name = payload.get("query_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JobError::Other("Missing query_name".to_string()))?;
        
        // Validate query_name is supported
        if !["wedding", "project", "travel"].contains(&query_name) {
            return Err(JobError::Other(format!("Unknown query: {}", query_name)));
        }
        
        // Parse start_date and days same as LoadJobHandler
        // ...
        
        Ok(QueryJobPayload { query_name: query_name.to_string(), start_date, days })
    }
}
```

### Example 2: Project Query Implementation

```rust
// In src/queries/project.rs
pub async fn find_project_dates(
    pool: &DatabasePool,
    criteria: &ProjectCriteria,
) -> Result<QueryResult<ProjectCandidate>, QueryError> {
    // Good project start criteria:
    // - Mercury direct (not retrograde)
    // - Moon in favorable signs (avoid Scorpio, Capricorn)
    // - Mars well-aspected (no squares/oppositions)
    
    let rows = sqlx::query(
        r#"
        SELECT 
            pp.time,
            pp.zodiac_sign as moon_sign,
            COALESCE(asum.total_favorable, 0) as favorable_aspects,
            COALESCE(asum.total_challenging, 0) as challenging_aspects
        FROM planet_positions pp
        LEFT JOIN aspect_summaries asum ON pp.time = asum.time AND asum.body_id = $3 -- Mars
        WHERE pp.body_id = $4    -- Moon
          AND pp.time >= $1 AND pp.time <= $2
          AND pp.zodiac_sign = ANY($5)  -- Exclude Scorpio(7), Capricorn(9)
          AND NOT EXISTS (
              SELECT 1 FROM lunar_conditions lc 
              WHERE lc.time = pp.time AND lc.is_void_of_course = true
          )
          -- Mercury must be direct (check retrograde_periods)
          AND NOT EXISTS (
              SELECT 1 FROM retrograde_periods rp
              WHERE rp.body_id = $6  -- Mercury
                AND pp.time BETWEEN rp.start_time AND rp.end_time
          )
        ORDER BY asum.total_favorable DESC NULLS LAST, pp.time
        LIMIT $7
        "#
    )
    .bind(start_datetime)
    .bind(end_datetime)
    .bind(body_ids::MARS)
    .bind(body_ids::MOON)
    .bind(&favorable_signs)  // Exclude 7, 9
    .bind(body_ids::MERCURY)
    .bind(criteria.limit as i64)
    .fetch_all(pool.pool())
    .await?;
    
    // Map to ProjectCandidate structs...
}
```

### Example 3: Travel Query Implementation

```rust
// In src/queries/travel.rs
pub async fn find_travel_dates(
    pool: &DatabasePool,
    criteria: &TravelCriteria,
) -> Result<QueryResult<TravelCandidate>, QueryError> {
    // Good travel criteria:
    // - Moon not void-of-course
    // - Mercury direct (communication/transport)
    // - No challenging Mars aspects (avoid accidents)
    // - Favorable Moon signs for the purpose
    
    // Similar SQL pattern to wedding/project
    // Focus on Mercury direct + Moon conditions
}
```

### Example 4: API Route Handler for Queries

```rust
// In src/server/routes/queries.rs (following jobs.rs pattern)
pub async fn query_handler(
    State(state): State<AppState>,
    Path(query_name): Path<String>,
    Json(request): Json<QueryRequest>,
) -> impl IntoResponse {
    // Validate query_name
    if !["wedding", "project", "travel"].contains(&query_name.as_str()) {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "unknown_query" }))).into_response();
    }
    
    // Validate date format and days range (same as load_handler)
    // ...
    
    // Build payload
    let payload = json!({
        "query_name": query_name,
        "start_date": request.start_date,
        "days": request.days,
    });
    
    if request.sync.unwrap_or(false) {
        match state.executor().execute_sync(JobType::Query, payload).await {
            Ok(job) => {
                let response = QuerySyncResponse {
                    job_id: job.id,
                    status: job.status,
                    result: job.result,
                };
                (StatusCode::OK, Json(json!(response))).into_response()
            }
            Err(e) => { /* error handling */ }
        }
    } else {
        // Async mode...
    }
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Direct query execution | Job-wrapped queries with data pre-flight | Phase 7 | Queries automatically load missing data |
| Inline query logic | Template registry pattern | Phase 7 | Extensible, testable query system |
| Manual result formatting | Structured QueryResult<T> with serde | Phase 3 | Consistent JSON output |

**Deprecated/outdated:**
- None — Phase 7 adds new patterns on top of existing infrastructure

---

## Open Questions

1. **Project Query Criteria**
   - What we know: Mercury direct is important, Moon sign matters
   - What's unclear: Exact favorable signs, Mars aspect thresholds
   - Recommendation: Start with Mercury direct + favorable Moon signs (Taurus, Cancer, Leo, Libra, Pisces), exclude Scorpio/Capricorn

2. **Travel Query Criteria**
   - What we know: Moon not VoC, Mercury direct for communication/transport
   - What's unclear: Specific aspect requirements, sign preferences
   - Recommendation: Moon direct + Mercury direct + no challenging Mars aspects

3. **Query Result Limits**
   - What we know: Wedding query uses default limit of 10
   - What's unclear: Should project/travel have different defaults?
   - Recommendation: Use same default (10) with optional limit parameter in payload

4. **Partial Data Loading Failure**
   - What we know: LoadJobHandler continues on per-date failure
   - What's unclear: Should query proceed if some dates fail to load?
   - Recommendation: Query proceeds with loaded dates; include warning in result about missing dates

---

## Validation Architecture

**Note:** `workflow.nyquist_validation` is `true` in config.json, but this phase implements handlers/routes, not core business logic. Testing strategy:

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (built-in) + sqlx test |
| Config file | None — tests use in-memory/assert patterns |
| Quick run | `cargo test -p astro-clock query` |
| Full suite | `cargo test` |
| Integration tests | Require running TimescaleDB instance |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Notes |
|--------|----------|-----------|-------|
| QUERY-06 | Wedding query returns candidates | Unit + Integration | Mock query function for unit |
| QUERY-07 | Project query returns candidates | Unit + Integration | New test in tests/ |
| QUERY-08 | Travel query returns candidates | Unit + Integration | New test in tests/ |
| QUERY-09 | Payload parsing accepts start_date/days | Unit | Test parse_payload() directly |
| QUERY-10 | Auto-load missing data | Integration | Requires DB |
| QUERY-11 | Sync/async modes work | Integration | Test via executor |
| RESULT-05 | Results in JSON format | Unit | Test serialization |

### Wave 0 Gaps
- [ ] `src/queries/project.rs` — New module for project query logic
- [ ] `src/queries/travel.rs` — New module for travel query logic
- [ ] `src/jobs/handlers/query.rs` — QueryJobHandler implementation
- [ ] `src/jobs/registry.rs` — QueryTemplateRegistry
- [ ] `src/server/routes/queries.rs` — Query API endpoints

---

## Sources

### Primary (HIGH confidence)
- `src/jobs/handlers/load.rs` — LoadJobHandler pattern to follow
- `src/jobs/executor.rs` — JobHandler trait, JobExecutor
- `src/queries/wedding.rs` — Query function pattern
- `src/queries/types.rs` — Criteria and result type patterns
- `.planning/research/ARCHITECTURE.md` — QueryTemplateRegistry design

### Secondary (MEDIUM confidence)
- `src/server/routes/jobs.rs` — API route patterns
- `src/cli/app.rs` — CLI command patterns
- `.planning/phases/03-query-system/03-RESEARCH.md` — Query system background

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — Patterns established in Phases 5-6
- Architecture: HIGH — ARCHITECTURE.md already designed the pattern
- Pitfalls: MEDIUM — Based on LoadJobHandler experience, but query domain adds complexity

**Research date:** 2026-03-01
**Valid until:** 2026-04-01 (stable patterns)