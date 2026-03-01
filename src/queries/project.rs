use chrono::{DateTime, Utc};
use sqlx::Row;
use std::time::Instant;

use crate::database::pool::DatabasePool;
use crate::database::schema::body_ids;
use crate::queries::error::QueryError;
use crate::queries::types::{
    FAVORABLE_PROJECT_SIGNS, ProjectCandidate, ProjectCriteria, QueryResult, ZodiacSign,
};

/// Find optimal dates for starting projects based on astrological criteria
///
/// Criteria:
/// - Moon in favorable signs (Taurus, Cancer, Leo, Libra, Aquarius, Pisces, Aries)
///   Excludes Scorpio (deep transformation) and Capricorn (heavy restrictions)
/// - Mercury direct (not retrograde) for clear communication and planning
/// - Exclude void-of-course Moon periods
/// - Sort by favorable aspects count descending, then by date
pub async fn find_project_dates(
    pool: &DatabasePool,
    criteria: &ProjectCriteria,
) -> Result<QueryResult<ProjectCandidate>, QueryError> {
    // Validate criteria
    criteria
        .validate()
        .map_err(|e| QueryError::InvalidCriteria(e.to_string()))?;

    let start_time = Instant::now();

    // Convert FAVORABLE_PROJECT_SIGNS to i16 IDs
    let favorable_signs: Vec<i16> = FAVORABLE_PROJECT_SIGNS.iter().map(|s| s.to_id()).collect();

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

    // Optimized query using aspect_summaries JOIN
    // Filters:
    // - Moon in favorable signs
    // - Not during retrograde periods (Mercury direct)
    // - Not void-of-course
    // Orders by favorable aspects to Mercury (for planning/communication)
    let rows = sqlx::query(
        r#"
        SELECT 
            pp.time,
            pp.zodiac_sign as moon_sign,
            COALESCE(asum.total_favorable, 0) as favorable_aspects
        FROM planet_positions pp
        LEFT JOIN aspect_summaries asum 
            ON pp.time = asum.time 
            AND asum.body_id = $3  -- Mercury (for project planning/communication)
        WHERE pp.body_id = $4    -- Moon
          AND pp.time >= $1 
          AND pp.time <= $2
          AND pp.zodiac_sign = ANY($5)
          AND NOT EXISTS (
              SELECT 1 FROM lunar_conditions lc 
              WHERE lc.time = pp.time AND lc.is_void_of_course = true
          )
          AND NOT EXISTS (
              SELECT 1 FROM retrograde_periods rp
              WHERE rp.body_id = $6  -- Mercury
                AND pp.time BETWEEN rp.start_time AND rp.end_time
          )
        ORDER BY asum.total_favorable DESC NULLS LAST, pp.time
        LIMIT $7
        "#,
    )
    .bind(start_datetime)
    .bind(end_datetime)
    .bind(body_ids::MERCURY)
    .bind(body_ids::MOON)
    .bind(&favorable_signs)
    .bind(body_ids::MERCURY)
    .bind(criteria.limit as i64)
    .fetch_all(pool.pool())
    .await?;

    let rows_examined = rows.len();

    let candidates: Vec<ProjectCandidate> = rows
        .into_iter()
        .map(|row| {
            let time: DateTime<Utc> = row.try_get("time").unwrap_or_else(|_| Utc::now());
            let moon_sign_id: i16 = row.try_get("moon_sign").unwrap_or(0);
            let favorable_aspects: i16 = row.try_get("favorable_aspects").unwrap_or(0);

            ProjectCandidate {
                datetime: time,
                moon_sign: ZodiacSign::from_id(moon_sign_id)
                    .unwrap_or(ZodiacSign::Aries),
                mercury_direct: true, // We filter for this in SQL
                favorable_aspects,
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_project_candidate_serialization() {
        let candidate = ProjectCandidate {
            datetime: Utc::now(),
            moon_sign: ZodiacSign::Taurus,
            mercury_direct: true,
            favorable_aspects: 5,
        };

        let json = serde_json::to_string(&candidate).unwrap();
        assert!(json.contains("Taurus"));
        assert!(json.contains("mercury_direct"));
        assert!(json.contains("true"));

        let deserialized: ProjectCandidate = serde_json::from_str(&json).unwrap();
        assert!(deserialized.mercury_direct);
        assert_eq!(deserialized.favorable_aspects, 5);
    }

    #[test]
    fn test_project_criteria_builder() {
        let criteria = ProjectCriteria::new(
            NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 6, 30).unwrap(),
        )
        .with_limit(20);

        assert_eq!(criteria.limit, 20);
        assert!(criteria.validate().is_ok());
    }
}
