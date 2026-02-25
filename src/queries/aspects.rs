use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::Row;
use std::time::Instant;

use crate::database::pool::DatabasePool;
use crate::queries::error::QueryError;
use crate::queries::types::{
    AspectCriteria, AspectType, Body, ExactAspect, QueryResult,
};

/// Find exact aspects within a date range
///
/// Filters by orb threshold, aspect types, and body pairs.
/// Returns aspects sorted by time and orb (tightest first).
pub async fn find_exact_aspects(
    pool: &DatabasePool,
    criteria: &AspectCriteria,
) -> Result<QueryResult<ExactAspect>, QueryError> {
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
    let orb_f64: f64 = criteria.orb_threshold.try_into().unwrap_or(1.0);

    // Build dynamic query based on filters
    let mut query = String::from(
        r#"
        SELECT 
            time,
            body1_id,
            body2_id,
            aspect_type,
            orb,
            applying
        FROM aspects
        WHERE time >= $1 
          AND time <= $2
          AND orb <= $3
        "#,
    );

    let mut has_aspect_types = false;

    // Add aspect type filter if specified
    if let Some(ref types) = criteria.aspect_types {
        if !types.is_empty() {
            query.push_str(" AND aspect_type = ANY($4)");
            has_aspect_types = true;
        }
    }

    // Add body pair filter if specified
    if let Some(ref pairs) = criteria.body_pairs {
        if !pairs.is_empty() {
            // Build (body1_id, body2_id) conditions
            let pair_conditions: Vec<String> = pairs
                .iter()
                .map(|(b1, b2)| {
                    let id1 = b1.to_id();
                    let id2 = b2.to_id();
                    // Ensure ordering (body1_id < body2_id per schema constraint)
                    if id1 < id2 {
                        format!("(body1_id = {} AND body2_id = {})", id1, id2)
                    } else {
                        format!("(body1_id = {} AND body2_id = {})", id2, id1)
                    }
                })
                .collect();

            query.push_str(" AND (");
            query.push_str(&pair_conditions.join(" OR "));
            query.push(')');
        }
    }

    query.push_str(" ORDER BY time, orb");

    // Execute query with bindings
    let mut sqlx_query = sqlx::query(&query)
        .bind(start_datetime)
        .bind(end_datetime)
        .bind(orb_f64);

    // Bind aspect types if present
    if has_aspect_types {
        if let Some(ref types) = criteria.aspect_types {
            let type_ids: Vec<i16> = types.iter().map(|t| t.to_id()).collect();
            sqlx_query = sqlx_query.bind(type_ids);
        }
    }

    let rows = sqlx_query.fetch_all(pool.pool()).await?;

    let rows_examined = rows.len();

    let aspects: Vec<ExactAspect> = rows
        .into_iter()
        .filter_map(|row| {
            let time: DateTime<Utc> = row.try_get("time").ok()?;
            let body1_id: i16 = row.try_get("body1_id").ok()?;
            let body2_id: i16 = row.try_get("body2_id").ok()?;
            let aspect_type_id: i16 = row.try_get("aspect_type").ok()?;
            let orb_f64: f64 = row.try_get("orb").ok()?;
            let applying: bool = row.try_get("applying").ok()?;

            let orb = Decimal::from_f64_retain(orb_f64)?;

            Some(ExactAspect {
                datetime: time,
                body1: Body::from_id(body1_id)?,
                body2: Body::from_id(body2_id)?,
                aspect_type: AspectType::from_id(aspect_type_id)?,
                orb,
                applying,
            })
        })
        .collect();

    let execution_time_ms = start_time.elapsed().as_millis() as u64;

    Ok(QueryResult {
        data: aspects,
        execution_time_ms,
        rows_examined,
        cache_hit: false,
    })
}

/// Find aspects for a specific body pair in a date range
pub async fn find_body_pair_aspects(
    pool: &DatabasePool,
    body1: Body,
    body2: Body,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<QueryResult<ExactAspect>, QueryError> {
    let criteria = AspectCriteria {
        start_date,
        end_date,
        orb_threshold: Decimal::from(1),
        aspect_types: None,
        body_pairs: Some(vec![(body1, body2)]),
    };
    find_exact_aspects(pool, &criteria).await
}

/// Find all aspects of a specific type in a date range
pub async fn find_aspects_by_type(
    pool: &DatabasePool,
    aspect_type: AspectType,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<QueryResult<ExactAspect>, QueryError> {
    let criteria = AspectCriteria {
        start_date,
        end_date,
        orb_threshold: Decimal::from(1),
        aspect_types: Some(vec![aspect_type]),
        body_pairs: None,
    };
    find_exact_aspects(pool, &criteria).await
}

/// Find applying aspects (orb decreasing) in a date range
pub async fn find_applying_aspects(
    pool: &DatabasePool,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<QueryResult<ExactAspect>, QueryError> {
    let criteria = AspectCriteria::new(start_date, end_date);
    let all_aspects = find_exact_aspects(pool, &criteria).await?;

    let applying: Vec<ExactAspect> = all_aspects
        .data
        .into_iter()
        .filter(|a| a.applying)
        .collect();

    Ok(QueryResult {
        rows_examined: applying.len(),
        execution_time_ms: all_aspects.execution_time_ms,
        data: applying,
        cache_hit: all_aspects.cache_hit,
    })
}
