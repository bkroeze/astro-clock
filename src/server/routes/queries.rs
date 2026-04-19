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
    /// Number of days to query (1-366). Mutually exclusive with end_date.
    #[serde(default)]
    pub days: Option<i64>,
    /// End date in YYYY-MM-DD format. Mutually exclusive with days.
    /// If provided, days will be calculated from start_date to end_date.
    #[serde(default)]
    pub end_date: Option<String>,
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

    // Determine days from either days field or end_date
    let (days, end_date_str) = match (&request.days, &request.end_date) {
        (Some(days), None) => {
            // Validate days range (1-366)
            if *days <= 0 || *days > 366 {
                let error = ErrorResponse {
                    error: "invalid_days".to_string(),
                    message: "days must be between 1 and 366".to_string(),
                };
                return (StatusCode::BAD_REQUEST, Json(serde_json::json!(error))).into_response();
            }
            // Calculate end_date from start_date + days - 1
            let start = chrono::NaiveDate::parse_from_str(&request.start_date, "%Y-%m-%d").unwrap();
            let end = start + chrono::Duration::days(*days - 1);
            (*days, end.format("%Y-%m-%d").to_string())
        }
        (None, Some(end_date)) => {
            // Validate end_date format
            if !is_valid_date(end_date) {
                let error = ErrorResponse {
                    error: "invalid_date".to_string(),
                    message: "end_date must be in YYYY-MM-DD format".to_string(),
                };
                return (StatusCode::BAD_REQUEST, Json(serde_json::json!(error))).into_response();
            }
            // Calculate days from start_date to end_date
            let start = chrono::NaiveDate::parse_from_str(&request.start_date, "%Y-%m-%d").unwrap();
            let end = chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d").unwrap();
            
            if end < start {
                let error = ErrorResponse {
                    error: "invalid_date_range".to_string(),
                    message: "end_date must be on or after start_date".to_string(),
                };
                return (StatusCode::BAD_REQUEST, Json(serde_json::json!(error))).into_response();
            }
            
            let days = end.signed_duration_since(start).num_days() + 1;
            if days <= 0 || days > 366 {
                let error = ErrorResponse {
                    error: "invalid_days".to_string(),
                    message: "date range must be between 1 and 366 days".to_string(),
                };
                return (StatusCode::BAD_REQUEST, Json(serde_json::json!(error))).into_response();
            }
            (days, end_date.clone())
        }
        (Some(_), Some(_)) => {
            let error = ErrorResponse {
                error: "invalid_request".to_string(),
                message: "Provide either days or end_date, not both".to_string(),
            };
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!(error))).into_response();
        }
        (None, None) => {
            let error = ErrorResponse {
                error: "missing_parameter".to_string(),
                message: "Either days or end_date is required".to_string(),
            };
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!(error))).into_response();
        }
    };

    // Build payload for job
    let payload = serde_json::json!({
        "query_name": query_name,
        "start_date": request.start_date,
        "end_date": end_date_str,
        "days": days,
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

    // Determine days from either days field or end_date
    let (days, end_date_str) = match (&request.days, &request.end_date) {
        (Some(days), None) => {
            // Validate days range (1-366)
            if *days <= 0 || *days > 366 {
                let error = ErrorResponse {
                    error: "invalid_days".to_string(),
                    message: "days must be between 1 and 366".to_string(),
                };
                return (StatusCode::BAD_REQUEST, Json(serde_json::json!(error))).into_response();
            }
            // Calculate end_date from start_date + days - 1
            let start = chrono::NaiveDate::parse_from_str(&request.start_date, "%Y-%m-%d").unwrap();
            let end = start + chrono::Duration::days(*days - 1);
            (*days, end.format("%Y-%m-%d").to_string())
        }
        (None, Some(end_date)) => {
            // Validate end_date format
            if !is_valid_date(end_date) {
                let error = ErrorResponse {
                    error: "invalid_date".to_string(),
                    message: "end_date must be in YYYY-MM-DD format".to_string(),
                };
                return (StatusCode::BAD_REQUEST, Json(serde_json::json!(error))).into_response();
            }
            // Calculate days from start_date to end_date
            let start = chrono::NaiveDate::parse_from_str(&request.start_date, "%Y-%m-%d").unwrap();
            let end = chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d").unwrap();
            
            if end < start {
                let error = ErrorResponse {
                    error: "invalid_date_range".to_string(),
                    message: "end_date must be on or after start_date".to_string(),
                };
                return (StatusCode::BAD_REQUEST, Json(serde_json::json!(error))).into_response();
            }
            
            let days = end.signed_duration_since(start).num_days() + 1;
            if days <= 0 || days > 366 {
                let error = ErrorResponse {
                    error: "invalid_days".to_string(),
                    message: "date range must be between 1 and 366 days".to_string(),
                };
                return (StatusCode::BAD_REQUEST, Json(serde_json::json!(error))).into_response();
            }
            (days, end_date.clone())
        }
        (Some(_), Some(_)) => {
            let error = ErrorResponse {
                error: "invalid_request".to_string(),
                message: "Provide either days or end_date, not both".to_string(),
            };
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!(error))).into_response();
        }
        (None, None) => {
            let error = ErrorResponse {
                error: "missing_parameter".to_string(),
                message: "Either days or end_date is required".to_string(),
            };
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!(error))).into_response();
        }
    };

    // Build payload for job
    let payload = serde_json::json!({
        "query_name": query_name,
        "start_date": request.start_date,
        "end_date": end_date_str,
        "days": days,
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
    fn test_query_request_deserialization_with_days() {
        let json = r#"{"start_date":"2024-06-01","days":30,"sync":true}"#;
        let request: QueryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.start_date, "2024-06-01");
        assert_eq!(request.days, Some(30));
        assert_eq!(request.end_date, None);
        assert_eq!(request.sync, Some(true));
    }

    #[test]
    fn test_query_request_deserialization_with_end_date() {
        let json = r#"{"start_date":"2024-06-01","end_date":"2024-06-30","sync":true}"#;
        let request: QueryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.start_date, "2024-06-01");
        assert_eq!(request.days, None);
        assert_eq!(request.end_date, Some("2024-06-30".to_string()));
        assert_eq!(request.sync, Some(true));
    }

    #[test]
    fn test_query_request_deserialization_no_sync() {
        let json = r#"{"start_date":"2024-06-01","days":30}"#;
        let request: QueryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.start_date, "2024-06-01");
        assert_eq!(request.days, Some(30));
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

    #[test]
    fn test_wedding_query_request_deserialization() {
        let json = r#"{"start_date":"2024-06-01","days":30,"sync":true}"#;
        let request: QueryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.start_date, "2024-06-01");
        assert_eq!(request.days, Some(30));
        assert_eq!(request.sync, Some(true));
    }

    #[test]
    fn test_project_query_request_deserialization() {
        // Same structure as wedding, just verifying the type works
        let json = r#"{"start_date":"2024-07-15","end_date":"2024-09-13","sync":false}"#;
        let request: QueryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.start_date, "2024-07-15");
        assert_eq!(request.end_date, Some("2024-09-13".to_string()));
        assert_eq!(request.sync, Some(false));
    }

    #[test]
    fn test_travel_query_request_defaults() {
        let json = r#"{"start_date":"2024-08-01","days":14}"#;
        let request: QueryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.start_date, "2024-08-01");
        assert_eq!(request.days, Some(14));
        assert_eq!(request.sync, None); // Should default to async
    }

    // ========================================================================
    // DATE BOUNDS CALCULATION TESTS
    // ========================================================================

    #[test]
    fn test_days_to_end_date_calculation() {
        // When days is provided, end_date = start_date + days - 1
        let start = chrono::NaiveDate::parse_from_str("2024-06-01", "%Y-%m-%d").unwrap();
        let days: i64 = 30;
        let expected_end = start + chrono::Duration::days(days - 1);
        assert_eq!(expected_end.format("%Y-%m-%d").to_string(), "2024-06-30");
    }

    #[test]
    fn test_end_date_to_days_calculation() {
        // When end_date is provided, days = (end - start).days() + 1
        let start = chrono::NaiveDate::parse_from_str("2024-06-01", "%Y-%m-%d").unwrap();
        let end = chrono::NaiveDate::parse_from_str("2024-06-30", "%Y-%m-%d").unwrap();
        let days = end.signed_duration_since(start).num_days() + 1;
        assert_eq!(days, 30);
    }

    #[test]
    fn test_single_day_range() {
        // start_date == end_date means 1 day
        let start = chrono::NaiveDate::parse_from_str("2024-06-01", "%Y-%m-%d").unwrap();
        let end = chrono::NaiveDate::parse_from_str("2024-06-01", "%Y-%m-%d").unwrap();
        let days = end.signed_duration_since(start).num_days() + 1;
        assert_eq!(days, 1);
    }

    #[test]
    fn test_date_range_boundaries() {
        // Test boundary conditions for 366-day max
        let start = chrono::NaiveDate::parse_from_str("2024-01-01", "%Y-%m-%d").unwrap();
        
        // 365 days - valid
        let end_365 = start + chrono::Duration::days(364);
        let days_365 = end_365.signed_duration_since(start).num_days() + 1;
        assert_eq!(days_365, 365);
        assert!(days_365 <= 366);
        
        // 366 days - valid (max)
        let end_366 = start + chrono::Duration::days(365);
        let days_366 = end_366.signed_duration_since(start).num_days() + 1;
        assert_eq!(days_366, 366);
        assert!(days_366 <= 366);
        
        // 367 days - invalid (exceeds max)
        let end_367 = start + chrono::Duration::days(366);
        let days_367 = end_367.signed_duration_since(start).num_days() + 1;
        assert_eq!(days_367, 367);
        assert!(days_367 > 366);
    }

    #[test]
    fn test_leap_year_date_range() {
        // 2024 is a leap year
        let start = chrono::NaiveDate::parse_from_str("2024-01-01", "%Y-%m-%d").unwrap();
        let end = chrono::NaiveDate::parse_from_str("2024-12-31", "%Y-%m-%d").unwrap();
        let days = end.signed_duration_since(start).num_days() + 1;
        assert_eq!(days, 366); // Leap year has 366 days
    }

    #[test]
    fn test_non_leap_year_date_range() {
        // 2023 is not a leap year
        let start = chrono::NaiveDate::parse_from_str("2023-01-01", "%Y-%m-%d").unwrap();
        let end = chrono::NaiveDate::parse_from_str("2023-12-31", "%Y-%m-%d").unwrap();
        let days = end.signed_duration_since(start).num_days() + 1;
        assert_eq!(days, 365);
    }

    #[test]
    fn test_request_with_only_start_date_fails_validation() {
        let json = r#"{"start_date":"2024-06-01"}"#;
        let request: QueryRequest = serde_json::from_str(json).unwrap();
        // Neither days nor end_date provided - should be caught by handler validation
        assert!(request.days.is_none());
        assert!(request.end_date.is_none());
    }

    #[test]
    fn test_request_with_both_days_and_end_date() {
        let json = r#"{"start_date":"2024-06-01","days":30,"end_date":"2024-06-30"}"#;
        let request: QueryRequest = serde_json::from_str(json).unwrap();
        // Both provided - should be caught by handler validation
        assert!(request.days.is_some());
        assert!(request.end_date.is_some());
    }

    #[test]
    fn test_request_with_end_date_before_start_date() {
        let json = r#"{"start_date":"2024-06-30","end_date":"2024-06-01"}"#;
        let request: QueryRequest = serde_json::from_str(json).unwrap();
        // end_date before start_date - should be caught by handler validation
        let start = chrono::NaiveDate::parse_from_str(&request.start_date, "%Y-%m-%d").unwrap();
        let end = chrono::NaiveDate::parse_from_str(request.end_date.as_ref().unwrap(), "%Y-%m-%d").unwrap();
        assert!(end < start);
    }

    #[test]
    fn test_date_consistency_across_month_boundary() {
        // Test crossing month boundaries
        let start = chrono::NaiveDate::parse_from_str("2024-06-25", "%Y-%m-%d").unwrap();
        let end = chrono::NaiveDate::parse_from_str("2024-07-05", "%Y-%m-%d").unwrap();
        let days = end.signed_duration_since(start).num_days() + 1;
        assert_eq!(days, 11); // 25, 26, 27, 28, 29, 30 (June) + 1, 2, 3, 4, 5 (July) = 11 days
    }

    #[test]
    fn test_date_consistency_across_year_boundary() {
        // Test crossing year boundaries
        let start = chrono::NaiveDate::parse_from_str("2024-12-25", "%Y-%m-%d").unwrap();
        let end = chrono::NaiveDate::parse_from_str("2025-01-05", "%Y-%m-%d").unwrap();
        let days = end.signed_duration_since(start).num_days() + 1;
        assert_eq!(days, 12); // Dec 25-31 (7 days) + Jan 1-5 (5 days) = 12 days
    }
}
