//! API integration tests using the test database infrastructure.
//!
//! These tests require a running TimescaleDB instance with seed data loaded.
//! Run `just test-db-setup` first, then `just test-integration` to execute.
//!
//! All tests in this file are marked `#[ignore]` by default because they
//! require database connectivity. Use `cargo test -- --ignored` or
//! `just test-integration` to run them.

mod common;

/// Verify the test helper module exposes the expected constants.
#[test]
fn test_helper_constants_are_valid() {
    use common::*;

    // Date range constants
    assert_eq!(SEED_START_DATE, "2025-01-01");
    assert_eq!(SEED_END_DATE, "2025-03-02");
    assert_eq!(SEED_DAY_COUNT, 60);

    // Body count
    assert_eq!(TOTAL_BODIES, 10);

    // Parsed dates
    let start = seed_start_date();
    let end = seed_end_date();
    assert!(end > start, "End date should be after start date");

    // Verify the day span matches (SEED_DAY_COUNT is the span, not inclusive count)
    let span = (end - start).num_days() as usize;
    assert_eq!(span, SEED_DAY_COUNT, "Day span should be {} days", SEED_DAY_COUNT);
}

/// Verify body ID constants are consistent.
#[test]
fn test_body_id_constants() {
    use common::body_ids::*;

    assert_eq!(SUN, 0);
    assert_eq!(MOON, 1);
    assert_eq!(MERCURY, 2);
    assert_eq!(VENUS, 3);
    assert_eq!(MARS, 4);
    assert_eq!(JUPITER, 5);
    assert_eq!(SATURN, 6);
    assert_eq!(URANUS, 7);
    assert_eq!(NEPTUNE, 8);
    assert_eq!(PLUTO, 9);
}

/// Verify aspect type constants are consistent.
#[test]
fn test_aspect_type_constants() {
    use common::aspect_type_ids::*;

    assert_eq!(CONJUNCTION, 0);
    assert_eq!(SEXTILE, 1);
    assert_eq!(SQUARE, 2);
    assert_eq!(TRINE, 3);
    assert_eq!(OPPOSITION, 4);
}

/// Verify zodiac sign constants are consistent.
#[test]
fn test_zodiac_sign_constants() {
    use common::zodiac_sign_ids::*;

    assert_eq!(ARIES, 0);
    assert_eq!(TAURUS, 1);
    assert_eq!(GEMINI, 2);
    assert_eq!(CANCER, 3);
    assert_eq!(LEO, 4);
    assert_eq!(VIRGO, 5);
    assert_eq!(LIBRA, 6);
    assert_eq!(SCORPIO, 7);
    assert_eq!(SAGITTARIUS, 8);
    assert_eq!(CAPRICORN, 9);
    assert_eq!(AQUARIUS, 10);
    assert_eq!(PISCES, 11);
}

/// Test database connection and seed data verification.
#[test]
#[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
fn test_seed_data_loaded() {
    use common::*;

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        let pool = test_pool().await;

        // Verify seed data covers the expected date range
        let days = verify_seed_data_loaded(&pool).await;
        assert_day_coverage(days);

        // Verify all 10 bodies have position data
        let counts = position_counts_per_body(&pool).await;
        assert_all_bodies_present(&counts);

        // Each body should have at least one record per day
        for (body_id, count) in &counts {
            assert!(
                *count > 0,
                "Body {} should have position records in seed range",
                body_id,
            );
        }

        pool.close().await;
    });
}

/// Test that aspects table has data for the seed range.
#[test]
#[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
fn test_aspects_present() {
    use common::*;

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        let pool = test_pool().await;

        let aspect_count = count_aspects(&pool).await;
        assert!(
            aspect_count > 0,
            "Aspects table should contain records for the seed range",
        );

        pool.close().await;
    });
}

/// Test that retrograde periods are present in the seed range.
#[test]
#[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
fn test_retrograde_periods_present() {
    use common::*;

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        let pool = test_pool().await;

        let retrograde_count = count_retrograde_periods(&pool).await;
        assert!(
            retrograde_count > 0,
            "Retrograde periods should exist within the 60-day seed range",
        );

        pool.close().await;
    });
}

/// Test that lunar conditions and VoC periods are present.
#[test]
#[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
fn test_lunar_conditions_present() {
    use common::*;

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        let pool = test_pool().await;

        let lunar_count = count_lunar_conditions(&pool).await;
        assert!(
            lunar_count > 0,
            "Lunar conditions should exist for the seed range",
        );

        // VoC periods should also exist in a 60-day range
        let voc_count = count_voc_periods(&pool).await;
        assert!(
            voc_count > 0,
            "VoC Moon periods should exist in a 60-day range",
        );

        // Moon should transit through multiple signs in 60 days
        let moon_signs = moon_signs_in_range(&pool).await;
        assert!(
            moon_signs.len() > 1,
            "Moon should transit through multiple zodiac signs in 60 days",
        );

        pool.close().await;
    });
}
