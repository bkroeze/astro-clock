use crate::chart::{
    planet, ChartCalculator, ChartConfig, ChartData, Error, HouseCusps, HouseSystem,
    PlanetPosition, Position,
};

pub struct SwissEphChartCalculator {
    config: ChartConfig,
}

impl SwissEphChartCalculator {
    pub fn new(config: ChartConfig) -> Result<Self, Error> {
        swiss_eph::safe::set_ephe_path("");
        Ok(Self { config })
    }

    fn get_planet(planet_name: &str) -> Result<swiss_eph::safe::Planet, Error> {
        use swiss_eph::safe::Planet;
        match planet_name {
            planet::SUN => Ok(Planet::Sun),
            planet::MOON => Ok(Planet::Moon),
            planet::MERCURY => Ok(Planet::Mercury),
            planet::VENUS => Ok(Planet::Venus),
            planet::MARS => Ok(Planet::Mars),
            planet::JUPITER => Ok(Planet::Jupiter),
            planet::SATURN => Ok(Planet::Saturn),
            planet::URANUS => Ok(Planet::Uranus),
            planet::NEPTUNE => Ok(Planet::Neptune),
            planet::PLUTO => Ok(Planet::Pluto),
            planet::CHIRON => Ok(Planet::Chiron),
            planet::MEAN_NODE => Ok(Planet::MeanNode),
            planet::TRUE_NODE => Ok(Planet::TrueNode),
            _ => Err(Error::UnknownPlanet(planet_name.to_string())),
        }
    }

    fn get_house_system(system: &HouseSystem) -> swiss_eph::safe::HouseSystem {
        use swiss_eph::safe::HouseSystem as HS;
        match system {
            HouseSystem::Placidus => HS::Placidus,
            HouseSystem::Koch => HS::Koch,
            HouseSystem::Equal => HS::Equal,
            HouseSystem::Whole => HS::WholeSign,
            HouseSystem::Porphyry => HS::Porphyrius,
            HouseSystem::Regiomontanus => HS::Regiomontanus,
            HouseSystem::Campanus => HS::Campanus,
            HouseSystem::Morinus => HS::Morinus,
        }
    }

    fn calculate_planet_position(&self, planet_name: &str) -> Result<Position, Error> {
        let planet = Self::get_planet(planet_name)?;
        let flags = swiss_eph::safe::CalcFlags::new().with_speed();

        let result = swiss_eph::safe::calc(self.config.julian_day, planet, flags)
            .map_err(|e| Error::SwissEph(e.to_string()))?;

        Ok(Position::new(
            result.longitude,
            result.latitude,
            result.distance,
            result.longitude_speed,
            result.latitude_speed,
            result.distance_speed,
        ))
    }
}

impl ChartCalculator for SwissEphChartCalculator {
    fn new(config: ChartConfig) -> Result<Self, Error> {
        Self::new(config)
    }

    fn calculate_planets(&self) -> Result<Vec<PlanetPosition>, Error> {
        let mut planets = Vec::new();

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
            planet::CHIRON,
            planet::MEAN_NODE,
            planet::TRUE_NODE,
        ]
        .iter()
        {
            let position = self.calculate_planet_position(planet_name)?;
            let retrograde = position.speed_lon < 0.0;
            planets.push(PlanetPosition::new(planet_name, position, retrograde));
        }

        Ok(planets)
    }

    fn calculate_houses(&self) -> Result<HouseCusps, Error> {
        let hs = Self::get_house_system(&self.config.house_system);

        let result = swiss_eph::safe::houses(
            self.config.julian_day,
            self.config.geo_pos.latitude,
            self.config.geo_pos.longitude,
            hs,
        )
        .map_err(|e| Error::SwissEph(e.to_string()))?;

        let dc = (result.ascendant + 180.0) % 360.0;
        let ic = (result.mc + 180.0) % 360.0;

        Ok(HouseCusps {
            asc: result.ascendant,
            mc: result.mc,
            dc,
            ic,
            houses: result.cusps,
            system: self.config.house_system.clone(),
        })
    }

    fn calculate_sidereal_time(&self) -> Result<f64, Error> {
        Ok(swiss_eph::safe::sidereal_time(self.config.julian_day))
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
    use crate::chart::GeoPos;

    #[test]
    fn test_swiss_eph_chart_calculator_creation() {
        let config = ChartConfig::new(
            HouseSystem::Placidus,
            GeoPos::new(40.7128, -74.0060, 0.0),
            2451545.0,
        );

        let calculator = SwissEphChartCalculator::new(config);
        assert!(calculator.is_ok());
    }

    #[test]
    fn test_planet_position_calculation() {
        let config = ChartConfig::new(
            HouseSystem::Placidus,
            GeoPos::new(40.7128, -74.0060, 0.0),
            2451545.0,
        );

        let calculator = SwissEphChartCalculator::new(config).unwrap();
        let planets = calculator.calculate_planets().unwrap();

        assert!(!planets.is_empty());
        let sun = planets.iter().find(|p| p.name == planet::SUN).unwrap();
        assert!(sun.position.longitude > 0.0);
        assert!(sun.position.longitude < 360.0);
    }

    #[test]
    fn test_house_calculation() {
        let config = ChartConfig::new(
            HouseSystem::Placidus,
            GeoPos::new(40.7128, -74.0060, 0.0),
            2451545.0,
        );

        let calculator = SwissEphChartCalculator::new(config).unwrap();
        let houses = calculator.calculate_houses().unwrap();

        assert!(houses.asc > 0.0 && houses.asc < 360.0);
        assert!(houses.mc > 0.0 && houses.mc < 360.0);
        assert_eq!(houses.houses.len(), 12);
    }
}
