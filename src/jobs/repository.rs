use crate::jobs::error::{JobError, JobResult};
use crate::jobs::types::{Job, JobStatus, JobType};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::{Pool, Postgres, QueryBuilder};
use uuid::Uuid;

/// Opaque cursor encoding a (created_at, id) boundary row.
/// Clients treat this as an opaque string — the encoding may change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobCursor {
    pub created_at: DateTime<Utc>,
    pub id: Uuid,
}

impl JobCursor {
    /// Encode cursor as base64url-no-pad JSON string
    pub fn encode(&self) -> String {
        let json = serde_json::to_string(self).expect("cursor serialization infallible");
        URL_SAFE_NO_PAD.encode(json)
    }

    /// Decode cursor from base64url-no-pad string
    pub fn decode(s: &str) -> Result<Self, String> {
        let json = URL_SAFE_NO_PAD
            .decode(s)
            .map_err(|e| format!("Invalid cursor encoding: {}", e))?;
        serde_json::from_slice(&json).map_err(|e| format!("Invalid cursor data: {}", e))
    }
}

/// Direction of cursor pagination
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorDirection {
    /// Forward = older jobs (default direction, DESC order)
    Forward,
    /// Backward = newer jobs (reversed within page to maintain DESC order)
    Backward,
}

/// Parsed filter parameters for listing jobs.
/// Shared contract between the handler (which parses query strings) and the repository.
#[derive(Debug, Clone, Default)]
pub struct JobListFilters {
    pub status: Vec<JobStatus>,
    pub job_type: Vec<JobType>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

/// Repository for job CRUD operations and job queue claiming
#[derive(Debug, Clone)]
pub struct JobRepository {
    pool: Pool<Postgres>,
}

impl JobRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    /// Get reference to the connection pool
    pub fn pool(&self) -> &Pool<Postgres> {
        &self.pool
    }

    /// Create a new job in pending status
    pub async fn create_job(&self, job_type: JobType, payload: JsonValue) -> JobResult<Job> {
        let job = Job::new(job_type, payload);

        sqlx::query_as::<_, Job>(
            r#"
            INSERT INTO jobs (id, job_type, status, payload, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(job.id)
        .bind(&job.job_type)
        .bind(&job.status)
        .bind(&job.payload)
        .bind(job.created_at)
        .bind(job.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(JobError::from)
    }

    /// Get job by ID
    pub async fn get_job(&self, id: Uuid) -> JobResult<Option<Job>> {
        sqlx::query_as::<_, Job>("SELECT * FROM jobs WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(JobError::from)
    }

    /// Delete a job by ID.
    /// Returns true if a row was deleted, false if the job was not found.
    pub async fn delete_job(&self, id: Uuid) -> JobResult<bool> {
        let result = sqlx::query("DELETE FROM jobs WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(JobError::from)?;
        Ok(result.rows_affected() > 0)
    }

    /// Claim next pending job using FOR UPDATE SKIP LOCKED (race-free)
    /// Returns the claimed job or None if no jobs available
    pub async fn claim_next_job(&self, worker_id: &str) -> JobResult<Option<Job>> {
        let now = Utc::now();

        sqlx::query_as::<_, Job>(
            r#"
            UPDATE jobs 
            SET status = 'in_process', 
                started_at = $1, 
                updated_at = $1,
                worker_id = $2
            WHERE id = (
                SELECT id FROM jobs 
                WHERE status = 'pending' 
                ORDER BY created_at 
                FOR UPDATE SKIP LOCKED
                LIMIT 1
            )
            RETURNING *
            "#,
        )
        .bind(now)
        .bind(worker_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(JobError::from)
    }

    /// Update job status with validation
    pub async fn update_job_status(
        &self,
        id: Uuid,
        new_status: JobStatus,
        result: Option<JsonValue>,
        error: Option<JsonValue>,
    ) -> JobResult<Job> {
        // Get current job to validate transition
        let current = self.get_job(id).await?;
        let job = current.ok_or_else(|| JobError::NotFound(id.to_string()))?;

        let current_status = job
            .status_enum()
            .ok_or_else(|| JobError::Other(format!("Invalid status: {}", job.status)))?;

        if !current_status.can_transition_to(new_status) {
            return Err(JobError::InvalidStatusTransition(
                current_status.to_string(),
                new_status.to_string(),
            ));
        }

        let now = Utc::now();

        sqlx::query_as::<_, Job>(
            r#"
            UPDATE jobs 
            SET status = $1, 
                updated_at = $2,
                completed_at = CASE WHEN $1 IN ('complete', 'failed') THEN $2 ELSE completed_at END,
                result = COALESCE($3, result),
                error = COALESCE($4, error)
            WHERE id = $5
            RETURNING *
            "#,
        )
        .bind(new_status.as_ref())
        .bind(now)
        .bind(result)
        .bind(error)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(JobError::from)
    }

    /// List jobs with dynamic filtering and cursor-based pagination.
    ///
    /// Returns up to `count` jobs plus one extra row to detect whether another page exists.
    /// The caller should check if `result.len() > count` and, if so, truncate to `count`
    /// and use the last returned row's `(created_at, id)` as the next cursor.
    ///
    /// For backward pagination, rows are fetched in ASC order and then reversed
    /// so the caller always receives DESC order.
    #[allow(unused_assignments)]
    pub async fn list_jobs(
        &self,
        filters: JobListFilters,
        count: i64,
        cursor: Option<JobCursor>,
        direction: CursorDirection,
    ) -> JobResult<Vec<Job>> {
        let mut query = QueryBuilder::new("SELECT * FROM jobs");
        let mut has_where = false;

        // --- Filter clauses (shared across all cursor modes) ---

        if !filters.status.is_empty() {
            query.push(" WHERE status IN (");
            let mut separated = query.separated(", ");
            for s in &filters.status {
                separated.push_bind(s.as_ref());
            }
            query.push(")");
            has_where = true;
        }

        if !filters.job_type.is_empty() {
            if has_where {
                query.push(" AND job_type IN (");
            } else {
                query.push(" WHERE job_type IN (");
                has_where = true;
            }
            let mut separated = query.separated(", ");
            for jt in &filters.job_type {
                separated.push_bind(jt.as_ref());
            }
            query.push(")");
        }

        if let Some(after) = filters.created_after {
            if has_where {
                query.push(" AND created_at >= ");
            } else {
                query.push(" WHERE created_at >= ");
                has_where = true;
            }
            query.push_bind(after);
        }

        if let Some(before) = filters.created_before {
            if has_where {
                query.push(" AND created_at <= ");
            } else {
                query.push(" WHERE created_at <= ");
                has_where = true;
            }
            query.push_bind(before);
        }

        // --- Cursor clause ---

        match (&cursor, direction) {
            (Some(cur), CursorDirection::Forward) => {
                // Forward: rows strictly before the cursor (older)
                if has_where {
                    query.push(" AND (created_at, id) < (");
                } else {
                    query.push(" WHERE (created_at, id) < (");
                }
                query.push_bind(cur.created_at);
                query.push(", ");
                query.push_bind(cur.id);
                query.push(")");
            }
            (Some(cur), CursorDirection::Backward) => {
                // Backward: rows strictly after the cursor (newer)
                if has_where {
                    query.push(" AND (created_at, id) > (");
                } else {
                    query.push(" WHERE (created_at, id) > (");
                }
                query.push_bind(cur.created_at);
                query.push(", ");
                query.push_bind(cur.id);
                query.push(")");
            }
            (None, _) => { /* First page — no cursor condition */ }
        }

        // --- ORDER BY + LIMIT ---
        //
        // Forward / no cursor: DESC (newest first)
        // Backward: ASC so we pick the N rows immediately newer than the cursor,
        //           then we reverse below to maintain DESC order for the caller.

        if direction == CursorDirection::Backward && cursor.is_some() {
            query.push(" ORDER BY created_at ASC, id ASC LIMIT ");
        } else {
            query.push(" ORDER BY created_at DESC, id DESC LIMIT ");
        }
        query.push_bind(count + 1); // +1 to detect next-page existence

        let mut jobs = query
            .build_query_as::<Job>()
            .fetch_all(&self.pool)
            .await
            .map_err(JobError::from)?;

        // Reverse backward pages so caller always sees DESC order
        if direction == CursorDirection::Backward && cursor.is_some() {
            jobs.reverse();
        }

        Ok(jobs)
    }

    /// Count jobs with dynamic filtering
    #[allow(unused_assignments)]
    pub async fn count_jobs(&self, filters: JobListFilters) -> JobResult<i64> {
        let mut query = QueryBuilder::new("SELECT COUNT(*) FROM jobs");
        let mut has_where = false;

        if !filters.status.is_empty() {
            query.push(" WHERE status IN (");
            let mut separated = query.separated(", ");
            for s in &filters.status {
                separated.push_bind(s.as_ref());
            }
            query.push(")");
            has_where = true;
        }

        if !filters.job_type.is_empty() {
            if has_where {
                query.push(" AND job_type IN (");
            } else {
                query.push(" WHERE job_type IN (");
                has_where = true;
            }
            let mut separated = query.separated(", ");
            for jt in &filters.job_type {
                separated.push_bind(jt.as_ref());
            }
            query.push(")");
        }

        if let Some(after) = filters.created_after {
            if has_where {
                query.push(" AND created_at >= ");
            } else {
                query.push(" WHERE created_at >= ");
                has_where = true;
            }
            query.push_bind(after);
        }

        if let Some(before) = filters.created_before {
            if has_where {
                query.push(" AND created_at <= ");
            } else {
                query.push(" WHERE created_at <= ");
                has_where = true;
            }
            query.push_bind(before);
        }

        let count: (i64,) = query
            .build_query_as()
            .fetch_one(&self.pool)
            .await
            .map_err(JobError::from)?;

        Ok(count.0)
    }
}

/// Repository for tracking loaded date ranges
#[derive(Debug, Clone)]
pub struct LoadedDaysRepository {
    pool: Pool<Postgres>,
}

impl LoadedDaysRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    /// Mark a date as loaded with coverage information
    pub async fn mark_day_loaded(
        &self,
        date: chrono::NaiveDate,
        coverage_minutes: i16,
        job_id: Option<Uuid>,
    ) -> JobResult<()> {
        sqlx::query(
            r#"
            INSERT INTO loaded_days (date, coverage_minutes, loaded_at, job_id)
            VALUES ($1, $2, NOW(), $3)
            ON CONFLICT (date) DO UPDATE SET
                coverage_minutes = EXCLUDED.coverage_minutes,
                loaded_at = EXCLUDED.loaded_at,
                job_id = EXCLUDED.job_id
            "#,
        )
        .bind(date)
        .bind(coverage_minutes)
        .bind(job_id)
        .execute(&self.pool)
        .await
        .map_err(JobError::from)?;

        Ok(())
    }

    /// Check if a date is already loaded
    pub async fn is_day_loaded(&self, date: chrono::NaiveDate) -> JobResult<bool> {
        let row: Option<(i64,)> =
            sqlx::query_as("SELECT 1 FROM loaded_days WHERE date = $1 LIMIT 1")
                .bind(date)
                .fetch_optional(&self.pool)
                .await
                .map_err(JobError::from)?;

        Ok(row.is_some())
    }

    /// Get dates in a range that are NOT loaded
    pub async fn get_missing_dates(
        &self,
        start: chrono::NaiveDate,
        days: i64,
    ) -> JobResult<Vec<chrono::NaiveDate>> {
        // Generate series of dates and find which aren't in loaded_days
        let rows: Vec<(chrono::NaiveDate,)> = sqlx::query_as(
            r#"
            SELECT gs::date as date
            FROM generate_series($1::date, $1::date + ($2 || ' days')::interval, '1 day'::interval) gs
            WHERE gs::date NOT IN (
                SELECT date FROM loaded_days 
                WHERE date BETWEEN $1 AND $1 + ($2 || ' days')::interval
            )
            ORDER BY gs::date
            "#
        )
        .bind(start)
        .bind(days)
        .fetch_all(&self.pool)
        .await
        .map_err(JobError::from)?;

        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    /// Get loaded date range summary
    pub async fn get_loaded_range_summary(
        &self,
    ) -> JobResult<Option<(chrono::NaiveDate, chrono::NaiveDate, i64)>> {
        let row: Option<(chrono::NaiveDate, chrono::NaiveDate, i64)> =
            sqlx::query_as("SELECT MIN(date), MAX(date), COUNT(*) FROM loaded_days")
                .fetch_optional(&self.pool)
                .await
                .map_err(JobError::from)?;

        Ok(row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_cursor_encode_decode_roundtrip() {
        let cursor = JobCursor {
            created_at: Utc.with_ymd_and_hms(2025, 6, 15, 10, 30, 0).unwrap(),
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        };
        let encoded = cursor.encode();
        let decoded = JobCursor::decode(&encoded).expect("decode should succeed");
        assert_eq!(decoded.created_at, cursor.created_at);
        assert_eq!(decoded.id, cursor.id);
    }

    #[test]
    fn test_cursor_decode_invalid_base64() {
        let result = JobCursor::decode("!!!not-base64!!!");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("Invalid cursor encoding"),
            "Error should mention encoding failure, got: {}",
            err
        );
    }

    #[test]
    fn test_cursor_decode_invalid_json() {
        // Valid base64 of non-JSON payload
        use base64::Engine;
        use base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let encoded = URL_SAFE_NO_PAD.encode("this is not json");
        let result = JobCursor::decode(&encoded);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("Invalid cursor data"),
            "Error should mention data failure, got: {}",
            err
        );
    }

    #[test]
    fn test_cursor_encode_no_padding() {
        let cursor = JobCursor {
            created_at: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
            id: Uuid::nil(),
        };
        let encoded = cursor.encode();
        assert!(
            !encoded.contains('='),
            "Encoded cursor should not contain padding characters, got: {}",
            encoded
        );
    }
}
