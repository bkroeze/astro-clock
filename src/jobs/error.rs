use sqlx::Error as SqlxError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JobError {
    #[error("Database error: {0}")]
    Database(#[from] SqlxError),

    #[error("Job not found: {0}")]
    NotFound(String),

    #[error("Invalid job status transition: {0} -> {1}")]
    InvalidStatusTransition(String, String),

    #[error("Job already claimed by another worker")]
    AlreadyClaimed,

    #[error("JSON serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Unknown error: {0}")]
    Other(String),
}

pub type JobResult<T> = Result<T, JobError>;
