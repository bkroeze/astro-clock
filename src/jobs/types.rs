use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use uuid::Uuid;

/// Job types supported by the system
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum_macros::Display,
    strum_macros::AsRefStr,
    strum_macros::EnumString,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    Load,  // Load planetary data for date range
    Query, // Execute named query (wedding, project, travel)
}

/// Job status with state machine enforcement
/// Valid transitions: pending → in_process → (complete | failed)
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum_macros::Display,
    strum_macros::AsRefStr,
    strum_macros::EnumString,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    InProcess,
    Complete,
    Failed,
}

impl JobStatus {
    /// Check if a status transition is valid
    pub fn can_transition_to(&self, new_status: JobStatus) -> bool {
        matches!(
            (self, new_status),
            (JobStatus::Pending, JobStatus::InProcess)
                | (JobStatus::InProcess, JobStatus::Complete)
                | (JobStatus::InProcess, JobStatus::Failed)
        )
    }
}

/// Job entity representing an async operation
#[derive(Debug, Clone, FromRow)]
pub struct Job {
    pub id: Uuid,
    pub job_type: String, // Stored as string in DB, parse to JobType
    pub status: String,   // Stored as string in DB, parse to JobStatus
    pub payload: Option<JsonValue>,
    pub result: Option<JsonValue>,
    pub error: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Job {
    /// Create a new job with pending status
    pub fn new(job_type: JobType, payload: JsonValue) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            job_type: job_type.to_string(),
            status: JobStatus::Pending.to_string(),
            payload: Some(payload),
            result: None,
            error: None,
            created_at: now,
            updated_at: now,
            started_at: None,
            completed_at: None,
        }
    }

    /// Parse job_type string to enum
    pub fn job_type_enum(&self) -> Option<JobType> {
        match self.job_type.as_str() {
            "load" => Some(JobType::Load),
            "query" => Some(JobType::Query),
            _ => None,
        }
    }

    /// Parse status string to enum
    pub fn status_enum(&self) -> Option<JobStatus> {
        match self.status.as_str() {
            "pending" => Some(JobStatus::Pending),
            "in_process" => Some(JobStatus::InProcess),
            "complete" => Some(JobStatus::Complete),
            "failed" => Some(JobStatus::Failed),
            _ => None,
        }
    }

    /// Get duration since job was started (if in progress)
    pub fn duration(&self) -> Option<chrono::Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end - start),
            (Some(start), None) => Some(Utc::now() - start),
            _ => None,
        }
    }
}
