use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::chart::ChartCalculator;
use crate::output_handler::{OutputFormat, OutputHandler};

#[derive(Parser, Debug)]
#[command(name = "astro-clock")]
#[command(about = "An astrological chart rendering application", long_about = None)]
#[command(version)]
pub struct Cli {
    /// Path to configuration file
    #[arg(short, long, global = true, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress non-essential output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate an astrological chart
    Chart {
        /// Latitude for chart calculation (-90 to 90)
        #[arg(long, value_name = "LAT")]
        lat: Option<f64>,

        /// Longitude for chart calculation (-180 to 180)
        #[arg(long, value_name = "LON")]
        lon: Option<f64>,

        /// Time for chart calculation (ISO 8601 format, defaults to now)
        #[arg(short, long, value_name = "TIME")]
        time: Option<String>,

        /// Output file path
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Output format (png/webp/md)
        #[arg(short, long, value_name = "FORMAT", default_value = "png")]
        format: String,

        /// Maximum orb for aspect detection in markdown output (in degrees)
        #[arg(long, value_name = "ORB")]
        orb: Option<f64>,

        /// House system for chart calculation (Placidus, Koch, Equal, Whole, Porphyry, Regiomontanus, Campanus, Morinus, Alcabitus, Topocentric, Vehlow)
        #[arg(long, value_name = "SYSTEM")]
        house: Option<String>,
    },

    /// Serve charts over HTTP
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,

        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },

    /// Analyze chart aspects
    Aspects {
        /// Latitude for chart calculation (-90 to 90)
        #[arg(long, value_name = "LAT")]
        lat: Option<f64>,

        /// Longitude for chart calculation (-180 to 180)
        #[arg(long, value_name = "LON")]
        lon: Option<f64>,

        /// Time for chart calculation (ISO 8601 format, defaults to now)
        #[arg(short, long, value_name = "TIME")]
        time: Option<String>,

        /// Maximum orb for aspect detection (in degrees)
        #[arg(short, long, value_name = "ORB", default_value = "3")]
        orb: f64,

        /// House system for chart calculation (Placidus, Koch, Equal, Whole, Porphyry, Regiomontanus, Campanus, Morinus, Alcabitus, Topocentric, Vehlow)
        #[arg(long, value_name = "SYSTEM")]
        house: Option<String>,
    },
}

#[derive(Debug)]
pub struct App {
    pub cli: Cli,
}

impl App {
    pub fn new() -> Self {
        let cli = Cli::parse();
        Self { cli }
    }

    pub fn run(self) -> Result<(), crate::errors::Error> {
        let level = if self.cli.quiet {
            tracing::Level::ERROR
        } else if self.cli.verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        };

        crate::logging::init_logging(level);

        let config = crate::config::AppConfig::from_file_or_default(self.cli.config.as_ref())?;
        tracing::info!("Starting Astro Clock");
        tracing::debug!("CLI arguments: {:#?}", self.cli);

        match &self.cli.command {
            Commands::Chart {
                lat,
                lon,
                time,
                output,
                format,
                orb,
                house,
            } => {
                tracing::info!("Running chart generation mode");
                tracing::debug!(
                    "Chart options: lat={:?}, lon={:?}, time={:?}, output={:?}, format={}",
                    lat, lon, time, output, format
                );

                // Parse format
                let output_format = OutputFormat::from_str(format);

                // Determine output filename
                let output_path = match output {
                    Some(path) => path.clone(),
                    None => {
                        let now = chrono::Local::now();
                        let filename = format!(
                            "{}.{}",
                            now.format("%m%d%y-%H%M%S"),
                            output_format.extension()
                        );
                        std::path::PathBuf::from(filename)
                    }
                };

                // Ensure correct extension
                let output_path = Self::ensure_extension(output_path, output_format.extension());

                // Parse time or use current time
                let julian_day = match time {
                    Some(time_str) => {
                        let datetime = chrono::DateTime::parse_from_rfc3339(time_str)
                            .map_err(|e| crate::errors::Error::Config(format!("Invalid time format: {}", e)))?;
                        crate::ephemeris::julian_day_from_chrono(datetime.with_timezone(&chrono::Utc))
                    }
                    None => {
                        let now = chrono::Utc::now();
                        crate::ephemeris::julian_day_from_chrono(now)
                    }
                };

                // Get coordinates from CLI args, config, or default to 0,0
                let latitude = lat.or(config.chart.location.latitude).unwrap_or(0.0);
                let longitude = lon.or(config.chart.location.longitude).unwrap_or(0.0);

                // Get house system from CLI or config
                let house_system_str = house.as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or_else(|| config.chart.house_system.as_str());
                let house_system = Self::parse_house_system(house_system_str)?;

                tracing::info!(
                    "Generating chart for lat={}, lon={}, jd={}, house_system={}",
                    latitude,
                    longitude,
                    julian_day,
                    house_system
                );

                // Create chart config
                let geo_pos = crate::chart::GeoPos::new(latitude, longitude, 0.0);
                let chart_config = crate::chart::ChartConfig::new(
                    house_system,
                    geo_pos,
                    julian_day,
                );

                // Calculate chart data
                let calculator = crate::swiss_eph_impl::SwissEphChartCalculator::new(chart_config)
                    .map_err(|e| crate::errors::Error::Chart(e.to_string()))?;
                let chart_data = calculator
                    .calculate_chart()
                    .map_err(|e| crate::errors::Error::Chart(e.to_string()))?;

                // Save chart using output handler
                if output_format == OutputFormat::Markdown {
                    // Use orb value from CLI or config, default to 3.0
                    let orb_value = orb.unwrap_or(config.chart.orb);
                    OutputHandler::save_markdown_with_orb(&chart_data, &output_path, orb_value)?;
                } else {
                    OutputHandler::save_chart(&chart_data, output_format, &output_path)?;
                }

                println!("Chart saved to: {}", output_path.display());

                Ok(())
            }
            Commands::Aspects { lat, lon, time, orb, house } => {
                tracing::info!("Running aspects analysis mode");

                // Parse time or use current time
                let julian_day = match time {
                    Some(time_str) => {
                        let datetime = chrono::DateTime::parse_from_rfc3339(time_str)
                            .map_err(|e| crate::errors::Error::Config(format!("Invalid time format: {}", e)))?;
                        crate::ephemeris::julian_day_from_chrono(datetime.with_timezone(&chrono::Utc))
                    }
                    None => {
                        let now = chrono::Utc::now();
                        crate::ephemeris::julian_day_from_chrono(now)
                    }
                };

                // Get coordinates from CLI args, config, or default to 0,0
                let latitude = lat.or(config.chart.location.latitude).unwrap_or(0.0);
                let longitude = lon.or(config.chart.location.longitude).unwrap_or(0.0);
                let orb_value = *orb;

                // Get house system from CLI or config
                let house_system_str = house.as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or_else(|| config.chart.house_system.as_str());
                let house_system = Self::parse_house_system(house_system_str)?;

                tracing::info!(
                    "Analyzing aspects for lat={}, lon={}, jd={}, orb={}, house_system={}",
                    latitude,
                    longitude,
                    julian_day,
                    orb_value,
                    house_system
                );

                // Create chart config
                let geo_pos = crate::chart::GeoPos::new(latitude, longitude, 0.0);
                let chart_config = crate::chart::ChartConfig::new(
                    house_system,
                    geo_pos,
                    julian_day,
                );

                // Calculate chart data
                let calculator = crate::swiss_eph_impl::SwissEphChartCalculator::new(chart_config)
                    .map_err(|e| crate::errors::Error::Chart(e.to_string()))?;
                let chart_data = calculator
                    .calculate_chart()
                    .map_err(|e| crate::errors::Error::Chart(e.to_string()))?;

                // Analyze aspects
                use crate::aspects::{analyze_aspects, AspectConfig};
                let analysis = analyze_aspects(&chart_data.planets, AspectConfig::new(orb_value));

                // Print results
                println!("\n## Aspects (orb: {}°)\n", orb_value);

                if let Some(void_info) = &analysis.moon_void_of_course {
                    println!("**{}**\n", void_info);
                }

                if !analysis.aspects.is_empty() {
                    println!("| Aspect | Planet 1 | Planet 2 | Orb |");
                    println!("|--------|----------|----------|-----|");
                    for aspect in &analysis.aspects {
                        println!(
                            "| {} {} | {} | {} | {:.2}° |",
                            aspect.aspect_type.symbol(),
                            aspect.aspect_type.name(),
                            aspect.planet1,
                            aspect.planet2,
                            aspect.orb
                        );
                    }
                } else {
                    println!("*No major aspects within orb*");
                }

                if !analysis.grand_trines.is_empty() {
                    println!("\n### Grand Trines\n");
                    for gt in &analysis.grand_trines {
                        println!("- {} △ {} △ {}", gt.planet1, gt.planet2, gt.planet3);
                    }
                }

                Ok(())
            }
            Commands::Serve { port, host } => {
                tracing::info!("Starting HTTP server on {}:{}", host, port);
                let server = crate::server::Server::new(host.clone(), *port);
                tokio::runtime::Runtime::new()?.block_on(async { server.run().await })?;
                Ok(())
            }
        }
    }

    fn parse_house_system(s: &str) -> Result<crate::chart::HouseSystem, crate::errors::Error> {
        match s.to_lowercase().as_str() {
            "placidus" => Ok(crate::chart::HouseSystem::Placidus),
            "koch" => Ok(crate::chart::HouseSystem::Koch),
            "equal" => Ok(crate::chart::HouseSystem::Equal),
            "whole" | "wholesign" | "whole_sign" => Ok(crate::chart::HouseSystem::Whole),
            "porphyry" => Ok(crate::chart::HouseSystem::Porphyry),
            "regiomontanus" => Ok(crate::chart::HouseSystem::Regiomontanus),
            "campanus" => Ok(crate::chart::HouseSystem::Campanus),
            "morinus" => Ok(crate::chart::HouseSystem::Morinus),
            "alcabitus" => Ok(crate::chart::HouseSystem::Alcabitus),
            "topocentric" => Ok(crate::chart::HouseSystem::Topocentric),
            "vehlow" => Ok(crate::chart::HouseSystem::Vehlow),
            _ => Err(crate::errors::Error::Config(format!(
                "Invalid house system: '{}'. Valid options: Placidus, Koch, Equal, Whole, Porphyry, Regiomontanus, Campanus, Morinus, Alcabitus, Topocentric, Vehlow",
                s
            ))),
        }
    }

    fn ensure_extension(path: PathBuf, extension: &str) -> PathBuf {
        let path_str = path.to_string_lossy();
        let expected_ext = format!(".{}", extension);
        if path_str.ends_with(&expected_ext) {
            path
        } else {
            // Remove any existing extension and add the correct one
            let mut new_path = path;
            if let Some(existing_ext) = new_path.extension() {
                let existing_ext = existing_ext.to_string_lossy();
                if existing_ext != extension {
                    new_path.set_extension(extension);
                }
            } else {
                new_path.set_extension(extension);
            }
            new_path
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_help() {
        let mut cmd = Cli::command();
        let _help = cmd.render_help();
    }

    #[test]
    fn test_chart_help() {
        let mut cmd = Cli::command();
        let chart_cmd = cmd
            .find_subcommand_mut("chart")
            .expect("chart subcommand exists");
        let _help = chart_cmd.render_help();
    }

    #[test]
    fn test_serve_help() {
        let mut cmd = Cli::command();
        let serve_cmd = cmd
            .find_subcommand_mut("serve")
            .expect("serve subcommand exists");
        let _help = serve_cmd.render_help();
    }

    #[test]
    fn test_ensure_extension() {
        let path = PathBuf::from("myfile");
        let result = App::ensure_extension(path, "png");
        assert_eq!(result.to_string_lossy(), "myfile.png");

        let path = PathBuf::from("myfile.png");
        let result = App::ensure_extension(path, "png");
        assert_eq!(result.to_string_lossy(), "myfile.png");

        let path = PathBuf::from("myfile");
        let result = App::ensure_extension(path, "md");
        assert_eq!(result.to_string_lossy(), "myfile.md");

        let path = PathBuf::from("myfile.txt");
        let result = App::ensure_extension(path, "md");
        assert_eq!(result.to_string_lossy(), "myfile.md");
    }
}
