use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "astro-clock")]
#[command(about = "An astrological chart rendering application", long_about = None)]
pub struct Cli {
    /// Path to configuration file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Date for chart calculation (YYYY-MM-DD)
    #[arg(short = 'd', long, value_name = "DATE")]
    pub date: Option<String>,

    /// Time for chart calculation (HH:MM)
    #[arg(short = 't', long, value_name = "TIME")]
    pub time: Option<String>,

    /// Location for chart calculation (lat,lon)
    #[arg(short = 'l', long, value_name = "LOCATION")]
    pub location: Option<String>,

    /// Output file path
    #[arg(short = 'o', long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Serve charts over HTTP
    #[arg(short = 's', long)]
    pub serve: bool,

    /// Enable verbose logging
    #[arg(short = 'v', long)]
    pub verbose: bool,
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

    pub fn run(self) -> Result<(), super::errors::Error> {
        if self.cli.verbose {
            let subscriber = tracing_subscriber::fmt()
                .with_max_level(tracing::Level::DEBUG)
                .with_target(false)
                .with_thread_ids(false)
                .with_thread_names(false);
            subscriber.init();
        }

        tracing::info!("Starting Astro Clock");
        tracing::debug!("CLI arguments: {:#?}", self.cli);

        if self.cli.serve {
            tracing::info!("Starting HTTP server...");
            // TODO: Implement server
            println!("Server mode not yet implemented");
        } else {
            tracing::info!("Running in chart generation mode");
            // TODO: Implement chart generation
            println!("Chart generation mode not yet implemented");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        let app = App::new();
        assert!(true); // Basic test to ensure compilation
    }

    #[test]
    fn test_cli_help() {
        let result = std::panic::catch_unwind(|| {
            Cli::command().render_help(std::io::stdout()).unwrap();
        });
        assert!(result.is_ok());
    }
}
