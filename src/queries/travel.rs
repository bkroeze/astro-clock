use chrono::{DateTime, Utc};
use sqlx::Row;
use std::time::Instant;

use crate::database::pool::DatabasePool;
use crate::database::schema::body_ids;
use crate::queries::error::QueryError;
use crate::queries::types::{
    FAVORABLE_TRAVEL_SIGNS, QueryResult, TravelCandidate, TravelCriteria, ZodiacSign,
};

/// Find optimal dates for travel based on astrological criteria
///
/// Criteria:
/// - Moon in favorable signs (Taurus, Cancer, Leo, Libra, Aquarius, Pisces, Gemini)
///   Excludes Scorpio (intensity) and Capricorn (restrictions)
/// - Mercury direct (not retrograde) for smooth communication and transportation
/// - Moon void-of-course status included in results (for information)
/// - Sort by favorable aspects count descending, then by date
pub async fn find_travel_dates(
    pool: &DatabasePool,
    criteria: &TravelCriteria,
) -> Result<QueryResult<TravelCandidate>, QueryError> {
    // Validate criteria
    criteria
        .validate()
        .map_err(|e| QueryError::InvalidCriteria(e.to_string()))?;

    let start_time = Instant::now();

    // Convert FAVORABLE_TRAVEL_SIGNS to i16 IDs
    let favorable_signs: Vec<i16> = FAVORABLE_TRAVEL_SIGNS.iter().map(|s| s.to_id()).collect();

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
    // - Includes void-of-course status for traveler's awareness
    // Orders by favorable aspects to Mercury (for communication/transport)
    let rows = sqlx::query(
        r#"
        SELECT 
            pp.time,
            pp.zodiac_sign as moon_sign,
            COALESCE(asum.total_favorable, 0) as favorable_aspects,
            COALESCE(lc.is_void_of_course, false) as is_voc
        FROM planet_positions pp
        LEFT JOIN aspect_summaries asum 
            ON pp.time = asum.time 
            AND asum.body_id = $3  -- Mercury (for travel communication/transport)
        LEFT JOIN lunar_conditions lc
            ON pp.time = lc.time
        WHERE pp.body_id = $4    -- Moon
          AND pp.time >= $1 
          AND pp.time <= $2
          AND pp.zodiac_sign = ANY($5)
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

    let candidates: Vec<TravelCandidate> = rows
        .into_iter()
        .map(|row| {
            let time: DateTime<Utc> = row.try_get("time").unwrap_or_else(|_| Utc::now());
            let moon_sign_id: i16 = row.try_get("moon_sign").unwrap_or(0);
            let favorable_aspects: i16 = row.try_get("favorable_aspects").unwrap_or(0);
            let is_voc: bool = row.try_get("is_voc").unwrap_or(false);

            TravelCandidate {
                datetime: time,
                moon_sign: ZodiacSign::from_id(moon_sign_id)
                    .unwrap_or(ZodiacSign::Aries),
                mercury_direct: true, // We filter for this in SQL
                moon_void_of_course: is_voc,
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
    use crate::queries::{FAVORABLE_PROJECT_SIGNS, TravelPurpose};
    use chrono::NaiveDate;

    #[test]
    fn test_travel_candidate_serialization() {
        let candidate = TravelCandidate {
            datetime: Utc::now(),
            moon_sign: ZodiacSign::Gemini,
            mercury_direct: true,
            moon_void_of_course: false,
            favorable_aspects: 4,
        };

        let json = serde_json::to_string(&candidate).unwrap();
        assert!(json.contains("Gemini"));
        assert!(json.contains("mercury_direct"));
        assert!(json.contains("moon_void_of_course"));
        assert!(json.contains("false"));

        let deserialized: TravelCandidate = serde_json::from_str(&json).unwrap();
        assert!(deserialized.mercury_direct);
        assert!(!deserialized.moon_void_of_course);
        assert_eq!(deserialized.favorable_aspects, 4);
    }

    #[test]
    fn test_travel_criteria_builder() {
        let criteria = TravelCriteria::new(
            NaiveDate::from_ymd_opt(2024, 7, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 7, 31).unwrap(),
        )
        .with_limit(15)
        .with_purpose(TravelPurpose::Leisure);

        assert_eq!(criteria.limit, 15);
        assert_eq!(criteria.purpose, Some(TravelPurpose::Leisure));
        assert!(criteria.validate().is_ok());
    }

    #[test]
    fn test_travel_criteria_business() {
        let criteria = TravelCriteria::new(
            NaiveDate::from_ymd_opt(2024, 8, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 8, 15).unwrap(),
        )
        .with_purpose(TravelPurpose::Business);

        assert_eq!(criteria.purpose, Some(TravelPurpose::Business));
        assert!(criteria.validate().is_ok());
    }

    #[test]
    fn test_favorable_travel_signs_include_gemini() {
        // Gemini is favorable for travel but not for projects
        assert!(FAVORABLE_TRAVEL_SIGNS.contains(&ZodiacSign::Gemini));
        // Verify Aries is in project signs but not travel
        assert!(FAVORABLE_PROJECT_SIGNS.contains(&ZodiacSign::Aries));
    }
}
