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

/// Verify the discovered seed data constants are internally consistent.
#[test]
fn test_discovered_seed_data_constants() {
    use common::*;

    // Total positions = 10 bodies × positions_per_body
    assert_eq!(
        TOTAL_POSITIONS,
        POSITIONS_PER_BODY * TOTAL_BODIES as i64,
        "Total positions should equal positions_per_body × total_bodies"
    );

    // Aspect type counts should sum to total
    let aspect_sum = known_aspects::CONJUNCTIONS
        + known_aspects::SEXTILES
        + known_aspects::SQUARES
        + known_aspects::TRINES
        + known_aspects::OPPOSITIONS;
    assert_eq!(aspect_sum, TOTAL_ASPECTS, "Aspect type counts should sum to total");

    // Lunar conditions = 1 per minute × minutes per day × days
    assert_eq!(
        TOTAL_LUNAR_CONDITIONS,
        POSITIONS_PER_BODY,
        "Lunar conditions should have 1 record per minute for the range"
    );

    // All retrograde bodies should be valid body IDs
    for (body_id, _) in known_retrogrades::RETROGRADE_BODIES {
        assert!(
            *body_id >= 0 && *body_id <= 9,
            "Retrograde body ID {} should be 0-9",
            body_id
        );
    }

    // Moon should visit all 12 signs in 60 days (> 2 full lunar cycles)
    assert!(
        known_lunar::MOON_SIGN_COUNT == TOTAL_ZODIAC_SIGNS,
        "Moon should transit all {} zodiac signs",
        TOTAL_ZODIAC_SIGNS,
    );
}

/// Test database connection and seed data verification with deterministic values.
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

        // Each body should have exactly POSITIONS_PER_BODY records
        for (body_id, count) in &counts {
            assert_eq!(
                *count, POSITIONS_PER_BODY,
                "Body {} should have exactly {} position records",
                body_id, POSITIONS_PER_BODY
            );
        }

        pool.close().await;
    });
}

/// Test that aspects table has the expected deterministic counts.
#[test]
#[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
fn test_aspects_present() {
    use common::*;

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        let pool = test_pool().await;

        let aspect_count = count_aspects(&pool).await;
        assert_eq!(
            aspect_count, TOTAL_ASPECTS,
            "Aspects table should contain exactly {} records for the seed range",
            TOTAL_ASPECTS,
        );

        pool.close().await;
    });
}

/// Test that retrograde bodies match the known deterministic values.
#[test]
#[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
fn test_retrograde_bodies_match_known() {
    use common::*;

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        let pool = test_pool().await;

        // Query retrograde minute counts per body
        let rows: Vec<(i16, i64)> = sqlx::query_as(
            "SELECT body_id, COUNT(*) FROM planet_positions WHERE retrograde = true GROUP BY body_id ORDER BY body_id",
        )
        .fetch_all(&pool)
        .await
        .expect("Failed to query retrograde counts");

        // Verify each known retrograde body is present
        for (known_body, known_minutes) in known_retrogrades::RETROGRADE_BODIES {
            let found = rows.iter().find(|(b, _)| *b == *known_body);
            assert!(
                found.is_some(),
                "Body {} should have retrograde records",
                known_body
            );
            let (_, actual_minutes) = found.unwrap();
            assert_eq!(
                *actual_minutes, *known_minutes,
                "Body {} retrograde minutes should be {}",
                known_body, known_minutes
            );
        }

        // Verify non-retrograde bodies have zero retrograde records
        for non_retro_body in known_retrogrades::NON_RETROGRADE_BODIES {
            let found = rows.iter().find(|(b, _)| *b == *non_retro_body);
            assert!(
                found.is_none(),
                "Body {} should NOT have retrograde records",
                non_retro_body
            );
        }

        pool.close().await;
    });
}

/// Test that lunar conditions match the deterministic values.
#[test]
#[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
fn test_lunar_conditions_match_known() {
    use common::*;

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        let pool = test_pool().await;

        // Total lunar conditions
        let lunar_count = count_lunar_conditions(&pool).await;
        assert_eq!(
            lunar_count, TOTAL_LUNAR_CONDITIONS,
            "Lunar conditions should be {}",
            TOTAL_LUNAR_CONDITIONS,
        );

        // VoC periods — count distinct VoC transitions
        let voc_count = count_voc_periods(&pool).await;
        assert_eq!(
            voc_count, known_lunar::VOC_MINUTES,
            "VoC minutes should be {}",
            known_lunar::VOC_MINUTES,
        );

        // Moon signs — should cover all 12 signs
        let moon_signs = moon_signs_in_range(&pool).await;
        assert_eq!(
            moon_signs.len(),
            known_lunar::MOON_SIGN_COUNT,
            "Moon should transit through {} zodiac signs",
            known_lunar::MOON_SIGN_COUNT,
        );

        pool.close().await;
    });
}
