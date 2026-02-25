use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ============================================================================
// LEGACY SCHEMA (maintained for backward compatibility)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ChartRecord {
    pub id: i32,
    pub created_at: String,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
    pub julian_day: f64,
    pub planets: Vec<PlanetPositionRecord>,
    pub houses: Vec<f64>,
    pub sidereal_time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PlanetPositionRecord {
    name: String,
    pub longitude: f64,
    pub latitude: f64,
    pub distance: f64,
    pub speed_lon: f64,
    pub speed_lat: f64,
    pub speed_dist: f64,
    pub retrograde: bool,
}

// ============================================================================
// NEW TIMESCALEDB SCHEMA (Phase 1 - Database Schema)
// ============================================================================

/// Planet position at a specific time
///
/// Maps to the `planet_positions` hypertable.
/// Stores 1-minute resolution positions for all celestial bodies.
///
/// Body IDs: 0=Sun, 1=Moon, 2=Mercury, 3=Venus, 4=Mars,
///           5=Jupiter, 6=Saturn, 7=Uranus, 8=Neptune, 9=Pluto
///
/// Zodiac signs: 0=Aries, 1=Taurus, 2=Gemini, 3=Cancer, 4=Leo, 5=Virgo,
///               6=Libra, 7=Scorpio, 8=Sagittarius, 9=Capricorn, 10=Aquarius, 11=Pisces
#[derive(Debug, Clone, FromRow)]
pub struct PlanetPosition {
    /// Timestamp of the position record
    pub time: chrono::DateTime<chrono::Utc>,
    /// Body ID (0-9 for Sun through Pluto)
    pub body_id: i16,
    /// Ecliptic longitude in degrees (0-360)
    pub longitude: Decimal,
    /// Ecliptic latitude in degrees
    pub latitude: Decimal,
    /// Distance from Earth in Astronomical Units (AU)
    pub distance: Decimal,
    /// Daily motion in longitude (degrees per day)
    pub speed_lon: Decimal,
    /// Whether the body is in retrograde motion
    pub retrograde: bool,
    /// Zodiac sign (0-11)
    pub zodiac_sign: i16,
}

/// Astrological aspect between two bodies
///
/// Maps to the `aspects` hypertable.
/// Stores pre-calculated astrological aspects between celestial bodies.
///
/// Aspect types: 0=Conjunction (0°), 1=Sextile (60°), 2=Square (90°),
///               3=Trine (120°), 4=Opposition (180°)
#[derive(Debug, Clone, FromRow)]
pub struct Aspect {
    /// Timestamp of the aspect
    pub time: chrono::DateTime<chrono::Utc>,
    /// First body ID (0-9)
    pub body1_id: i16,
    /// Second body ID (0-9), always greater than body1_id
    pub body2_id: i16,
    /// Aspect type (0-4)
    pub aspect_type: i16,
    /// Orb from exact aspect in degrees (typically 0-10°)
    pub orb: Decimal,
    /// True if the aspect is applying (orb decreasing)
    pub applying: bool,
}

/// Lunar conditions at a specific time
///
/// Maps to the `lunar_conditions` hypertable.
/// Stores Moon phases, illumination, and void-of-course periods.
///
/// Moon phases: 0=New, 1=Waxing Crescent, 2=First Quarter, 3=Waxing Gibbous,
///              4=Full, 5=Waning Gibbous, 6=Last Quarter, 7=Waning Crescent
#[derive(Debug, Clone, FromRow)]
pub struct LunarCondition {
    /// Timestamp of the lunar condition
    pub time: chrono::DateTime<chrono::Utc>,
    /// Moon phase (0-7)
    pub moon_phase: i16,
    /// Moon's zodiac sign (0-11)
    pub moon_sign: i16,
    /// Moon phase angle in degrees (0-360)
    pub moon_phase_angle: Decimal,
    /// Moon illumination fraction (0.0-1.0)
    pub moon_illumination: Decimal,
    /// True if Moon is void-of-course
    pub is_void_of_course: bool,
    /// Start of void-of-course period (null if not VoC)
    pub voc_start: Option<chrono::DateTime<chrono::Utc>>,
    /// End of void-of-course period (null if not VoC)
    pub voc_end: Option<chrono::DateTime<chrono::Utc>>,
}

/// Geographic location for house cusp calculations
///
/// Maps to the `locations` table.
/// Normalized location data to reduce storage by ~60% for repeated locations.
#[derive(Debug, Clone, FromRow)]
pub struct Location {
    /// Unique location ID
    pub id: i32,
    /// Optional location name (e.g., "New York, NY")
    pub name: Option<String>,
    /// Latitude in decimal degrees (-90 to 90)
    pub latitude: Decimal,
    /// Longitude in decimal degrees (-180 to 180)
    pub longitude: Decimal,
    /// IANA timezone name (e.g., "America/New_York")
    pub timezone: Option<String>,
}

/// House cusps for a specific location and time
///
/// Maps to the `house_cusps` hypertable.
/// Stores calculated house cusps for specific locations and times.
///
/// House systems: 0=Placidus, 1=Koch, 2=Equal, 3=Whole Sign, 4=Porphyry,
///                5=Regiomontanus, 6=Campanus, 7=Morinus, 8=Topocentric,
///                9=Alcabitius, 10=Azimuthal
#[derive(Debug, Clone, FromRow)]
pub struct HouseCusp {
    /// Timestamp of the house cusp calculation
    pub time: chrono::DateTime<chrono::Utc>,
    /// Foreign key to locations.id
    pub location_id: i32,
    /// House system used (0-10)
    pub house_system: i16,
    /// 1st house cusp longitude
    pub cusp_1: Decimal,
    /// 2nd house cusp longitude
    pub cusp_2: Decimal,
    /// 3rd house cusp longitude
    pub cusp_3: Decimal,
    /// 4th house cusp longitude (IC)
    pub cusp_4: Decimal,
    /// 5th house cusp longitude
    pub cusp_5: Decimal,
    /// 6th house cusp longitude
    pub cusp_6: Decimal,
    /// 7th house cusp longitude (Descendant)
    pub cusp_7: Decimal,
    /// 8th house cusp longitude
    pub cusp_8: Decimal,
    /// 9th house cusp longitude
    pub cusp_9: Decimal,
    /// 10th house cusp longitude (Midheaven/MC)
    pub cusp_10: Decimal,
    /// 11th house cusp longitude
    pub cusp_11: Decimal,
    /// 12th house cusp longitude
    pub cusp_12: Decimal,
    /// Ascendant (same as cusp_1)
    pub ascendant: Decimal,
    /// Midheaven (same as cusp_10)
    pub mc: Decimal,
}

/// Pre-aggregated aspect counts per body per time
///
/// Maps to the `aspect_summaries` table.
/// Eliminates correlated subqueries in wedding/electional queries.
/// Provides 51× performance improvement for aspect counting queries.
#[derive(Debug, Clone, FromRow)]
pub struct AspectSummary {
    /// Timestamp of the summary
    pub time: chrono::DateTime<chrono::Utc>,
    /// Body ID (0-9)
    pub body_id: i16,
    /// Number of conjunction aspects (0° ± orb)
    pub conjunctions: i16,
    /// Number of sextile aspects (60° ± orb)
    pub sextiles: i16,
    /// Number of square aspects (90° ± orb)
    pub squares: i16,
    /// Number of trine aspects (120° ± orb)
    pub trines: i16,
    /// Number of opposition aspects (180° ± orb)
    pub oppositions: i16,
    /// Sum of trines and sextiles ("favorable" aspects)
    pub total_favorable: i16,
    /// Sum of squares and oppositions ("challenging" aspects)
    pub total_challenging: i16,
}

// ============================================================================
// SCHEMA HELPER
// ============================================================================

pub struct ChartSchema;

#[allow(dead_code)]
impl ChartSchema {
    pub fn create_table_sql() -> &'static str {
        r#"
CREATE TABLE IF NOT EXISTS charts (
    id SERIAL PRIMARY KEY,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    latitude DOUBLE PRECISION NOT NULL,
    longitude DOUBLE PRECISION NOT NULL,
    altitude DOUBLE PRECISION DEFAULT 0,
    julian_day DOUBLE PRECISION NOT NULL,
    planets JSONB NOT NULL,
    houses JSONB NOT NULL,
    sidereal_time DOUBLE PRECISION NOT NULL
);
"#
    }
}

// ============================================================================
// TYPE ALIASES FOR DOMAIN CLARITY
// ============================================================================

/// Body ID constants for type safety
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

/// Aspect type constants
pub mod aspect_types {
    pub const CONJUNCTION: i16 = 0;
    pub const SEXTILE: i16 = 1;
    pub const SQUARE: i16 = 2;
    pub const TRINE: i16 = 3;
    pub const OPPOSITION: i16 = 4;
}

/// Zodiac sign constants
pub mod zodiac_signs {
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

/// Moon phase constants
pub mod moon_phases {
    pub const NEW: i16 = 0;
    pub const WAXING_CRESCENT: i16 = 1;
    pub const FIRST_QUARTER: i16 = 2;
    pub const WAXING_GIBBOUS: i16 = 3;
    pub const FULL: i16 = 4;
    pub const WANING_GIBBOUS: i16 = 5;
    pub const LAST_QUARTER: i16 = 6;
    pub const WANING_CRESCENT: i16 = 7;
}

/// House system constants
pub mod house_systems {
    pub const PLACIDUS: i16 = 0;
    pub const KOCH: i16 = 1;
    pub const EQUAL: i16 = 2;
    pub const WHOLE_SIGN: i16 = 3;
    pub const PORPHYRY: i16 = 4;
    pub const REGIOMONTANUS: i16 = 5;
    pub const CAMPANUS: i16 = 6;
    pub const MORINUS: i16 = 7;
    pub const TOPOCENTRIC: i16 = 8;
    pub const ALCABITIUS: i16 = 9;
    pub const AZIMUTHAL: i16 = 10;
}
