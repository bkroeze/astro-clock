use crate::chart::{
    constants, planet, ChartCalculator, ChartConfig, ChartData, Error, GeoPos, HouseCusps,
    HouseSystem, PlanetPosition, Position,
};
use chrono::{DateTime, Datelike, Duration, Timelike, Utc};
use std::sync::Arc;

use swiss_eph::safe::*;

pub struct SwissEphChartCalculator {
    config: ChartConfig,
    eph: Arc<SwissEph>,
}

impl SwissEphChartCalculator {
    pub fn new(config: ChartConfig) -> Result<Self, Error> {
        // Initialize Swiss Ephemeris
        let eph = Arc::new(SwissEph::new()?);

        // Set ephemeris path if needed (for external files)
        // safe::set_ephe_path("/path/to/ephe/files");

        Ok(Self { config, eph })
    }

    fn julday_from_datetime(dt: DateTime<Utc>) -> f64 {
        let date = dt.naive_utc();
        let time = dt.time();
        let jd = julday(
            date.year() as i32,
            date.month() as i32,
            date.day() as i32,
            time.hour() as f64 + time.minute() as f64 / 60.0 + time.second() as f64 / 3600.0,
        );
        jd
    }

    fn get_planet_id(planet_name: &str) -> Result<i32, Error> {
        match planet_name {
            planet::SUN => Ok(SE_SUN),
            planet::MOON => Ok(SE_MOON),
            planet::MERCURY => Ok(SE_MERCURY),
            planet::VENUS => Ok(SE_VENUS),
            planet::MARS => Ok(SE_MARS),
            planet::JUPITER => Ok(SE_JUPITER),
            planet::SATURN => Ok(SE_SATURN),
            planet::URANUS => Ok(SE_URANUS),
            planet::NEPTUNE => Ok(SE_NEPTUNE),
            planet::PLUTO => Ok(SE_PLUTO),
            planet::CHIRON => Ok(SE_CHIRON),
            planet::MEAN_NODE => Ok(SE_MEAN_NODE),
            planet::TRUE_NODE => Ok(SE_TRUE_NODE),
            planet::ASCENDANT => Ok(SE_ASC),
            planet::MIDHEAVEN => Ok(SE_MC),
            _ => Err(Error::UnknownPlanet(planet_name.to_string())),
        }
    }

    fn calculate_planet_position(
        &self,
        planet_name: &str,
        flags: CalcFlags,
    ) -> Result<Position, Error> {
        let planet_id = Self::get_planet_id(planet_name)?;
        let position = self.eph.calc_ut(self.config.julian_day, planet_id, flags)?;

        Ok(Position::new(
            position.longitude,
            position.latitude,
            position.distance,
            position.speed.longitude,
            position.speed.latitude,
            position.speed.distance,
        ))
    }

    fn calculate_house_cusps(&self) -> Result<HouseCusps, Error> {
        let house_system_id = match self.config.house_system {
            HouseSystem::Placidus => SE_HOUSES_PLACIDUS,
            HouseSystem::Koch => SE_HOUSES_KOCH,
            HouseSystem::Equal => SE_HOUSES_EQUAL,
            HouseSystem::Whole => SE_HOUSES_WHOLE_SIGN,
            HouseSystem::Porphyry => SE_HOUSES_PORPHYRIUS,
            HouseSystem::Regiomontanus => SE_HOUSES_REGIOMONTANUS,
            HouseSystem::Campanus => SE_HOUSES_CAMPANUS,
            HouseSystem::Morinus => SE_HOUSES_MORINUS,
        };

        let flags = CalcFlags::new();
        let houses = self.eph.houses(
            self.config.julian_day,
            house_system_id,
            self.config.geo_pos.latitude,
            self.config.geo_pos.longitude,
            flags,
        )?;

        Ok(HouseCusps {
            asc: houses.asc,
            mc: houses.mc,
            dc: houses.dc,
            ic: houses.ic,
            houses: houses.houses,
            system: self.config.house_system.clone(),
        })
    }

    fn calculate_sidereal_time(&self) -> Result<f64, Error> {
        let flags = CalcFlags::new();
        let sidereal_time = self.eph.sidereal_time(self.config.julian_day, flags)?;
        Ok(sidereal_time)
    }
}

impl ChartCalculator for SwissEphChartCalculator {
    fn new(config: ChartConfig) -> Result<Self, Error>
    where
        Self: Sized,
    {
        Self::new(config)
    }

    fn calculate_planets(&self) -> Result<Vec<PlanetPosition>, Error> {
        let flags = CalcFlags::new().with_speed();
        let mut planets = Vec::new();

        // Main planets
        for &planet_name in [
            planet::SUN,
            planet::MOON,
            planet::MERCURY,
            planet::VENUS,
            planet::MARS,
            planet::JUPITER,
            planet::SATURN,
            planet::URANUS,
            planet::NEPTUNE,
            planet::PLUTO,
        ]
        .iter()
        {
            let position = self.calculate_planet_position(planet_name, flags)?;
            let retrograde = position.speed_lon < 0.0;
            planets.push(PlanetPosition::new(planet_name, position, retrograde));
        }

        // Lunar nodes
        let mean_node_pos = self.calculate_planet_position(planet::MEAN_NODE, flags)?;
        let true_node_pos = self.calculate_planet_position(planet::TRUE_NODE, flags)?;
        planets.push(PlanetPosition::new(planet::MEAN_NODE, mean_node_pos, false));
        planets.push(PlanetPosition::new(planet::TRUE_NODE, true_node_pos, false));

        // Chiron
        let chiron_pos = self.calculate_planet_position(planet::CHIRON, flags)?;
        planets.push(PlanetPosition::new(planet::CHIRON, chiron_pos, false));

        Ok(planets)
    }

    fn calculate_houses(&self) -> Result<HouseCusps, Error> {
        self.calculate_house_cusps()
    }

    fn calculate_sidereal_time(&self) -> Result<f64, Error> {
        self.calculate_sidereal_time()
    }

    fn calculate_chart(&self) -> Result<ChartData, Error> {
        let planets = self.calculate_planets()?;
        let houses = self.calculate_houses()?;
        let sidereal_time = self.calculate_sidereal_time()?;

        Ok(ChartData {
            geo_pos: self.config.geo_pos.clone(),
            julian_day: self.config.julian_day,
            planets,
            houses,
            sidereal_time,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_swiss_eph_chart_calculator_creation() {
        let config = ChartConfig::new(
            HouseSystem::Placidus,
            GeoPos::new(40.7128, -74.0060, 0.0),
            2451545.0, // Jan 1, 2000, 12:00 UTC
        );

        let calculator = SwissEphChartCalculator::new(config);
        assert!(calculator.is_ok());
    }

    #[test]
    fn test_planet_position_calculation() {
        let config = ChartConfig::new(
            HouseSystem::Placidus,
            GeoPos::new(40.7128, -74.0060, 0.0),
            2451545.0, // Jan 1, 2000, 12:00 UTC
        );

        let calculator = SwissEphChartCalculator::new(config).unwrap();
        let planets = calculator.calculate_planets().unwrap();

        assert!(!planets.is_empty());
        assert_eq!(planets.len(), 13); // 10 main planets + 3 lunar points

        // Check Sun position
        let sun = planets.iter().find(|p| p.name == planet::SUN).unwrap();
        assert!(sun.position.longitude > 0.0);
        assert!(sun.position.longitude < 360.0);
    }

    #[test]
    fn test_house_calculation() {
        let config = ChartConfig::new(
            HouseSystem::Placidus,
            GeoPos::new(40.7128, -74.0060, 0.0),
            2451545.0, // Jan 1, 2000, 12:00 UTC
        );

        let calculator = SwissEphChartCalculator::new(config).unwrap();
        let houses = calculator.calculate_houses().unwrap();

        assert!(houses.asc > 0.0 && houses.asc < 360.0);
        assert!(houses.mc > 0.0 && houses.mc < 360.0);
        assert_eq!(houses.houses.len(), 12);
    }

    #[test]
    fn test_sidereal_time_calculation() {
        let config = ChartConfig::new(
            HouseSystem::Placidus,
            GeoPos::new(40.7128, -74.0060, 0.0),
            2451545.0, // Jan 1, 2000, 12:00 UTC
        );

        let calculator = SwissEphChartCalculator::new(config).unwrap();
        let sidereal_time = calculator.calculate_sidereal_time().unwrap();

        assert!(sidereal_time > 0.0 && sidereal_time < 360.0);
    }

    #[test]
    fn test_full_chart_calculation() {
        let config = ChartConfig::new(
            HouseSystem::Placidus,
            GeoPos::new(40.7128, -74.0060, 0.0),
            2451545.0, // Jan 1, 2000, 12:00 UTC
        );

        let calculator = SwissEphChartCalculator::new(config).unwrap();
        let chart = calculator.calculate_chart().unwrap();

        assert_eq!(chart.geo_pos.latitude, 40.7128);
        assert_eq!(chart.geo_pos.longitude, -74.0060);
        assert_eq!(chart.julian_day, 2451545.0);
        assert!(!chart.planets.is_empty());
        assert!(chart.sidereal_time > 0.0 && chart.sidereal_time < 360.0);
    }
}
