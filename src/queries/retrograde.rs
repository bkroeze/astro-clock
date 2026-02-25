use chrono::{DateTime, NaiveDate, Utc};
use sqlx::Row;
use std::time::Instant;

use crate::database::pool::DatabasePool;
use crate::queries::error::QueryError;
use crate::queries::types::{
    Body, QueryResult, RetrogradeCriteria, RetrogradePeriod, RetrogradeStatus,
};

/// Find planetary retrograde periods within a date range
///
/// Returns retrograde periods with status calculation (Direct, Retrograde,
/// PreShadow, PostShadow) based on the query date range.
pub async fn find_retrograde_periods(
    pool: &DatabasePool,
    criteria: &RetrogradeCriteria,
) -> Result<QueryResult<RetrogradePeriod>, QueryError> {
    // Validate criteria
    criteria
        .validate()
        .map_err(|e| QueryError::InvalidCriteria(e.to_string()))?;

    let start_time = Instant::now();

    let start_datetime = criteria
        .start_date
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();
    let end_datetime = criteria
        .end_date
        .and_hms_opt(23, 59, 59)
        .unwrap()
        .and_utc();

    // Build and execute query based on whether planet filter is specified
    let rows = if let Some(ref planets) = criteria.planets {
        let body_ids: Vec<i16> = planets.iter().map(|b| b.to_id()).collect();
        sqlx::query(
            r#"
            SELECT 
                body_id,
                retrograde_start,
                retrograde_end,
                pre_shadow_start,
                post_shadow_end
            FROM retrograde_periods
            WHERE body_id = ANY($1)
              AND (
                  (retrograde_start >= $2 AND retrograde_start <= $3)
                  OR (retrograde_end >= $2 AND retrograde_end <= $3)
                  OR (retrograde_start <= $2 AND retrograde_end >= $3)
              )
            ORDER BY retrograde_start, body_id
            "#,
        )
        .bind(&body_ids)
        .bind(start_datetime)
        .bind(end_datetime)
        .fetch_all(pool.pool())
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT 
                body_id,
                retrograde_start,
                retrograde_end,
                pre_shadow_start,
                post_shadow_end
            FROM retrograde_periods
            WHERE (
                (retrograde_start >= $1 AND retrograde_start <= $2)
                OR (retrograde_end >= $1 AND retrograde_end <= $2)
                OR (retrograde_start <= $1 AND retrograde_end >= $2)
            )
            ORDER BY retrograde_start, body_id
            "#,
        )
        .bind(start_datetime)
        .bind(end_datetime)
        .fetch_all(pool.pool())
        .await?
    };

    let rows_examined = rows.len();

    let periods: Vec<RetrogradePeriod> = rows
        .into_iter()
        .filter_map(|row| {
            let body_id: i16 = row.try_get("body_id").ok()?;
            let retrograde_start: DateTime<Utc> = row.try_get("retrograde_start").ok()?;
            let retrograde_end: DateTime<Utc> = row.try_get("retrograde_end").ok()?;
            let pre_shadow_start: Option<DateTime<Utc>> = row.try_get("pre_shadow_start").ok()?;
            let post_shadow_end: Option<DateTime<Utc>> = row.try_get("post_shadow_end").ok()?;

            // Calculate status based on the start of the query date range
            let status = calculate_status(
                start_datetime,
                retrograde_start,
                retrograde_end,
                pre_shadow_start,
                post_shadow_end,
            );

            Some(RetrogradePeriod {
                planet: Body::from_id(body_id)?,
                start: retrograde_start,
                end: retrograde_end,
                shadow_start: pre_shadow_start,
                shadow_end: post_shadow_end,
                status,
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

/// Calculate retrograde status for a given query time
fn calculate_status(
    query_time: DateTime<Utc>,
    retrograde_start: DateTime<Utc>,
    retrograde_end: DateTime<Utc>,
    pre_shadow_start: Option<DateTime<Utc>>,
    post_shadow_end: Option<DateTime<Utc>>,
) -> RetrogradeStatus {
    // Check if query time is within retrograde period
    if query_time >= retrograde_start && query_time <= retrograde_end {
        return RetrogradeStatus::Retrograde;
    }

    // Check pre-shadow period
    if let Some(shadow_start) = pre_shadow_start {
        if query_time >= shadow_start && query_time < retrograde_start {
            return RetrogradeStatus::PreShadow;
        }
    }

    // Check post-shadow period
    if let Some(shadow_end) = post_shadow_end {
        if query_time > retrograde_end && query_time <= shadow_end {
            return RetrogradeStatus::PostShadow;
        }
    }

    RetrogradeStatus::Direct
}

/// Check if a specific planet is in retrograde at a given time
pub async fn is_retrograde_at_time(
    pool: &DatabasePool,
    body: Body,
    datetime: DateTime<Utc>,
) -> Result<Option<RetrogradeStatus>, QueryError> {
    let rows = sqlx::query(
        r#"
        SELECT 
            retrograde_start,
            retrograde_end,
            pre_shadow_start,
            post_shadow_end
        FROM retrograde_periods
        WHERE body_id = $1
          AND retrograde_start <= $2
          AND retrograde_end >= $2
        LIMIT 1
        "#,
    )
    .bind(body.to_id())
    .bind(datetime)
    .fetch_all(pool.pool())
    .await?;

    if rows.is_empty() {
        return Ok(None);
    }

    let row = &rows[0];
    let retrograde_start: DateTime<Utc> = row.try_get("retrograde_start").unwrap_or(datetime);
    let retrograde_end: DateTime<Utc> = row.try_get("retrograde_end").unwrap_or(datetime);
    let pre_shadow_start: Option<DateTime<Utc>> = row.try_get("pre_shadow_start").ok();
    let post_shadow_end: Option<DateTime<Utc>> = row.try_get("post_shadow_end").ok();

    let status = calculate_status(
        datetime,
        retrograde_start,
        retrograde_end,
        pre_shadow_start,
        post_shadow_end,
    );

    Ok(Some(status))
}

/// Find all retrograde periods for a specific planet in a date range
pub async fn find_planet_retrograde_periods(
    pool: &DatabasePool,
    body: Body,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<QueryResult<RetrogradePeriod>, QueryError> {
    let criteria = RetrogradeCriteria {
        start_date,
        end_date,
        planets: Some(vec![body]),
    };
    find_retrograde_periods(pool, &criteria).await
}
