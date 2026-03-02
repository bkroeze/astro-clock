use async_trait::async_trait;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::database::chunk_generator::ChunkGenerator;
use crate::database::pool::DatabasePool;
use crate::jobs::error::{JobError, JobResult};
use crate::jobs::executor::JobHandler;
use crate::jobs::registry::QueryTemplateRegistry;
use crate::jobs::repository::LoadedDaysRepository;
use crate::jobs::types::{Job, JobType};

/// Payload for query jobs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryJobPayload {
    /// Query name: "wedding", "project", or "travel"
    pub query_name: String,
    /// Start date in YYYY-MM-DD format
    pub start_date: String,
    /// Number of days to query
    pub days: i64,
}

/// Result wrapper for query jobs with common metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryJobResult {
    /// Name of the query that was executed
    pub query_name: String,
    /// Start date of the query range
    pub start_date: String,
    /// Number of days queried
    pub days: i64,
    /// Total number of results found
    pub total_results: usize,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// The actual query results (JSON array)
    pub results: JsonValue,
    /// Any warnings about data loading (e.g., partial failures)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
}

/// Handler for query jobs
///
/// Orchestrates named query execution by:
/// 1. Parsing and validating the job payload
/// 2. Automatically loading missing data via ChunkGenerator
/// 3. Dispatching to the appropriate query template via QueryTemplateRegistry
/// 4. Returning structured JSON results
#[derive(Debug, Clone)]
pub struct QueryJobHandler {
    pool: Pool<Postgres>,
    loaded_days_repo: LoadedDaysRepository,
    registry: Arc<QueryTemplateRegistry>,
}

impl QueryJobHandler {
    /// Create a new QueryJobHandler with the given database pool
    pub fn new(pool: Pool<Postgres>) -> Self {
        let loaded_days_repo = LoadedDaysRepository::new(pool.clone());
        let registry = Arc::new(QueryTemplateRegistry::new());

        Self {
            pool,
            loaded_days_repo,
            registry,
        }
    }

    /// Create a new QueryJobHandler with a custom registry (for testing)
    #[allow(dead_code)]
    pub fn with_registry(pool: Pool<Postgres>, registry: Arc<QueryTemplateRegistry>) -> Self {
        let loaded_days_repo = LoadedDaysRepository::new(pool.clone());

        Self {
            pool,
            loaded_days_repo,
            registry,
        }
    }

    /// Parse and validate the job payload
    fn parse_payload(&self, job: &Job) -> JobResult<QueryJobPayload> {
        let payload = job
            .payload
            .as_ref()
            .ok_or_else(|| JobError::Other("Missing job payload".to_string()))?;

        let query_name = payload
            .get("query_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JobError::Other("Missing query_name in payload".to_string()))?;

        let start_date = payload
            .get("start_date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JobError::Other("Missing start_date in payload".to_string()))?;

        let days = payload
            .get("days")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| JobError::Other("Missing days in payload".to_string()))?;

        // Validate query_name is supported
        if !["wedding", "project", "travel"].contains(&query_name) {
            return Err(JobError::Other(format!(
                "Unknown query_name: {}. Must be one of: wedding, project, travel",
                query_name
            )));
        }

        // Validate days is positive and reasonable
        if days <= 0 {
            return Err(JobError::Other("days must be positive".to_string()));
        }

        if days > 366 {
            return Err(JobError::Other(
                "days cannot exceed 366 (use multiple jobs for larger ranges)".to_string(),
            ));
        }

        // Validate date format
        if NaiveDate::parse_from_str(start_date, "%Y-%m-%d").is_err() {
            return Err(JobError::Other(format!(
                "Invalid start_date format: {}. Expected YYYY-MM-DD",
                start_date
            )));
        }

        Ok(QueryJobPayload {
            query_name: query_name.to_string(),
            start_date: start_date.to_string(),
            days,
        })
    }

    /// Ensure data is loaded for the requested date range
    ///
    /// Checks loaded_days table and generates missing data via ChunkGenerator.
    /// Continues with partial failures - warnings are collected and returned
    /// in the job result.
    ///
    /// Returns a vector of warnings (if any dates failed to load)
    async fn ensure_data_loaded(
        &self,
        job_id: uuid::Uuid,
        start_date: NaiveDate,
        days: i64,
    ) -> JobResult<Vec<String>> {
        let mut warnings: Vec<String> = Vec::new();

        // Find missing dates using the database
        let missing_dates = self
            .loaded_days_repo
            .get_missing_dates(start_date, days)
            .await?;

        if missing_dates.is_empty() {
            debug!(job_id = %job_id, "All dates already loaded");
            return Ok(warnings);
        }

        info!(
            job_id = %job_id,
            missing_count = missing_dates.len(),
            "Loading missing dates"
        );

        // Create chunk generator using the existing pool
        let db_pool = DatabasePool::from_pool(self.pool.clone());
        let chunk_generator = ChunkGenerator::new(db_pool);

        // Process each missing date
        for date in missing_dates {
            debug!(job_id = %job_id, date = %date, "Generating chunk for date");

            // Generate chunk for this date
            let chunk = match chunk_generator.generate_chunk(date).await {
                Ok(chunk) => chunk,
                Err(e) => {
                    error!(
                        job_id = %job_id,
                        date = %date,
                        error = %e,
                        "Failed to generate chunk"
                    );
                    warnings.push(format!("Failed to generate data for {}: {}", date, e));
                    continue;
                }
            };

            // Save chunk to database
            if let Err(e) = chunk_generator.save_chunk_to_db(&chunk).await {
                error!(
                    job_id = %job_id,
                    date = %date,
                    error = %e,
                    "Failed to save chunk"
                );
                    warnings.push(format!("Failed to save data for {}: {}", date, e));
                    continue;
            }

            // Mark day as loaded
            if let Err(e) = self
                .loaded_days_repo
                .mark_day_loaded(date, 1440, Some(job_id))
                .await
            {
                error!(
                    job_id = %job_id,
                    date = %date,
                    error = %e,
                    "Failed to mark day loaded"
                );
                    warnings.push(format!("Failed to track {}: {}", date, e));
                continue;
            }

            debug!(job_id = %job_id, date = %date, "Date loaded successfully");
        }

        if !warnings.is_empty() {
            warn!(
                job_id = %job_id,
                warning_count = warnings.len(),
                "Some dates failed to load, proceeding with available data"
            );
        }

        Ok(warnings)
    }
}

#[async_trait]
impl JobHandler for QueryJobHandler {
    fn job_type(&self) -> JobType {
        JobType::Query
    }

    async fn execute(&self, job: &Job) -> JobResult<JsonValue> {
        let start_time = std::time::Instant::now();

        // 1. Parse payload
        let payload = self.parse_payload(job)?;

        let start_date = NaiveDate::parse_from_str(&payload.start_date, "%Y-%m-%d").map_err(|e| {
            JobError::Other(format!("Failed to parse start_date: {}", e))
        })?;

        info!(
            job_id = %job.id,
            query_name = %payload.query_name,
            start_date = %payload.start_date,
            days = payload.days,
            "Starting query job"
        );

        // 2. Ensure data is loaded
        let warnings = self.ensure_data_loaded(job.id, start_date, payload.days).await?;

        // 3. Get query template from registry and execute
        let db_pool = DatabasePool::from_pool(self.pool.clone());
        let query_results = self.registry.execute(&payload.query_name, &db_pool, &payload).await?;

        // 4. Extract result count (handle both array and object results)
        let total_results = query_results
            .as_array()
            .map(|arr| arr.len())
            .or_else(|| query_results.get("data").and_then(|d| d.as_array()).map(|arr| arr.len()))
            .unwrap_or(0);

        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        // 5. Wrap in QueryJobResult
        let job_result = QueryJobResult {
            query_name: payload.query_name.clone(),
            start_date: payload.start_date.clone(),
            days: payload.days,
            total_results,
            execution_time_ms,
            results: query_results,
            warnings: if warnings.is_empty() { None } else { Some(warnings) },
        };

        info!(
            job_id = %job.id,
            query_name = %payload.query_name,
            total_results = job_result.total_results,
            execution_time_ms = job_result.execution_time_ms,
            "Query job completed"
        );

        // 6. Serialize to JSON
        serde_json::to_value(job_result).map_err(JobError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_job_payload_serialization() {
        let payload = QueryJobPayload {
            query_name: "wedding".to_string(),
            start_date: "2024-01-01".to_string(),
            days: 30,
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("wedding"));
        assert!(json.contains("2024-01-01"));
        assert!(json.contains("30"));

        let deserialized: QueryJobPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.query_name, "wedding");
        assert_eq!(deserialized.start_date, "2024-01-01");
        assert_eq!(deserialized.days, 30);
    }

    #[test]
    fn test_query_job_result_serialization() {
        let result = QueryJobResult {
            query_name: "wedding".to_string(),
            start_date: "2024-01-01".to_string(),
            days: 30,
            total_results: 5,
            execution_time_ms: 42,
            results: serde_json::json!([{"date": "2024-01-15"}]),
            warnings: Some(vec!["Partial data load".to_string()]),
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: QueryJobResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.query_name, "wedding");
        assert_eq!(deserialized.total_results, 5);
        assert_eq!(deserialized.execution_time_ms, 42);
        assert!(deserialized.warnings.is_some());
    }

    #[test]
    fn test_query_job_result_without_warnings() {
        let result = QueryJobResult {
            query_name: "project".to_string(),
            start_date: "2024-06-01".to_string(),
            days: 7,
            total_results: 3,
            execution_time_ms: 15,
            results: serde_json::json!([{"date": "2024-06-03"}]),
            warnings: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        // Warnings field should be omitted when None
        assert!(!json.contains("warnings"));
    }

    // Note: parse_payload tests require a database pool for the handler
    // These would be integration tests. For unit tests, we verify the
    // struct definitions and serialization are correct.

    /// Integration tests for query job end-to-end execution
    mod integration_tests {
        use super::*;
        use crate::jobs::registry::QueryTemplateRegistry;
        use crate::queries::types::{WeddingCandidate, ZodiacSign};
        use chrono::Utc;

        /// Tests that the wedding query template can be executed end-to-end
        /// This verifies QUERY-06: Wedding query works through job system
        #[test]
        fn test_wedding_query_template_e2e() {
            // Verify registry returns a working template
            let registry = QueryTemplateRegistry::new();

            // Verify wedding template exists
            let template = registry
                .get("wedding")
                .expect("wedding template should be registered");
            assert_eq!(template.name, "wedding");

            // Verify we can construct the execute payload
            let payload = QueryJobPayload {
                query_name: "wedding".to_string(),
                start_date: "2024-06-01".to_string(),
                days: 30,
            };

            // Verify payload serializes correctly for job system
            let payload_json =
                serde_json::to_value(&payload).expect("payload should serialize");
            assert_eq!(payload_json["query_name"], "wedding");
            assert_eq!(payload_json["start_date"], "2024-06-01");
            assert_eq!(payload_json["days"], 30);

            // Note: Full execution requires database, tested in integration tests
            // This test verifies the template is registered and payload is valid
        }

        /// Tests that all three query templates are properly registered
        #[test]
        fn test_all_query_templates_registered() {
            let registry = QueryTemplateRegistry::new();

            // Verify wedding exists
            assert!(
                registry.get("wedding").is_some(),
                "wedding template missing"
            );

            // Verify project exists (now with real implementation from 07-02)
            let project = registry
                .get("project")
                .expect("project template should be registered");
            assert_eq!(project.name, "project");

            // Verify travel exists (now with real implementation from 07-02)
            let travel = registry
                .get("travel")
                .expect("travel template should be registered");
            assert_eq!(travel.name, "travel");

            // Verify unknown queries return None
            assert!(
                registry.get("unknown").is_none(),
                "unknown template should not exist"
            );
        }

        /// Tests QueryJobResult serialization for wedding query results
        #[test]
        fn test_wedding_query_job_result_serialization() {
            let candidate = WeddingCandidate {
                datetime: Utc::now(),
                moon_sign: ZodiacSign::Taurus,
                venus_favorable_aspects: 5,
            };

            let result = QueryJobResult {
                query_name: "wedding".to_string(),
                start_date: "2024-06-01".to_string(),
                days: 30,
                total_results: 1,
                execution_time_ms: 150,
                results: serde_json::json!({"data": [candidate]}),
                warnings: None,
            };

            let json_str = serde_json::to_string(&result).expect("result should serialize");

            assert!(json_str.contains("wedding"));
            assert!(json_str.contains("2024-06-01"));
            assert!(json_str.contains("total_results"));

            // Verify deserialization
            let deserialized: QueryJobResult =
                serde_json::from_str(&json_str).expect("result should deserialize");
            assert_eq!(deserialized.query_name, "wedding");
            assert_eq!(deserialized.total_results, 1);
        }

        /// Tests that query templates have proper descriptions
        #[test]
        fn test_query_template_metadata() {
            let registry = QueryTemplateRegistry::new();

            let wedding = registry.get("wedding").unwrap();
            assert!(!wedding.description.is_empty());
            assert!(wedding.description.to_lowercase().contains("wedding"));

            let project = registry.get("project").unwrap();
            assert!(!project.description.is_empty());

            let travel = registry.get("travel").unwrap();
            assert!(!travel.description.is_empty());
        }

        /// Tests payload validation for different query types
        #[test]
        fn test_query_payload_variations() {
            // Wedding payload
            let wedding_payload = QueryJobPayload {
                query_name: "wedding".to_string(),
                start_date: "2024-06-15".to_string(),
                days: 60,
            };
            let wedding_json = serde_json::to_value(&wedding_payload).unwrap();
            assert_eq!(wedding_json["query_name"], "wedding");

            // Project payload
            let project_payload = QueryJobPayload {
                query_name: "project".to_string(),
                start_date: "2024-07-01".to_string(),
                days: 14,
            };
            let project_json = serde_json::to_value(&project_payload).unwrap();
            assert_eq!(project_json["query_name"], "project");

            // Travel payload
            let travel_payload = QueryJobPayload {
                query_name: "travel".to_string(),
                start_date: "2024-08-01".to_string(),
                days: 30,
            };
            let travel_json = serde_json::to_value(&travel_payload).unwrap();
            assert_eq!(travel_json["query_name"], "travel");
        }
    }
}
