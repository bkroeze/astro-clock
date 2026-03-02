//! Job API route handlers
//!
//! Provides endpoints for:
//! - POST /api/v1/load - Create and execute load jobs (sync or async)
//! - GET /api/v1/jobs/:id - Get job status and details

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::jobs::repository::JobRepository;
use crate::jobs::types::{Job, JobType};
use crate::server::state::AppState;

/// Request to create a load job
#[derive(Debug, Deserialize)]
pub struct LoadRequest {
    /// Start date in YYYY-MM-DD format
    pub start_date: String,
    /// Number of days to load (1-365)
    pub days: i64,
    /// If true, execute synchronously and wait for completion
    /// If false or omitted, execute asynchronously and return job-id immediately
    #[serde(default)]
    pub sync: Option<bool>,
}

/// Response for successful sync load execution
#[derive(Debug, Serialize)]
pub struct LoadSyncResponse {
    pub job_id: Uuid,
    pub status: String,
    pub result: Option<JsonValue>,
}

/// Response for async load submission
#[derive(Debug, Serialize)]
pub struct LoadAsyncResponse {
    pub job_id: Uuid,
    pub status: String,
    pub poll_url: String,
}

/// Response for job status query
#[derive(Debug, Serialize)]
pub struct JobResponse {
    pub job_id: Uuid,
    pub job_type: String,
    pub status: String,
    pub payload: Option<JsonValue>,
    pub result: Option<JsonValue>,
    pub error: Option<JobErrorResponse>,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

/// Error details for failed jobs
#[derive(Debug, Serialize)]
pub struct JobErrorResponse {
    pub code: String,
    pub message: String,
}

/// Error response body
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

/// Request parameters for listing jobs
#[derive(Debug, Deserialize)]
pub struct ListJobsRequest {
    /// Filter by status (pending, in_process, complete, failed)
    pub status: Option<String>,
    /// Maximum number of jobs to return (default: 20, max: 100)
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// Offset for pagination (default: 0)
    #[serde(default = "default_offset")]
    pub offset: i64,
}

fn default_limit() -> i64 { 20 }
fn default_offset() -> i64 { 0 }

/// Response for job listing
#[derive(Debug, Serialize)]
pub struct ListJobsResponse {
    pub jobs: Vec<JobResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// POST /api/v1/load - Create and execute a load job
///
/// In sync mode (sync=true): Blocks until completion, returns full job with result
/// In async mode (sync=false or omitted): Returns immediately with job-id for polling
pub async fn load_handler(
    State(state): State<AppState>,
    Json(request): Json<LoadRequest>,
) -> impl IntoResponse {
    // Validate days parameter (must be 1-365)
    if request.days < 1 || request.days > 365 {
        let error = ErrorResponse {
            error: "invalid_days".to_string(),
            message: format!("Days must be between 1 and 365, got {}", request.days),
        };
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!(error))).into_response();
    }

    // Validate date format (YYYY-MM-DD)
    if NaiveDate::parse_from_str(&request.start_date, "%Y-%m-%d").is_err() {
        let error = ErrorResponse {
            error: "invalid_date".to_string(),
            message: format!(
                "Invalid date format: '{}', expected YYYY-MM-DD",
                request.start_date
            ),
        };
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!(error))).into_response();
    }

    // Build payload JSON
    let payload = serde_json::json!({
        "start_date": request.start_date,
        "days": request.days,
    });

    let sync_mode = request.sync.unwrap_or(false);

    if sync_mode {
        // Synchronous execution: block until completion
        match state.executor().execute_sync(JobType::Load, payload).await {
            Ok(job) => {
                let response = LoadSyncResponse {
                    job_id: job.id,
                    status: job.status,
                    result: job.result,
                };
                (StatusCode::OK, Json(serde_json::json!(response))).into_response()
            }
            Err(e) => {
                tracing::error!("Sync load job failed: {}", e);
                let error = ErrorResponse {
                    error: "execution_failed".to_string(),
                    message: format!("{}", e),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!(error))).into_response()
            }
        }
    } else {
        // Asynchronous execution: return job-id immediately
        match state.executor().execute_async(JobType::Load, payload).await {
            Ok(job_id) => {
                let response = LoadAsyncResponse {
                    job_id,
                    status: "pending".to_string(),
                    poll_url: format!("/api/v1/jobs/{}", job_id),
                };
                (StatusCode::ACCEPTED, Json(serde_json::json!(response))).into_response()
            }
            Err(e) => {
                tracing::error!("Async load job submission failed: {}", e);
                let error = ErrorResponse {
                    error: "submission_failed".to_string(),
                    message: format!("{}", e),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!(error))).into_response()
            }
        }
    }
}

/// GET /api/v1/jobs/:id - Get job details and status
///
/// Returns full job information including:
/// - Job metadata (id, type, status, timestamps)
/// - Payload (input parameters)
/// - Result (output data, if complete)
/// - Error (error details, if failed)
pub async fn get_job_handler(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> impl IntoResponse {
    let repository = JobRepository::new(state.get_pool());

    match repository.get_job(job_id).await {
        Ok(Some(job)) => {
            let response = build_job_response(job);
            (StatusCode::OK, Json(serde_json::json!(response))).into_response()
        }
        Ok(None) => {
            let error = ErrorResponse {
                error: "not_found".to_string(),
                message: format!("Job {} not found", job_id),
            };
            (StatusCode::NOT_FOUND, Json(serde_json::json!(error))).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get job {}: {}", job_id, e);
            let error = ErrorResponse {
                error: "query_failed".to_string(),
                message: format!("{}", e),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!(error))).into_response()
        }
    }
}

/// GET /api/v1/jobs - List recent jobs with pagination
///
/// Query parameters:
/// - status: Filter by status (optional)
/// - limit: Max jobs to return (default: 20, max: 100)
/// - offset: Pagination offset (default: 0)
pub async fn list_jobs_handler(
    State(state): State<AppState>,
    Query(params): Query<ListJobsRequest>,
) -> impl IntoResponse {
    let repository = JobRepository::new(state.get_pool());

    // Validate and cap limit
    let limit = params.limit.max(1).min(100);
    let offset = params.offset.max(0);

    // Parse status filter if provided
    let status_filter = params.status.clone().and_then(|s| match s.as_str() {
        "pending" => Some(crate::jobs::types::JobStatus::Pending),
        "in_process" => Some(crate::jobs::types::JobStatus::InProcess),
        "complete" => Some(crate::jobs::types::JobStatus::Complete),
        "failed" => Some(crate::jobs::types::JobStatus::Failed),
        _ => None,
    });

    // If invalid status was provided, return error
    if params.status.is_some() && status_filter.is_none() {
        let error = ErrorResponse {
            error: "invalid_status".to_string(),
            message: format!(
                "Invalid status filter: '{}'. Valid values: pending, in_process, complete, failed",
                params.status.unwrap()
            ),
        };
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!(error))).into_response();
    }

    // Get total count for pagination
    let total_result = repository.count_jobs(status_filter).await;

    match total_result {
        Ok(total) => {
            match repository.list_jobs(status_filter, limit, offset).await {
                Ok(jobs) => {
                    let responses: Vec<JobResponse> = jobs
                        .into_iter()
                        .map(build_job_response)
                        .collect();

                    let response = ListJobsResponse {
                        jobs: responses,
                        total,
                        limit,
                        offset,
                    };

                    (StatusCode::OK, Json(serde_json::json!(response))).into_response()
                }
                Err(e) => {
                    tracing::error!("Failed to list jobs: {}", e);
                    let error = ErrorResponse {
                        error: "query_failed".to_string(),
                        message: format!("Failed to retrieve jobs: {}", e),
                    };
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!(error))).into_response()
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to count jobs: {}", e);
            let error = ErrorResponse {
                error: "query_failed".to_string(),
                message: format!("Failed to count jobs: {}", e),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!(error))).into_response()
        }
    }
}

/// Build a JobResponse from a Job entity
fn build_job_response(job: Job) -> JobResponse {
    let error_response = job.error.map(|err| {
        // Try to extract code and message from error JSON
        let code = err
            .get("code")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown_error")
            .to_string();
        let message = err
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("An unknown error occurred")
            .to_string();
        JobErrorResponse { code, message }
    });

    JobResponse {
        job_id: job.id,
        job_type: job.job_type,
        status: job.status,
        payload: job.payload,
        result: job.result,
        error: error_response,
        created_at: job.created_at.to_rfc3339(),
        updated_at: job.updated_at.to_rfc3339(),
        started_at: job.started_at.map(|t| t.to_rfc3339()),
        completed_at: job.completed_at.map(|t| t.to_rfc3339()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jobs::types::{Job, JobType};

    #[test]
    fn test_load_request_deserialization() {
        let json = r#"{"start_date":"2024-01-01","days":7}"#;
        let req: LoadRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.start_date, "2024-01-01");
        assert_eq!(req.days, 7);
        assert_eq!(req.sync, None);
    }

    #[test]
    fn test_load_request_with_sync() {
        let json = r#"{"start_date":"2024-01-01","days":7,"sync":true}"#;
        let req: LoadRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.sync, Some(true));
    }

    #[test]
    fn test_load_request_with_explicit_async() {
        let json = r#"{"start_date":"2024-01-01","days":7,"sync":false}"#;
        let req: LoadRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.sync, Some(false));
    }

    #[test]
    fn test_load_sync_response_serialization() {
        let response = LoadSyncResponse {
            job_id: Uuid::nil(),
            status: "complete".to_string(),
            result: Some(serde_json::json!({"loaded": 7})),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("complete"));
        assert!(json.contains("loaded"));
    }

    #[test]
    fn test_load_async_response_serialization() {
        let response = LoadAsyncResponse {
            job_id: Uuid::nil(),
            status: "pending".to_string(),
            poll_url: "/api/v1/jobs/00000000-0000-0000-0000-000000000000".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("pending"));
        assert!(json.contains("/api/v1/jobs/"));
    }

    #[test]
    fn test_job_response_building() {
        let job = Job::new(JobType::Load, serde_json::json!({"test": true}));
        let response = build_job_response(job);
        assert_eq!(response.job_type, "load");
        assert_eq!(response.status, "pending");
    }

    #[test]
    fn test_job_error_response_extraction() {
        let mut job = Job::new(JobType::Load, serde_json::json!({}));
        job.error = Some(serde_json::json!({
            "code": "validation_failed",
            "message": "Invalid date range"
        }));

        let response = build_job_response(job);
        assert!(response.error.is_some());
        let err = response.error.unwrap();
        assert_eq!(err.code, "validation_failed");
        assert_eq!(err.message, "Invalid date range");
    }

    #[test]
    fn test_error_response_serialization() {
        let error = ErrorResponse {
            error: "test_error".to_string(),
            message: "Something went wrong".to_string(),
        };
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("test_error"));
        assert!(json.contains("Something went wrong"));
    }

    #[test]
    fn test_list_jobs_request_deserialization() {
        // Test with all parameters
        let json = r#"{"status":"complete","limit":10,"offset":5}"#;
        let req: ListJobsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.status, Some("complete".to_string()));
        assert_eq!(req.limit, 10);
        assert_eq!(req.offset, 5);
    }

    #[test]
    fn test_list_jobs_request_defaults() {
        // Test with empty JSON (should use defaults)
        let json = r#"{}"#;
        let req: ListJobsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.status, None);
        assert_eq!(req.limit, 20);  // default_limit
        assert_eq!(req.offset, 0);   // default_offset
    }

    #[test]
    fn test_list_jobs_request_partial() {
        // Test with only status filter
        let json = r#"{"status":"pending"}"#;
        let req: ListJobsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.status, Some("pending".to_string()));
        assert_eq!(req.limit, 20);  // default
        assert_eq!(req.offset, 0);   // default
    }

    #[test]
    fn test_list_jobs_response_serialization() {
        let job = Job::new(JobType::Load, serde_json::json!({"test": true}));
        let job_response = build_job_response(job);

        let response = ListJobsResponse {
            jobs: vec![job_response],
            total: 1,
            limit: 20,
            offset: 0,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"total\":1"));
        assert!(json.contains("\"limit\":20"));
        assert!(json.contains("\"offset\":0"));
        assert!(json.contains("\"jobs\""));
    }
}
