use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::Row;
use thiserror::Error;

use super::pool::DatabasePool;
use super::schema;

/// Errors that can occur during multi-resolution operations
#[derive(Error, Debug)]
pub enum MultiResolutionError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Conversion error: {0}")]
    Conversion(String),
    #[error("Invalid body ID: {0}")]
    InvalidBodyId(i16),
}

/// Body category determines the appropriate resolution for storage
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyCategory {
    /// Moon moves rapidly, requires 1-minute resolution
    Moon,
    /// Inner planets (Sun, Mercury, Venus, Mars) - 5-minute resolution
    Inner,
    /// Outer planets (Jupiter, Saturn, Uranus, Neptune, Pluto) - 60-minute resolution
    Outer,
}

/// Resolution levels for planet position data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    /// 1-minute resolution for Moon
    OneMinute,
    /// 5-minute resolution for inner planets
    FiveMinute,
    /// 60-minute resolution for outer planets
    SixtyMinute,
}

/// Get the body category for a given body ID
///
/// Body ID mapping:
/// - 0 = Sun (Inner)
/// - 1 = Moon (Moon)
/// - 2 = Mercury (Inner)
/// - 3 = Venus (Inner)
/// - 4 = Mars (Inner)
/// - 5 = Jupiter (Outer)
/// - 6 = Saturn (Outer)
/// - 7 = Uranus (Outer)
/// - 8 = Neptune (Outer)
/// - 9 = Pluto (Outer)
pub fn get_body_category(body_id: i16) -> Result<BodyCategory, MultiResolutionError> {
    match body_id {
        1 => Ok(BodyCategory::Moon),
        0 | 2 | 3 | 4 => Ok(BodyCategory::Inner),
        5..=9 => Ok(BodyCategory::Outer),
        _ => Err(MultiResolutionError::InvalidBodyId(body_id)),
    }
}

/// Get the appropriate resolution for a given body ID
pub fn get_resolution_for_body(body_id: i16) -> Result<Resolution, MultiResolutionError> {
    let category = get_body_category(body_id)?;
    Ok(match category {
        BodyCategory::Moon => Resolution::OneMinute,
        BodyCategory::Inner => Resolution::FiveMinute,
        BodyCategory::Outer => Resolution::SixtyMinute,
    })
}

/// Get the table name for a given resolution
pub fn get_table_for_resolution(res: Resolution) -> &'static str {
    match res {
        Resolution::OneMinute => "planet_positions",
        Resolution::FiveMinute => "planet_positions_5min",
        Resolution::SixtyMinute => "planet_positions_60min",
    }
}

/// Manages multi-resolution data loading from TimescaleDB continuous aggregates
///
/// Provides efficient querying by loading data from the appropriate resolution
/// table based on body category. Outer planets use 60-minute resolution,
/// inner planets use 5-minute, and Moon uses full 1-minute resolution.
#[derive(Clone)]
pub struct MultiResolutionManager {
    pool: DatabasePool,
}

impl MultiResolutionManager {
    /// Create a new MultiResolutionManager with the given database pool
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    /// Load positions for a single body from the appropriate resolution table
    ///
    /// # Arguments
    /// * `body_id` - The celestial body ID (0-9)
    /// * `start` - Start time for the query range
    /// * `end` - End time for the query range
    ///
    /// # Returns
    /// * `Ok(Vec<PlanetPosition>)` - Position records from the appropriate table
    /// * `Err(MultiResolutionError)` - If database query fails
    ///
    /// # Resolution Selection
    /// - Moon (1): Queries planet_positions (1-minute)
    /// - Inner planets (0, 2-4): Queries planet_positions_5min
    /// - Outer planets (5-9): Queries planet_positions_60min
    pub async fn load_positions_for_body(
        &self,
        body_id: i16,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<schema::PlanetPosition>, MultiResolutionError> {
        let resolution = get_resolution_for_body(body_id)?;
        let table = get_table_for_resolution(resolution);

        // For continuous aggregates, the time column is named 'bucket'
        // For the raw table, it's named 'time'
        let time_column = match resolution {
            Resolution::OneMinute => "time",
            _ => "bucket",
        };

        let query = format!(
            r#"
            SELECT {time_col} as time, body_id, longitude, latitude, distance, 
                   speed_lon, retrograde, zodiac_sign
            FROM {table}
            WHERE body_id = $1 
              AND {time_col} >= $2 
              AND {time_col} <= $3
            ORDER BY {time_col}
            "#,
            time_col = time_column,
            table = table
        );

        let rows = sqlx::query(&query)
            .bind(body_id)
            .bind(start)
            .bind(end)
            .fetch_all(self.pool.pool())
            .await?;

        let mut positions = Vec::with_capacity(rows.len());
        for row in rows {
            let longitude: f64 = row.try_get("longitude")?;
            let latitude: f64 = row.try_get("latitude")?;
            let distance: f64 = row.try_get("distance")?;
            let speed_lon: f64 = row.try_get("speed_lon")?;

            positions.push(schema::PlanetPosition {
                time: row.try_get("time")?,
                body_id: row.try_get("body_id")?,
                longitude: Decimal::from_f64_retain(longitude)
                    .ok_or_else(|| MultiResolutionError::Conversion("longitude".to_string()))?,
                latitude: Decimal::from_f64_retain(latitude)
                    .ok_or_else(|| MultiResolutionError::Conversion("latitude".to_string()))?,
                distance: Decimal::from_f64_retain(distance)
                    .ok_or_else(|| MultiResolutionError::Conversion("distance".to_string()))?,
                speed_lon: Decimal::from_f64_retain(speed_lon)
                    .ok_or_else(|| MultiResolutionError::Conversion("speed_lon".to_string()))?,
                retrograde: row.try_get("retrograde")?,
                zodiac_sign: row.try_get("zodiac_sign")?,
            });
        }

        Ok(positions)
    }

    /// Load positions for multiple bodies, querying appropriate tables for each
    ///
    /// This method groups bodies by resolution and queries each table once,
    /// then combines and sorts the results by time.
    ///
    /// # Arguments
    /// * `body_ids` - Slice of celestial body IDs to query
    /// * `start` - Start time for the query range
    /// * `end` - End time for the query range
    ///
    /// # Returns
    /// * `Ok(Vec<PlanetPosition>)` - Combined position records sorted by time
    /// * `Err(MultiResolutionError)` - If any database query fails
    pub async fn load_positions_multi_resolution(
        &self,
        body_ids: &[i16],
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<schema::PlanetPosition>, MultiResolutionError> {
        // Group bodies by resolution to minimize queries
        let mut moon_bodies: Vec<i16> = Vec::new();
        let mut inner_bodies: Vec<i16> = Vec::new();
        let mut outer_bodies: Vec<i16> = Vec::new();

        for &body_id in body_ids {
            match get_body_category(body_id)? {
                BodyCategory::Moon => moon_bodies.push(body_id),
                BodyCategory::Inner => inner_bodies.push(body_id),
                BodyCategory::Outer => outer_bodies.push(body_id),
            }
        }

        let mut all_positions: Vec<schema::PlanetPosition> = Vec::new();

        // Query each resolution table if there are bodies for that category
        if !moon_bodies.is_empty() {
            let positions = self
                .load_positions_for_resolution(Resolution::OneMinute, &moon_bodies, start, end)
                .await?;
            all_positions.extend(positions);
        }

        if !inner_bodies.is_empty() {
            let positions = self
                .load_positions_for_resolution(Resolution::FiveMinute, &inner_bodies, start, end)
                .await?;
            all_positions.extend(positions);
        }

        if !outer_bodies.is_empty() {
            let positions = self
                .load_positions_for_resolution(Resolution::SixtyMinute, &outer_bodies, start, end)
                .await?;
            all_positions.extend(positions);
        }

        // Sort by time, then by body_id for consistent ordering
        all_positions.sort_by(|a, b| a.time.cmp(&b.time).then_with(|| a.body_id.cmp(&b.body_id)));

        Ok(all_positions)
    }

    /// Load positions for multiple bodies from a specific resolution table
    async fn load_positions_for_resolution(
        &self,
        resolution: Resolution,
        body_ids: &[i16],
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<schema::PlanetPosition>, MultiResolutionError> {
        if body_ids.is_empty() {
            return Ok(Vec::new());
        }

        let table = get_table_for_resolution(resolution);
        let time_column = match resolution {
            Resolution::OneMinute => "time",
            _ => "bucket",
        };

        // Build IN clause for body_ids
        let body_id_list: Vec<String> = body_ids.iter().map(|id| id.to_string()).collect();
        let body_id_str = body_id_list.join(",");

        let query = format!(
            r#"
            SELECT {time_col} as time, body_id, longitude, latitude, distance, 
                   speed_lon, retrograde, zodiac_sign
            FROM {table}
            WHERE body_id IN ({body_ids})
              AND {time_col} >= $1 
              AND {time_col} <= $2
            ORDER BY {time_col}, body_id
            "#,
            time_col = time_column,
            table = table,
            body_ids = body_id_str
        );

        let rows = sqlx::query(&query)
            .bind(start)
            .bind(end)
            .fetch_all(self.pool.pool())
            .await?;

        let mut positions = Vec::with_capacity(rows.len());
        for row in rows {
            let longitude: f64 = row.try_get("longitude")?;
            let latitude: f64 = row.try_get("latitude")?;
            let distance: f64 = row.try_get("distance")?;
            let speed_lon: f64 = row.try_get("speed_lon")?;

            positions.push(schema::PlanetPosition {
                time: row.try_get("time")?,
                body_id: row.try_get("body_id")?,
                longitude: Decimal::from_f64_retain(longitude)
                    .ok_or_else(|| MultiResolutionError::Conversion("longitude".to_string()))?,
                latitude: Decimal::from_f64_retain(latitude)
                    .ok_or_else(|| MultiResolutionError::Conversion("latitude".to_string()))?,
                distance: Decimal::from_f64_retain(distance)
                    .ok_or_else(|| MultiResolutionError::Conversion("distance".to_string()))?,
                speed_lon: Decimal::from_f64_retain(speed_lon)
                    .ok_or_else(|| MultiResolutionError::Conversion("speed_lon".to_string()))?,
                retrograde: row.try_get("retrograde")?,
                zodiac_sign: row.try_get("zodiac_sign")?,
            });
        }

        Ok(positions)
    }

    /// Get the resolution that would be used for a given body ID
    ///
    /// Convenience method for checking resolution without loading data
    pub fn get_resolution(&self, body_id: i16) -> Result<Resolution, MultiResolutionError> {
        get_resolution_for_body(body_id)
    }

    /// Get the table name that would be queried for a given body ID
    ///
    /// Convenience method for debugging and logging
    pub fn get_table_for_body(&self, body_id: i16) -> Result<&'static str, MultiResolutionError> {
        let resolution = get_resolution_for_body(body_id)?;
        Ok(get_table_for_resolution(resolution))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_body_category() {
        // Moon
        assert_eq!(get_body_category(1).unwrap(), BodyCategory::Moon);

        // Inner planets
        assert_eq!(get_body_category(0).unwrap(), BodyCategory::Inner); // Sun
        assert_eq!(get_body_category(2).unwrap(), BodyCategory::Inner); // Mercury
        assert_eq!(get_body_category(3).unwrap(), BodyCategory::Inner); // Venus
        assert_eq!(get_body_category(4).unwrap(), BodyCategory::Inner); // Mars

        // Outer planets
        assert_eq!(get_body_category(5).unwrap(), BodyCategory::Outer); // Jupiter
        assert_eq!(get_body_category(6).unwrap(), BodyCategory::Outer); // Saturn
        assert_eq!(get_body_category(7).unwrap(), BodyCategory::Outer); // Uranus
        assert_eq!(get_body_category(8).unwrap(), BodyCategory::Outer); // Neptune
        assert_eq!(get_body_category(9).unwrap(), BodyCategory::Outer); // Pluto

        // Invalid
        assert!(get_body_category(10).is_err());
        assert!(get_body_category(-1).is_err());
    }

    #[test]
    fn test_get_resolution_for_body() {
        assert_eq!(
            get_resolution_for_body(1).unwrap(),
            Resolution::OneMinute // Moon
        );
        assert_eq!(
            get_resolution_for_body(0).unwrap(),
            Resolution::FiveMinute // Sun
        );
        assert_eq!(
            get_resolution_for_body(5).unwrap(),
            Resolution::SixtyMinute // Jupiter
        );
    }

    #[test]
    fn test_get_table_for_resolution() {
        assert_eq!(
            get_table_for_resolution(Resolution::OneMinute),
            "planet_positions"
        );
        assert_eq!(
            get_table_for_resolution(Resolution::FiveMinute),
            "planet_positions_5min"
        );
        assert_eq!(
            get_table_for_resolution(Resolution::SixtyMinute),
            "planet_positions_60min"
        );
    }
}
