//! Job API route handlers
//!
//! Provides endpoints for:
//! - POST /api/v1/load - Create and execute load jobs (sync or async)
//! - GET /api/v1/jobs/:id - Get job status and details

#![cfg(feature = "db")]

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use std::str::FromStr;

use crate::jobs::repository::{JobListFilters, JobRepository, CursorDirection};
use crate::jobs::types::{Job, JobStatus, JobType};
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
    /// Filter by status — comma-separated for multi-value (e.g. "complete,failed")
    pub status: Option<String>,
    /// Filter by job type — comma-separated for multi-value (e.g. "load,query")
    pub job_type: Option<String>,
    /// Filter jobs created on or after this timestamp (RFC3339 or YYYY-MM-DD)
    pub created_after: Option<String>,
    /// Filter jobs created on or before this timestamp (RFC3339 or YYYY-MM-DD)
    pub created_before: Option<String>,
    /// Maximum number of jobs to return (default: 20, max: 100)
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// Offset for pagination (default: 0)
    #[serde(default = "default_offset")]
    pub offset: i64,
}

fn default_limit() -> i64 { 20 }
fn default_offset() -> i64 { 0 }

/// Parse a date string as a UTC DateTime.
/// Tries RFC3339 first, then falls back to YYYY-MM-DD (interpreted as start-of-day UTC).
fn parse_date_param(s: &str) -> Result<DateTime<Utc>, String> {
    // Try RFC3339 first (e.g. "2025-01-15T10:30:00Z")
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.to_utc());
    }
    // Fall back to YYYY-MM-DD (start of day UTC)
    if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return date
            .and_hms_opt(0, 0, 0)
            .map(|nd| DateTime::<Utc>::from_naive_utc_and_offset(nd, Utc))
            .ok_or_else(|| format!("Invalid date: '{}'", s));
    }
    Err(format!(
        "Invalid date format: '{}'. Expected RFC3339 (2025-01-15T10:30:00Z) or YYYY-MM-DD",
        s
    ))
}

/// Parse a comma-separated string into a Vec<T> using FromStr.
/// Returns the first error encountered, if any.
fn parse_comma_separated<T: FromStr>(input: &str, label: &str) -> Result<Vec<T>, String> {
    let mut results = Vec::new();
    for part in input.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        match trimmed.parse::<T>() {
            Ok(v) => results.push(v),
            Err(_) => {
                return Err(format!(
                    "Invalid {} value: '{}'. Example valid values: {}",
                    label,
                    trimmed,
                    // Show the label contextually — callers provide meaningful labels
                    input
                ));
            }
        }
    }
    Ok(results)
}

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

/// GET /api/v1/jobs - List recent jobs with pagination and filtering
///
/// Query parameters:
/// - status: Comma-separated status filter (e.g. "complete,failed")
/// - job_type: Comma-separated job type filter (e.g. "load,query")
/// - created_after: Filter by created_at >= timestamp (RFC3339 or YYYY-MM-DD)
/// - created_before: Filter by created_at <= timestamp (RFC3339 or YYYY-MM-DD)
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

    // Parse comma-separated status filter
    let statuses: Vec<JobStatus> = match params.status.as_deref() {
        Some(s) => {
            let parsed = parse_comma_separated::<JobStatus>(s, "status");
            match parsed {
                Ok(v) => v,
                Err(msg) => {
                    let error = ErrorResponse {
                        error: "invalid_status".to_string(),
                        message: format!(
                            "{}. Valid values: pending, in_process, complete, failed",
                            msg
                        ),
                    };
                    return (StatusCode::BAD_REQUEST, Json(serde_json::json!(error)))
                        .into_response();
                }
            }
        }
        None => Vec::new(),
    };

    // Parse comma-separated job_type filter
    let job_types: Vec<JobType> = match params.job_type.as_deref() {
        Some(s) => {
            let parsed = parse_comma_separated::<JobType>(s, "job_type");
            match parsed {
                Ok(v) => v,
                Err(msg) => {
                    let error = ErrorResponse {
                        error: "invalid_job_type".to_string(),
                        message: format!(
                            "{}. Valid values: load, query",
                            msg
                        ),
                    };
                    return (StatusCode::BAD_REQUEST, Json(serde_json::json!(error)))
                        .into_response();
                }
            }
        }
        None => Vec::new(),
    };

    // Parse created_after date
    let created_after = match params.created_after.as_deref() {
        Some(s) => {
            match parse_date_param(s) {
                Ok(dt) => Some(dt),
                Err(msg) => {
                    let error = ErrorResponse {
                        error: "invalid_date".to_string(),
                        message: format!("Invalid created_after: {}", msg),
                    };
                    return (StatusCode::BAD_REQUEST, Json(serde_json::json!(error)))
                        .into_response();
                }
            }
        }
        None => None,
    };

    // Parse created_before date
    let created_before = match params.created_before.as_deref() {
        Some(s) => {
            match parse_date_param(s) {
                Ok(dt) => Some(dt),
                Err(msg) => {
                    let error = ErrorResponse {
                        error: "invalid_date".to_string(),
                        message: format!("Invalid created_before: {}", msg),
                    };
                    return (StatusCode::BAD_REQUEST, Json(serde_json::json!(error)))
                        .into_response();
                }
            }
        }
        None => None,
    };

    let filters = JobListFilters {
        status: statuses,
        job_type: job_types,
        created_after,
        created_before,
    };

    // Get total count for pagination
    let total_result = repository.count_jobs(filters.clone()).await;

    match total_result {
        Ok(total) => {
            match repository.list_jobs(filters, limit, None, CursorDirection::Forward).await {
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
    use chrono::{Datelike, Timelike};

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
        assert_eq!(req.job_type, None);
        assert_eq!(req.created_after, None);
        assert_eq!(req.created_before, None);
        assert_eq!(req.limit, 10);
        assert_eq!(req.offset, 5);
    }

    #[test]
    fn test_list_jobs_request_defaults() {
        // Test with empty JSON (should use defaults)
        let json = r#"{}"#;
        let req: ListJobsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.status, None);
        assert_eq!(req.job_type, None);
        assert_eq!(req.created_after, None);
        assert_eq!(req.created_before, None);
        assert_eq!(req.limit, 20);  // default_limit
        assert_eq!(req.offset, 0);   // default_offset
    }

    #[test]
    fn test_list_jobs_request_partial() {
        // Test with only status filter
        let json = r#"{"status":"pending"}"#;
        let req: ListJobsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.status, Some("pending".to_string()));
        assert_eq!(req.job_type, None);
        assert_eq!(req.limit, 20);  // default
        assert_eq!(req.offset, 0);   // default
    }

    #[test]
    fn test_list_jobs_request_all_new_fields() {
        let json = r#"{"status":"complete,failed","job_type":"load,query","created_after":"2025-01-01","created_before":"2025-03-01T23:59:59Z","limit":50,"offset":10}"#;
        let req: ListJobsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.status, Some("complete,failed".to_string()));
        assert_eq!(req.job_type, Some("load,query".to_string()));
        assert_eq!(req.created_after, Some("2025-01-01".to_string()));
        assert_eq!(req.created_before, Some("2025-03-01T23:59:59Z".to_string()));
        assert_eq!(req.limit, 50);
        assert_eq!(req.offset, 10);
    }

    // ── Multi-value comma-separated parsing tests ──

    #[test]
    fn test_parse_comma_separated_single_value() {
        let result = parse_comma_separated::<JobStatus>("complete", "status").unwrap();
        assert_eq!(result, vec![JobStatus::Complete]);
    }

    #[test]
    fn test_parse_comma_separated_multiple_values() {
        let result = parse_comma_separated::<JobStatus>("complete,failed", "status").unwrap();
        assert_eq!(result, vec![JobStatus::Complete, JobStatus::Failed]);
    }

    #[test]
    fn test_parse_comma_separated_all_statuses() {
        let result = parse_comma_separated::<JobStatus>("pending,in_process,complete,failed", "status").unwrap();
        assert_eq!(result, vec![JobStatus::Pending, JobStatus::InProcess, JobStatus::Complete, JobStatus::Failed]);
    }

    #[test]
    fn test_parse_comma_separated_job_types() {
        let result = parse_comma_separated::<JobType>("load,query", "job_type").unwrap();
        assert_eq!(result, vec![JobType::Load, JobType::Query]);
    }

    #[test]
    fn test_parse_comma_separated_invalid_status() {
        let result = parse_comma_separated::<JobStatus>("complete,bogus", "status");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("bogus"));
        assert!(err.contains("status"));
    }

    #[test]
    fn test_parse_comma_separated_invalid_job_type() {
        let result = parse_comma_separated::<JobType>("load,invalid", "job_type");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("invalid"));
        assert!(err.contains("job_type"));
    }

    #[test]
    fn test_parse_comma_separated_empty_string() {
        let result = parse_comma_separated::<JobStatus>("", "status").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_comma_separated_whitespace_handling() {
        let result = parse_comma_separated::<JobStatus>(" complete , failed ", "status").unwrap();
        assert_eq!(result, vec![JobStatus::Complete, JobStatus::Failed]);
    }

    // ── Date parsing tests ──

    #[test]
    fn test_parse_date_param_yyyy_mm_dd() {
        let dt = parse_date_param("2025-01-15").unwrap();
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 15);
        assert_eq!(dt.hour(), 0);
        assert_eq!(dt.minute(), 0);
        assert_eq!(dt.second(), 0);
    }

    #[test]
    fn test_parse_date_param_rfc3339() {
        let dt = parse_date_param("2025-01-15T10:30:00Z").unwrap();
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 15);
        assert_eq!(dt.hour(), 10);
        assert_eq!(dt.minute(), 30);
    }

    #[test]
    fn test_parse_date_param_rfc3339_with_offset() {
        let dt = parse_date_param("2025-01-15T10:30:00+05:00").unwrap();
        // Should convert to UTC: 10:30 +05:00 = 05:30 UTC
        assert_eq!(dt.hour(), 5);
        assert_eq!(dt.minute(), 30);
    }

    #[test]
    fn test_parse_date_param_invalid() {
        let result = parse_date_param("not-a-date");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not-a-date"));
    }

    #[test]
    fn test_parse_date_param_partial_date() {
        let result = parse_date_param("2025-13-01");
        assert!(result.is_err());
    }

    // ── Backward compatibility tests ──

    #[test]
    fn test_single_status_still_works() {
        // Single status values should parse to a Vec with one element
        let result = parse_comma_separated::<JobStatus>("pending", "status").unwrap();
        assert_eq!(result, vec![JobStatus::Pending]);

        let result = parse_comma_separated::<JobStatus>("in_process", "status").unwrap();
        assert_eq!(result, vec![JobStatus::InProcess]);

        let result = parse_comma_separated::<JobStatus>("complete", "status").unwrap();
        assert_eq!(result, vec![JobStatus::Complete]);

        let result = parse_comma_separated::<JobStatus>("failed", "status").unwrap();
        assert_eq!(result, vec![JobStatus::Failed]);
    }

    #[test]
    fn test_backward_compat_request_with_single_status() {
        // Existing clients sending ?status=complete should still work
        let json = r#"{"status":"complete","limit":20,"offset":0}"#;
        let req: ListJobsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.status, Some("complete".to_string()));
        // The handler will parse "complete" -> vec![JobStatus::Complete]
        let parsed = parse_comma_separated::<JobStatus>("complete", "status").unwrap();
        assert_eq!(parsed, vec![JobStatus::Complete]);
    }

    // ── Combined filter tests ──

    #[test]
    fn test_combined_status_and_job_type() {
        let statuses = parse_comma_separated::<JobStatus>("complete,failed", "status").unwrap();
        let job_types = parse_comma_separated::<JobType>("load", "job_type").unwrap();
        let after = parse_date_param("2025-01-01").unwrap();
        let before = parse_date_param("2025-03-01").unwrap();

        let filters = JobListFilters {
            status: statuses,
            job_type: job_types,
            created_after: Some(after),
            created_before: Some(before),
        };

        assert_eq!(filters.status.len(), 2);
        assert_eq!(filters.job_type.len(), 1);
        assert!(filters.created_after.is_some());
        assert!(filters.created_before.is_some());
    }

    #[test]
    fn test_filters_all_empty() {
        let filters = JobListFilters::default();
        assert!(filters.status.is_empty());
        assert!(filters.job_type.is_empty());
        assert!(filters.created_after.is_none());
        assert!(filters.created_before.is_none());
    }

    #[test]
    fn test_filters_status_only() {
        let statuses = parse_comma_separated::<JobStatus>("pending", "status").unwrap();
        let filters = JobListFilters {
            status: statuses,
            ..Default::default()
        };
        assert_eq!(filters.status.len(), 1);
        assert!(filters.job_type.is_empty());
    }

    #[test]
    fn test_filters_date_range_only() {
        let after = parse_date_param("2025-01-01").unwrap();
        let before = parse_date_param("2025-12-31").unwrap();
        let filters = JobListFilters {
            created_after: Some(after),
            created_before: Some(before),
            ..Default::default()
        };
        assert!(filters.status.is_empty());
        assert!(filters.job_type.is_empty());
        assert!(filters.created_after.is_some());
        assert!(filters.created_before.is_some());
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

    #[test]
    fn test_list_jobs_handler_response_format() {
        // This test verifies the response structure matches API-06 requirements
        let job = Job::new(
            JobType::Query,
            serde_json::json!({
                "query_name": "wedding",
                "start_date": "2024-06-01",
                "days": 30
            }),
        );

        let response = build_job_response(job);

        // Verify all API-06 required fields are present
        assert!(!response.job_id.to_string().is_empty());
        assert!(!response.job_type.is_empty());
        assert!(!response.status.is_empty());
        assert!(!response.created_at.is_empty());
        assert!(!response.updated_at.is_empty());
        // started_at and completed_at are optional
        // payload, result, error are optional

        // Serialize and verify JSON structure
        let json = serde_json::to_value(&response).unwrap();
        assert!(json.get("job_id").is_some());
        assert!(json.get("job_type").is_some());
        assert!(json.get("status").is_some());
        assert!(json.get("created_at").is_some());
        assert!(json.get("updated_at").is_some());
    }

    #[test]
    fn test_list_jobs_response_includes_pagination() {
        let job1 = Job::new(JobType::Load, serde_json::json!({"test": 1}));
        let job2 = Job::new(JobType::Query, serde_json::json!({"test": 2}));

        let response = ListJobsResponse {
            jobs: vec![build_job_response(job1), build_job_response(job2)],
            total: 2,
            limit: 20,
            offset: 0,
        };

        let json = serde_json::to_value(&response).unwrap();
        assert!(json.get("jobs").is_some());
        assert!(json.get("total").is_some());
        assert!(json.get("limit").is_some());
        assert!(json.get("offset").is_some());

        let jobs = json.get("jobs").unwrap().as_array().unwrap();
        assert_eq!(jobs.len(), 2);
    }

    #[test]
    fn test_job_response_with_optional_fields() {
        let mut job = Job::new(JobType::Load, serde_json::json!({"test": true}));

        // Set optional fields
        job.started_at = Some(chrono::Utc::now());
        job.completed_at = Some(chrono::Utc::now());
        job.result = Some(serde_json::json!({"loaded": 7}));

        let response = build_job_response(job);

        // Verify required fields
        assert!(!response.job_id.to_string().is_empty());
        assert_eq!(response.job_type, "load");
        assert_eq!(response.status, "pending");

        // Verify optional fields are present
        assert!(response.started_at.is_some());
        assert!(response.completed_at.is_some());
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_job_response_with_error() {
        let mut job = Job::new(JobType::Query, serde_json::json!({}));
        job.error = Some(serde_json::json!({
            "code": "validation_failed",
            "message": "Invalid query parameters"
        }));

        let response = build_job_response(job);

        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, "validation_failed");
        assert_eq!(error.message, "Invalid query parameters");
    }
}
