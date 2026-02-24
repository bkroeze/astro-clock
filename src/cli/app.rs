use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::chart::ChartCalculator;

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

        /// Output format (png/webp)
        #[arg(short, long, value_name = "FORMAT", default_value = "png")]
        format: String,
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

        let _config = crate::config::AppConfig::from_file_or_default(self.cli.config.as_ref())?;
        tracing::info!("Starting Astro Clock");
        tracing::debug!("CLI arguments: {:#?}", self.cli);

        match &self.cli.command {
            Commands::Chart {
                lat,
                lon,
                time,
                output,
                format,
            } => {
                tracing::info!("Running chart generation mode");
                tracing::debug!(
                    "Chart options: lat={:?}, lon={:?}, time={:?}, output={:?}, format={}",
                    lat,
                    lon,
                    time,
                    output,
                    format
                );

                // Determine output filename
                let output_path = match output {
                    Some(path) => path.clone(),
                    None => {
                        let now = chrono::Local::now();
                        let format_lower = format.to_lowercase();
                        let ext = if format_lower == "webp" { "webp" } else { "png" };
                        let filename = format!("{}.{}", now.format("%m%d%y-%H%M%S"), ext);
                        std::path::PathBuf::from(filename)
                    }
                };

                // Parse time or use current time
                let julian_day = match time {
                    Some(time_str) => {
                        // Parse ISO 8601 format
                        let datetime = chrono::DateTime::parse_from_rfc3339(time_str)
                            .map_err(|e| crate::errors::Error::Config(format!("Invalid time format: {}", e)))?;
                        crate::ephemeris::julian_day_from_chrono(datetime.with_timezone(&chrono::Utc))
                    }
                    None => {
                        let now = chrono::Utc::now();
                        crate::ephemeris::julian_day_from_chrono(now)
                    }
                };

                // Get coordinates (default to 0,0 if not provided)
                let latitude = lat.unwrap_or(0.0);
                let longitude = lon.unwrap_or(0.0);

                tracing::info!("Generating chart for lat={}, lon={}, jd={}", latitude, longitude, julian_day);

                // Create chart config
                let geo_pos = crate::chart::GeoPos::new(latitude, longitude, 0.0);
                let chart_config = crate::chart::ChartConfig::new(
                    crate::chart::HouseSystem::Placidus,
                    geo_pos,
                    julian_day,
                );

                // Calculate chart data
                let calculator = crate::swiss_eph_impl::SwissEphChartCalculator::new(chart_config)
                    .map_err(|e| crate::errors::Error::Chart(e.to_string()))?;
                let chart_data = calculator.calculate_chart()
                    .map_err(|e| crate::errors::Error::Chart(e.to_string()))?;

                // Render chart
                let size = crate::renderer::Size::new(800.0, 800.0);
                let mut renderer = crate::renderer::Renderer::new(size)
                    .map_err(|e| crate::errors::Error::Chart(e.to_string()))?;
                renderer.render_chart(&chart_data)
                    .map_err(|e| crate::errors::Error::Chart(e.to_string()))?;

                // Save based on format
                let format_lower = format.to_lowercase();
                match format_lower.as_str() {
                    "webp" => {
                        let output_str = output_path.to_string_lossy();
                        let output_str = if output_str.ends_with(".webp") {
                            output_str.to_string()
                        } else {
                            format!("{}.webp", output_str)
                        };
                        renderer.save_webp(&output_str)
                            .map_err(|e| crate::errors::Error::Chart(e.to_string()))?;
                        println!("Chart saved to: {}", output_str);
                    }
                    _ => {
                        // Default to PNG
                        let output_str = output_path.to_string_lossy();
                        let output_str = if output_str.ends_with(".png") {
                            output_str.to_string()
                        } else {
                            format!("{}.png", output_str)
                        };
                        renderer.save(&output_str)
                            .map_err(|e| crate::errors::Error::Chart(e.to_string()))?;
                        println!("Chart saved to: {}", output_str);
                    }
                }

                Ok(())
            }
            Commands::Serve { port, host } => {
                tracing::info!("Starting HTTP server on {}:{}", host, port);
                let server = crate::server::Server::new(host.clone(), *port);
                tokio::runtime::Runtime::new()?
                    .block_on(async { server.run().await })?;
                Ok(())
            }
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
}
