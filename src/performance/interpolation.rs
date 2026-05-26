use crate::database::schema::PlanetPosition;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

/// Errors that can occur during interpolation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpolationError {
    InvalidTimeRange,
    TimeOutOfRange,
}

impl std::fmt::Display for InterpolationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpolationError::InvalidTimeRange => write!(f, "End time must be after start time"),
            InterpolationError::TimeOutOfRange => {
                write!(f, "Query time outside interpolation range")
            }
        }
    }
}

impl std::error::Error for InterpolationError {}

/// Interpolate longitude handling 360° wraparound
/// Longitude wraps at 360°, so interpolating from 350° to 10° should go forward through 360°
pub fn interpolate_longitude(start: f64, end: f64, fraction: f64) -> f64 {
    // Calculate shortest path around the circle
    let diff = end - start;
    let adjusted_diff = if diff > 180.0 {
        diff - 360.0
    } else if diff < -180.0 {
        diff + 360.0
    } else {
        diff
    };

    let result = start + adjusted_diff * fraction;
    // Normalize to [0, 360)
    (result % 360.0 + 360.0) % 360.0
}

/// Interpolate latitude (simple linear interpolation)
pub fn interpolate_latitude(start: f64, end: f64, fraction: f64) -> f64 {
    start + (end - start) * fraction
}

/// Calculate zodiac sign from longitude (0-11 for Aries-Pisces)
fn calculate_zodiac_sign(longitude: f64) -> i16 {
    ((longitude / 30.0) as i16) % 12
}

/// Interpolate between two planet positions
/// Used when querying outer planets at higher resolution than stored (60-min)
pub fn interpolate_position(
    start: &PlanetPosition,
    end: &PlanetPosition,
    query_time: DateTime<Utc>,
) -> Result<PlanetPosition, InterpolationError> {
    if start.time >= end.time {
        return Err(InterpolationError::InvalidTimeRange);
    }

    if query_time < start.time || query_time > end.time {
        return Err(InterpolationError::TimeOutOfRange);
    }

    // Calculate fraction between start and end
    let total_duration = (end.time - start.time).num_seconds() as f64;
    let query_offset = (query_time - start.time).num_seconds() as f64;
    let fraction = query_offset / total_duration;

    let start_lon = start.longitude.to_f64().unwrap_or(0.0);
    let end_lon = end.longitude.to_f64().unwrap_or(0.0);
    let interpolated_lon = interpolate_longitude(start_lon, end_lon, fraction);

    Ok(PlanetPosition {
        time: query_time,
        body_id: start.body_id,
        longitude: Decimal::from_f64_retain(interpolated_lon).unwrap_or(Decimal::ZERO),
        latitude: Decimal::from_f64_retain(interpolate_latitude(
            start.latitude.to_f64().unwrap_or(0.0),
            end.latitude.to_f64().unwrap_or(0.0),
            fraction,
        ))
        .unwrap_or(Decimal::ZERO),
        distance: Decimal::from_f64_retain(interpolate_latitude(
            start.distance.to_f64().unwrap_or(0.0),
            end.distance.to_f64().unwrap_or(0.0),
            fraction,
        ))
        .unwrap_or(Decimal::ZERO),
        speed_lon: Decimal::from_f64_retain(interpolate_latitude(
            start.speed_lon.to_f64().unwrap_or(0.0),
            end.speed_lon.to_f64().unwrap_or(0.0),
            fraction,
        ))
        .unwrap_or(Decimal::ZERO),
        retrograde: start.retrograde, // Keep retrograde status from start point
        zodiac_sign: calculate_zodiac_sign(interpolated_lon),
    })
}

/// Interpolate multiple positions between two stored points
/// Returns interpolated positions for all query_times between start and end
pub fn interpolate_positions(
    start: &PlanetPosition,
    end: &PlanetPosition,
    query_times: &[DateTime<Utc>],
) -> Result<Vec<PlanetPosition>, InterpolationError> {
    query_times
        .iter()
        .map(|&t| interpolate_position(start, end, t))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    fn create_position(time: DateTime<Utc>, longitude: f64, latitude: f64) -> PlanetPosition {
        PlanetPosition {
            time,
            body_id: 5, // Jupiter
            longitude: Decimal::from_f64_retain(longitude).unwrap(),
            latitude: Decimal::from_f64_retain(latitude).unwrap(),
            distance: Decimal::from_f64_retain(5.2).unwrap(),
            speed_lon: Decimal::from_f64_retain(0.08).unwrap(),
            retrograde: false,
            zodiac_sign: calculate_zodiac_sign(longitude),
        }
    }

    #[test]
    fn test_interpolate_longitude_simple() {
        // Simple interpolation: 10° to 20°
        let result = interpolate_longitude(10.0, 20.0, 0.5);
        assert!((result - 15.0).abs() < 0.001);
    }

    #[test]
    fn test_interpolate_longitude_wraparound_forward() {
        // Wraparound: 350° to 10° should go forward through 360°
        let result = interpolate_longitude(350.0, 10.0, 0.5);
        // Should be at 0° (halfway between 350° and 10° going forward)
        assert!((result - 0.0).abs() < 0.001 || (result - 360.0).abs() < 0.001);
    }

    #[test]
    fn test_interpolate_longitude_wraparound_backward() {
        // Wraparound: 10° to 350° should go backward through 0°
        let result = interpolate_longitude(10.0, 350.0, 0.5);
        // Should be at 0° (halfway between 10° and 350° going backward)
        assert!((result - 0.0).abs() < 0.001 || (result - 360.0).abs() < 0.001);
    }

    #[test]
    fn test_interpolate_longitude_normalization() {
        // Result should always be in [0, 360)
        let result = interpolate_longitude(350.0, 20.0, 0.5);
        assert!(result >= 0.0 && result < 360.0);
    }

    #[test]
    fn test_interpolate_latitude() {
        let result = interpolate_latitude(5.0, 15.0, 0.5);
        assert!((result - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_zodiac_sign() {
        assert_eq!(calculate_zodiac_sign(0.0), 0); // Aries
        assert_eq!(calculate_zodiac_sign(30.0), 1); // Taurus
        assert_eq!(calculate_zodiac_sign(350.0), 11); // Pisces
    }

    #[test]
    fn test_interpolate_position_basic() {
        let start_time = Utc::now();
        let end_time = start_time + chrono::Duration::minutes(60);
        let query_time = start_time + chrono::Duration::minutes(30);

        let start = create_position(start_time, 100.0, 5.0);
        let end = create_position(end_time, 110.0, 15.0);

        let result = interpolate_position(&start, &end, query_time).unwrap();

        assert_eq!(result.time, query_time);
        assert_eq!(result.body_id, 5);
        // Longitude should be halfway: 105°
        let lon = result.longitude.to_f64().unwrap();
        assert!((lon - 105.0).abs() < 0.001);
        // Latitude should be halfway: 10°
        let lat = result.latitude.to_f64().unwrap();
        assert!((lat - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_interpolate_position_wraparound() {
        let start_time = Utc::now();
        let end_time = start_time + chrono::Duration::minutes(60);
        let query_time = start_time + chrono::Duration::minutes(30);

        let start = create_position(start_time, 355.0, 5.0);
        let end = create_position(end_time, 5.0, 15.0);

        let result = interpolate_position(&start, &end, query_time).unwrap();

        // Longitude should be at 0° (halfway between 355° and 5°)
        let lon = result.longitude.to_f64().unwrap();
        assert!((lon - 0.0).abs() < 0.001 || (lon - 360.0).abs() < 0.001);
    }

    #[test]
    fn test_interpolate_position_invalid_range() {
        let start_time = Utc::now();
        let end_time = start_time - chrono::Duration::minutes(60); // End before start

        let start = create_position(start_time, 100.0, 5.0);
        let end = create_position(end_time, 110.0, 15.0);

        let result = interpolate_position(&start, &end, start_time);
        assert!(matches!(result, Err(InterpolationError::InvalidTimeRange)));
    }

    #[test]
    fn test_interpolate_position_out_of_range() {
        let start_time = Utc::now();
        let end_time = start_time + chrono::Duration::minutes(60);
        let query_time = start_time - chrono::Duration::minutes(30); // Before start

        let start = create_position(start_time, 100.0, 5.0);
        let end = create_position(end_time, 110.0, 15.0);

        let result = interpolate_position(&start, &end, query_time);
        assert!(matches!(result, Err(InterpolationError::TimeOutOfRange)));
    }

    #[test]
    fn test_interpolate_positions_batch() {
        let start_time = Utc::now();
        let end_time = start_time + chrono::Duration::minutes(60);
        let query_times = vec![
            start_time + chrono::Duration::minutes(15),
            start_time + chrono::Duration::minutes(30),
            start_time + chrono::Duration::minutes(45),
        ];

        let start = create_position(start_time, 100.0, 5.0);
        let end = create_position(end_time, 110.0, 15.0);

        let results = interpolate_positions(&start, &end, &query_times).unwrap();

        assert_eq!(results.len(), 3);
        // At 25%: 102.5°
        let lon0 = results[0].longitude.to_f64().unwrap();
        assert!((lon0 - 102.5).abs() < 0.001);
        // At 50%: 105°
        let lon1 = results[1].longitude.to_f64().unwrap();
        assert!((lon1 - 105.0).abs() < 0.001);
        // At 75%: 107.5°
        let lon2 = results[2].longitude.to_f64().unwrap();
        assert!((lon2 - 107.5).abs() < 0.001);
    }

    #[test]
    fn test_error_display() {
        let err1 = InterpolationError::InvalidTimeRange;
        assert!(
            err1.to_string()
                .contains("End time must be after start time")
        );

        let err2 = InterpolationError::TimeOutOfRange;
        assert!(
            err2.to_string()
                .contains("Query time outside interpolation range")
        );
    }
}
