use std::{fmt, str::FromStr};

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GeoPos {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
}

impl GeoPos {
    pub fn new(latitude: f64, longitude: f64, altitude: f64) -> Self {
        Self {
            latitude,
            longitude,
            altitude,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Position {
    pub longitude: f64,
    pub latitude: f64,
    pub distance: f64,
    pub speed_lon: f64,
    pub speed_lat: f64,
    pub speed_dist: f64,
}

impl Position {
    pub fn new(
        longitude: f64,
        latitude: f64,
        distance: f64,
        speed_lon: f64,
        speed_lat: f64,
        speed_dist: f64,
    ) -> Self {
        Self {
            longitude,
            latitude,
            distance,
            speed_lon,
            speed_lat,
            speed_dist,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HouseCusps {
    pub asc: f64,
    pub mc: f64,
    pub dc: f64,
    pub ic: f64,
    pub houses: [f64; 12],
    pub system: HouseSystem,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum HouseSystem {
    Placidus,
    Koch,
    Equal,
    Whole,
    Porphyry,
    Regiomontanus,
    Campanus,
    Morinus,
    Alcabitus,
    Topocentric,
    Vehlow,
}

impl fmt::Display for HouseSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HouseSystem::Placidus => write!(f, "Placidus"),
            HouseSystem::Koch => write!(f, "Koch"),
            HouseSystem::Equal => write!(f, "Equal"),
            HouseSystem::Whole => write!(f, "Whole"),
            HouseSystem::Porphyry => write!(f, "Porphyry"),
            HouseSystem::Regiomontanus => write!(f, "Regiomontanus"),
            HouseSystem::Campanus => write!(f, "Campanus"),
            HouseSystem::Morinus => write!(f, "Morinus"),
            HouseSystem::Alcabitus => write!(f, "Alcabitus"),
            HouseSystem::Topocentric => write!(f, "Topocentric"),
            HouseSystem::Vehlow => write!(f, "Vehlow"),
        }
    }
}

impl FromStr for HouseSystem {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "placidus" => Ok(HouseSystem::Placidus),
            "koch" => Ok(HouseSystem::Koch),
            "equal" => Ok(HouseSystem::Equal),
            "whole" | "wholesign" | "whole_sign" => Ok(HouseSystem::Whole),
            "porphyry" => Ok(HouseSystem::Porphyry),
            "regiomontanus" => Ok(HouseSystem::Regiomontanus),
            "campanus" => Ok(HouseSystem::Campanus),
            "morinus" => Ok(HouseSystem::Morinus),
            "alcabitus" => Ok(HouseSystem::Alcabitus),
            "topocentric" => Ok(HouseSystem::Topocentric),
            "vehlow" => Ok(HouseSystem::Vehlow),
            _ => Err(format!(
                "Invalid house system: '{}'. Valid options: Placidus, Koch, Equal, Whole, Porphyry, Regiomontanus, Campanus, Morinus, Alcabitus, Topocentric, Vehlow",
                s
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChartData {
    pub geo_pos: GeoPos,
    pub julian_day: f64,
    pub planets: Vec<PlanetPosition>,
    pub houses: HouseCusps,
    pub sidereal_time: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlanetPosition {
    pub name: String,
    pub position: Position,
    pub retrograde: bool,
}

impl PlanetPosition {
    pub fn new(name: &str, position: Position, retrograde: bool) -> Self {
        Self {
            name: name.to_string(),
            position,
            retrograde,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChartConfig {
    pub house_system: HouseSystem,
    pub geo_pos: GeoPos,
    pub julian_day: f64,
}

impl ChartConfig {
    pub fn new(house_system: HouseSystem, geo_pos: GeoPos, julian_day: f64) -> Self {
        Self {
            house_system,
            geo_pos,
            julian_day,
        }
    }
}

pub trait ChartCalculator {
    fn new(config: ChartConfig) -> Result<Self, Error>
    where
        Self: Sized;

    fn calculate_planets(&self) -> Result<Vec<PlanetPosition>, Error>;
    fn calculate_houses(&self) -> Result<HouseCusps, Error>;
    fn calculate_sidereal_time(&self) -> Result<f64, Error>;
    fn calculate_chart(&self) -> Result<ChartData, Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Swiss Ephemeris error: {0}")]
    SwissEph(String),
    #[error("Invalid calculation parameters: {0}")]
    InvalidParams(String),
    #[error("Missing ephemeris data")]
    MissingEphemerisData,
    #[error("Unknown planet: {0}")]
    UnknownPlanet(String),
    #[error("Calculation failed: {0}")]
    CalculationFailed(String),
}

pub mod planet {
    pub const SUN: &str = "Sun";
    pub const MOON: &str = "Moon";
    pub const MERCURY: &str = "Mercury";
    pub const VENUS: &str = "Venus";
    pub const MARS: &str = "Mars";
    pub const JUPITER: &str = "Jupiter";
    pub const SATURN: &str = "Saturn";
    pub const URANUS: &str = "Uranus";
    pub const NEPTUNE: &str = "Neptune";
    pub const PLUTO: &str = "Pluto";
    pub const CHIRON: &str = "Chiron";
    pub const MEAN_NODE: &str = "Mean Node";
    pub const TRUE_NODE: &str = "True Node";
    pub const ASCENDANT: &str = "Ascendant";
    pub const MIDHEAVEN: &str = "Midheaven";
}

pub mod zodiac {
    pub const ARIEST: &str = "♈";
    pub const TAURUS: &str = "♉";
    pub const GEMINI: &str = "♊";
    pub const CANCER: &str = "♋";
    pub const LEO: &str = "♌";
    pub const VIRGO: &str = "♍";
    pub const LIBRA: &str = "♎";
    pub const SCORPIO: &str = "♏";
    pub const SAGITTARIUS: &str = "♐";
    pub const CAPRICORN: &str = "♑";
    pub const AQUARIUS: &str = "♒";
    pub const PISCES: &str = "♓";
}

pub mod house_system {
    pub const PLACIDUS: &str = "Placidus";
    pub const KOCH: &str = "Koch";
    pub const EQUAL: &str = "Equal";
    pub const WHOLE: &str = "Whole";
    pub const PORPHYRY: &str = "Porphyry";
    pub const REGIOMONTANUS: &str = "Regiomontanus";
    pub const CAMPANUS: &str = "Campanus";
    pub const MORINUS: &str = "Morinus";
    pub const ALCABITUS: &str = "Alcabitus";
    pub const TOPOCENTRIC: &str = "Topocentric";
    pub const VEHLOW: &str = "Vehlow";
}

pub mod constants {
    pub const DEG_TO_RAD: f64 = std::f64::consts::PI / 180.0;
    pub const RAD_TO_DEG: f64 = 180.0 / std::f64::consts::PI;
    pub const AU_TO_KM: f64 = 149_597_870.7;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geo_pos_creation() {
        let pos = GeoPos::new(40.7128, -74.0060, 0.0);
        assert_eq!(pos.latitude, 40.7128);
        assert_eq!(pos.longitude, -74.0060);
        assert_eq!(pos.altitude, 0.0);
    }

    #[test]
    fn test_position_creation() {
        let pos = Position::new(10.0, 20.0, 30.0, 0.1, 0.2, 0.3);
        assert_eq!(pos.longitude, 10.0);
        assert_eq!(pos.latitude, 20.0);
        assert_eq!(pos.distance, 30.0);
        assert_eq!(pos.speed_lon, 0.1);
        assert_eq!(pos.speed_lat, 0.2);
        assert_eq!(pos.speed_dist, 0.3);
    }

    #[test]
    fn test_house_system_display() {
        assert_eq!(HouseSystem::Placidus.to_string(), "Placidus");
        assert_eq!(HouseSystem::Koch.to_string(), "Koch");
    }
}
