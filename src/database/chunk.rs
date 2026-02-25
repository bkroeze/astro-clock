use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::time::Instant;
use thiserror::Error;

use super::schema;

// ============================================================================
// CONSTANTS
// ============================================================================

/// Number of days per chunk (1 day chunks align with TimescaleDB)
pub const CHUNK_SIZE_DAYS: i64 = 1;

/// Target cache size in chunks (~30MB at ~1MB per day)
pub const CACHE_SIZE_CHUNKS: usize = 30;

/// Minutes in a day (24 * 60)
pub const MINUTES_PER_DAY: u32 = 1440;

/// Number of celestial bodies (Sun through Pluto)
pub const BODIES_COUNT: usize = 10;

// ============================================================================
// ERROR TYPE
// ============================================================================

/// Errors that can occur during chunk operations
#[derive(Debug, Error)]
pub enum ChunkError {
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),
    #[error("Invalid longitude value: {0}")]
    InvalidLongitude(Decimal),
    #[error("Invalid latitude value: {0}")]
    InvalidLatitude(Decimal),
    #[error("Invalid body ID: {0}")]
    InvalidBodyId(i16),
    #[error("Invalid zodiac sign: {0}")]
    InvalidZodiacSign(i16),
    #[error("Invalid speed value: {0}")]
    InvalidSpeed(Decimal),
    #[error("Invalid moon phase: {0}")]
    InvalidMoonPhase(i16),
    #[error("Invalid illumination value: {0}")]
    InvalidIllumination(Decimal),
    #[error("Invalid aspect type: {0}")]
    InvalidAspectType(i16),
}

// ============================================================================
// CACHE KEY
// ============================================================================

/// Key for cache lookups by date
///
/// Implements Hash, Eq, Clone, and Copy for use as an LRU cache key.
/// The date is stored as NaiveDate for efficient hashing and comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkKey {
    pub date: NaiveDate,
}

impl ChunkKey {
    /// Create a new chunk key for the given date
    pub fn new(date: NaiveDate) -> Self {
        Self { date }
    }

    /// Create a key from a DateTime (extracts the date component)
    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        Self {
            date: dt.date_naive(),
        }
    }
}

// ============================================================================
// COMPACT PLANET POSITION
// ============================================================================

/// Memory-efficient planet position storage (~16 bytes vs ~72 bytes)
///
/// Uses packed representation and integer encoding to achieve ~4.5× size reduction.
/// All angles are stored in millidegrees (0.001° precision) for compactness.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct CompactPlanetPosition {
    /// Minutes since chunk start (0-1439 for a full day)
    pub timestamp_minutes: u32,
    /// Longitude in millidegrees (0-360000, 0.001° precision)
    pub longitude_millidegrees: i32,
    /// Latitude in millidegrees (-90000 to 90000)
    pub latitude_millidegrees: i32,
    /// Daily motion in longitude (millidegrees per day, fits in i16)
    pub speed_millidegrees: i16,
    /// Body ID (0-9 for Sun through Pluto)
    pub body_id: u8,
    /// Zodiac sign (0-11 for Aries through Pisces)
    pub zodiac_sign: u8,
    /// Whether the body is in retrograde motion
    pub is_retrograde: bool,
    /// Distance from Earth in milliaU (0.001 AU precision, fits in u16 for inner planets)
    /// For outer planets, this is capped at 65535 (65.535 AU)
    pub distance_milliau: u16,
}

impl CompactPlanetPosition {
    /// Convert from schema::PlanetPosition to compact representation
    ///
    /// # Arguments
    /// * `pos` - The schema position to convert
    /// * `chunk_date` - The date of the chunk (for calculating minutes offset)
    ///
    /// # Errors
    /// Returns ChunkError if any values are out of valid ranges
    pub fn from_schema(
        pos: &schema::PlanetPosition,
        chunk_date: NaiveDate,
    ) -> Result<Self, ChunkError> {
        // Calculate minutes since chunk start
        let chunk_start = chunk_date.and_hms_opt(0, 0, 0).ok_or_else(|| {
            ChunkError::InvalidTimestamp(format!("Invalid chunk date: {}", chunk_date))
        })?;
        let pos_naive = pos.time.naive_utc();
        let minutes_since_start = pos_naive.signed_duration_since(chunk_start).num_minutes();

        if minutes_since_start < 0 || minutes_since_start >= i64::from(MINUTES_PER_DAY) {
            return Err(ChunkError::InvalidTimestamp(format!(
                "Position time {} is outside chunk date {}",
                pos.time, chunk_date
            )));
        }

        // Convert Decimal degrees to millidegrees
        let longitude_millidegrees = (pos.longitude * Decimal::from(1000))
            .to_i32()
            .ok_or_else(|| ChunkError::InvalidLongitude(pos.longitude))?;

        let latitude_millidegrees = (pos.latitude * Decimal::from(1000))
            .to_i32()
            .ok_or_else(|| ChunkError::InvalidLatitude(pos.latitude))?;

        // Convert speed to millidegrees per day (fits in i16 for normal planetary motion)
        let speed_millidegrees = (pos.speed_lon * Decimal::from(1000))
            .to_i16()
            .ok_or_else(|| ChunkError::InvalidSpeed(pos.speed_lon))?;

        // Convert body_id and zodiac_sign to compact u8
        let body_id = pos
            .body_id
            .try_into()
            .map_err(|_| ChunkError::InvalidBodyId(pos.body_id))?;

        let zodiac_sign = pos
            .zodiac_sign
            .try_into()
            .map_err(|_| ChunkError::InvalidZodiacSign(pos.zodiac_sign))?;

        // Convert distance to milliaU (capped at u16::MAX for outer planets)
        let distance_milliau = (pos.distance * Decimal::from(1000))
            .to_u16()
            .unwrap_or(u16::MAX);

        Ok(Self {
            timestamp_minutes: minutes_since_start as u32,
            longitude_millidegrees,
            latitude_millidegrees,
            speed_millidegrees,
            body_id,
            zodiac_sign,
            is_retrograde: pos.retrograde,
            distance_milliau,
        })
    }

    /// Convert millidegrees back to Decimal degrees
    pub fn longitude_degrees(&self) -> Decimal {
        Decimal::from(self.longitude_millidegrees) / Decimal::from(1000)
    }

    /// Convert millidegrees back to Decimal degrees
    pub fn latitude_degrees(&self) -> Decimal {
        Decimal::from(self.latitude_millidegrees) / Decimal::from(1000)
    }

    /// Convert millidegrees per day back to Decimal
    pub fn speed_degrees(&self) -> Decimal {
        Decimal::from(self.speed_millidegrees) / Decimal::from(1000)
    }

    /// Convert milliaU back to Decimal AU
    pub fn distance_au(&self) -> Decimal {
        Decimal::from(self.distance_milliau) / Decimal::from(1000)
    }

    /// Get the full timestamp from chunk date
    pub fn timestamp(&self, chunk_date: NaiveDate) -> Option<DateTime<Utc>> {
        chunk_date
            .and_hms_opt(0, 0, 0)
            .and_then(|dt| {
                dt.checked_add_signed(chrono::Duration::minutes(i64::from(self.timestamp_minutes)))
            })
            .map(|naive| DateTime::from_naive_utc_and_offset(naive, Utc))
    }
}

// ============================================================================
// COMPACT ASPECT
// ============================================================================

/// Memory-efficient aspect storage
///
/// Stores astrological aspects between two celestial bodies at a specific time.
/// Body IDs are ordered (body1_id < body2_id) to avoid duplicate storage.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct CompactAspect {
    /// Minutes since chunk start (0-1439)
    pub timestamp_minutes: u32,
    /// First body ID (0-9, always less than body2_id)
    pub body1_id: u8,
    /// Second body ID (0-9, always greater than body1_id)
    pub body2_id: u8,
    /// Aspect type (0-4: conjunction, sextile, square, trine, opposition)
    pub aspect_type: u8,
    /// Orb from exact aspect in millidegrees (0-10000 for 0-10°)
    pub orb_millidegrees: u16,
    /// True if the aspect is applying (orb decreasing)
    pub applying: bool,
}

impl CompactAspect {
    /// Convert from schema::Aspect to compact representation
    ///
    /// # Arguments
    /// * `aspect` - The schema aspect to convert
    /// * `chunk_date` - The date of the chunk
    ///
    /// # Errors
    /// Returns ChunkError if values are out of valid ranges
    pub fn from_schema(aspect: &schema::Aspect, chunk_date: NaiveDate) -> Result<Self, ChunkError> {
        // Calculate minutes since chunk start
        let chunk_start = chunk_date.and_hms_opt(0, 0, 0).ok_or_else(|| {
            ChunkError::InvalidTimestamp(format!("Invalid chunk date: {}", chunk_date))
        })?;
        let aspect_naive = aspect.time.naive_utc();
        let minutes_since_start = aspect_naive
            .signed_duration_since(chunk_start)
            .num_minutes();

        if minutes_since_start < 0 || minutes_since_start >= i64::from(MINUTES_PER_DAY) {
            return Err(ChunkError::InvalidTimestamp(format!(
                "Aspect time {} is outside chunk date {}",
                aspect.time, chunk_date
            )));
        }

        // Convert body IDs to u8
        let body1_id = aspect
            .body1_id
            .try_into()
            .map_err(|_| ChunkError::InvalidBodyId(aspect.body1_id))?;
        let body2_id = aspect
            .body2_id
            .try_into()
            .map_err(|_| ChunkError::InvalidBodyId(aspect.body2_id))?;

        // Ensure ordering (body1_id < body2_id)
        let (b1, b2) = if body1_id < body2_id {
            (body1_id, body2_id)
        } else {
            (body2_id, body1_id)
        };

        // Convert aspect type to u8
        let aspect_type = aspect
            .aspect_type
            .try_into()
            .map_err(|_| ChunkError::InvalidAspectType(aspect.aspect_type))?;

        // Convert orb to millidegrees (capped at 10000 for 10°)
        let orb_millidegrees = (aspect.orb * Decimal::from(1000))
            .to_u16()
            .unwrap_or(10000)
            .min(10000);

        Ok(Self {
            timestamp_minutes: minutes_since_start as u32,
            body1_id: b1,
            body2_id: b2,
            aspect_type,
            orb_millidegrees,
            applying: aspect.applying,
        })
    }

    /// Convert millidegrees back to Decimal degrees
    pub fn orb_degrees(&self) -> Decimal {
        Decimal::from(self.orb_millidegrees) / Decimal::from(1000)
    }

    /// Get the full timestamp from chunk date
    pub fn timestamp(&self, chunk_date: NaiveDate) -> Option<DateTime<Utc>> {
        chunk_date
            .and_hms_opt(0, 0, 0)
            .and_then(|dt| {
                dt.checked_add_signed(chrono::Duration::minutes(i64::from(self.timestamp_minutes)))
            })
            .map(|naive| DateTime::from_naive_utc_and_offset(naive, Utc))
    }
}

// ============================================================================
// COMPACT LUNAR CONDITION
// ============================================================================

/// Memory-efficient lunar condition storage
///
/// Stores Moon phase, illumination, and void-of-course status.
/// Uses compact integer representations for all values.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct CompactLunarCondition {
    /// Minutes since chunk start (0-1439)
    pub timestamp_minutes: u32,
    /// Moon phase (0-7: New through Waning Crescent)
    pub moon_phase: u8,
    /// Moon's zodiac sign (0-11)
    pub moon_sign: u8,
    /// Moon phase angle in millidegrees (0-360000)
    pub moon_phase_angle_milli: u32,
    /// Moon illumination in permille (0-1000, representing 0.0-1.0)
    pub moon_illumination_permille: u16,
    /// True if Moon is void-of-course
    pub is_void_of_course: bool,
    /// Reserved padding for alignment (VoC start/end stored separately if needed)
    pub _padding: u8,
}

impl CompactLunarCondition {
    /// Convert from schema::LunarCondition to compact representation
    ///
    /// # Arguments
    /// * `condition` - The schema lunar condition to convert
    /// * `chunk_date` - The date of the chunk
    ///
    /// # Errors
    /// Returns ChunkError if values are out of valid ranges
    pub fn from_schema(
        condition: &schema::LunarCondition,
        chunk_date: NaiveDate,
    ) -> Result<Self, ChunkError> {
        // Calculate minutes since chunk start
        let chunk_start = chunk_date.and_hms_opt(0, 0, 0).ok_or_else(|| {
            ChunkError::InvalidTimestamp(format!("Invalid chunk date: {}", chunk_date))
        })?;
        let condition_naive = condition.time.naive_utc();
        let minutes_since_start = condition_naive
            .signed_duration_since(chunk_start)
            .num_minutes();

        if minutes_since_start < 0 || minutes_since_start >= i64::from(MINUTES_PER_DAY) {
            return Err(ChunkError::InvalidTimestamp(format!(
                "Lunar condition time {} is outside chunk date {}",
                condition.time, chunk_date
            )));
        }

        // Convert moon phase to u8
        let moon_phase = condition
            .moon_phase
            .try_into()
            .map_err(|_| ChunkError::InvalidMoonPhase(condition.moon_phase))?;

        // Convert moon sign to u8
        let moon_sign = condition
            .moon_sign
            .try_into()
            .map_err(|_| ChunkError::InvalidZodiacSign(condition.moon_sign))?;

        // Convert phase angle to millidegrees
        let moon_phase_angle_milli = (condition.moon_phase_angle * Decimal::from(1000))
            .to_u32()
            .ok_or_else(|| ChunkError::InvalidLongitude(condition.moon_phase_angle))?;

        // Convert illumination to permille (0-1000)
        let moon_illumination_permille = (condition.moon_illumination * Decimal::from(1000))
            .to_u16()
            .ok_or_else(|| ChunkError::InvalidIllumination(condition.moon_illumination))?;

        Ok(Self {
            timestamp_minutes: minutes_since_start as u32,
            moon_phase,
            moon_sign,
            moon_phase_angle_milli,
            moon_illumination_permille,
            is_void_of_course: condition.is_void_of_course,
            _padding: 0,
        })
    }

    /// Convert millidegrees back to Decimal degrees
    pub fn moon_phase_angle_degrees(&self) -> Decimal {
        Decimal::from(self.moon_phase_angle_milli) / Decimal::from(1000)
    }

    /// Convert permille back to Decimal (0.0-1.0)
    pub fn moon_illumination(&self) -> Decimal {
        Decimal::from(self.moon_illumination_permille) / Decimal::from(1000)
    }

    /// Get the full timestamp from chunk date
    pub fn timestamp(&self, chunk_date: NaiveDate) -> Option<DateTime<Utc>> {
        chunk_date
            .and_hms_opt(0, 0, 0)
            .and_then(|dt| {
                dt.checked_add_signed(chrono::Duration::minutes(i64::from(self.timestamp_minutes)))
            })
            .map(|naive| DateTime::from_naive_utc_and_offset(naive, Utc))
    }
}

// ============================================================================
// CHUNK DATA CONTAINER
// ============================================================================

/// Container for all astrological data in a single day chunk
///
/// This is the primary data structure stored in the LRU cache.
/// Contains all planet positions, aspects, and lunar conditions for a 24-hour period.
#[derive(Debug, Clone)]
pub struct ChunkData {
    /// The date this chunk represents
    pub date: NaiveDate,
    /// All planet positions for all bodies throughout the day
    /// Stored as a flat Vec for cache efficiency (not HashMap)
    pub planet_positions: Vec<CompactPlanetPosition>,
    /// All aspects occurring during the day
    pub aspects: Vec<CompactAspect>,
    /// All lunar condition records for the day
    pub lunar_conditions: Vec<CompactLunarCondition>,
    /// When this chunk was loaded into memory
    pub loaded_at: Instant,
}

impl ChunkData {
    /// Create a new empty chunk for the given date
    pub fn new(date: NaiveDate) -> Self {
        Self {
            date,
            planet_positions: Vec::with_capacity(MINUTES_PER_DAY as usize * BODIES_COUNT),
            aspects: Vec::new(),
            lunar_conditions: Vec::with_capacity(MINUTES_PER_DAY as usize),
            loaded_at: Instant::now(),
        }
    }

    /// Create a chunk from schema data
    ///
    /// # Arguments
    /// * `date` - The date of the chunk
    /// * `positions` - Planet positions from database
    /// * `aspects` - Aspects from database
    /// * `lunar` - Lunar conditions from database
    ///
    /// # Errors
    /// Returns ChunkError if any data cannot be converted
    pub fn from_schema_data(
        date: NaiveDate,
        positions: &[schema::PlanetPosition],
        aspects: &[schema::Aspect],
        lunar: &[schema::LunarCondition],
    ) -> Result<Self, ChunkError> {
        let mut chunk = Self::new(date);

        // Convert planet positions
        for pos in positions {
            chunk
                .planet_positions
                .push(CompactPlanetPosition::from_schema(pos, date)?);
        }

        // Convert aspects
        for aspect in aspects {
            chunk
                .aspects
                .push(CompactAspect::from_schema(aspect, date)?);
        }

        // Convert lunar conditions
        for condition in lunar {
            chunk
                .lunar_conditions
                .push(CompactLunarCondition::from_schema(condition, date)?);
        }

        Ok(chunk)
    }

    /// Get the approximate memory size of this chunk in bytes
    pub fn approximate_size(&self) -> usize {
        let positions_size =
            self.planet_positions.len() * std::mem::size_of::<CompactPlanetPosition>();
        let aspects_size = self.aspects.len() * std::mem::size_of::<CompactAspect>();
        let lunar_size = self.lunar_conditions.len() * std::mem::size_of::<CompactLunarCondition>();
        let base_size = std::mem::size_of::<Self>();

        base_size + positions_size + aspects_size + lunar_size
    }

    /// Get positions for a specific body at a specific minute
    pub fn get_position_at_minute(
        &self,
        body_id: u8,
        minute: u32,
    ) -> Option<&CompactPlanetPosition> {
        self.planet_positions
            .iter()
            .find(|p| p.body_id == body_id && p.timestamp_minutes == minute)
    }

    /// Get all positions for a specific body
    pub fn get_positions_for_body(
        &self,
        body_id: u8,
    ) -> impl Iterator<Item = &CompactPlanetPosition> {
        self.planet_positions
            .iter()
            .filter(move |p| p.body_id == body_id)
    }

    /// Get lunar condition at a specific minute
    pub fn get_lunar_at_minute(&self, minute: u32) -> Option<&CompactLunarCondition> {
        self.lunar_conditions
            .iter()
            .find(|l| l.timestamp_minutes == minute)
    }

    /// Get aspects at a specific minute
    pub fn get_aspects_at_minute(&self, minute: u32) -> impl Iterator<Item = &CompactAspect> {
        self.aspects
            .iter()
            .filter(move |a| a.timestamp_minutes == minute)
    }

    /// Check if this chunk is still fresh (loaded within the last hour)
    pub fn is_fresh(&self) -> bool {
        self.loaded_at.elapsed().as_secs() < 3600
    }
}

// ============================================================================
// SIZE VERIFICATION
// ============================================================================

/// Verify the size of compact types
///
/// These constants are checked at compile time to ensure memory efficiency targets.
pub const COMPACT_PLANET_POSITION_SIZE: usize = std::mem::size_of::<CompactPlanetPosition>();
pub const COMPACT_ASPECT_SIZE: usize = std::mem::size_of::<CompactAspect>();
pub const COMPACT_LUNAR_CONDITION_SIZE: usize = std::mem::size_of::<CompactLunarCondition>();
pub const CHUNK_KEY_SIZE: usize = std::mem::size_of::<ChunkKey>();

// Static assertions to ensure size targets are met
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compact_planet_position_size() {
        // Target: ~16 bytes (vs ~72 bytes for schema type)
        assert!(
            COMPACT_PLANET_POSITION_SIZE <= 24,
            "CompactPlanetPosition should be <= 24 bytes, got {}",
            COMPACT_PLANET_POSITION_SIZE
        );
    }

    #[test]
    fn test_compact_aspect_size() {
        // Should be around 12 bytes with packed representation
        assert!(
            COMPACT_ASPECT_SIZE <= 16,
            "CompactAspect should be <= 16 bytes, got {}",
            COMPACT_ASPECT_SIZE
        );
    }

    #[test]
    fn test_compact_lunar_condition_size() {
        // Should be around 16 bytes with packed representation
        assert!(
            COMPACT_LUNAR_CONDITION_SIZE <= 20,
            "CompactLunarCondition should be <= 20 bytes, got {}",
            COMPACT_LUNAR_CONDITION_SIZE
        );
    }

    #[test]
    fn test_chunk_key_traits() {
        // Verify ChunkKey implements required traits for LRU cache
        fn assert_hash<T: std::hash::Hash>() {}
        fn assert_eq<T: Eq>() {}
        fn assert_clone<T: Clone>() {}
        fn assert_copy<T: Copy>() {}

        assert_hash::<ChunkKey>();
        assert_eq::<ChunkKey>();
        assert_clone::<ChunkKey>();
        assert_copy::<ChunkKey>();
    }

    #[test]
    fn test_compact_types_are_copy() {
        // Verify all compact types implement Copy for efficient cache storage
        fn assert_copy<T: Copy>() {}

        assert_copy::<CompactPlanetPosition>();
        assert_copy::<CompactAspect>();
        assert_copy::<CompactLunarCondition>();
    }
}
