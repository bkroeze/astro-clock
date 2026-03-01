use async_trait::async_trait;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::{Pool, Postgres};
use tracing::{debug, error, info};

use crate::database::chunk_generator::ChunkGenerator;
use crate::database::pool::DatabasePool;
use crate::jobs::error::{JobError, JobResult};
use crate::jobs::executor::JobHandler;
use crate::jobs::repository::LoadedDaysRepository;
use crate::jobs::types::{Job, JobType};

/// Payload for load jobs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadJobPayload {
    /// Start date in YYYY-MM-DD format
    pub start_date: String,
    /// Number of days to load
    pub days: i64,
}

/// Result details for a single loaded date
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateLoadDetail {
    pub date: String,
    pub positions: usize,
    pub aspects: usize,
    pub lunar_conditions: usize,
}

/// Result details for a failed date
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateFailureDetail {
    pub date: String,
    pub error: String,
}

/// Result returned by LoadJobHandler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadJobResult {
    pub start_date: String,
    pub days_requested: i64,
    pub dates_loaded: usize,
    pub dates_skipped: usize,
    pub dates_failed: usize,
    pub total_positions: usize,
    pub total_aspects: usize,
    pub total_lunar_conditions: usize,
    pub loaded: Vec<DateLoadDetail>,
    pub skipped: Vec<String>,
    pub failed: Vec<DateFailureDetail>,
}

/// Handler for data loading jobs
///
/// Orchestrates day-level incremental data loading by:
/// 1. Detecting which dates in the requested range are already loaded
/// 2. Generating planetary data for missing dates using ChunkGenerator
/// 3. Saving data to database tables
/// 4. Marking loaded dates in the tracking table
/// 5. Returning structured results with statistics
#[derive(Debug, Clone)]
pub struct LoadJobHandler {
    loaded_days_repo: LoadedDaysRepository,
    pool: Pool<Postgres>,
}

impl LoadJobHandler {
    /// Create a new LoadJobHandler with the given database pool
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self {
            loaded_days_repo: LoadedDaysRepository::new(pool.clone()),
            pool,
        }
    }

    /// Parse and validate the job payload
    fn parse_payload(&self, job: &Job) -> JobResult<LoadJobPayload> {
        let payload = job
            .payload
            .as_ref()
            .ok_or_else(|| JobError::Other("Missing job payload".to_string()))?;

        let start_date = payload
            .get("start_date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JobError::Other("Missing start_date in payload".to_string()))?;

        let days = payload
            .get("days")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| JobError::Other("Missing days in payload".to_string()))?;

        // Validate days is positive and reasonable
        if days <= 0 {
            return Err(JobError::Other(
                "days must be positive".to_string(),
            ));
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

        Ok(LoadJobPayload {
            start_date: start_date.to_string(),
            days,
        })
    }

    /// Generate all dates in the requested range
    fn generate_date_range(&self, start: NaiveDate, days: i64) -> Vec<NaiveDate> {
        let mut dates = Vec::with_capacity(days as usize);
        let mut current = start;
        let end = start + chrono::Duration::days(days - 1);

        while current <= end {
            dates.push(current);
            current = current + chrono::Duration::days(1);
        }

        dates
    }
}

#[async_trait]
impl JobHandler for LoadJobHandler {
    fn job_type(&self) -> JobType {
        JobType::Load
    }

    async fn execute(&self, job: &Job) -> JobResult<JsonValue> {
        let payload = self.parse_payload(job)?;

        let start_date =
            NaiveDate::parse_from_str(&payload.start_date, "%Y-%m-%d").map_err(|e| {
                JobError::Other(format!("Failed to parse start_date: {}", e))
            })?;

        info!(
            job_id = %job.id,
            start_date = %payload.start_date,
            days = payload.days,
            "Starting load job"
        );

        // Find missing dates using the database
        let missing_dates = self
            .loaded_days_repo
            .get_missing_dates(start_date, payload.days)
            .await?;

        debug!(
            job_id = %job.id,
            missing_count = missing_dates.len(),
            "Found missing dates"
        );

        // Generate all dates in range to determine which are already loaded
        let all_dates = self.generate_date_range(start_date, payload.days);
        let missing_set: std::collections::HashSet<_> =
            missing_dates.iter().cloned().collect();

        let skipped_dates: Vec<String> = all_dates
            .iter()
            .filter(|d| !missing_set.contains(*d))
            .map(|d| d.to_string())
            .collect();

        if !skipped_dates.is_empty() {
            info!(
                job_id = %job.id,
                skipped_count = skipped_dates.len(),
                "Skipping already-loaded dates"
            );
        }

        // Create chunk generator using the existing pool
        // DatabasePool is a wrapper around Pool<Postgres>, so we create it directly
        let db_pool = DatabasePool::from_pool(self.pool.clone());
        
        let chunk_generator = ChunkGenerator::new(db_pool);

        // Track results
        let mut loaded_details: Vec<DateLoadDetail> = Vec::new();
        let mut failed_details: Vec<DateFailureDetail> = Vec::new();
        let mut total_positions = 0usize;
        let mut total_aspects = 0usize;
        let mut total_lunar = 0usize;

        // Process each missing date
        for date in missing_dates {
            debug!(job_id = %job.id, date = %date, "Processing date");

            // Generate chunk for this date
            let chunk = match chunk_generator.generate_chunk(date).await {
                Ok(chunk) => chunk,
                Err(e) => {
                    error!(job_id = %job.id, date = %date, error = %e, "Failed to generate chunk");
                    failed_details.push(DateFailureDetail {
                        date: date.to_string(),
                        error: e.to_string(),
                    });
                    continue;
                }
            };

            let positions_count = chunk.planet_positions.len();
            let aspects_count = chunk.aspects.len();
            let lunar_count = chunk.lunar_conditions.len();

            // Save chunk to database
            if let Err(e) = chunk_generator.save_chunk_to_db(&chunk).await {
                error!(job_id = %job.id, date = %date, error = %e, "Failed to save chunk");
                failed_details.push(DateFailureDetail {
                    date: date.to_string(),
                    error: format!("Save failed: {}", e),
                });
                continue;
            }

            // Mark day as loaded
            if let Err(e) = self
                .loaded_days_repo
                .mark_day_loaded(date, 1440, Some(job.id))
                .await
            {
                error!(job_id = %job.id, date = %date, error = %e, "Failed to mark day loaded");
                failed_details.push(DateFailureDetail {
                    date: date.to_string(),
                    error: format!("Tracking failed: {}", e),
                });
                continue;
            }

            // Track success
            loaded_details.push(DateLoadDetail {
                date: date.to_string(),
                positions: positions_count,
                aspects: aspects_count,
                lunar_conditions: lunar_count,
            });

            total_positions += positions_count;
            total_aspects += aspects_count;
            total_lunar += lunar_count;

            debug!(
                job_id = %job.id,
                date = %date,
                positions = positions_count,
                aspects = aspects_count,
                lunar = lunar_count,
                "Date loaded successfully"
            );
        }

        // Build result
        let result = LoadJobResult {
            start_date: payload.start_date.clone(),
            days_requested: payload.days,
            dates_loaded: loaded_details.len(),
            dates_skipped: skipped_dates.len(),
            dates_failed: failed_details.len(),
            total_positions,
            total_aspects,
            total_lunar_conditions: total_lunar,
            loaded: loaded_details,
            skipped: skipped_dates,
            failed: failed_details,
        };

        // Log summary
        info!(
            job_id = %job.id,
            loaded = result.dates_loaded,
            skipped = result.dates_skipped,
            failed = result.dates_failed,
            total_positions = result.total_positions,
            total_aspects = result.total_aspects,
            total_lunar = result.total_lunar_conditions,
            "Load job completed"
        );

        // Convert to JSON
        serde_json::to_value(result).map_err(JobError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_job_payload_serialization() {
        let payload = LoadJobPayload {
            start_date: "2024-01-01".to_string(),
            days: 30,
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("2024-01-01"));
        assert!(json.contains("30"));

        let deserialized: LoadJobPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.start_date, "2024-01-01");
        assert_eq!(deserialized.days, 30);
    }

    #[test]
    fn test_load_job_result_serialization() {
        let result = LoadJobResult {
            start_date: "2024-01-01".to_string(),
            days_requested: 7,
            dates_loaded: 5,
            dates_skipped: 1,
            dates_failed: 1,
            total_positions: 100800, // 10 bodies * 1440 minutes * 7 days
            total_aspects: 5000,
            total_lunar_conditions: 10080, // 1440 minutes * 7 days
            loaded: vec![DateLoadDetail {
                date: "2024-01-01".to_string(),
                positions: 14400,
                aspects: 1000,
                lunar_conditions: 1440,
            }],
            skipped: vec!["2024-01-02".to_string()],
            failed: vec![DateFailureDetail {
                date: "2024-01-03".to_string(),
                error: "Test error".to_string(),
            }],
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: LoadJobResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.dates_loaded, 5);
        assert_eq!(deserialized.dates_skipped, 1);
        assert_eq!(deserialized.dates_failed, 1);
        assert_eq!(deserialized.total_positions, 100800);
    }
}
