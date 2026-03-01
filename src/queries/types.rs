use chrono::{DateTime, Duration, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::error::QueryError;

// ============================================================================
// QUERY RESULT TYPE WITH METADATA
// ============================================================================

/// Generic query result with execution metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult<T> {
    pub data: Vec<T>,
    pub execution_time_ms: u64,
    pub rows_examined: usize,
    pub cache_hit: bool,
}

impl<T> QueryResult<T> {
    pub fn new(
        data: Vec<T>,
        execution_time_ms: u64,
        rows_examined: usize,
        cache_hit: bool,
    ) -> Self {
        Self {
            data,
            execution_time_ms,
            rows_examined,
            cache_hit,
        }
    }
}

// ============================================================================
// CRITERIA STRUCTS WITH VALIDATION
// ============================================================================

/// Criteria for wedding date queries (QUERY-01)
#[derive(Debug, Clone)]
pub struct WeddingCriteria {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub min_venus_aspects: i16,
    pub limit: usize,
}

impl WeddingCriteria {
    /// Create new wedding criteria with defaults
    pub fn new(start_date: NaiveDate, end_date: NaiveDate) -> Self {
        Self {
            start_date,
            end_date,
            min_venus_aspects: 2,
            limit: 10,
        }
    }

    /// Set minimum Venus aspects threshold
    pub fn with_min_venus_aspects(mut self, min: i16) -> Self {
        self.min_venus_aspects = min;
        self
    }

    /// Set result limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Validate criteria
    pub fn validate(&self) -> Result<(), QueryError> {
        if self.start_date > self.end_date {
            return Err(QueryError::InvalidCriteria(
                "Start date must be before end date".to_string(),
            ));
        }

        let max_range = Duration::days(365);
        let date_range = self.end_date.signed_duration_since(self.start_date);
        if date_range > max_range {
            return Err(QueryError::InvalidCriteria(
                "Date range cannot exceed 1 year".to_string(),
            ));
        }

        if self.min_venus_aspects < 0 {
            return Err(QueryError::InvalidCriteria(
                "Minimum Venus aspects cannot be negative".to_string(),
            ));
        }

        if self.limit == 0 || self.limit > 1000 {
            return Err(QueryError::InvalidCriteria(
                "Limit must be between 1 and 1000".to_string(),
            ));
        }

        Ok(())
    }
}

/// Criteria for void-of-course Moon queries (QUERY-02)
#[derive(Debug, Clone)]
pub struct VoCCriteria {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub min_duration: Option<Duration>,
}

impl VoCCriteria {
    /// Create new VoC criteria
    pub fn new(start_date: NaiveDate, end_date: NaiveDate) -> Self {
        Self {
            start_date,
            end_date,
            min_duration: None,
        }
    }

    /// Set minimum duration filter
    pub fn with_min_duration(mut self, duration: Duration) -> Self {
        self.min_duration = Some(duration);
        self
    }

    /// Validate criteria
    pub fn validate(&self) -> Result<(), QueryError> {
        if self.start_date > self.end_date {
            return Err(QueryError::InvalidCriteria(
                "Start date must be before end date".to_string(),
            ));
        }

        let max_range = Duration::days(365);
        let date_range = self.end_date.signed_duration_since(self.start_date);
        if date_range > max_range {
            return Err(QueryError::InvalidCriteria(
                "Date range cannot exceed 1 year".to_string(),
            ));
        }

        if let Some(duration) = self.min_duration {
            if duration < Duration::zero() {
                return Err(QueryError::InvalidCriteria(
                    "Minimum duration cannot be negative".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// Criteria for retrograde period queries (QUERY-03)
#[derive(Debug, Clone)]
pub struct RetrogradeCriteria {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub planets: Option<Vec<Body>>,
}

impl RetrogradeCriteria {
    /// Create new retrograde criteria
    pub fn new(start_date: NaiveDate, end_date: NaiveDate) -> Self {
        Self {
            start_date,
            end_date,
            planets: None,
        }
    }

    /// Filter by specific planets
    pub fn with_planets(mut self, planets: Vec<Body>) -> Self {
        self.planets = Some(planets);
        self
    }

    /// Validate criteria
    pub fn validate(&self) -> Result<(), QueryError> {
        if self.start_date > self.end_date {
            return Err(QueryError::InvalidCriteria(
                "Start date must be before end date".to_string(),
            ));
        }

        let max_range = Duration::days(365);
        let date_range = self.end_date.signed_duration_since(self.start_date);
        if date_range > max_range {
            return Err(QueryError::InvalidCriteria(
                "Date range cannot exceed 1 year".to_string(),
            ));
        }

        if let Some(ref planets) = self.planets {
            for body in planets {
                if !body.is_valid() {
                    return Err(QueryError::InvalidCriteria(format!(
                        "Invalid body ID: {:?}",
                        body
                    )));
                }
            }
        }

        Ok(())
    }
}

/// Criteria for exact aspect queries (QUERY-04)
#[derive(Debug, Clone)]
pub struct AspectCriteria {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub orb_threshold: Decimal,
    pub aspect_types: Option<Vec<AspectType>>,
    pub body_pairs: Option<Vec<(Body, Body)>>,
}

impl AspectCriteria {
    /// Create new aspect criteria with defaults
    pub fn new(start_date: NaiveDate, end_date: NaiveDate) -> Self {
        Self {
            start_date,
            end_date,
            orb_threshold: Decimal::from(1), // 1 degree default
            aspect_types: None,
            body_pairs: None,
        }
    }

    /// Set orb threshold (0-10 degrees)
    pub fn with_orb_threshold(mut self, orb: Decimal) -> Self {
        self.orb_threshold = orb;
        self
    }

    /// Filter by specific aspect types
    pub fn with_aspect_types(mut self, types: Vec<AspectType>) -> Self {
        self.aspect_types = Some(types);
        self
    }

    /// Filter by specific body pairs
    pub fn with_body_pairs(mut self, pairs: Vec<(Body, Body)>) -> Self {
        self.body_pairs = Some(pairs);
        self
    }

    /// Validate criteria
    pub fn validate(&self) -> Result<(), QueryError> {
        if self.start_date > self.end_date {
            return Err(QueryError::InvalidCriteria(
                "Start date must be before end date".to_string(),
            ));
        }

        let max_range = Duration::days(365);
        let date_range = self.end_date.signed_duration_since(self.start_date);
        if date_range > max_range {
            return Err(QueryError::InvalidCriteria(
                "Date range cannot exceed 1 year".to_string(),
            ));
        }

        // Orb threshold must be 0-10 degrees
        let zero = Decimal::from(0);
        let ten = Decimal::from(10);
        if self.orb_threshold < zero || self.orb_threshold > ten {
            return Err(QueryError::InvalidCriteria(
                "Orb threshold must be between 0 and 10 degrees".to_string(),
            ));
        }

        if let Some(ref pairs) = self.body_pairs {
            for (body1, body2) in pairs {
                if !body1.is_valid() || !body2.is_valid() {
                    return Err(QueryError::InvalidCriteria(
                        "Invalid body ID in body pairs".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }
}

// ============================================================================
// RESULT STRUCTS
// ============================================================================

/// Wedding date candidate result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeddingCandidate {
    pub datetime: DateTime<Utc>,
    pub moon_sign: ZodiacSign,
    pub venus_favorable_aspects: i16,
}

/// Void-of-course Moon period result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoCPeriod {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub duration: Duration,
    pub moon_sign: ZodiacSign,
}

/// Retrograde period result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrogradePeriod {
    pub planet: Body,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub shadow_start: Option<DateTime<Utc>>,
    pub shadow_end: Option<DateTime<Utc>>,
    pub status: RetrogradeStatus,
}

/// Exact aspect result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExactAspect {
    pub datetime: DateTime<Utc>,
    pub body1: Body,
    pub body2: Body,
    pub aspect_type: AspectType,
    pub orb: Decimal,
    pub applying: bool,
}

// ============================================================================
// ENUMS
// ============================================================================

/// Retrograde status for a planet at a specific time
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetrogradeStatus {
    Direct,
    Retrograde,
    PreShadow,
    PostShadow,
}

/// Celestial body
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(i16)]
pub enum Body {
    Sun = 0,
    Moon = 1,
    Mercury = 2,
    Venus = 3,
    Mars = 4,
    Jupiter = 5,
    Saturn = 6,
    Uranus = 7,
    Neptune = 8,
    Pluto = 9,
}

impl Body {
    /// Check if body ID is valid (0-9)
    pub fn is_valid(&self) -> bool {
        matches!(
            self,
            Body::Sun
                | Body::Moon
                | Body::Mercury
                | Body::Venus
                | Body::Mars
                | Body::Jupiter
                | Body::Saturn
                | Body::Uranus
                | Body::Neptune
                | Body::Pluto
        )
    }

    /// Convert from i16 body ID
    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            0 => Some(Body::Sun),
            1 => Some(Body::Moon),
            2 => Some(Body::Mercury),
            3 => Some(Body::Venus),
            4 => Some(Body::Mars),
            5 => Some(Body::Jupiter),
            6 => Some(Body::Saturn),
            7 => Some(Body::Uranus),
            8 => Some(Body::Neptune),
            9 => Some(Body::Pluto),
            _ => None,
        }
    }

    /// Convert to i16 body ID
    pub fn to_id(&self) -> i16 {
        *self as i16
    }
}

/// Astrological aspect type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(i16)]
pub enum AspectType {
    Conjunction = 0,
    Sextile = 1,
    Square = 2,
    Trine = 3,
    Opposition = 4,
}

impl AspectType {
    /// Get the angle for this aspect type in degrees
    pub fn angle(&self) -> i16 {
        match self {
            AspectType::Conjunction => 0,
            AspectType::Sextile => 60,
            AspectType::Square => 90,
            AspectType::Trine => 120,
            AspectType::Opposition => 180,
        }
    }

    /// Convert from i16 aspect type
    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            0 => Some(AspectType::Conjunction),
            1 => Some(AspectType::Sextile),
            2 => Some(AspectType::Square),
            3 => Some(AspectType::Trine),
            4 => Some(AspectType::Opposition),
            _ => None,
        }
    }

    /// Convert to i16 aspect type ID
    pub fn to_id(&self) -> i16 {
        *self as i16
    }

    /// Check if this is a "favorable" aspect (trine or sextile)
    pub fn is_favorable(&self) -> bool {
        matches!(self, AspectType::Trine | AspectType::Sextile)
    }

    /// Check if this is a "challenging" aspect (square or opposition)
    pub fn is_challenging(&self) -> bool {
        matches!(self, AspectType::Square | AspectType::Opposition)
    }
}

/// Zodiac sign
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(i16)]
pub enum ZodiacSign {
    Aries = 0,
    Taurus = 1,
    Gemini = 2,
    Cancer = 3,
    Leo = 4,
    Virgo = 5,
    Libra = 6,
    Scorpio = 7,
    Sagittarius = 8,
    Capricorn = 9,
    Aquarius = 10,
    Pisces = 11,
}

impl ZodiacSign {
    /// Get the sign name
    pub fn name(&self) -> &'static str {
        match self {
            ZodiacSign::Aries => "Aries",
            ZodiacSign::Taurus => "Taurus",
            ZodiacSign::Gemini => "Gemini",
            ZodiacSign::Cancer => "Cancer",
            ZodiacSign::Leo => "Leo",
            ZodiacSign::Virgo => "Virgo",
            ZodiacSign::Libra => "Libra",
            ZodiacSign::Scorpio => "Scorpio",
            ZodiacSign::Sagittarius => "Sagittarius",
            ZodiacSign::Capricorn => "Capricorn",
            ZodiacSign::Aquarius => "Aquarius",
            ZodiacSign::Pisces => "Pisces",
        }
    }

    /// Check if this is a "favorable" sign for weddings
    /// (Taurus, Cancer, Leo, Libra, Scorpio, Capricorn, Aquarius, Pisces)
    pub fn is_favorable_for_wedding(&self) -> bool {
        matches!(
            self,
            ZodiacSign::Taurus
                | ZodiacSign::Cancer
                | ZodiacSign::Leo
                | ZodiacSign::Libra
                | ZodiacSign::Scorpio
                | ZodiacSign::Capricorn
                | ZodiacSign::Aquarius
                | ZodiacSign::Pisces
        )
    }

    /// Convert from i16 zodiac sign ID
    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            0 => Some(ZodiacSign::Aries),
            1 => Some(ZodiacSign::Taurus),
            2 => Some(ZodiacSign::Gemini),
            3 => Some(ZodiacSign::Cancer),
            4 => Some(ZodiacSign::Leo),
            5 => Some(ZodiacSign::Virgo),
            6 => Some(ZodiacSign::Libra),
            7 => Some(ZodiacSign::Scorpio),
            8 => Some(ZodiacSign::Sagittarius),
            9 => Some(ZodiacSign::Capricorn),
            10 => Some(ZodiacSign::Aquarius),
            11 => Some(ZodiacSign::Pisces),
            _ => None,
        }
    }

    /// Convert to i16 zodiac sign ID
    pub fn to_id(&self) -> i16 {
        *self as i16
    }
}

// ============================================================================
// CONSTANTS
// ============================================================================

/// Favorable Moon signs for wedding dates
pub const FAVORABLE_WEDDING_SIGNS: [ZodiacSign; 8] = [
    ZodiacSign::Taurus,
    ZodiacSign::Cancer,
    ZodiacSign::Leo,
    ZodiacSign::Libra,
    ZodiacSign::Scorpio,
    ZodiacSign::Capricorn,
    ZodiacSign::Aquarius,
    ZodiacSign::Pisces,
];

/// All celestial bodies
pub const ALL_BODIES: [Body; 10] = [
    Body::Sun,
    Body::Moon,
    Body::Mercury,
    Body::Venus,
    Body::Mars,
    Body::Jupiter,
    Body::Saturn,
    Body::Uranus,
    Body::Neptune,
    Body::Pluto,
];

/// All aspect types
pub const ALL_ASPECT_TYPES: [AspectType; 5] = [
    AspectType::Conjunction,
    AspectType::Sextile,
    AspectType::Square,
    AspectType::Trine,
    AspectType::Opposition,
];

// ============================================================================
// PROJECT AND TRAVEL QUERY TYPES (QUERY-07, QUERY-08)
// ============================================================================

/// Purpose of travel (for future extensibility)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TravelPurpose {
    Business,
    Leisure,
    Relocation,
}

/// Criteria for project date queries (QUERY-07)
#[derive(Debug, Clone)]
pub struct ProjectCriteria {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub limit: usize,
}

impl ProjectCriteria {
    /// Create new project criteria with defaults
    pub fn new(start_date: NaiveDate, end_date: NaiveDate) -> Self {
        Self {
            start_date,
            end_date,
            limit: 10,
        }
    }

    /// Set result limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Validate criteria
    pub fn validate(&self) -> Result<(), QueryError> {
        if self.start_date > self.end_date {
            return Err(QueryError::InvalidCriteria(
                "Start date must be before end date".to_string(),
            ));
        }

        let max_range = Duration::days(365);
        let date_range = self.end_date.signed_duration_since(self.start_date);
        if date_range > max_range {
            return Err(QueryError::InvalidCriteria(
                "Date range cannot exceed 1 year".to_string(),
            ));
        }

        if self.limit == 0 || self.limit > 1000 {
            return Err(QueryError::InvalidCriteria(
                "Limit must be between 1 and 1000".to_string(),
            ));
        }

        Ok(())
    }
}

/// Criteria for travel date queries (QUERY-08)
#[derive(Debug, Clone)]
pub struct TravelCriteria {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub limit: usize,
    pub purpose: Option<TravelPurpose>,
}

impl TravelCriteria {
    /// Create new travel criteria with defaults
    pub fn new(start_date: NaiveDate, end_date: NaiveDate) -> Self {
        Self {
            start_date,
            end_date,
            limit: 10,
            purpose: None,
        }
    }

    /// Set result limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Set travel purpose
    pub fn with_purpose(mut self, purpose: TravelPurpose) -> Self {
        self.purpose = Some(purpose);
        self
    }

    /// Validate criteria
    pub fn validate(&self) -> Result<(), QueryError> {
        if self.start_date > self.end_date {
            return Err(QueryError::InvalidCriteria(
                "Start date must be before end date".to_string(),
            ));
        }

        let max_range = Duration::days(365);
        let date_range = self.end_date.signed_duration_since(self.start_date);
        if date_range > max_range {
            return Err(QueryError::InvalidCriteria(
                "Date range cannot exceed 1 year".to_string(),
            ));
        }

        if self.limit == 0 || self.limit > 1000 {
            return Err(QueryError::InvalidCriteria(
                "Limit must be between 1 and 1000".to_string(),
            ));
        }

        Ok(())
    }
}

/// Project date candidate result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCandidate {
    pub datetime: DateTime<Utc>,
    pub moon_sign: ZodiacSign,
    pub mercury_direct: bool,
    pub favorable_aspects: i16,
}

/// Travel date candidate result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TravelCandidate {
    pub datetime: DateTime<Utc>,
    pub moon_sign: ZodiacSign,
    pub mercury_direct: bool,
    pub moon_void_of_course: bool,
    pub favorable_aspects: i16,
}

/// Favorable Moon signs for starting projects (avoid Scorpio, Capricorn)
pub const FAVORABLE_PROJECT_SIGNS: [ZodiacSign; 7] = [
    ZodiacSign::Taurus,
    ZodiacSign::Cancer,
    ZodiacSign::Leo,
    ZodiacSign::Libra,
    ZodiacSign::Aquarius,
    ZodiacSign::Pisces,
    ZodiacSign::Aries,
];

/// Favorable Moon signs for travel (same as wedding but exclude Scorpio/Capricorn)
pub const FAVORABLE_TRAVEL_SIGNS: [ZodiacSign; 7] = [
    ZodiacSign::Taurus,
    ZodiacSign::Cancer,
    ZodiacSign::Leo,
    ZodiacSign::Libra,
    ZodiacSign::Aquarius,
    ZodiacSign::Pisces,
    ZodiacSign::Gemini,
];

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_criteria_validation() {
        // Valid criteria
        let criteria = ProjectCriteria::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        );
        assert!(criteria.validate().is_ok());

        // Invalid: start after end
        let invalid = ProjectCriteria::new(
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );
        assert!(invalid.validate().is_err());

        // Invalid: range too large
        let too_large = ProjectCriteria::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 6, 1).unwrap(),
        );
        assert!(too_large.validate().is_err());

        // Invalid: limit too large
        let bad_limit = ProjectCriteria::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        )
        .with_limit(1001);
        assert!(bad_limit.validate().is_err());

        // Invalid: limit zero
        let zero_limit = ProjectCriteria::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        )
        .with_limit(0);
        assert!(zero_limit.validate().is_err());
    }

    #[test]
    fn test_travel_criteria_validation() {
        // Valid criteria
        let criteria = TravelCriteria::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        )
        .with_purpose(TravelPurpose::Business);
        assert!(criteria.validate().is_ok());

        // Invalid: start after end
        let invalid = TravelCriteria::new(
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );
        assert!(invalid.validate().is_err());

        // Invalid: range too large
        let too_large = TravelCriteria::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 6, 1).unwrap(),
        );
        assert!(too_large.validate().is_err());

        // Valid: limit within bounds
        let valid_limit = TravelCriteria::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        )
        .with_limit(100);
        assert!(valid_limit.validate().is_ok());
    }

    #[test]
    fn test_candidate_serialization() {
        let project = ProjectCandidate {
            datetime: Utc::now(),
            moon_sign: ZodiacSign::Taurus,
            mercury_direct: true,
            favorable_aspects: 5,
        };

        let json = serde_json::to_string(&project).unwrap();
        assert!(json.contains("Taurus"));

        let travel = TravelCandidate {
            datetime: Utc::now(),
            moon_sign: ZodiacSign::Cancer,
            mercury_direct: true,
            moon_void_of_course: false,
            favorable_aspects: 3,
        };

        let json = serde_json::to_string(&travel).unwrap();
        assert!(json.contains("Cancer"));
    }

    #[test]
    fn test_favorable_signs_constants() {
        // Verify project signs don't include Scorpio or Capricorn
        assert!(!FAVORABLE_PROJECT_SIGNS.contains(&ZodiacSign::Scorpio));
        assert!(!FAVORABLE_PROJECT_SIGNS.contains(&ZodiacSign::Capricorn));

        // Verify travel signs don't include Scorpio or Capricorn
        assert!(!FAVORABLE_TRAVEL_SIGNS.contains(&ZodiacSign::Scorpio));
        assert!(!FAVORABLE_TRAVEL_SIGNS.contains(&ZodiacSign::Capricorn));

        // Verify both include Taurus (should be favorable for both)
        assert!(FAVORABLE_PROJECT_SIGNS.contains(&ZodiacSign::Taurus));
        assert!(FAVORABLE_TRAVEL_SIGNS.contains(&ZodiacSign::Taurus));
    }
}
