use clap::{Parser, Subcommand};
use std::path::PathBuf;

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

        /// Output format (human/json/plain)
        #[arg(short, long, value_name = "FORMAT", default_value = "human")]
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

        let _ = tracing_subscriber::fmt()
            .with_max_level(level)
            .with_target(false)
            .try_init();

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
                println!("Chart generation mode");
            }
            Commands::Serve { port, host } => {
                tracing::info!("Starting HTTP server on {}:{}", host, port);
                println!("Server mode not yet implemented");
            }
        }

        Ok(())
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
