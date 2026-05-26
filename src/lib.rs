pub mod aspects;
pub mod chart;
pub mod cli;
pub mod config;
#[cfg(feature = "db")]
pub mod database;
pub mod ephemeris;
pub mod errors;
#[cfg(feature = "db")]
pub mod jobs;
pub mod logging;
pub mod output_handler;
pub mod performance;
#[cfg(feature = "db")]
pub mod queries;
pub mod renderer;
pub mod server;
pub mod svg_glyphs;
pub mod svg_renderer;

pub mod swiss_eph_impl;

pub use chart::{
    ChartCalculator, ChartData, Error as ChartError, GeoPos, HouseCusps, HouseSystem,
    PlanetPosition, Position,
};
pub use renderer::{Color, Point, Rect, Renderer, Size};
pub use swiss_eph_impl::SwissEphChartCalculator;

#[cfg(test)]
mod tests {
    #[test]
    fn test_swiss_eph_integration() {
        let config = crate::chart::ChartConfig::new(
            crate::chart::HouseSystem::Placidus,
            crate::chart::GeoPos::new(40.7128, -74.0060, 0.0),
            2451545.0,
        );

        let calculator = crate::swiss_eph_impl::SwissEphChartCalculator::new(config);
        assert!(calculator.is_ok());
    }
}
