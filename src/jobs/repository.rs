use crate::jobs::error::{JobError, JobResult};
use crate::jobs::types::{Job, JobStatus, JobType};
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::{Pool, Postgres, QueryBuilder};
use uuid::Uuid;

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
    pub async fn create_job(
        &self,
        job_type: JobType,
        payload: JsonValue,
    ) -> JobResult<Job> {
        let job = Job::new(job_type, payload);
        
        sqlx::query_as::<_, Job>(
            r#"
            INSERT INTO jobs (id, job_type, status, payload, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#
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
    pub async fn get_job(&self, id: Uuid
    ) -> JobResult<Option<Job>> {
        sqlx::query_as::<_, Job>(
            "SELECT * FROM jobs WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(JobError::from)
    }

    /// Claim next pending job using FOR UPDATE SKIP LOCKED (race-free)
    /// Returns the claimed job or None if no jobs available
    pub async fn claim_next_job(
        &self,
        worker_id: &str,
    ) -> JobResult<Option<Job>> {
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
            "#
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
        
        let current_status = job.status_enum()
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
            "#
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

    /// List recent jobs with pagination
    pub async fn list_jobs(
        &self,
        status: Option<JobStatus>,
        limit: i64,
        offset: i64,
    ) -> JobResult<Vec<Job>> {
        let mut query = QueryBuilder::new("SELECT * FROM jobs");
        
        if let Some(s) = status {
            query.push(" WHERE status = ");
            query.push_bind(s.to_string());
        }
        
        query.push(" ORDER BY created_at DESC LIMIT ");
        query.push_bind(limit);
        query.push(" OFFSET ");
        query.push_bind(offset);
        
        query.build_query_as::<Job>()
            .fetch_all(&self.pool)
            .await
            .map_err(JobError::from)
    }

    /// Count jobs by status
    pub async fn count_jobs(&self, status: Option<JobStatus>
    ) -> JobResult<i64> {
        let count: (i64,) = if let Some(s) = status {
            sqlx::query_as("SELECT COUNT(*) FROM jobs WHERE status = $1")
                .bind(s.as_ref())
                .fetch_one(&self.pool)
                .await?
        } else {
            sqlx::query_as("SELECT COUNT(*) FROM jobs")
                .fetch_one(&self.pool)
                .await?
        };
        
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
            "#
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
    pub async fn is_day_loaded(
        &self,
        date: chrono::NaiveDate,
    ) -> JobResult<bool> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT 1 FROM loaded_days WHERE date = $1 LIMIT 1"
        )
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
        let row: Option<(chrono::NaiveDate, chrono::NaiveDate, i64)> = sqlx::query_as(
            "SELECT MIN(date), MAX(date), COUNT(*) FROM loaded_days"
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(JobError::from)?;
        
        Ok(row)
    }
}
