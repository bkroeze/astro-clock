//! Shared test helper module for integration tests.
//!
//! Provides:
//! - Database connection helper (reads TEST_PG_URL, creates pool)
//! - Seed data constants (date range, known planet positions, expected query results)
//! - Test fixture functions (verify seed data loaded, get specific position assertions)
//!
//! This module is shared across integration test files via `mod common;`.

#![allow(dead_code)] // Public API for future integration tests

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::postgres::PgPoolOptions;
use sqlx::Pool;
use sqlx::Postgres;

// ============================================================================
// SEED DATA CONSTANTS
// ============================================================================

/// Seed data start date (inclusive)
pub const SEED_START_DATE: &str = "2025-01-01";

/// Seed data end date (inclusive)
pub const SEED_END_DATE: &str = "2025-03-02";

/// Number of days covered by seed data
pub const SEED_DAY_COUNT: usize = 60;

/// Total number of celestial bodies (Sun through Pluto)
pub const TOTAL_BODIES: usize = 10;

/// Number of zodiac signs
pub const TOTAL_ZODIAC_SIGNS: usize = 12;

/// Number of aspect types
pub const TOTAL_ASPECT_TYPES: usize = 5;

/// Seed data start as NaiveDate
pub fn seed_start_date() -> NaiveDate {
    NaiveDate::parse_from_str(SEED_START_DATE, "%Y-%m-%d").unwrap()
}

/// Seed data end as NaiveDate
pub fn seed_end_date() -> NaiveDate {
    NaiveDate::parse_from_str(SEED_END_DATE, "%Y-%m-%d").unwrap()
}

/// Seed data start as DateTime<Utc> (midnight UTC)
pub fn seed_start_utc() -> DateTime<Utc> {
    seed_start_date()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
}

/// Seed data end as DateTime<Utc> (end of day)
pub fn seed_end_utc() -> DateTime<Utc> {
    seed_end_date()
        .and_hms_opt(23, 59, 59)
        .unwrap()
        .and_utc()
}

/// Body ID constants (matching database schema)
pub mod body_ids {
    pub const SUN: i16 = 0;
    pub const MOON: i16 = 1;
    pub const MERCURY: i16 = 2;
    pub const VENUS: i16 = 3;
    pub const MARS: i16 = 4;
    pub const JUPITER: i16 = 5;
    pub const SATURN: i16 = 6;
    pub const URANUS: i16 = 7;
    pub const NEPTUNE: i16 = 8;
    pub const PLUTO: i16 = 9;
}

/// Aspect type constants (matching database schema)
pub mod aspect_type_ids {
    pub const CONJUNCTION: i16 = 0;
    pub const SEXTILE: i16 = 1;
    pub const SQUARE: i16 = 2;
    pub const TRINE: i16 = 3;
    pub const OPPOSITION: i16 = 4;
}

/// Zodiac sign ID constants (matching database schema)
pub mod zodiac_sign_ids {
    pub const ARIES: i16 = 0;
    pub const TAURUS: i16 = 1;
    pub const GEMINI: i16 = 2;
    pub const CANCER: i16 = 3;
    pub const LEO: i16 = 4;
    pub const VIRGO: i16 = 5;
    pub const LIBRA: i16 = 6;
    pub const SCORPIO: i16 = 7;
    pub const SAGITTARIUS: i16 = 8;
    pub const CAPRICORN: i16 = 9;
    pub const AQUARIUS: i16 = 10;
    pub const PISCES: i16 = 11;
}

/// Moon phase constants (matching database schema)
pub mod moon_phase_ids {
    pub const NEW: i16 = 0;
    pub const WAXING_CRESCENT: i16 = 1;
    pub const FIRST_QUARTER: i16 = 2;
    pub const WAXING_GIBBOUS: i16 = 3;
    pub const FULL: i16 = 4;
    pub const WANING_GIBBOUS: i16 = 5;
    pub const LAST_QUARTER: i16 = 6;
    pub const WANING_CRESCENT: i16 = 7;
}

// ============================================================================
// KNOWN ASTRONOMICAL VALUES FOR SEED RANGE
// Queried from test database after loading 60 days of seed data (2025-01-01 to 2025-03-02)
// ============================================================================

/// Total position records in seed data (10 bodies × 86400 minutes/day × 60 days)
pub const TOTAL_POSITIONS: i64 = 864_000;

/// Positions per body per day (1440 minutes × 60 days = 86400)
pub const POSITIONS_PER_BODY: i64 = 86_400;

/// Total aspect records in seed data
pub const TOTAL_ASPECTS: i64 = 1_465_067;

/// Total lunar conditions records in seed data (86400 minutes × 1 day-condition)
pub const TOTAL_LUNAR_CONDITIONS: i64 = 86_400;

/// Retrograde bodies during seed range with their retrograde minute counts.
/// Venus (3): 1402 min, Mars (4): 76441 min, Jupiter (5): 48102 min, Uranus (7): 41304 min
pub mod known_retrogrades {
    /// Bodies that are retrograde during the seed range, with their total retrograde minutes
    pub const RETROGRADE_BODIES: &[(i16, i64)] = &[
        (3, 1_402),   // Venus — brief retrograde at start of range
        (4, 76_441),  // Mars — retrograde majority of range
        (5, 48_102),  // Jupiter — retrograde most of range
        (7, 41_304),  // Uranus — retrograde much of range
    ];

    /// Bodies that are NOT retrograde during the seed range
    pub const NON_RETROGRADE_BODIES: &[i16] = &[
        0, // Sun — never retrograde
        1, // Moon — never retrograde
        2, // Mercury — not retrograde in this range
        6, // Saturn — not retrograde in this range
        8, // Neptune — not retrograde in this range
        9, // Pluto — not retrograde in this range
    ];

    /// Retrograde days per body
    pub const RETROGRADE_DAYS: &[(i16, i64)] = &[
        (3, 1),   // Venus
        (4, 54),  // Mars
        (5, 34),  // Jupiter
        (7, 29),  // Uranus
    ];

    /// Zodiac signs occupied by retrograde bodies
    pub const RETROGRADE_ZODIAC_SIGNS: &[(i16, i16)] = &[
        (3, 0),   // Venus in Aries
        (4, 3),   // Mars in Cancer
        (4, 4),   // Mars in Leo
        (5, 2),   // Jupiter in Gemini
        (7, 1),   // Uranus in Taurus
    ];
}

/// Aspect count breakdown by type
pub mod known_aspects {
    pub const CONJUNCTIONS: i64 = 178_226;
    pub const SEXTILES: i64 = 630_355;
    pub const SQUARES: i64 = 265_324;
    pub const TRINES: i64 = 295_184;
    pub const OPPOSITIONS: i64 = 95_978;
}

/// Known lunar data for the seed range
pub mod known_lunar {
    /// Number of distinct void-of-course periods
    pub const VOC_PERIODS: i64 = 68;

    /// Total VoC minutes
    pub const VOC_MINUTES: i64 = 48_294;

    /// Number of Moon sign changes (transits through zodiac)
    pub const MOON_SIGN_CHANGES: i64 = 27;

    /// All 12 zodiac signs are visited by the Moon
    pub const MOON_SIGN_COUNT: usize = 12;

    /// All 8 Moon phases appear in the range
    pub const MOON_PHASE_COUNT: usize = 8;
}

/// Known planetary longitude values for validation.
///
/// These provide sanity-check bounds for positions that should be present
/// in the seed data. Longitudes are always 0-360 degrees.
pub mod known_positions {
    /// Sun longitude on 2025-01-01 00:00 UTC: ~280.8° (Capricorn, sign 9)
    pub const SUN_LONGITUDE_JAN01_APPROX: (f64, f64) = (280.0, 282.0);
    /// Sun longitude on 2025-03-02 00:00 UTC: ~341.7° (Pisces, sign 11)
    pub const SUN_LONGITUDE_MAR02_APPROX: (f64, f64) = (340.0, 343.0);
    /// Moon moves ~13 degrees per day, covering all 360° in ~27.3 days
    pub const MOON_LONGITUDE_RANGE: (f64, f64) = (0.0, 360.0);
}

// ============================================================================
// DATABASE CONNECTION HELPER
// ============================================================================

/// Get the TEST_PG_URL from the environment, panicking with a clear message if unset.
pub fn test_pg_url() -> String {
    std::env::var("TEST_PG_URL").expect(
        "TEST_PG_URL must be set for integration tests. \
         Run `just test-db-setup` first, or set it manually.",
    )
}

/// Create a new database connection pool using TEST_PG_URL.
///
/// Uses a small pool (max 5 connections) suitable for test use.
/// This function reads TEST_PG_URL from the environment and connects.
pub async fn test_pool() -> Pool<Postgres> {
    let url = test_pg_url();
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .expect("Failed to connect to test database. Is TimescaleDB running?")
}

/// Create a database pool from an explicit URL (useful for parameterized tests).
pub async fn pool_from_url(url: &str) -> Pool<Postgres> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(url)
        .await
        .expect("Failed to connect to database")
}

// ============================================================================
// TEST FIXTURE FUNCTIONS
// ============================================================================

/// Verify that seed data is loaded by checking planet_positions has rows
/// for the expected date range.
///
/// Returns the count of distinct dates found in planet_positions.
pub async fn verify_seed_data_loaded(pool: &Pool<Postgres>) -> usize {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT DATE(time)) FROM planet_positions WHERE time >= $1 AND time <= $2",
    )
    .bind(seed_start_utc())
    .bind(seed_end_utc())
    .fetch_one(pool)
    .await
    .expect("Failed to query planet_positions for seed data verification");

    count.0 as usize
}

/// Get the count of planet_position records for a specific body in the seed range.
pub async fn count_positions_for_body(pool: &Pool<Postgres>, body_id: i16) -> i64 {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM planet_positions WHERE body_id = $1 AND time >= $2 AND time <= $3",
    )
    .bind(body_id)
    .bind(seed_start_utc())
    .bind(seed_end_utc())
    .fetch_one(pool)
    .await
    .expect("Failed to count positions for body");

    count.0
}

/// Get the count of aspects in the seed range.
pub async fn count_aspects(pool: &Pool<Postgres>) -> i64 {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM aspects WHERE time >= $1 AND time <= $2",
    )
    .bind(seed_start_utc())
    .bind(seed_end_utc())
    .fetch_one(pool)
    .await
    .expect("Failed to count aspects");

    count.0
}

/// Get the count of retrograde periods in the seed range.
pub async fn count_retrograde_periods(pool: &Pool<Postgres>) -> i64 {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM retrograde_periods WHERE retrograde_start <= $1 OR retrograde_end >= $2",
    )
    .bind(seed_end_utc())
    .bind(seed_start_utc())
    .fetch_one(pool)
    .await
    .expect("Failed to count retrograde periods");

    count.0
}

/// Get the count of lunar conditions records in the seed range.
pub async fn count_lunar_conditions(pool: &Pool<Postgres>) -> i64 {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM lunar_conditions WHERE time >= $1 AND time <= $2",
    )
    .bind(seed_start_utc())
    .bind(seed_end_utc())
    .fetch_one(pool)
    .await
    .expect("Failed to count lunar conditions");

    count.0
}

/// Get the count of VoC (void-of-course) Moon periods in the seed range.
pub async fn count_voc_periods(pool: &Pool<Postgres>) -> i64 {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM lunar_conditions WHERE is_void_of_course = true AND time >= $1 AND time <= $2",
    )
    .bind(seed_start_utc())
    .bind(seed_end_utc())
    .fetch_one(pool)
    .await
    .expect("Failed to count VoC periods");

    count.0
}

/// Get the distinct zodiac signs the Moon transits through in the seed range.
pub async fn moon_signs_in_range(pool: &Pool<Postgres>) -> Vec<i16> {
    let rows: Vec<(i16,)> = sqlx::query_as(
        "SELECT DISTINCT moon_sign FROM lunar_conditions WHERE time >= $1 AND time <= $2 ORDER BY moon_sign",
    )
    .bind(seed_start_utc())
    .bind(seed_end_utc())
    .fetch_all(pool)
    .await
    .expect("Failed to query Moon signs");

    rows.into_iter().map(|r| r.0).collect()
}

/// Assert that seed data covers the expected number of days.
/// Panics with a descriptive message if the count doesn't match.
pub fn assert_day_coverage(actual_days: usize) {
    assert!(
        actual_days >= SEED_DAY_COUNT,
        "Expected at least {} days of seed data, found {}",
        SEED_DAY_COUNT,
        actual_days,
    );
}

/// Assert that all 10 bodies have position data.
/// Takes a map of body_id -> row_count and verifies each body has rows.
pub fn assert_all_bodies_present(body_counts: &[(i16, i64)]) {
    let bodies_found: std::collections::HashSet<i16> =
        body_counts.iter().map(|(id, _)| *id).collect();

    for body_id in 0..10 {
        assert!(
            bodies_found.contains(&body_id),
            "Body ID {} missing from planet_positions",
            body_id,
        );
    }
}

/// Get position count per body for the entire seed range.
/// Returns sorted by body_id.
pub async fn position_counts_per_body(pool: &Pool<Postgres>) -> Vec<(i16, i64)> {
    let rows: Vec<(i16, i64)> = sqlx::query_as(
        "SELECT body_id, COUNT(*) FROM planet_positions WHERE time >= $1 AND time <= $2 GROUP BY body_id ORDER BY body_id",
    )
    .bind(seed_start_utc())
    .bind(seed_end_utc())
    .fetch_all(pool)
    .await
    .expect("Failed to get position counts per body");

    rows
}
