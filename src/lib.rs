pub mod chart;
pub mod cli;
pub mod config;
pub mod errors;
pub mod logging;
pub mod swiss_eph_impl;
pub mod renderer;

pub use chart::{Chart, ChartCalculator, ChartError};
pub use swiss_eph_impl::SwissEphChartCalculator;
pub use renderer::{Renderer, Point, Size, Color, Rect};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_swiss_eph_integration() {
        let calculator = SwissEphChartCalculator::new();
        let chart = calculator.calculate_chart(
            Utc.ymd(1990, 1, 1).and_hms(12, 0, 0),
            GeographicPosition::new(40.7128, -74.0060),
            HouseSystem::Placidus,
        ).unwrap();
        
        assert_eq!(chart.planets.len(), 10);
        assert_eq!(chart.house_cusps.len(), 12);
        
        println!("Swiss Ephemeris integration test passed!");
    }
}
}

fn main() {
    if let Err(err) = astro_clock() {
        eprintln!("⚠️  Error: {}\n", err);
        std::process::exit(1);
    }
}

fn astro_clock() -> Result<(), Error> {
    init_logging();

    println!("Astro Clock v0.1.0");
    println!("Swiss Ephemeris integration: Testing...");

    // Test Swiss Ephemeris integration
    let config = crate::chart::ChartConfig::new(
        crate::chart::HouseSystem::Placidus,
        crate::chart::GeoPos::new(40.7128, -74.0060, 0.0),
        2451545.0, // Jan 1, 2000, 12:00 UTC
    );

    let calculator = crate::swiss_eph_impl::SwissEphChartCalculator::new(config);
    match calculator {
        Ok(calc) => {
            println!("✅ Swiss Ephemeris initialized successfully");
            
            // Calculate planets
            let planets = calc.calculate_planets().unwrap();
            println!("✅ Calculated {} planets", planets.len());
            
            // Calculate houses
            let houses = calc.calculate_houses().unwrap();
            println!("✅ Calculated house cusps");
            println!("Ascendant: {:.2}°", houses.asc);
            println!("Midheaven: {:.2}°", houses.mc);
            
            // Calculate full chart
            let chart = calc.calculate_chart().unwrap();
            println!("✅ Full chart calculated");
            println!("Sidereal time: {:.2}°", chart.sidereal_time);
            
            // Show Sun position
            let sun = planets.iter().find(|p| p.name == "Sun").unwrap();
            println!("Sun position: {:.2}°", sun.position.longitude);
            
            Ok(())
        }
        Err(err) => {
            eprintln!("❌ Swiss Ephemeris initialization failed: {}", err);
            Err(err.into())
        }
    }
}
