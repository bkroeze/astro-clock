//! Job API route handlers
//!
//! Provides endpoints for:
//! - POST /api/v1/load - Create and execute load jobs (sync or async)
//! - GET /api/v1/jobs/:id - Get job status and details

#![cfg(feature = "db")]

use axum::extract::{OriginalUri, Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use std::str::FromStr;

use crate::jobs::repository::{CursorDirection, JobCursor, JobListFilters, JobRepository};
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
    #[serde(default = "default_count")]
    pub count: i64,
    /// Opaque pagination cursor from a previous response's next/prev URL
    #[serde(default)]
    pub cursor: Option<String>,
}

fn default_count() -> i64 { 20 }

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
    /// Full URL for the next page of results, or None if this is the last page
    pub next: Option<String>,
    /// Full URL for the previous page of results, or None if this is the first page
    pub prev: Option<String>,
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

/// GET /api/v1/jobs - List recent jobs with cursor-based pagination and filtering
///
/// Query parameters:
/// - status: Comma-separated status filter (e.g. "complete,failed")
/// - job_type: Comma-separated job type filter (e.g. "load,query")
/// - created_after: Filter by created_at >= timestamp (RFC3339 or YYYY-MM-DD)
/// - created_before: Filter by created_at <= timestamp (RFC3339 or YYYY-MM-DD)
/// - count: Max jobs to return (default: 20, max: 100)
/// - cursor: Opaque pagination cursor from a previous response's next/prev URL
pub async fn list_jobs_handler(
    State(state): State<AppState>,
    OriginalUri(original_uri): OriginalUri,
    Query(params): Query<ListJobsRequest>,
) -> impl IntoResponse {
    let repository = JobRepository::new(state.get_pool());

    // Validate and cap count
    let count = params.count.max(1).min(100);

    // Determine base path from OriginalUri (strip query string)
    let base_path = original_uri.path().to_string();

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

    // Decode cursor if present
    let (cursor, direction) = match params.cursor.as_deref() {
        Some(cursor_str) => {
            match JobCursor::decode(cursor_str) {
                Ok(cur) => (Some(cur), CursorDirection::Forward),
                Err(msg) => {
                    let error = ErrorResponse {
                        error: "invalid_cursor".to_string(),
                        message: format!("Invalid cursor: {}", msg),
                    };
                    return (StatusCode::BAD_REQUEST, Json(serde_json::json!(error)))
                        .into_response();
                }
            }
        }
        None => (None, CursorDirection::Forward),
    };

    let filters = JobListFilters {
        status: statuses,
        job_type: job_types,
        created_after,
        created_before,
    };

    // Fetch count+1 rows to detect next page
    match repository.list_jobs(filters.clone(), count, cursor.clone(), direction).await {
        Ok(mut jobs) => {
            // Determine has_next: we fetched count+1 rows
            let has_next = jobs.len() > count as usize;
            if has_next {
                jobs.truncate(count as usize);
            }

            // Determine has_prev: if we had a cursor and got results, there's a previous page
            let has_prev = cursor.is_some() && !jobs.is_empty();

            // First page (no cursor): prev is always None
            let prev = if cursor.is_none() {
                None
            } else if has_prev {
                // Use first job for prev cursor
                let first = &jobs[0];
                let prev_cursor = JobCursor {
                    created_at: first.created_at,
                    id: first.id,
                };
                Some(build_page_url(
                    &base_path,
                    count,
                    &prev_cursor,
                    params.status.as_deref(),
                    params.job_type.as_deref(),
                    params.created_after.as_deref(),
                    params.created_before.as_deref(),
                ))
            } else {
                None
            };

            // Build next URL from last job's cursor
            let next = if has_next {
                let last = &jobs[jobs.len() - 1];
                let next_cursor = JobCursor {
                    created_at: last.created_at,
                    id: last.id,
                };
                Some(build_page_url(
                    &base_path,
                    count,
                    &next_cursor,
                    params.status.as_deref(),
                    params.job_type.as_deref(),
                    params.created_after.as_deref(),
                    params.created_before.as_deref(),
                ))
            } else {
                None
            };

            let responses: Vec<JobResponse> = jobs
                .into_iter()
                .map(build_job_response)
                .collect();

            let response = ListJobsResponse {
                jobs: responses,
                next,
                prev,
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

/// Build a pagination URL that preserves all active filter parameters.
///
/// The URL includes `count`, the encoded `cursor`, and any non-empty filter
/// values so that paginating through a filtered result set stays filtered.
fn build_page_url(
    base_path: &str,
    count: i64,
    cursor: &JobCursor,
    status: Option<&str>,
    job_type: Option<&str>,
    created_after: Option<&str>,
    created_before: Option<&str>,
) -> String {
    let mut params: Vec<String> = Vec::new();
    params.push(format!("count={}", count));
    params.push(format!("cursor={}", cursor.encode()));
    if let Some(s) = status {
        params.push(format!("status={}", s));
    }
    if let Some(jt) = job_type {
        params.push(format!("job_type={}", jt));
    }
    if let Some(ca) = created_after {
        params.push(format!("created_after={}", ca));
    }
    if let Some(cb) = created_before {
        params.push(format!("created_before={}", cb));
    }
    format!("{}?{}", base_path, params.join("&"))
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
    use base64::Engine;
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
        let json = r#"{"status":"complete","count":10,"cursor":"abc123"}"#;
        let req: ListJobsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.status, Some("complete".to_string()));
        assert_eq!(req.job_type, None);
        assert_eq!(req.created_after, None);
        assert_eq!(req.created_before, None);
        assert_eq!(req.count, 10);
        assert_eq!(req.cursor, Some("abc123".to_string()));
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
        assert_eq!(req.count, 20);   // default_count
        assert_eq!(req.cursor, None);
    }

    #[test]
    fn test_list_jobs_request_partial() {
        // Test with only status filter
        let json = r#"{"status":"pending"}"#;
        let req: ListJobsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.status, Some("pending".to_string()));
        assert_eq!(req.job_type, None);
        assert_eq!(req.count, 20);   // default
        assert_eq!(req.cursor, None);
    }

    #[test]
    fn test_list_jobs_request_all_new_fields() {
        let json = r#"{"status":"complete,failed","job_type":"load,query","created_after":"2025-01-01","created_before":"2025-03-01T23:59:59Z","count":50,"cursor":"eyJjcmVhdGVkX2F0IjoiMjAyNS0wMS0wMVQwMDowMDowMFoiLCJpZCI6IjU1MGU4NDAtZTI5Yi00MWQ0LWE3MTYtNDQ2NjU1NDQwMDAwIn0"}"#;
        let req: ListJobsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.status, Some("complete,failed".to_string()));
        assert_eq!(req.job_type, Some("load,query".to_string()));
        assert_eq!(req.created_after, Some("2025-01-01".to_string()));
        assert_eq!(req.created_before, Some("2025-03-01T23:59:59Z".to_string()));
        assert_eq!(req.count, 50);
        assert!(req.cursor.is_some());
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
        let json = r#"{"status":"complete","count":20}"#;
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
            next: None,
            prev: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"next\":null"));
        assert!(json.contains("\"prev\":null"));
        assert!(json.contains("\"jobs\""));
        // Ensure old fields are NOT present
        assert!(!json.contains("\"total\""));
        assert!(!json.contains("\"limit\""));
        assert!(!json.contains("\"offset\""));
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

        let cursor = JobCursor {
            created_at: chrono::Utc::now(),
            id: Uuid::new_v4(),
        };

        let response = ListJobsResponse {
            jobs: vec![build_job_response(job1), build_job_response(job2)],
            next: Some(build_page_url(
                "/api/v1/jobs",
                20,
                &cursor,
                None,
                None,
                None,
                None,
            )),
            prev: None,
        };

        let json = serde_json::to_value(&response).unwrap();
        assert!(json.get("jobs").is_some());
        assert!(json.get("next").is_some());
        assert!(json.get("prev").is_some());

        let jobs = json.get("jobs").unwrap().as_array().unwrap();
        assert_eq!(jobs.len(), 2);

        // next should be a URL with count and cursor
        let next_url = json.get("next").unwrap().as_str().unwrap();
        assert!(next_url.contains("count=20"));
        assert!(next_url.contains("cursor="));
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

    // ── Cursor handler-layer tests ──

    #[test]
    fn test_list_jobs_request_with_count_and_cursor() {
        let json = r#"{"count":5,"cursor":"eyJjcmVhdGVkX2F0IjoiMjAyNS0wNi0xNVQxMDozMDowMFoiLCJpZCI6IjU1MGU4NDAtZTI5Yi00MWQ0LWE3MTYtNDQ2NjU1NDQwMDAwIn0"}"#;
        let req: ListJobsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.count, 5);
        assert!(req.cursor.is_some());
        assert_eq!(req.status, None);
    }

    #[test]
    fn test_cursor_decode_in_handler() {
        // Invalid base64 should fail with descriptive error
        let bad_cursor = "!!!invalid!!!";
        let result = JobCursor::decode(bad_cursor);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("Invalid cursor encoding"),
            "Error should mention encoding, got: {}",
            err
        );

        // Valid base64 of non-JSON should fail with data error
        let bad_json_cursor = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode("not json at all");
        let result = JobCursor::decode(&bad_json_cursor);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("Invalid cursor data"),
            "Error should mention data, got: {}",
            err
        );
    }

    #[test]
    fn test_build_page_url_with_filters() {
        let cursor = JobCursor {
            created_at: chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 2025, 6, 15, 10, 30, 0).unwrap(),
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        };

        let url = build_page_url(
            "/api/v1/jobs",
            10,
            &cursor,
            Some("complete,failed"),
            Some("load"),
            Some("2025-01-01"),
            Some("2025-06-30"),
        );

        assert!(url.starts_with("/api/v1/jobs?"));
        assert!(url.contains("count=10"));
        assert!(url.contains("cursor="));
        assert!(url.contains("status=complete,failed"));
        assert!(url.contains("job_type=load"));
        assert!(url.contains("created_after=2025-01-01"));
        assert!(url.contains("created_before=2025-06-30"));
    }

    #[test]
    fn test_build_page_url_without_filters() {
        let cursor = JobCursor {
            created_at: chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 2025, 6, 15, 10, 30, 0).unwrap(),
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        };

        let url = build_page_url(
            "/api/v1/jobs",
            20,
            &cursor,
            None,
            None,
            None,
            None,
        );

        assert!(url.starts_with("/api/v1/jobs?"));
        assert!(url.contains("count=20"));
        assert!(url.contains("cursor="));
        // Should NOT contain filter params
        assert!(!url.contains("status="));
        assert!(!url.contains("job_type="));
        assert!(!url.contains("created_after="));
        assert!(!url.contains("created_before="));
    }

    #[test]
    fn test_first_page_no_prev() {
        // When no cursor is provided (first page), prev should always be None.
        // This is a structural test — we verify the response shape directly.
        let response = ListJobsResponse {
            jobs: vec![],
            next: None,
            prev: None,  // No cursor → no prev
        };

        let json = serde_json::to_value(&response).unwrap();
        assert!(json.get("prev").unwrap().is_null());
        assert!(json.get("next").unwrap().is_null());
    }

    #[test]
    fn test_build_page_url_preserves_partial_filters() {
        let cursor = JobCursor {
            created_at: chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 2025, 1, 1, 0, 0, 0).unwrap(),
            id: Uuid::nil(),
        };

        // Only status filter, no others
        let url = build_page_url(
            "/api/v1/jobs",
            5,
            &cursor,
            Some("pending"),
            None,
            None,
            None,
        );

        assert!(url.contains("status=pending"));
        assert!(!url.contains("job_type="));
        assert!(!url.contains("created_after="));
    }

    #[test]
    fn test_cursor_url_is_valid() {
        // Verify the cursor in the URL doesn't contain characters that need escaping
        let cursor = JobCursor {
            created_at: chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 2025, 6, 15, 10, 30, 0).unwrap(),
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        };

        let url = build_page_url(
            "/api/v1/jobs",
            20,
            &cursor,
            None,
            None,
            None,
            None,
        );

        // Extract cursor value from URL
        let cursor_part = url.split("cursor=").nth(1).unwrap();
        // Cursor should not contain URL-unsafe characters (no = padding, no +, no /)
        assert!(!cursor_part.contains('='));
        assert!(!cursor_part.contains('+'));
    }

    // ── Comprehensive cursor-specific tests (T03) ──

    #[test]
    fn test_cursor_roundtrip_various_timestamps() {
        // Past timestamp
        let cursor_past = JobCursor {
            created_at: chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 2020, 1, 1, 0, 0, 0).unwrap(),
            id: Uuid::nil(),
        };
        let encoded = cursor_past.encode();
        let decoded = JobCursor::decode(&encoded).unwrap();
        assert_eq!(decoded.created_at, cursor_past.created_at);
        assert_eq!(decoded.id, cursor_past.id);

        // Future timestamp
        let cursor_future = JobCursor {
            created_at: chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 2099, 12, 31, 23, 59, 59).unwrap(),
            id: Uuid::new_v4(),
        };
        let encoded = cursor_future.encode();
        let decoded = JobCursor::decode(&encoded).unwrap();
        assert_eq!(decoded.created_at, cursor_future.created_at);
        assert_eq!(decoded.id, cursor_future.id);

        // Edge case: leap second boundary (Feb 29)
        let cursor_leap = JobCursor {
            created_at: chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 2024, 2, 29, 12, 0, 0).unwrap(),
            id: Uuid::parse_str("ffffffff-ffff-ffff-ffff-ffffffffffff").unwrap(),
        };
        let encoded = cursor_leap.encode();
        let decoded = JobCursor::decode(&encoded).unwrap();
        assert_eq!(decoded.created_at, cursor_leap.created_at);
        assert_eq!(decoded.id, cursor_leap.id);

        // Edge case: Unix epoch
        let cursor_epoch = JobCursor {
            created_at: chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 1970, 1, 1, 0, 0, 0).unwrap(),
            id: Uuid::nil(),
        };
        let encoded = cursor_epoch.encode();
        let decoded = JobCursor::decode(&encoded).unwrap();
        assert_eq!(decoded.created_at, cursor_epoch.created_at);
    }

    #[test]
    fn test_build_page_url_preserves_status_filter() {
        let cursor = JobCursor {
            created_at: chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 2025, 6, 15, 10, 30, 0).unwrap(),
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        };

        let url = build_page_url(
            "/api/v1/jobs",
            20,
            &cursor,
            Some("complete,failed"),
            None,
            None,
            None,
        );

        assert!(
            url.contains("status=complete,failed"),
            "URL should preserve multi-value status filter, got: {}",
            url
        );
        assert!(url.contains("count=20"));
        assert!(url.contains("cursor="));
        // No other filters present
        assert!(!url.contains("job_type="));
        assert!(!url.contains("created_after="));
        assert!(!url.contains("created_before="));
    }

    #[test]
    fn test_build_page_url_preserves_job_type_filter() {
        let cursor = JobCursor {
            created_at: chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 2025, 6, 15, 10, 30, 0).unwrap(),
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        };

        let url = build_page_url(
            "/api/v1/jobs",
            50,
            &cursor,
            None,
            Some("load"),
            None,
            None,
        );

        assert!(
            url.contains("job_type=load"),
            "URL should preserve job_type filter, got: {}",
            url
        );
        assert!(url.contains("count=50"));
        // No other filters present
        assert!(!url.contains("status="));
        assert!(!url.contains("created_after="));
        assert!(!url.contains("created_before="));
    }

    #[test]
    fn test_build_page_url_preserves_date_filters() {
        let cursor = JobCursor {
            created_at: chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 2025, 6, 15, 10, 30, 0).unwrap(),
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        };

        let url = build_page_url(
            "/api/v1/jobs",
            10,
            &cursor,
            None,
            None,
            Some("2025-01-01"),
            Some("2025-12-31T23:59:59Z"),
        );

        assert!(
            url.contains("created_after=2025-01-01"),
            "URL should preserve created_after filter, got: {}",
            url
        );
        assert!(
            url.contains("created_before=2025-12-31T23:59:59Z"),
            "URL should preserve created_before filter, got: {}",
            url
        );
        // No other filters present
        assert!(!url.contains("status="));
        assert!(!url.contains("job_type="));
    }

    #[test]
    fn test_build_page_url_minimal() {
        // Minimal URL: only count and cursor, no filters at all
        let cursor = JobCursor {
            created_at: chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 2025, 6, 15, 10, 30, 0).unwrap(),
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        };

        let url = build_page_url(
            "/api/v1/jobs",
            5,
            &cursor,
            None,
            None,
            None,
            None,
        );

        // Should start with base path
        assert!(url.starts_with("/api/v1/jobs?"));
        // Must have count and cursor
        assert!(url.contains("count=5"));
        assert!(url.contains("cursor="));
        // Must NOT have any filter params
        assert!(!url.contains("status="));
        assert!(!url.contains("job_type="));
        assert!(!url.contains("created_after="));
        assert!(!url.contains("created_before="));
        // Verify the URL has exactly 2 query params
        let query_part = url.split('?').nth(1).unwrap();
        let params: Vec<&str> = query_part.split('&').collect();
        assert_eq!(params.len(), 2, "Minimal URL should have exactly 2 params (count, cursor), got: {:?}", params);
    }

    #[test]
    fn test_empty_results_no_next_prev() {
        // When 0 results returned, both next and prev should be None
        let response = ListJobsResponse {
            jobs: vec![],
            next: None,
            prev: None,
        };

        let json = serde_json::to_value(&response).unwrap();
        assert!(
            json.get("next").unwrap().is_null(),
            "Empty results should have next=null"
        );
        assert!(
            json.get("prev").unwrap().is_null(),
            "Empty results should have prev=null"
        );
        assert!(
            json.get("jobs").unwrap().as_array().unwrap().is_empty(),
            "Empty results should have empty jobs array"
        );
        // Ensure response doesn't leak old pagination fields
        assert!(json.get("total").is_none());
        assert!(json.get("limit").is_none());
        assert!(json.get("offset").is_none());
    }

    #[test]
    fn test_single_page_no_next() {
        // When results fit in one page (no extra row fetched), next should be None
        let job1 = Job::new(JobType::Load, serde_json::json!({"test": 1}));
        let job2 = Job::new(JobType::Query, serde_json::json!({"test": 2}));

        let response = ListJobsResponse {
            jobs: vec![build_job_response(job1), build_job_response(job2)],
            next: None,  // Fits in one page
            prev: None,  // First page
        };

        let json = serde_json::to_value(&response).unwrap();
        assert!(
            json.get("next").unwrap().is_null(),
            "Single page should have next=null"
        );
        assert!(
            json.get("prev").unwrap().is_null(),
            "First page should have prev=null"
        );
        let jobs = json.get("jobs").unwrap().as_array().unwrap();
        assert_eq!(jobs.len(), 2, "Should have exactly 2 jobs");
        // Ensure old fields are absent
        assert!(json.get("total").is_none());
        assert!(json.get("offset").is_none());
    }
}
