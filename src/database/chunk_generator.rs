use chrono::{DateTime, NaiveDate, Utc};
use std::time::Instant;
use thiserror::Error;

use super::chunk::{ChunkData, CompactAspect, CompactLunarCondition, CompactPlanetPosition, BODIES_COUNT, MINUTES_PER_DAY};
use super::pool::DatabasePool;

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
    pub async fn generate_chunk(
        &self,
        date: NaiveDate,
    ) -> Result<ChunkData, ChunkGeneratorError> {
        // Ensure ephemeris is initialized
        crate::ephemeris::Ephemeris::ensure_initialized()
            .map_err(|e| ChunkGeneratorError::SwissEph(e.to_string()))?;

        let mut positions = Vec::with_capacity(MINUTES_PER_DAY as usize * BODIES_COUNT);
        let mut aspects = Vec::new();
        let mut lunar_conditions = Vec::with_capacity(MINUTES_PER_DAY as usize);

        // Generate data for each minute of the day
        for minute in 0..MINUTES_PER_DAY {
            let timestamp = date
                .and_hms_opt(0, minute, 0)
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

            // Calculate aspects for this minute
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
    fn calculate_aspects(
        &self,
        positions: &[CompactPlanetPosition],
        minute: u32,
    ) -> Vec<CompactAspect> {
        const MAJOR_ASPECTS: [(u8, f64); 5] = [
            (0, 0.0),   // Conjunction
            (1, 60.0),  // Sextile
            (2, 90.0),  // Square
            (3, 120.0), // Trine
            (4, 180.0), // Opposition
        ];
        const DEFAULT_ORB: f64 = 10.0; // degrees

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

                for (aspect_type, target_angle) in MAJOR_ASPECTS {
                    let orb = (diff - target_angle).abs();

                    if orb <= DEFAULT_ORB {
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
    pub async fn save_chunk_to_db(
        &self,
        chunk: &ChunkData,
    ) -> Result<(), ChunkGeneratorError> {
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
        use rust_decimal::Decimal;

        if positions.is_empty() {
            return Ok(());
        }

        // Build batch insert using UNNEST for efficiency
        let times: Vec<DateTime<Utc>> = positions
            .iter()
            .map(|p| {
                chunk_date
                    .and_hms_opt(0, p.timestamp_minutes, 0)
                    .unwrap()
                    .and_utc()
            })
            .collect();

        let body_ids: Vec<i16> = positions.iter().map(|p| p.body_id as i16).collect();
        let longitudes: Vec<Decimal> = positions
            .iter()
            .map(|p| Decimal::from(p.longitude_millidegrees) / Decimal::from(1000))
            .collect();
        let latitudes: Vec<Decimal> = positions
            .iter()
            .map(|p| Decimal::from(p.latitude_millidegrees) / Decimal::from(1000))
            .collect();
        let distances: Vec<Decimal> = positions
            .iter()
            .map(|p| Decimal::from(p.distance_milliau) / Decimal::from(1000))
            .collect();
        let speeds: Vec<Decimal> = positions
            .iter()
            .map(|p| Decimal::from(p.speed_millidegrees) / Decimal::from(1000))
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
                $3::decimal[],
                $4::decimal[],
                $5::decimal[],
                $6::decimal[],
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
        use rust_decimal::Decimal;

        if aspects.is_empty() {
            return Ok(());
        }

        let times: Vec<DateTime<Utc>> = aspects
            .iter()
            .map(|a| {
                chunk_date
                    .and_hms_opt(0, a.timestamp_minutes, 0)
                    .unwrap()
                    .and_utc()
            })
            .collect();
        let body1_ids: Vec<i16> = aspects.iter().map(|a| a.body1_id as i16).collect();
        let body2_ids: Vec<i16> = aspects.iter().map(|a| a.body2_id as i16).collect();
        let aspect_types: Vec<i16> = aspects.iter().map(|a| a.aspect_type as i16).collect();
        let orbs: Vec<Decimal> = aspects
            .iter()
            .map(|a| Decimal::from(a.orb_millidegrees) / Decimal::from(1000))
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
                $5::decimal[],
                $6::boolean[]
            )
            ON CONFLICT (time, body1_id, body2_id) DO NOTHING
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
        use rust_decimal::Decimal;

        if lunar.is_empty() {
            return Ok(());
        }

        let times: Vec<DateTime<Utc>> = lunar
            .iter()
            .map(|l| {
                chunk_date
                    .and_hms_opt(0, l.timestamp_minutes, 0)
                    .unwrap()
                    .and_utc()
            })
            .collect();
        let moon_phases: Vec<i16> = lunar.iter().map(|l| l.moon_phase as i16).collect();
        let moon_signs: Vec<i16> = lunar.iter().map(|l| l.moon_sign as i16).collect();
        let phase_angles: Vec<Decimal> = lunar
            .iter()
            .map(|l| Decimal::from(l.moon_phase_angle_milli) / Decimal::from(1000))
            .collect();
        let illuminations: Vec<Decimal> = lunar
            .iter()
            .map(|l| Decimal::from(l.moon_illumination_permille) / Decimal::from(1000))
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
                $4::decimal[],
                $5::decimal[],
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
}
