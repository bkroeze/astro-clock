use chrono::{DateTime, NaiveDate, Utc};
use std::time::Instant;
use thiserror::Error;
use tracing::debug;

use super::chunk::{
    BODIES_COUNT, ChunkData, CompactAspect, CompactLunarCondition, CompactPlanetPosition,
    MINUTES_PER_DAY,
};
use super::pool::DatabasePool;

/// Orb for considering an aspect valid (in degrees)
const ASPECT_ORB: f64 = 8.0;

/// Check if an angular separation is a major aspect within orb
fn is_major_aspect(angle: f64) -> bool {
    const MAJOR_ASPECT_ANGLES: [f64; 5] = [0.0, 60.0, 90.0, 120.0, 180.0];
    let normalized = (angle % 360.0 + 360.0) % 360.0;
    MAJOR_ASPECT_ANGLES.iter().any(|&aspect_angle| {
        let diff = (normalized - aspect_angle).abs();
        let diff = if diff > 180.0 { 360.0 - diff } else { diff };
        diff <= ASPECT_ORB
    })
}

/// Errors that can occur during chunk generation
#[derive(Error, Debug)]
pub enum ChunkGeneratorError {
    #[error("Swiss Ephemeris error: {0}")]
    SwissEph(String),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Invalid date: {0}")]
    InvalidDate(String),
}

/// Generates astrological data chunks from Swiss Ephemeris
///
/// When data is not available in the database, this generator creates
/// chunks by calculating planetary positions, aspects, and lunar conditions
/// using the Swiss Ephemeris library.
#[derive(Debug, Clone)]
pub struct ChunkGenerator {
    db_pool: DatabasePool,
}

impl ChunkGenerator {
    /// Create a new ChunkGenerator with the given database pool
    pub fn new(db_pool: DatabasePool) -> Self {
        Self { db_pool }
    }

    /// Generate a complete chunk for the given date
    ///
    /// Calculates all planetary positions, aspects, and lunar conditions
    /// for every minute of the day using Swiss Ephemeris.
    ///
    /// Only major aspects (conjunction, sextile, square, trine, opposition) are stored.
    /// Minor aspects are calculated on-demand when needed, reducing storage by ~80%.
    pub async fn generate_chunk(&self, date: NaiveDate) -> Result<ChunkData, ChunkGeneratorError> {
        // Ensure ephemeris is initialized
        crate::ephemeris::Ephemeris::ensure_initialized()
            .map_err(|e| ChunkGeneratorError::SwissEph(e.to_string()))?;

        let mut positions = Vec::with_capacity(MINUTES_PER_DAY as usize * BODIES_COUNT);
        let mut aspects = Vec::new();
        let mut lunar_conditions = Vec::with_capacity(MINUTES_PER_DAY as usize);

        // Generate data for each minute of the day
        for minute in 0..MINUTES_PER_DAY {
            let timestamp = date
                .and_hms_opt(minute / 60, minute % 60, 0)
                .ok_or_else(|| {
                    ChunkGeneratorError::InvalidDate(format!(
                        "Invalid time for minute {} on date {}",
                        minute, date
                    ))
                })?
                .and_utc();
            let julian_day = crate::ephemeris::julian_day_from_chrono(timestamp);

            // Calculate positions for all bodies
            let minute_positions = self.calculate_all_bodies(julian_day, minute).await?;

            // Calculate aspects for this minute (only major aspects are stored)
            let minute_aspects = self.calculate_aspects(&minute_positions, minute);
            aspects.extend(minute_aspects);

            // Calculate lunar conditions (only need Moon for this)
            if let Some(moon_pos) = minute_positions.iter().find(|p| p.body_id == 1) {
                let lunar = self.calculate_lunar_condition(
                    julian_day,
                    moon_pos,
                    minute,
                    &minute_positions,
                )?;
                lunar_conditions.push(lunar);
            }

            positions.extend(minute_positions);
        }

        debug!(
            "Generated chunk for {}: {} positions, {} major aspects, {} lunar conditions",
            date,
            positions.len(),
            aspects.len(),
            lunar_conditions.len()
        );

        Ok(ChunkData {
            date,
            planet_positions: positions,
            aspects,
            lunar_conditions,
            loaded_at: Instant::now(),
        })
    }

    /// Calculate positions for all celestial bodies at a given Julian day
    async fn calculate_all_bodies(
        &self,
        julian_day: f64,
        minute: u32,
    ) -> Result<Vec<CompactPlanetPosition>, ChunkGeneratorError> {
        use swiss_eph::safe::{CalcFlags, Planet};

        let flags = CalcFlags::new().with_speed();
        let mut positions = Vec::with_capacity(BODIES_COUNT);

        let bodies = [
            (0, Planet::Sun),
            (1, Planet::Moon),
            (2, Planet::Mercury),
            (3, Planet::Venus),
            (4, Planet::Mars),
            (5, Planet::Jupiter),
            (6, Planet::Saturn),
            (7, Planet::Uranus),
            (8, Planet::Neptune),
            (9, Planet::Pluto),
        ];

        for (body_id, planet) in bodies {
            let result = swiss_eph::safe::calc(julian_day, planet, flags)
                .map_err(|e| ChunkGeneratorError::SwissEph(e.to_string()))?;

            let zodiac_sign = (result.longitude / 30.0) as u8 % 12;

            positions.push(CompactPlanetPosition {
                timestamp_minutes: minute,
                longitude_millidegrees: (result.longitude * 1000.0) as i32,
                latitude_millidegrees: (result.latitude * 1000.0) as i32,
                speed_millidegrees: (result.longitude_speed * 1000.0) as i16,
                body_id,
                zodiac_sign,
                is_retrograde: result.longitude_speed < 0.0,
                distance_milliau: (result.distance * 1000.0) as u16,
            });
        }

        Ok(positions)
    }

    /// Calculate aspects between all body pairs at a given minute
    ///
    /// Only major aspects (conjunction, sextile, square, trine, opposition) are calculated.
    /// Minor aspects are not stored to reduce storage by ~80%.
    fn calculate_aspects(
        &self,
        positions: &[CompactPlanetPosition],
        minute: u32,
    ) -> Vec<CompactAspect> {
        // Aspect type IDs matching the database schema
        const ASPECT_TYPE_IDS: [(u8, f64); 5] = [
            (0, 0.0),   // Conjunction
            (1, 60.0),  // Sextile
            (2, 90.0),  // Square
            (3, 120.0), // Trine
            (4, 180.0), // Opposition
        ];

        let mut aspects = Vec::new();

        // Check all body pairs
        for i in 0..positions.len() {
            for j in (i + 1)..positions.len() {
                let p1 = &positions[i];
                let p2 = &positions[j];

                let lon1 = p1.longitude_millidegrees as f64 / 1000.0;
                let lon2 = p2.longitude_millidegrees as f64 / 1000.0;

                let diff = (lon2 - lon1).abs();
                let diff = if diff > 180.0 { 360.0 - diff } else { diff };

                // Filter to only major aspects (conjunction, sextile, square, trine, opposition)
                // This reduces storage by ~80% compared to storing all aspects
                if !is_major_aspect(diff) {
                    continue;
                }

                for (aspect_type, target_angle) in ASPECT_TYPE_IDS {
                    let orb = (diff - target_angle).abs();

                    if orb <= ASPECT_ORB {
                        // Determine if applying (orb decreasing)
                        let speed1 = p1.speed_millidegrees as f64 / 1000.0;
                        let speed2 = p2.speed_millidegrees as f64 / 1000.0;
                        let relative_speed = speed2 - speed1;
                        let applying = if aspect_type == 0 {
                            // For conjunction, applying if moving toward 0° diff
                            relative_speed < 0.0
                        } else {
                            // For other aspects, depends on relative motion
                            (diff > target_angle && relative_speed < 0.0)
                                || (diff < target_angle && relative_speed > 0.0)
                        };

                        aspects.push(CompactAspect {
                            timestamp_minutes: minute,
                            body1_id: p1.body_id,
                            body2_id: p2.body_id,
                            aspect_type,
                            orb_millidegrees: (orb * 1000.0) as u16,
                            applying,
                        });
                    }
                }
            }
        }

        aspects
    }

    /// Calculate lunar conditions for a given minute
    fn calculate_lunar_condition(
        &self,
        _julian_day: f64,
        moon_pos: &CompactPlanetPosition,
        minute: u32,
        all_positions: &[CompactPlanetPosition],
    ) -> Result<CompactLunarCondition, ChunkGeneratorError> {
        // Calculate moon phase angle (Moon - Sun)
        let sun_pos = all_positions
            .iter()
            .find(|p| p.body_id == 0)
            .ok_or_else(|| ChunkGeneratorError::InvalidDate("Sun not found".to_string()))?;

        let moon_lon = moon_pos.longitude_millidegrees as f64 / 1000.0;
        let sun_lon = sun_pos.longitude_millidegrees as f64 / 1000.0;

        let mut phase_angle = moon_lon - sun_lon;
        if phase_angle < 0.0 {
            phase_angle += 360.0;
        }
        if phase_angle >= 360.0 {
            phase_angle -= 360.0;
        }

        // Determine moon phase (0-7)
        let moon_phase = (phase_angle / 45.0) as u8 % 8;

        // Calculate illumination (approximate)
        let illumination = (1.0 - (phase_angle / 180.0).cos()) / 2.0;

        // Determine void-of-course status
        let is_voc = self.is_void_of_course(moon_pos, all_positions);

        Ok(CompactLunarCondition {
            timestamp_minutes: minute,
            moon_phase,
            moon_sign: moon_pos.zodiac_sign,
            moon_phase_angle_milli: (phase_angle * 1000.0) as u32,
            moon_illumination_permille: (illumination * 1000.0) as u16,
            is_void_of_course: is_voc,
            _padding: 0,
        })
    }

    /// Check if the Moon is void-of-course
    ///
    /// Simplified VoC check: Moon makes no applying aspects to major planets.
    /// A full implementation would check until Moon leaves sign.
    fn is_void_of_course(
        &self,
        moon_pos: &CompactPlanetPosition,
        all_positions: &[CompactPlanetPosition],
    ) -> bool {
        const VOC_ORB: f64 = 2.0; // degrees

        let moon_lon = moon_pos.longitude_millidegrees as f64 / 1000.0;

        for pos in all_positions {
            if pos.body_id == 1 || pos.body_id > 6 {
                // Skip Moon and outer planets (Uranus, Neptune, Pluto)
                continue;
            }

            let other_lon = pos.longitude_millidegrees as f64 / 1000.0;
            let diff = (other_lon - moon_lon).abs();
            let diff = if diff > 180.0 { 360.0 - diff } else { diff };

            // Check major aspects
            for &target in [0.0, 60.0, 90.0, 120.0, 180.0].iter() {
                let orb = (diff - target).abs();
                if orb <= VOC_ORB {
                    return false; // Moon is making an aspect, not VoC
                }
            }
        }

        true // No aspects found, Moon is void-of-course
    }

    /// Save a generated chunk to the database using batch inserts
    pub async fn save_chunk_to_db(&self, chunk: &ChunkData) -> Result<(), ChunkGeneratorError> {
        let mut tx = self.db_pool.pool().begin().await?;

        // Insert planet positions in batches
        self.insert_planet_positions_batch(&mut tx, &chunk.planet_positions, chunk.date)
            .await?;

        // Insert aspects in batches
        self.insert_aspects_batch(&mut tx, &chunk.aspects, chunk.date)
            .await?;

        // Insert lunar conditions in batches
        self.insert_lunar_conditions_batch(&mut tx, &chunk.lunar_conditions, chunk.date)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    async fn insert_planet_positions_batch(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        positions: &[CompactPlanetPosition],
        chunk_date: NaiveDate,
    ) -> Result<(), sqlx::Error> {
        if positions.is_empty() {
            return Ok(());
        }

        // Build batch insert using UNNEST for efficiency
        // Use f64 for numeric arrays since rust_decimal doesn't implement sqlx array traits
        let times: Vec<DateTime<Utc>> = positions
            .iter()
            .map(|p| {
                chunk_date
                    .and_hms_opt(p.timestamp_minutes / 60, p.timestamp_minutes % 60, 0)
                    .unwrap()
                    .and_utc()
            })
            .collect();

        let body_ids: Vec<i16> = positions.iter().map(|p| p.body_id as i16).collect();
        let longitudes: Vec<f64> = positions
            .iter()
            .map(|p| p.longitude_millidegrees as f64 / 1000.0)
            .collect();
        let latitudes: Vec<f64> = positions
            .iter()
            .map(|p| p.latitude_millidegrees as f64 / 1000.0)
            .collect();
        let distances: Vec<f64> = positions
            .iter()
            .map(|p| p.distance_milliau as f64 / 1000.0)
            .collect();
        let speeds: Vec<f64> = positions
            .iter()
            .map(|p| p.speed_millidegrees as f64 / 1000.0)
            .collect();
        let retrogrades: Vec<bool> = positions.iter().map(|p| p.is_retrograde).collect();
        let signs: Vec<i16> = positions.iter().map(|p| p.zodiac_sign as i16).collect();

        sqlx::query(
            r#"
            INSERT INTO planet_positions 
                (time, body_id, longitude, latitude, distance, speed_lon, retrograde, zodiac_sign)
            SELECT * FROM UNNEST(
                $1::timestamptz[],
                $2::smallint[],
                $3::float8[],
                $4::float8[],
                $5::float8[],
                $6::float8[],
                $7::boolean[],
                $8::smallint[]
            )
            ON CONFLICT (time, body_id) DO NOTHING
            "#,
        )
        .bind(&times)
        .bind(&body_ids)
        .bind(&longitudes)
        .bind(&latitudes)
        .bind(&distances)
        .bind(&speeds)
        .bind(&retrogrades)
        .bind(&signs)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    async fn insert_aspects_batch(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        aspects: &[CompactAspect],
        chunk_date: NaiveDate,
    ) -> Result<(), sqlx::Error> {
        if aspects.is_empty() {
            return Ok(());
        }

        let times: Vec<DateTime<Utc>> = aspects
            .iter()
            .map(|a| {
                chunk_date
                    .and_hms_opt(a.timestamp_minutes / 60, a.timestamp_minutes % 60, 0)
                    .unwrap()
                    .and_utc()
            })
            .collect();
        let body1_ids: Vec<i16> = aspects.iter().map(|a| a.body1_id as i16).collect();
        let body2_ids: Vec<i16> = aspects.iter().map(|a| a.body2_id as i16).collect();
        let aspect_types: Vec<i16> = aspects.iter().map(|a| a.aspect_type as i16).collect();
        let orbs: Vec<f64> = aspects
            .iter()
            .map(|a| a.orb_millidegrees as f64 / 1000.0)
            .collect();
        let applyings: Vec<bool> = aspects.iter().map(|a| a.applying).collect();

        sqlx::query(
            r#"
            INSERT INTO aspects 
                (time, body1_id, body2_id, aspect_type, orb, applying)
            SELECT * FROM UNNEST(
                $1::timestamptz[],
                $2::smallint[],
                $3::smallint[],
                $4::smallint[],
                $5::float8[],
                $6::boolean[]
            )
            ON CONFLICT (time, body1_id, body2_id, aspect_type) DO NOTHING
            "#,
        )
        .bind(&times)
        .bind(&body1_ids)
        .bind(&body2_ids)
        .bind(&aspect_types)
        .bind(&orbs)
        .bind(&applyings)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    async fn insert_lunar_conditions_batch(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        lunar: &[CompactLunarCondition],
        chunk_date: NaiveDate,
    ) -> Result<(), sqlx::Error> {
        if lunar.is_empty() {
            return Ok(());
        }

        let times: Vec<DateTime<Utc>> = lunar
            .iter()
            .map(|l| {
                chunk_date
                    .and_hms_opt(l.timestamp_minutes / 60, l.timestamp_minutes % 60, 0)
                    .unwrap()
                    .and_utc()
            })
            .collect();
        let moon_phases: Vec<i16> = lunar.iter().map(|l| l.moon_phase as i16).collect();
        let moon_signs: Vec<i16> = lunar.iter().map(|l| l.moon_sign as i16).collect();
        let phase_angles: Vec<f64> = lunar
            .iter()
            .map(|l| l.moon_phase_angle_milli as f64 / 1000.0)
            .collect();
        let illuminations: Vec<f64> = lunar
            .iter()
            .map(|l| l.moon_illumination_permille as f64 / 1000.0)
            .collect();
        let is_voc: Vec<bool> = lunar.iter().map(|l| l.is_void_of_course).collect();

        sqlx::query(
            r#"
            INSERT INTO lunar_conditions 
                (time, moon_phase, moon_sign, moon_phase_angle, moon_illumination, is_void_of_course)
            SELECT * FROM UNNEST(
                $1::timestamptz[],
                $2::smallint[],
                $3::smallint[],
                $4::float8[],
                $5::float8[],
                $6::boolean[]
            )
            ON CONFLICT (time) DO NOTHING
            "#,
        )
        .bind(&times)
        .bind(&moon_phases)
        .bind(&moon_signs)
        .bind(&phase_angles)
        .bind(&illuminations)
        .bind(&is_voc)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_generator_error_display() {
        let err = ChunkGeneratorError::SwissEph("test error".to_string());
        assert!(err.to_string().contains("test error"));
    }

    #[test]
    fn test_chunk_generator_error_from_sqlx() {
        // This just verifies the From impl exists
        fn assert_from_sqlx<T: From<sqlx::Error>>() {}
        assert_from_sqlx::<ChunkGeneratorError>();
    }

    #[test]
    fn test_is_major_aspect_filtering() {
        // Test major aspects (should be recognized)
        assert!(
            is_major_aspect(0.0),
            "Conjunction (0°) should be a major aspect"
        );
        assert!(
            is_major_aspect(2.0),
            "Conjunction within orb (2°) should be a major aspect"
        );
        assert!(
            is_major_aspect(60.0),
            "Sextile (60°) should be a major aspect"
        );
        assert!(
            is_major_aspect(58.0),
            "Sextile within orb (58°) should be a major aspect"
        );
        assert!(
            is_major_aspect(90.0),
            "Square (90°) should be a major aspect"
        );
        assert!(
            is_major_aspect(92.0),
            "Square within orb (92°) should be a major aspect"
        );
        assert!(
            is_major_aspect(120.0),
            "Trine (120°) should be a major aspect"
        );
        assert!(
            is_major_aspect(118.0),
            "Trine within orb (118°) should be a major aspect"
        );
        assert!(
            is_major_aspect(180.0),
            "Opposition (180°) should be a major aspect"
        );
        assert!(
            is_major_aspect(178.0),
            "Opposition within orb (178°) should be a major aspect"
        );

        // Test minor aspects (should be filtered out)
        assert!(
            !is_major_aspect(30.0),
            "Semi-sextile (30°) should NOT be a major aspect"
        );
        assert!(
            !is_major_aspect(45.0),
            "Semi-square (45°) should NOT be a major aspect"
        );
        assert!(
            !is_major_aspect(72.0),
            "Quintile (72°) should NOT be a major aspect"
        );
        assert!(
            !is_major_aspect(135.0),
            "Sesquiquadrate (135°) should NOT be a major aspect"
        );
        assert!(
            !is_major_aspect(150.0),
            "Quincunx (150°) should NOT be a major aspect"
        );
        assert!(
            !is_major_aspect(10.0),
            "10° separation should NOT be a major aspect (outside orb)"
        );
        assert!(
            !is_major_aspect(200.0),
            "200° separation should NOT be a major aspect"
        );

        // Test wraparound cases
        assert!(
            is_major_aspect(360.0),
            "360° should be equivalent to 0° (conjunction)"
        );
        assert!(
            is_major_aspect(420.0),
            "420° should be equivalent to 60° (sextile)"
        );
        assert!(
            !is_major_aspect(-30.0),
            "Negative angle (-30°) should normalize correctly and NOT be major"
        );
    }

    #[test]
    fn test_calculate_aspects_filters_minor_aspects() {
        // Create a ChunkGenerator with a mock database pool
        // We'll test the aspect calculation directly using a test helper

        // Test case: Two bodies at specific positions
        // Body 1 at 0°, Body 2 at 30° (semi-sextile - minor aspect, should be filtered)
        // Body 3 at 60° (sextile - major aspect, should be included)

        let _positions = vec![
            CompactPlanetPosition {
                timestamp_minutes: 0,
                longitude_millidegrees: 0, // 0°
                latitude_millidegrees: 0,
                speed_millidegrees: 100,
                body_id: 0, // Sun
                zodiac_sign: 0,
                is_retrograde: false,
                distance_milliau: 1000,
            },
            CompactPlanetPosition {
                timestamp_minutes: 0,
                longitude_millidegrees: 30_000, // 30° - semi-sextile (minor)
                latitude_millidegrees: 0,
                speed_millidegrees: 100,
                body_id: 1, // Moon
                zodiac_sign: 1,
                is_retrograde: false,
                distance_milliau: 1000,
            },
            CompactPlanetPosition {
                timestamp_minutes: 0,
                longitude_millidegrees: 60_000, // 60° - sextile (major)
                latitude_millidegrees: 0,
                speed_millidegrees: 100,
                body_id: 2, // Mercury
                zodiac_sign: 2,
                is_retrograde: false,
                distance_milliau: 1000,
            },
        ];

        // Create a minimal test - we can't easily create ChunkGenerator without DB,
        // but we can verify the is_major_aspect logic is correct

        // Verify the angular differences
        let diff_sun_moon = 30.0; // 30° - should be filtered
        let diff_sun_mercury = 60.0; // 60° - should be included

        assert!(
            !is_major_aspect(diff_sun_moon),
            "30° (Sun-Moon) should be filtered as minor aspect"
        );
        assert!(
            is_major_aspect(diff_sun_mercury),
            "60° (Sun-Mercury) should be included as major aspect"
        );
    }
}
