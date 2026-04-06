use serde::Serialize;

use crate::chart::{planet, PlanetPosition};

/// The type of astrological aspect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AspectType {
    Conjunction,
    Opposition,
    Square,
    Trine,
    Sextile,
    GrandTrine,
}

impl AspectType {
    /// Get the angle for this aspect type (in degrees)
    pub fn angle(&self) -> f64 {
        match self {
            AspectType::Conjunction => 0.0,
            AspectType::Opposition => 180.0,
            AspectType::Square => 90.0,
            AspectType::Trine => 120.0,
            AspectType::Sextile => 60.0,
            AspectType::GrandTrine => 120.0,
        }
    }

    /// Get the display name for this aspect
    pub fn name(&self) -> &'static str {
        match self {
            AspectType::Conjunction => "Conjunction",
            AspectType::Opposition => "Opposition",
            AspectType::Square => "Square",
            AspectType::Trine => "Trine",
            AspectType::Sextile => "Sextile",
            AspectType::GrandTrine => "Grand Trine",
        }
    }

    /// Get the symbol for this aspect
    pub fn symbol(&self) -> &'static str {
        match self {
            AspectType::Conjunction => "☌",
            AspectType::Opposition => "☍",
            AspectType::Square => "□",
            AspectType::Trine => "△",
            AspectType::Sextile => "✶",
            AspectType::GrandTrine => "△△",
        }
    }
}

/// Represents a single aspect between two planets
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Aspect {
    pub aspect_type: AspectType,
    pub planet1: String,
    pub planet2: String,
    pub longitude1: f64,
    pub longitude2: f64,
    pub orb: f64,
}

impl Aspect {
    pub fn new(
        aspect_type: AspectType,
        planet1: &str,
        planet2: &str,
        longitude1: f64,
        longitude2: f64,
        orb: f64,
    ) -> Self {
        Self {
            aspect_type,
            planet1: planet1.to_string(),
            planet2: planet2.to_string(),
            longitude1,
            longitude2,
            orb,
        }
    }
}

/// Represents a grand trine (three planets in trine aspect)
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GrandTrine {
    pub planet1: String,
    pub planet2: String,
    pub planet3: String,
    pub longitude1: f64,
    pub longitude2: f64,
    pub longitude3: f64,
}

impl GrandTrine {
    pub fn new(
        planet1: &str,
        planet2: &str,
        planet3: &str,
        longitude1: f64,
        longitude2: f64,
        longitude3: f64,
    ) -> Self {
        Self {
            planet1: planet1.to_string(),
            planet2: planet2.to_string(),
            planet3: planet3.to_string(),
            longitude1,
            longitude2,
            longitude3,
        }
    }
}

/// Configuration for aspect detection
#[derive(Debug, Clone, Copy)]
pub struct AspectConfig {
    /// The maximum orb (in degrees) for considering an aspect valid
    pub orb: f64,
}

impl Default for AspectConfig {
    fn default() -> Self {
        Self { orb: 3.0 }
    }
}

impl AspectConfig {
    pub fn new(orb: f64) -> Self {
        Self { orb }
    }
}

/// Calculate the shortest angular distance between two longitudes
fn angular_distance(long1: f64, long2: f64) -> f64 {
    let diff = (long1 - long2).abs();
    if diff > 180.0 {
        360.0 - diff
    } else {
        diff
    }
}

/// Check if two planets form a specific aspect within the given orb
fn check_aspect(
    planet1: &PlanetPosition,
    planet2: &PlanetPosition,
    aspect_type: AspectType,
    orb: f64,
) -> Option<Aspect> {
    let target_angle = aspect_type.angle();
    let distance = angular_distance(planet1.position.longitude, planet2.position.longitude);
    let orb_value = (distance - target_angle).abs();

    if orb_value <= orb {
        Some(Aspect::new(
            aspect_type,
            &planet1.name,
            &planet2.name,
            planet1.position.longitude,
            planet2.position.longitude,
            orb_value,
        ))
    } else {
        None
    }
}

/// Find all aspects between planets
pub fn find_aspects(planets: &[PlanetPosition], config: AspectConfig) -> Vec<Aspect> {
    let mut aspects = Vec::new();
    let aspect_types = [
        AspectType::Conjunction,
        AspectType::Opposition,
        AspectType::Square,
        AspectType::Trine,
        AspectType::Sextile,
    ];

    // Check each pair of planets once
    for i in 0..planets.len() {
        for j in (i + 1)..planets.len() {
            for aspect_type in &aspect_types {
                if let Some(aspect) =
                    check_aspect(&planets[i], &planets[j], *aspect_type, config.orb)
                {
                    aspects.push(aspect);
                }
            }
        }
    }

    // Sort aspects by orb (smallest first - most exact aspects)
    aspects.sort_by(|a, b| a.orb.partial_cmp(&b.orb).unwrap());

    aspects
}

/// Find all grand trines in the chart
pub fn find_grand_trines(planets: &[PlanetPosition], config: AspectConfig) -> Vec<GrandTrine> {
    let mut grand_trines = Vec::new();
    let mut found_combinations: std::collections::HashSet<(usize, usize, usize)> =
        std::collections::HashSet::new();

    // Find all trine aspects first
    let mut trines: Vec<(usize, usize)> = Vec::new();
    for i in 0..planets.len() {
        for j in (i + 1)..planets.len() {
            if check_aspect(&planets[i], &planets[j], AspectType::Trine, config.orb).is_some() {
                trines.push((i, j));
            }
        }
    }

    // Look for grand trines (three planets where each pair forms a trine)
    for i in 0..trines.len() {
        let (a, b) = trines[i];
        for j in (i + 1)..trines.len() {
            let (c, d) = trines[j];

            // Check if these two trines share a planet and form a grand trine
            let planets_in_trines = [a, b, c, d];
            let unique_planets: Vec<_> = planets_in_trines
                .iter()
                .cloned()
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            if unique_planets.len() == 3 {
                // We have three unique planets, check if they all form trines
                let mut indices = [unique_planets[0], unique_planets[1], unique_planets[2]];
                indices.sort(); // Sort to ensure consistent ordering
                let (p1, p2, p3) = (indices[0], indices[1], indices[2]);

                // Skip if we already found this combination
                if found_combinations.contains(&(p1, p2, p3)) {
                    continue;
                }

                // Verify all three pairs form trines
                let has_trine_12 = trines
                    .iter()
                    .any(|(x, y)| (*x == p1 && *y == p2) || (*x == p2 && *y == p1));
                let has_trine_13 = trines
                    .iter()
                    .any(|(x, y)| (*x == p1 && *y == p3) || (*x == p3 && *y == p1));
                let has_trine_23 = trines
                    .iter()
                    .any(|(x, y)| (*x == p2 && *y == p3) || (*x == p3 && *y == p2));

                if has_trine_12 && has_trine_13 && has_trine_23 {
                    found_combinations.insert((p1, p2, p3));
                    grand_trines.push(GrandTrine::new(
                        &planets[p1].name,
                        &planets[p2].name,
                        &planets[p3].name,
                        planets[p1].position.longitude,
                        planets[p2].position.longitude,
                        planets[p3].position.longitude,
                    ));
                }
            }
        }
    }

    grand_trines
}

/// Check if the Moon is void of course
/// Moon is void of course when it will not make any more major aspects
/// before leaving its current sign
pub fn is_moon_void_of_course(planets: &[PlanetPosition], config: AspectConfig) -> Option<String> {
    let moon = planets.iter().find(|p| p.name == planet::MOON)?;
    let moon_sign_start = (moon.position.longitude / 30.0).floor() * 30.0;
    let moon_sign_end = moon_sign_start + 30.0;

    // Major aspects to check
    let aspect_angles = [0.0, 60.0, 90.0, 120.0, 180.0];

    // Check if Moon will make any major aspects before leaving sign
    for planet in planets {
        if planet.name == planet::MOON {
            continue;
        }

        // Calculate how far the Moon needs to travel to aspect this planet
        for angle in &aspect_angles {
            let target_longitude = (planet.position.longitude + angle) % 360.0;
            let distance = if target_longitude >= moon.position.longitude {
                target_longitude - moon.position.longitude
            } else {
                target_longitude + 360.0 - moon.position.longitude
            };

            // If this aspect happens before Moon leaves sign, Moon is not void
            if distance < (moon_sign_end - moon.position.longitude) {
                if distance <= config.orb || (distance - *angle).abs() <= config.orb {
                    return None;
                }
            }
        }
    }

    Some(format!(
        "Moon is void of course (at {:.2}° {}, will not aspect before leaving sign)",
        moon.position.longitude % 30.0,
        get_sign_name(moon.position.longitude)
    ))
}

fn get_sign_name(longitude: f64) -> &'static str {
    let signs = [
        "Aries",
        "Taurus",
        "Gemini",
        "Cancer",
        "Leo",
        "Virgo",
        "Libra",
        "Scorpio",
        "Sagittarius",
        "Capricorn",
        "Aquarius",
        "Pisces",
    ];
    let sign_index = (longitude / 30.0) as usize % 12;
    signs[sign_index]
}

/// Results of aspect analysis
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AspectAnalysis {
    pub aspects: Vec<Aspect>,
    pub grand_trines: Vec<GrandTrine>,
    pub moon_void_of_course: Option<String>,
}

/// Analyze a chart for all aspects and configurations
pub fn analyze_aspects(planets: &[PlanetPosition], config: AspectConfig) -> AspectAnalysis {
    AspectAnalysis {
        aspects: find_aspects(planets, config),
        grand_trines: find_grand_trines(planets, config),
        moon_void_of_course: is_moon_void_of_course(planets, config),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chart::{PlanetPosition, Position};

    fn create_planet(name: &str, longitude: f64) -> PlanetPosition {
        PlanetPosition::new(
            name,
            Position::new(longitude, 0.0, 1.0, 0.0, 0.0, 0.0),
            false,
        )
    }

    #[test]
    fn test_angular_distance() {
        assert_eq!(angular_distance(0.0, 10.0), 10.0);
        assert_eq!(angular_distance(10.0, 0.0), 10.0);
        assert_eq!(angular_distance(350.0, 10.0), 20.0);
        assert_eq!(angular_distance(10.0, 350.0), 20.0);
        assert_eq!(angular_distance(180.0, 0.0), 180.0);
        assert_eq!(angular_distance(0.0, 180.0), 180.0);
    }

    #[test]
    fn test_find_conjunction() {
        let planets = vec![
            create_planet(planet::SUN, 10.0),
            create_planet(planet::MOON, 12.0), // 2° orb conjunction
        ];

        let config = AspectConfig::new(3.0);
        let aspects = find_aspects(&planets, config);

        assert_eq!(aspects.len(), 1);
        assert_eq!(aspects[0].aspect_type, AspectType::Conjunction);
        assert_eq!(aspects[0].planet1, planet::SUN);
        assert_eq!(aspects[0].planet2, planet::MOON);
        assert!((aspects[0].orb - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_find_opposition() {
        let planets = vec![
            create_planet(planet::SUN, 10.0),
            create_planet(planet::MOON, 190.0), // 180° opposition
        ];

        let config = AspectConfig::new(3.0);
        let aspects = find_aspects(&planets, config);

        let opposition = aspects
            .iter()
            .find(|a| a.aspect_type == AspectType::Opposition);
        assert!(opposition.is_some());
    }

    #[test]
    fn test_find_square() {
        let planets = vec![
            create_planet(planet::SUN, 10.0),
            create_planet(planet::MARS, 100.0), // 90° square
        ];

        let config = AspectConfig::new(3.0);
        let aspects = find_aspects(&planets, config);

        let square = aspects.iter().find(|a| a.aspect_type == AspectType::Square);
        assert!(square.is_some());
    }

    #[test]
    fn test_find_trine() {
        let planets = vec![
            create_planet(planet::SUN, 10.0),
            create_planet(planet::JUPITER, 130.0), // 120° trine
        ];

        let config = AspectConfig::new(3.0);
        let aspects = find_aspects(&planets, config);

        let trine = aspects.iter().find(|a| a.aspect_type == AspectType::Trine);
        assert!(trine.is_some());
    }

    #[test]
    fn test_find_sextile() {
        let planets = vec![
            create_planet(planet::SUN, 10.0),
            create_planet(planet::VENUS, 70.0), // 60° sextile
        ];

        let config = AspectConfig::new(3.0);
        let aspects = find_aspects(&planets, config);

        let sextile = aspects
            .iter()
            .find(|a| a.aspect_type == AspectType::Sextile);
        assert!(sextile.is_some());
    }

    #[test]
    fn test_aspect_outside_orb() {
        let planets = vec![
            create_planet(planet::SUN, 0.0),
            create_planet(planet::MOON, 10.0), // 10° apart, outside 3° orb for conjunction
        ];

        let config = AspectConfig::new(3.0);
        let aspects = find_aspects(&planets, config);

        assert_eq!(aspects.len(), 0);
    }

    #[test]
    fn test_find_grand_trine() {
        let planets = vec![
            create_planet(planet::SUN, 0.0),
            create_planet(planet::MOON, 120.0),
            create_planet(planet::MARS, 240.0),
        ];

        let config = AspectConfig::new(3.0);
        let grand_trines = find_grand_trines(&planets, config);

        assert_eq!(grand_trines.len(), 1);
    }

    #[test]
    fn test_moon_void_of_course() {
        // Moon at 25° Aries, no other planets within aspect orb before 30°
        let planets = vec![
            create_planet(planet::MOON, 25.0),
            create_planet(planet::SUN, 100.0), // Not in aspect range
            create_planet(planet::MARS, 200.0), // Not in aspect range
        ];

        let config = AspectConfig::new(3.0);
        let result = is_moon_void_of_course(&planets, config);

        assert!(result.is_some());
        assert!(result.unwrap().contains("void of course"));
    }

    #[test]
    fn test_moon_not_void_of_course() {
        // Moon at 25° Aries, but will conjunct Sun at 28° Aries before leaving sign
        let planets = vec![
            create_planet(planet::MOON, 25.0),
            create_planet(planet::SUN, 28.0), // Moon will conjunct before leaving Aries
        ];

        let config = AspectConfig::new(3.0);
        let result = is_moon_void_of_course(&planets, config);

        assert!(result.is_none());
    }

    #[test]
    fn test_analyze_aspects() {
        let planets = vec![
            create_planet(planet::SUN, 0.0),
            create_planet(planet::MOON, 180.0), // Opposition
            create_planet(planet::MARS, 90.0),  // Square to Sun
        ];

        let config = AspectConfig::new(3.0);
        let analysis = analyze_aspects(&planets, config);

        assert!(analysis.aspects.len() >= 2); // At least opposition and square
        assert!(analysis.grand_trines.is_empty());
    }

    #[test]
    fn test_aspect_type_properties() {
        assert_eq!(AspectType::Conjunction.angle(), 0.0);
        assert_eq!(AspectType::Opposition.angle(), 180.0);
        assert_eq!(AspectType::Square.angle(), 90.0);
        assert_eq!(AspectType::Trine.angle(), 120.0);
        assert_eq!(AspectType::Sextile.angle(), 60.0);

        assert_eq!(AspectType::Conjunction.name(), "Conjunction");
        assert_eq!(AspectType::Opposition.name(), "Opposition");
        assert_eq!(AspectType::Square.name(), "Square");
        assert_eq!(AspectType::Trine.name(), "Trine");
        assert_eq!(AspectType::Sextile.name(), "Sextile");
        assert_eq!(AspectType::GrandTrine.name(), "Grand Trine");
    }
}
