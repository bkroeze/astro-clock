use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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
