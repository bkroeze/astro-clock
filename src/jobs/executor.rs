use crate::jobs::error::{JobError, JobResult};
use crate::jobs::repository::JobRepository;
use crate::jobs::types::{Job, JobStatus, JobType};
use async_trait::async_trait;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::oneshot;
use uuid::Uuid;

/// Handler for executing specific job types
/// Implementors handle the actual business logic for a job type
#[async_trait]
pub trait JobHandler: Send + Sync {
    /// Job type this handler processes
    fn job_type(&self) -> JobType;

    /// Execute the job logic
    /// Returns Ok(result) on success, Err(error_json) on failure
    async fn execute(&self, job: &Job) -> JobResult<JsonValue>;
}

/// Executor for managing job lifecycle and execution modes
#[derive(Clone)]
pub struct JobExecutor {
    repository: JobRepository,
    handlers: Arc<HashMap<JobType, Arc<dyn JobHandler>>>,
    worker_id: String,
}

impl JobExecutor {
    /// Create a new executor with the given repository and handlers
    pub fn new(
        repository: JobRepository,
        handlers: Vec<Arc<dyn JobHandler>>,
        worker_id: String,
    ) -> Self {
        let handler_map: HashMap<_, _> = handlers.into_iter().map(|h| (h.job_type(), h)).collect();

        Self {
            repository,
            handlers: Arc::new(handler_map),
            worker_id,
        }
    }

    /// Get handler for a job type
    fn get_handler(&self, job_type: JobType) -> JobResult<Arc<dyn JobHandler>> {
        self.handlers
            .get(&job_type)
            .cloned()
            .ok_or_else(|| JobError::Other(format!("No handler registered for job type: {}", job_type)))
    }

    /// Execute a job synchronously - blocks until completion
    /// Returns the completed job with result or error
    ///
    /// This is used by CLI commands where the user waits for results.
    pub async fn execute_sync(
        &self,
        job_type: JobType,
        payload: JsonValue,
    ) -> JobResult<Job> {
        // 1. Create the job in pending status
        let job = self.repository.create_job(job_type, payload).await?;

        // 2. Claim the job immediately (we just created it)
        let job = self.claim_job_for_worker(job.id).await?;

        // 3. Execute the job (potentially CPU-intensive)
        let handler = self.get_handler(job_type)?;
        let job_clone = job.clone();

        // Use spawn_blocking for CPU-intensive work to avoid blocking async runtime
        let (tx, rx) = oneshot::channel();
        let handler_clone = Arc::clone(&handler);

        tokio::task::spawn_blocking(move || {
            // Block on the async handler execution
            let runtime = tokio::runtime::Handle::current();
            let result = runtime.block_on(async { handler_clone.execute(&job_clone).await });
            let _ = tx.send(result);
        });

        // Wait for completion
        let result = rx
            .await
            .map_err(|_| JobError::Other("Job execution channel closed".to_string()))?;

        // 4. Update job status based on result
        match result {
            Ok(result_json) => {
                self.repository
                    .update_job_status(job.id, JobStatus::Complete, Some(result_json), None)
                    .await
            }
            Err(e) => {
                let error_json = serde_json::json!({
                    "message": e.to_string(),
                });
                self.repository
                    .update_job_status(job.id, JobStatus::Failed, None, Some(error_json))
                    .await
            }
        }
    }

    /// Execute a job asynchronously - returns job-id immediately
    /// Job runs in background, caller polls for completion
    ///
    /// This is used by HTTP API where we return immediately with a job-id.
    pub async fn execute_async(
        &self,
        job_type: JobType,
        payload: JsonValue,
    ) -> JobResult<Uuid> {
        // 1. Create the job in pending status
        let job = self.repository.create_job(job_type, payload).await?;
        let job_id = job.id;

        // 2. Spawn background task to execute the job
        let executor = self.clone();

        tokio::spawn(async move {
            // Process the job in background
            if let Err(e) = executor.process_job_background(job_id).await {
                tracing::error!("Background job {} failed: {}", job_id, e);
            }
        });

        // 3. Return job-id immediately
        Ok(job_id)
    }

    /// Poll for job status and result
    pub async fn poll_job_status(&self, job_id: Uuid) -> JobResult<Option<Job>> {
        self.repository.get_job(job_id).await
    }

    /// Process a job in the background (used by execute_async)
    async fn process_job_background(&self, job_id: Uuid) -> JobResult<()> {
        // 1. Try to claim the job
        let job = match self.repository.claim_next_job(&self.worker_id).await? {
            Some(j) if j.id == job_id => j,
            Some(_) => {
                // Job claimed by another worker, that's fine
                return Ok(());
            }
            None => {
                // No job available or job not found
                return Ok(());
            }
        };

        let job_type = job.job_type_enum().ok_or_else(|| {
            JobError::Other(format!("Invalid job type: {}", job.job_type))
        })?;

        // 2. Get handler and execute
        let handler = self.get_handler(job_type)?;
        let result = handler.execute(&job).await;

        // 3. Update status
        match result {
            Ok(result_json) => {
                self.repository
                    .update_job_status(job_id, JobStatus::Complete, Some(result_json), None)
                    .await?;
            }
            Err(e) => {
                let error_json = serde_json::json!({
                    "message": e.to_string(),
                });
                self.repository
                    .update_job_status(job_id, JobStatus::Failed, None, Some(error_json))
                    .await?;
            }
        }

        Ok(())
    }

    /// Claim a specific job by ID (for sync execution where we just created it)
    async fn claim_job_for_worker(&self, job_id: Uuid) -> JobResult<Job> {
        use chrono::Utc;

        // Direct update to claim this specific job
        sqlx::query_as::<_, Job>(
            r#"
            UPDATE jobs
            SET status = 'in_process',
                started_at = $1,
                updated_at = $1,
                worker_id = $2
            WHERE id = $3 AND status = 'pending'
            RETURNING *
            "#,
        )
        .bind(Utc::now())
        .bind(&self.worker_id)
        .bind(job_id)
        .fetch_one(self.repository.pool())
        .await
        .map_err(JobError::from)
    }
}
