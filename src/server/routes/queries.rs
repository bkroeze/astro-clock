//! Query API route handlers
//!
//! Provides endpoints for:
//! - POST /api/v1/query/:query_name - Execute named queries (wedding, project, travel)
//!
//! Supports both synchronous (blocking) and asynchronous execution modes.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::jobs::types::JobType;
use crate::server::state::AppState;

/// Request to execute a named query
#[derive(Debug, Clone, Deserialize)]
pub struct QueryRequest {
    /// Start date in YYYY-MM-DD format
    pub start_date: String,
    /// Number of days to query (1-366)
    pub days: i64,
    /// If true, execute synchronously and wait for completion
    /// If false or omitted, execute asynchronously and return job-id immediately
    #[serde(default)]
    pub sync: Option<bool>,
}

/// Response for successful sync query execution
#[derive(Debug, Clone, Serialize)]
pub struct QuerySyncResponse {
    pub job_id: Uuid,
    pub status: String,
    pub result: Option<JsonValue>,
}

/// Response for async query submission
#[derive(Debug, Clone, Serialize)]
pub struct QueryAsyncResponse {
    pub job_id: Uuid,
    pub status: String,
    pub message: String,
}

/// Error response body
#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

/// POST /api/v1/query/:query_name - Execute a named query
///
/// Supported query names: "wedding", "project", "travel"
///
/// In sync mode (sync=true): Blocks until completion, returns full job with result
/// In async mode (sync=false or omitted): Returns immediately with job-id for polling
pub async fn query_handler(
    State(state): State<AppState>,
    Path(query_name): Path<String>,
    Json(request): Json<QueryRequest>,
) -> impl IntoResponse {
    // Validate query name
    let valid_queries = ["wedding", "project", "travel"];
    if !valid_queries.contains(&query_name.as_str()) {
        let error = ErrorResponse {
            error: "unknown_query".to_string(),
            message: format!(
                "Query '{}' not found. Valid queries: wedding, project, travel",
                query_name
            ),
        };
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!(error))).into_response();
    }

    // Validate date format (YYYY-MM-DD)
    if !is_valid_date(&request.start_date) {
        let error = ErrorResponse {
            error: "invalid_date".to_string(),
            message: "start_date must be in YYYY-MM-DD format".to_string(),
        };
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!(error))).into_response();
    }

    // Validate days range (1-366)
    if request.days <= 0 || request.days > 366 {
        let error = ErrorResponse {
            error: "invalid_days".to_string(),
            message: "days must be between 1 and 366".to_string(),
        };
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!(error))).into_response();
    }

    // Build payload for job
    let payload = serde_json::json!({
        "query_name": query_name,
        "start_date": request.start_date,
        "days": request.days,
    });

    let is_sync = request.sync.unwrap_or(false);

    if is_sync {
        // Synchronous execution
        match state.executor().execute_sync(JobType::Query, payload).await {
            Ok(job) => {
                let response = QuerySyncResponse {
                    job_id: job.id,
                    status: job.status,
                    result: job.result,
                };
                (StatusCode::OK, Json(serde_json::json!(response))).into_response()
            }
            Err(e) => {
                tracing::error!("Sync query job failed: {}", e);
                let error = ErrorResponse {
                    error: "execution_failed".to_string(),
                    message: format!("Query execution failed: {}", e),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!(error)))
                    .into_response()
            }
        }
    } else {
        // Asynchronous execution
        match state.executor().execute_async(JobType::Query, payload).await {
            Ok(job_id) => {
                let response = QueryAsyncResponse {
                    job_id,
                    status: "pending".to_string(),
                    message: format!(
                        "Query '{}' started. Poll GET /api/v1/jobs/{} for status",
                        query_name, job_id
                    ),
                };
                (StatusCode::ACCEPTED, Json(serde_json::json!(response))).into_response()
            }
            Err(e) => {
                tracing::error!("Async query job submission failed: {}", e);
                let error = ErrorResponse {
                    error: "job_creation_failed".to_string(),
                    message: format!("Failed to create query job: {}", e),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!(error)))
                    .into_response()
            }
        }
    }
}

/// POST /api/v1/query/wedding - Execute wedding date query
pub async fn wedding_query_handler(
    State(state): State<AppState>,
    Json(request): Json<QueryRequest>,
) -> impl IntoResponse {
    execute_named_query(state, "wedding", request).await
}

/// POST /api/v1/query/project - Execute project start date query
pub async fn project_query_handler(
    State(state): State<AppState>,
    Json(request): Json<QueryRequest>,
) -> impl IntoResponse {
    execute_named_query(state, "project", request).await
}

/// POST /api/v1/query/travel - Execute travel date query
pub async fn travel_query_handler(
    State(state): State<AppState>,
    Json(request): Json<QueryRequest>,
) -> impl IntoResponse {
    execute_named_query(state, "travel", request).await
}

/// Shared query execution logic
async fn execute_named_query(
    state: AppState,
    query_name: &str,
    request: QueryRequest,
) -> Response {
    // Validate date format (YYYY-MM-DD)
    if !is_valid_date(&request.start_date) {
        let error = ErrorResponse {
            error: "invalid_date".to_string(),
            message: "start_date must be in YYYY-MM-DD format".to_string(),
        };
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!(error))).into_response();
    }

    // Validate days range (1-366)
    if request.days <= 0 || request.days > 366 {
        let error = ErrorResponse {
            error: "invalid_days".to_string(),
            message: "days must be between 1 and 366".to_string(),
        };
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!(error))).into_response();
    }

    // Build payload for job
    let payload = serde_json::json!({
        "query_name": query_name,
        "start_date": request.start_date,
        "days": request.days,
    });

    let is_sync = request.sync.unwrap_or(false);

    if is_sync {
        // Synchronous execution
        match state.executor().execute_sync(JobType::Query, payload).await {
            Ok(job) => {
                let response = QuerySyncResponse {
                    job_id: job.id,
                    status: job.status,
                    result: job.result,
                };
                (StatusCode::OK, Json(serde_json::json!(response))).into_response()
            }
            Err(e) => {
                tracing::error!("Sync {} query job failed: {}", query_name, e);
                let error = ErrorResponse {
                    error: "execution_failed".to_string(),
                    message: format!("Query execution failed: {}", e),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!(error)))
                    .into_response()
            }
        }
    } else {
        // Asynchronous execution
        match state.executor().execute_async(JobType::Query, payload).await {
            Ok(job_id) => {
                let response = QueryAsyncResponse {
                    job_id,
                    status: "pending".to_string(),
                    message: format!(
                        "Query '{}' started. Poll GET /api/v1/jobs/{} for status",
                        query_name, job_id
                    ),
                };
                (StatusCode::ACCEPTED, Json(serde_json::json!(response))).into_response()
            }
            Err(e) => {
                tracing::error!("Async {} query job submission failed: {}", query_name, e);
                let error = ErrorResponse {
                    error: "job_creation_failed".to_string(),
                    message: format!("Failed to create query job: {}", e),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!(error)))
                    .into_response()
            }
        }
    }
}

/// Validate date string is in YYYY-MM-DD format
fn is_valid_date(date_str: &str) -> bool {
    chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_request_deserialization() {
        let json = r#"{"start_date":"2024-06-01","days":30,"sync":true}"#;
        let request: QueryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.start_date, "2024-06-01");
        assert_eq!(request.days, 30);
        assert_eq!(request.sync, Some(true));
    }

    #[test]
    fn test_query_request_deserialization_no_sync() {
        let json = r#"{"start_date":"2024-06-01","days":30}"#;
        let request: QueryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.start_date, "2024-06-01");
        assert_eq!(request.days, 30);
        assert_eq!(request.sync, None);
    }

    #[test]
    fn test_query_sync_response_serialization() {
        let response = QuerySyncResponse {
            job_id: Uuid::new_v4(),
            status: "complete".to_string(),
            result: Some(serde_json::json!({"candidates": []})),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("complete"));
        assert!(json.contains("candidates"));
    }

    #[test]
    fn test_query_async_response_serialization() {
        let response = QueryAsyncResponse {
            job_id: Uuid::nil(),
            status: "pending".to_string(),
            message: "Query 'wedding' started".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("pending"));
        assert!(json.contains("wedding"));
    }

    #[test]
    fn test_error_response_serialization() {
        let error = ErrorResponse {
            error: "unknown_query".to_string(),
            message: "Query 'invalid' not found".to_string(),
        };
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("unknown_query"));
        assert!(json.contains("invalid"));
    }

    #[test]
    fn test_is_valid_date_valid() {
        assert!(is_valid_date("2024-06-01"));
        assert!(is_valid_date("2024-12-31"));
        assert!(is_valid_date("2024-02-29")); // Leap year
    }

    #[test]
    fn test_is_valid_date_invalid() {
        assert!(!is_valid_date("2024-13-01")); // Invalid month
        assert!(!is_valid_date("2024-06-32")); // Invalid day
        assert!(!is_valid_date("06-01-2024")); // Wrong format
        assert!(!is_valid_date("invalid"));
        assert!(!is_valid_date(""));
    }
}
