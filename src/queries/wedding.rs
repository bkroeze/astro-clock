use chrono::{DateTime, Utc};
use sqlx::Row;
use std::time::Instant;

use crate::database::pool::DatabasePool;
use crate::database::schema::body_ids;
use crate::queries::error::QueryError;
use crate::queries::types::{QueryResult, WeddingCandidate, WeddingCriteria, ZodiacSign};

/// Find optimal wedding dates based on astrological criteria
///
/// Criteria:
/// - Moon in favorable signs (Taurus, Cancer, Leo, Libra, Scorpio, Capricorn, Aquarius, Pisces)
/// - Minimum Venus favorable aspects threshold
/// - Exclude void-of-course Moon periods
/// - Sort by Venus aspect count descending, then by date
pub async fn find_wedding_dates(
    pool: &DatabasePool,
    criteria: &WeddingCriteria,
) -> Result<QueryResult<WeddingCandidate>, QueryError> {
    // Validate criteria
    criteria
        .validate()
        .map_err(|e| QueryError::InvalidCriteria(e.to_string()))?;

    let start_time = Instant::now();

    // Favorable Moon signs per CONTEXT.md: Taurus(1), Cancer(3), Leo(4), Libra(6),
    // Scorpio(7), Capricorn(9), Aquarius(10), Pisces(11)
    let favorable_signs = vec![
        1i16, // Taurus
        3,    // Cancer
        4,    // Leo
        6,    // Libra
        7,    // Scorpio
        9,    // Capricorn
        10,   // Aquarius
        11,   // Pisces
    ];

    let start_datetime = criteria.start_date.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let end_datetime = criteria.end_date.and_hms_opt(23, 59, 59).unwrap().and_utc();

    // Optimized query using aspect_summaries JOIN (51× faster than correlated subquery)
    let rows = sqlx::query(
        r#"
        SELECT 
            pp.time,
            pp.zodiac_sign as moon_sign,
            COALESCE(asum.total_favorable, 0) as favorable_aspects
        FROM planet_positions pp
        LEFT JOIN aspect_summaries asum 
            ON pp.time = asum.time 
            AND asum.body_id = $3  -- Venus
        WHERE pp.body_id = $4    -- Moon
          AND pp.time >= $1 
          AND pp.time <= $2
          AND pp.zodiac_sign = ANY($5)
          AND NOT EXISTS (
              SELECT 1 FROM lunar_conditions lc 
              WHERE lc.time = pp.time AND lc.is_void_of_course = true
          )
          AND COALESCE(asum.total_favorable, 0) >= $6
        ORDER BY asum.total_favorable DESC NULLS LAST, pp.time
        LIMIT $7
        "#,
    )
    .bind(start_datetime)
    .bind(end_datetime)
    .bind(body_ids::VENUS)
    .bind(body_ids::MOON)
    .bind(&favorable_signs)
    .bind(criteria.min_venus_aspects)
    .bind(criteria.limit as i64)
    .fetch_all(pool.pool())
    .await?;

    let rows_examined = rows.len();

    let candidates: Vec<WeddingCandidate> = rows
        .into_iter()
        .map(|row| {
            let time: DateTime<Utc> = row.try_get("time").unwrap_or_else(|_| Utc::now());
            let moon_sign_id: i16 = row.try_get("moon_sign").unwrap_or(0);
            let favorable_aspects: i16 = row.try_get("favorable_aspects").unwrap_or(0);

            WeddingCandidate {
                datetime: time,
                moon_sign: ZodiacSign::from_id(moon_sign_id)
                    .unwrap_or(crate::queries::types::ZodiacSign::Aries),
                venus_favorable_aspects: favorable_aspects,
            }
        })
        .collect();

    let execution_time_ms = start_time.elapsed().as_millis() as u64;

    Ok(QueryResult {
        data: candidates,
        execution_time_ms,
        rows_examined,
        cache_hit: false,
    })
}

/// Find wedding dates with extended criteria
///
/// This variant allows specifying custom favorable Moon signs
/// for more personalized wedding date selection.
pub async fn find_wedding_dates_with_signs(
    pool: &DatabasePool,
    criteria: &WeddingCriteria,
    favorable_signs: &[i16],
) -> Result<QueryResult<WeddingCandidate>, QueryError> {
    // Validate criteria
    criteria
        .validate()
        .map_err(|e| QueryError::InvalidCriteria(e.to_string()))?;

    let start_time = Instant::now();

    let start_datetime = criteria.start_date.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let end_datetime = criteria.end_date.and_hms_opt(23, 59, 59).unwrap().and_utc();

    let rows = sqlx::query(
        r#"
        SELECT 
            pp.time,
            pp.zodiac_sign as moon_sign,
            COALESCE(asum.total_favorable, 0) as favorable_aspects
        FROM planet_positions pp
        LEFT JOIN aspect_summaries asum 
            ON pp.time = asum.time 
            AND asum.body_id = $3  -- Venus
        WHERE pp.body_id = $4    -- Moon
          AND pp.time >= $1 
          AND pp.time <= $2
          AND pp.zodiac_sign = ANY($5)
          AND NOT EXISTS (
              SELECT 1 FROM lunar_conditions lc 
              WHERE lc.time = pp.time AND lc.is_void_of_course = true
          )
          AND COALESCE(asum.total_favorable, 0) >= $6
        ORDER BY asum.total_favorable DESC NULLS LAST, pp.time
        LIMIT $7
        "#,
    )
    .bind(start_datetime)
    .bind(end_datetime)
    .bind(body_ids::VENUS)
    .bind(body_ids::MOON)
    .bind(favorable_signs)
    .bind(criteria.min_venus_aspects)
    .bind(criteria.limit as i64)
    .fetch_all(pool.pool())
    .await?;

    let rows_examined = rows.len();

    let candidates: Vec<WeddingCandidate> = rows
        .into_iter()
        .map(|row| {
            let time: DateTime<Utc> = row.try_get("time").unwrap_or_else(|_| Utc::now());
            let moon_sign_id: i16 = row.try_get("moon_sign").unwrap_or(0);
            let favorable_aspects: i16 = row.try_get("favorable_aspects").unwrap_or(0);

            WeddingCandidate {
                datetime: time,
                moon_sign: ZodiacSign::from_id(moon_sign_id)
                    .unwrap_or(crate::queries::types::ZodiacSign::Aries),
                venus_favorable_aspects: favorable_aspects,
            }
        })
        .collect();

    let execution_time_ms = start_time.elapsed().as_millis() as u64;

    Ok(QueryResult {
        data: candidates,
        execution_time_ms,
        rows_examined,
        cache_hit: false,
    })
}
