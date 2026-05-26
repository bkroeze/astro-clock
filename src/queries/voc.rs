use chrono::{DateTime, Duration, NaiveDate, Utc};
use sqlx::Row;
use std::time::Instant;

use crate::database::pool::DatabasePool;
use crate::queries::error::QueryError;
use crate::queries::types::{QueryResult, VoCCriteria, VoCPeriod, ZodiacSign};

/// Find void-of-course Moon periods within a date range
///
/// Uses pre-calculated is_void_of_course flag from lunar_conditions table.
/// Aggregates contiguous VoC periods and returns start/end times with duration.
pub async fn find_voc_periods(
    pool: &DatabasePool,
    criteria: &VoCCriteria,
) -> Result<QueryResult<VoCPeriod>, QueryError> {
    // Validate criteria
    criteria
        .validate()
        .map_err(|e| QueryError::InvalidCriteria(e.to_string()))?;

    let start_time = Instant::now();

    let start_datetime = criteria.start_date.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let end_datetime = criteria.end_date.and_hms_opt(23, 59, 59).unwrap().and_utc();

    // Query to find VoC periods using gap-and-island pattern
    // This aggregates contiguous rows where is_void_of_course = true
    let rows = sqlx::query(
        r#"
        WITH voc_groups AS (
            SELECT 
                time,
                moon_sign,
                is_void_of_course,
                -- Create a group identifier that increments when VoC status changes
                SUM(CASE WHEN is_void_of_course THEN 0 ELSE 1 END) 
                    OVER (ORDER BY time) as grp
            FROM lunar_conditions
            WHERE time >= $1 
              AND time <= $2
              AND is_void_of_course = true
        ),
        voc_periods AS (
            SELECT 
                MIN(time) as start_time,
                MAX(time) as end_time,
                MAX(time) - MIN(time) as duration,
                MIN(moon_sign) as moon_sign  -- Moon sign at start of VoC
            FROM voc_groups
            GROUP BY grp
        )
        SELECT 
            start_time,
            end_time,
            duration,
            moon_sign
        FROM voc_periods
        WHERE ($3::interval IS NULL OR duration >= $3)
        ORDER BY start_time
        "#,
    )
    .bind(start_datetime)
    .bind(end_datetime)
    .bind(
        criteria
            .min_duration
            .map(|d| format!("{} seconds", d.num_seconds())),
    )
    .fetch_all(pool.pool())
    .await?;

    let rows_examined = rows.len();

    let periods: Vec<VoCPeriod> = rows
        .into_iter()
        .filter_map(|row| {
            let start: DateTime<Utc> = row.try_get("start_time").ok()?;
            let end: DateTime<Utc> = row.try_get("end_time").ok()?;
            let duration_pg: Option<String> = row.try_get("duration").ok();
            let moon_sign_id: i16 = row.try_get("moon_sign").unwrap_or(0);

            // Parse PostgreSQL interval format or calculate from timestamps
            let duration = duration_pg
                .and_then(|d| parse_pg_interval(&d))
                .unwrap_or_else(|| end - start);

            Some(VoCPeriod {
                start,
                end,
                duration,
                moon_sign: ZodiacSign::from_id(moon_sign_id)
                    .unwrap_or(crate::queries::types::ZodiacSign::Aries),
            })
        })
        .collect();

    let execution_time_ms = start_time.elapsed().as_millis() as u64;

    Ok(QueryResult {
        data: periods,
        execution_time_ms,
        rows_examined,
        cache_hit: false,
    })
}

/// Find all VoC periods without minimum duration filtering
///
/// Convenience function for getting all VoC periods in a range.
pub async fn find_all_voc_periods(
    pool: &DatabasePool,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<QueryResult<VoCPeriod>, QueryError> {
    let criteria = VoCCriteria {
        start_date,
        end_date,
        min_duration: None,
    };
    find_voc_periods(pool, &criteria).await
}

/// Check if a specific datetime is during a VoC period
///
/// Returns the VoC period if the datetime falls within one,
/// or None if the Moon is not void-of-course at that time.
pub async fn is_voc_at_time(
    pool: &DatabasePool,
    datetime: DateTime<Utc>,
) -> Result<Option<VoCPeriod>, QueryError> {
    let rows = sqlx::query(
        r#"
        SELECT 
            time,
            moon_sign,
            is_void_of_course
        FROM lunar_conditions
        WHERE time = $1
        LIMIT 1
        "#,
    )
    .bind(datetime)
    .fetch_all(pool.pool())
    .await?;

    if rows.is_empty() {
        return Ok(None);
    }

    let row = &rows[0];
    let is_voc: bool = row.try_get("is_void_of_course").unwrap_or(false);

    if !is_voc {
        return Ok(None);
    }

    // Find the VoC period containing this datetime
    let date = datetime.date_naive();
    let criteria = VoCCriteria {
        start_date: date - chrono::Duration::days(1),
        end_date: date + chrono::Duration::days(1),
        min_duration: None,
    };

    let result = find_voc_periods(pool, &criteria).await?;

    // Find the period containing our datetime
    let period = result
        .data
        .into_iter()
        .find(|p| p.start <= datetime && p.end >= datetime);

    Ok(period)
}

/// Parse PostgreSQL interval string to Duration
fn parse_pg_interval(interval: &str) -> Option<Duration> {
    // PostgreSQL interval format: "HH:MM:SS" or "DD days HH:MM:SS"
    // For simplicity, we'll use a basic parser
    if let Some(days_idx) = interval.find(" days ") {
        let days: i64 = interval[..days_idx].parse().ok()?;
        let time_part = &interval[days_idx + 6..];
        let parts: Vec<&str> = time_part.split(':').collect();
        if parts.len() == 3 {
            let hours: i64 = parts[0].parse().ok()?;
            let minutes: i64 = parts[1].parse().ok()?;
            let seconds: i64 = parts[2].parse().ok()?;
            return Some(
                Duration::days(days)
                    + Duration::hours(hours)
                    + Duration::minutes(minutes)
                    + Duration::seconds(seconds),
            );
        }
    }

    // Try HH:MM:SS format
    let parts: Vec<&str> = interval.split(':').collect();
    if parts.len() == 3 {
        let hours: i64 = parts[0].parse().ok()?;
        let minutes: i64 = parts[1].parse().ok()?;
        let seconds: i64 = parts[2].parse().ok()?;
        return Some(
            Duration::hours(hours) + Duration::minutes(minutes) + Duration::seconds(seconds),
        );
    }

    None
}
