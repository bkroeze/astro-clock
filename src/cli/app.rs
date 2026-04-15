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

/// Job subcommands for managing async jobs
#[derive(Subcommand, Debug)]
pub enum JobCommands {
    /// Get job status and results
    Status {
        /// Job ID (UUID format)
        job_id: String,
    },
    /// List recent jobs
    List {
        /// Filter by status (pending, in_process, complete, failed)
        #[arg(long, value_name = "STATUS")]
        status: Option<String>,
        /// Number of jobs to show (default: 20)
        #[arg(long, value_name = "N")]
        limit: Option<i64>,
        /// Offset for pagination (default: 0)
        #[arg(long, value_name = "N")]
        offset: Option<i64>,
    },
}

/// Subcommands for named queries (wedding, project, travel)
#[derive(Subcommand, Debug)]
pub enum QueryCommands {
    /// Find auspicious wedding dates
    Wedding {
        /// Start date (YYYY-MM-DD)
        #[arg(long, value_name = "DATE")]
        start: String,
        /// Number of days to query (1-366)
        #[arg(long, value_name = "N")]
        days: i64,
        /// Execute synchronously and wait for completion
        #[arg(long)]
        sync: bool,
    },
    /// Find good dates to start projects
    Project {
        #[arg(long, value_name = "DATE")]
        start: String,
        #[arg(long, value_name = "N")]
        days: i64,
        #[arg(long)]
        sync: bool,
    },
    /// Find favorable travel dates
    Travel {
        #[arg(long, value_name = "DATE")]
        start: String,
        #[arg(long, value_name = "N")]
        days: i64,
        #[arg(long)]
        sync: bool,
    },
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

        /// Output format (png/webp/svg/md)
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

    /// Load planetary data for a date range
    Load {
        /// Start date (YYYY-MM-DD)
        #[arg(long, value_name = "DATE")]
        start: String,

        /// Number of days to load (1-365)
        #[arg(long, value_name = "N")]
        days: i64,

        /// Execute synchronously and wait for completion
        #[arg(long)]
        sync: bool,
    },

    /// Execute named queries (wedding, project, travel)
    #[command(subcommand)]
    Query(QueryCommands),

    /// Manage async jobs
    #[command(subcommand)]
    Job(JobCommands),
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
            Commands::Load { start, days, sync } => {
                tracing::info!("Running data load mode: start={}, days={}, sync={}", start, days, sync);

                // Validate date format
                let _start_date = chrono::NaiveDate::parse_from_str(start, "%Y-%m-%d")
                    .map_err(|_| crate::errors::Error::Config(
                        format!("Invalid date format: {}. Expected YYYY-MM-DD", start)
                    ))?;

                // Validate days range
                if *days < 1 || *days > 365 {
                    return Err(crate::errors::Error::Config(
                        format!("Days must be between 1 and 365, got {}", days)
                    ));
                }

                // Check if db feature is enabled
                #[cfg(feature = "db")]
                {
                    use crate::database::pool::DatabasePool;
                    use crate::jobs::executor::JobExecutor;
                    use crate::jobs::handlers::LoadJobHandler;
                    use crate::jobs::repository::JobRepository;
                    use crate::jobs::types::JobType;
                    use serde_json::json;

                    // Get database URL from environment or config
                    let db_url = std::env::var("DATABASE_URL")
                        .unwrap_or_else(|_| config.database.url.clone());

                    // Use a single runtime for pool creation and execution to avoid
                    // "pool timed out" from using connections across dropped runtimes
                    let rt = tokio::runtime::Runtime::new()?;

                    if *sync {
                        let result: crate::errors::Result<_> = rt.block_on(async {
                            let db_pool = DatabasePool::connect(&db_url).await
                                .map_err(|e| crate::errors::Error::Database(
                                    format!("Failed to connect to database: {}", e)
                                ))?;
                            let pool = db_pool.pool().clone();
                            let job_repo = JobRepository::new(pool.clone());
                            let load_handler = LoadJobHandler::new(pool);
                            let executor = JobExecutor::new(
                                job_repo,
                                vec![std::sync::Arc::new(load_handler)],
                                "cli-worker".to_string(),
                            );
                            let payload = json!({
                                "start_date": start,
                                "days": days
                            });
                            executor.execute_sync(JobType::Load, payload).await
                                .map_err(|e| crate::errors::Error::Chart(format!("{}", e)))
                        });

                        let result = result.map_err(|e| crate::errors::Error::Chart(format!("Load failed: {}", e)))?;

                        println!("Loading planetary data for {} days starting from {}...", days, start);
                        {
                            let job = result;
                            if let Some(ref job_result) = job.result {
                                if let Ok(load_result) = serde_json::from_value::<crate::jobs::handlers::LoadJobResult>(job_result.clone()) {
                                    println!("\n✓ Load completed successfully");
                                    println!("  Dates loaded: {}", load_result.dates_loaded);
                                    println!("  Dates skipped: {}", load_result.dates_skipped);
                                    println!("  Dates failed: {}", load_result.dates_failed);
                                    println!("  Total positions: {}", load_result.total_positions);
                                    println!("  Total aspects: {}", load_result.total_aspects);
                                    println!("  Total lunar conditions: {}", load_result.total_lunar_conditions);
                                    
                                    if !load_result.failed.is_empty() {
                                        println!("\nFailed dates:");
                                        for failure in &load_result.failed {
                                            println!("  - {}: {}", failure.date, failure.error);
                                        }
                                    }
                                } else {
                                    println!("✓ Load completed: {:?}", job.result);
                                }
                            } else if let Some(error) = job.error {
                                println!("✗ Load failed: {:?}", error);
                            }
                        }
                    } else {
                        let job_id: crate::errors::Result<_> = rt.block_on(async {
                            let db_pool = DatabasePool::connect(&db_url).await
                                .map_err(|e| crate::errors::Error::Database(
                                    format!("Failed to connect to database: {}", e)
                                ))?;
                            let pool = db_pool.pool().clone();
                            let job_repo = JobRepository::new(pool.clone());
                            let load_handler = LoadJobHandler::new(pool);
                            let executor = JobExecutor::new(
                                job_repo,
                                vec![std::sync::Arc::new(load_handler)],
                                "cli-worker".to_string(),
                            );
                            let payload = json!({
                                "start_date": start,
                                "days": days
                            });
                            executor.execute_async(JobType::Load, payload).await
                                .map_err(|e| crate::errors::Error::Chart(format!("{}", e)))
                        });

                        let job_id = job_id.map_err(|e| crate::errors::Error::Chart(format!("Failed to start load job: {}", e)))?;

                        {
                            let id = job_id;
                            println!("Load job started in background");
                            println!("Job ID: {}", id);
                            println!("Poll status: /api/v1/jobs/{}", id);
                        }
                    }

                    Ok(())
                }

                #[cfg(not(feature = "db"))]
                {
                    Err(crate::errors::Error::Config(
                        "Database support not enabled. Build with --features db to use the load command.".to_string()
                    ))
                }
            }
            Commands::Query(query_cmd) => {
                self.handle_query_command(query_cmd, &config)
            }
            Commands::Job(job_cmd) => {
                self.handle_job_command(job_cmd, &config)
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

    fn handle_query_command(
        &self,
        query_cmd: &QueryCommands,
        config: &crate::config::AppConfig,
    ) -> Result<(), crate::errors::Error> {
        // Extract query parameters based on variant
        let (query_name, start, days, sync) = match query_cmd {
            QueryCommands::Wedding { start, days, sync } => ("wedding", start, days, sync),
            QueryCommands::Project { start, days, sync } => ("project", start, days, sync),
            QueryCommands::Travel { start, days, sync } => ("travel", start, days, sync),
        };

        tracing::info!(
            "Running query mode: query={}, start={}, days={}, sync={}",
            query_name,
            start,
            days,
            sync
        );

        // Validate date format
        let _start_date = chrono::NaiveDate::parse_from_str(start, "%Y-%m-%d").map_err(|_| {
            crate::errors::Error::Config(format!(
                "Invalid date format: {}. Expected YYYY-MM-DD",
                start
            ))
        })?;

        // Validate days range (1-366 for queries)
        if *days < 1 || *days > 366 {
            return Err(crate::errors::Error::Config(format!(
                "Days must be between 1 and 366, got {}",
                days
            )));
        }

        // Check if db feature is enabled
        #[cfg(feature = "db")]
        {
            use crate::database::pool::DatabasePool;
            use crate::jobs::executor::JobExecutor;
            use crate::jobs::handlers::QueryJobHandler;
            use crate::jobs::repository::JobRepository;
            use crate::jobs::types::JobType;
            use serde_json::json;

            // Get database URL from environment or config
            let db_url = std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| config.database.url.clone());

            // Create database pool — use a single runtime to avoid "pool timed out"
            // from connections bound to a dropped runtime.
            let rt = tokio::runtime::Runtime::new()?;

            let pool = rt.block_on(async {
                let db_pool = DatabasePool::connect(&db_url)
                    .await
                    .map_err(|e| {
                        crate::errors::Error::Config(format!(
                            "Failed to connect to database: {}",
                            e
                        ))
                    })?;
                Ok::<_, crate::errors::Error>(db_pool.pool().clone())
            })?;

            // Create repositories and handler
            let job_repo = JobRepository::new(pool.clone());
            let query_handler = QueryJobHandler::new(pool);

            // Create executor with the query handler
            let executor = JobExecutor::new(
                job_repo,
                vec![std::sync::Arc::new(query_handler)],
                "cli-worker".to_string(),
            );

            // Build payload
            let payload = json!({
                "query_name": query_name,
                "start_date": start,
                "days": days
            });

            // Execute based on sync flag
            if *sync {
                println!(
                    "Running {} query for {} days starting from {}...",
                    query_name, days, start
                );

                let result = rt.block_on(async {
                    executor.execute_sync(JobType::Query, payload).await
                });

                match result {
                    Ok(job) => {
                        if let Some(ref result) = job.result {
                            // Deserialize and pretty-print the result
                            if let Ok(query_result) = serde_json::from_value::<
                                crate::jobs::handlers::QueryJobResult,
                            >(result.clone())
                            {
                                println!("\n✓ Query completed successfully");
                                println!("  Query: {}", query_result.query_name);
                                println!("  Date range: {} to {} ({} days)",
                                    query_result.start_date,
                                    query_result.start_date, // Note: we'd need to calculate end date
                                    query_result.days
                                );
                                println!("  Total results: {}", query_result.total_results);
                                println!("  Execution time: {}ms", query_result.execution_time_ms);

                                // Pretty-print the results
                                if query_result.total_results > 0 {
                                    println!("\n  Results:");
                                    if let Ok(results_json) = serde_json::to_string_pretty(&query_result.results) {
                                        // Indent the JSON output
                                        for line in results_json.lines() {
                                            println!("    {}", line);
                                        }
                                    }
                                }

                                if let Some(warnings) = query_result.warnings {
                                    if !warnings.is_empty() {
                                        println!("\n  Warnings:");
                                        for warning in &warnings {
                                            println!("    - {}", warning);
                                        }
                                    }
                                }
                            } else {
                                println!("✓ Query completed: {:?}", job.result);
                            }
                        } else if let Some(error) = job.error {
                            println!("✗ Query failed: {:?}", error);
                        }
                    }
                    Err(e) => {
                        return Err(crate::errors::Error::Chart(format!(
                            "Query failed: {}",
                            e
                        )));
                    }
                }
            } else {
                let job_id = rt.block_on(async {
                    executor.execute_async(JobType::Query, payload).await
                });

                match job_id {
                    Ok(id) => {
                        println!("{} query job started in background", query_name);
                        println!("Job ID: {}", id);
                        println!("Poll status: /api/v1/jobs/{}", id);
                    }
                    Err(e) => {
                        return Err(crate::errors::Error::Chart(format!(
                            "Failed to start query job: {}",
                            e
                        )));
                    }
                }
            }

            Ok(())
        }

        #[cfg(not(feature = "db"))]
        {
            Err(crate::errors::Error::Config(
                "Database support not enabled. Build with --features db to use the query command."
                    .to_string(),
            ))
        }
    }

    fn handle_job_command(
        &self,
        job_cmd: &JobCommands,
        config: &crate::config::AppConfig,
    ) -> Result<(), crate::errors::Error> {
        #[cfg(feature = "db")]
        {
            match job_cmd {
                JobCommands::Status { job_id } => {
                    self.handle_job_status(job_id, config)
                }
                JobCommands::List { status, limit, offset } => {
                    self.handle_job_list(status.as_ref(), *limit, *offset, config)
                }
            }
        }

        #[cfg(not(feature = "db"))]
        {
            Err(crate::errors::Error::Config(
                "Database support not enabled. Build with --features db to use the job command.".to_string()
            ))
        }
    }

    #[cfg(feature = "db")]
    fn handle_job_status(
        &self,
        job_id_str: &str,
        config: &crate::config::AppConfig,
    ) -> Result<(), crate::errors::Error> {
        use uuid::Uuid;
        use crate::jobs::repository::JobRepository;

        // Validate UUID format
        let job_id = match Uuid::parse_str(job_id_str) {
            Ok(id) => id,
            Err(_) => {
                return Err(crate::errors::Error::Config(
                    format!("Invalid job ID format: {}. Expected UUID.", job_id_str)
                ));
            }
        };

        // Get database URL from environment or config
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| config.database.url.clone());

        // Create database pool — use a single runtime to avoid "pool timed out" from
        // connections bound to a dropped runtime.
        let rt = tokio::runtime::Runtime::new()?;

        let pool = rt.block_on(async {
            let db_pool = crate::database::pool::DatabasePool::connect(&db_url).await
                .map_err(|e| crate::errors::Error::Config(
                    format!("Failed to connect to database: {}", e)
                ))?;
            Ok::<_, crate::errors::Error>(db_pool.pool().clone())
        })?;

        // Query job
        let job = rt.block_on(async {
            let repo = JobRepository::new(pool);
            repo.get_job(job_id).await
        });

        match job {
            Ok(Some(job)) => {
                println!("Job: {}", job.id);
                println!("Type: {}", job.job_type);
                println!("Status: {}", job.status);
                println!("Created: {}", job.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
                println!("Updated: {}", job.updated_at.format("%Y-%m-%d %H:%M:%S UTC"));
                println!("Started: {}", job.started_at.map(|d| d.format("%Y-%m-%d %H:%M:%S UTC").to_string()).unwrap_or_else(|| "N/A".to_string()));
                println!("Completed: {}", job.completed_at.map(|d| d.format("%Y-%m-%d %H:%M:%S UTC").to_string()).unwrap_or_else(|| "N/A".to_string()));
                println!();

                if let Some(ref payload) = job.payload {
                    println!("Payload:");
                    match serde_json::to_string_pretty(payload) {
                        Ok(json) => println!("{}", json),
                        Err(_) => println!("{:?}", payload),
                    }
                } else {
                    println!("Payload:");
                    println!("N/A");
                }
                println!();

                if let Some(ref result) = job.result {
                    println!("Result:");
                    match serde_json::to_string_pretty(result) {
                        Ok(json) => println!("{}", json),
                        Err(_) => println!("{:?}", result),
                    }
                } else {
                    println!("Result:");
                    println!("N/A");
                }
                println!();

                if let Some(ref error) = job.error {
                    println!("Error:");
                    match serde_json::to_string_pretty(error) {
                        Ok(json) => println!("{}", json),
                        Err(_) => println!("{:?}", error),
                    }
                } else {
                    println!("Error:");
                    println!("N/A");
                }
            }
            Ok(None) => {
                println!("Job {} not found", job_id);
            }
            Err(e) => {
                return Err(crate::errors::Error::Chart(format!("Failed to get job: {}", e)));
            }
        }

        Ok(())
    }

    #[cfg(feature = "db")]
    fn handle_job_list(
        &self,
        status_filter: Option<&String>,
        limit: Option<i64>,
        offset: Option<i64>,
        config: &crate::config::AppConfig,
    ) -> Result<(), crate::errors::Error> {
        use crate::jobs::types::JobStatus;
        use crate::jobs::repository::JobRepository;

        // Parse status filter
        let status = match status_filter {
            Some(s) => {
                match s.as_str() {
                    "pending" => Some(JobStatus::Pending),
                    "in_process" => Some(JobStatus::InProcess),
                    "complete" => Some(JobStatus::Complete),
                    "failed" => Some(JobStatus::Failed),
                    _ => {
                        return Err(crate::errors::Error::Config(
                            format!("Invalid status: '{}'. Valid options: pending, in_process, complete, failed", s)
                        ));
                    }
                }
            }
            None => None,
        };

        // Set defaults and cap limit
        let limit = limit.unwrap_or(20).min(100);
        let offset = offset.unwrap_or(0);

        // Get database URL from environment or config
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| config.database.url.clone());

        // Create database pool — use a single runtime to avoid "pool timed out" from
        // connections bound to a dropped runtime.
        let rt = tokio::runtime::Runtime::new()?;

        let pool = rt.block_on(async {
            let db_pool = crate::database::pool::DatabasePool::connect(&db_url).await
                .map_err(|e| crate::errors::Error::Config(
                    format!("Failed to connect to database: {}", e)
                ))?;
            Ok::<_, crate::errors::Error>(db_pool.pool().clone())
        })?;

        // Query jobs
        let result: Result<(Vec<_>, i64), crate::jobs::error::JobError> = rt.block_on(async {
            let repo = JobRepository::new(pool);
            let jobs = repo.list_jobs(status, limit, offset).await?;
            let total = repo.count_jobs(status).await?;
            Ok::<_, crate::jobs::error::JobError>((jobs, total))
        });

        match result {
            Ok((jobs, total)) => {
                if jobs.is_empty() {
                    if let Some(s) = status_filter {
                        println!("No {} jobs found", s);
                    } else {
                        println!("No jobs found");
                    }
                } else {
                    println!("Jobs (showing {} of {}):\n", jobs.len(), total);
                    println!("{:<40} {:<10} {:<12} {:<20}", "ID", "Type", "Status", "Created");
                    println!("{}", "-".repeat(82));

                    for job in jobs {
                        let id_short = format!("{}...", &job.id.to_string()[..8]);
                        let created = job.created_at.format("%Y-%m-%d %H:%M");
                        let status_str = format!("{:>10}", job.status);
                        let type_str = format!("{:<10}", job.job_type);
                        println!("{:<40} {:<10} {:<12} {:<20}", id_short, type_str, status_str, created);
                    }

                    if total > (offset + limit) {
                        println!("\nUse --offset {} to see more results", offset + limit);
                    }
                }
            }
            Err(e) => {
                return Err(crate::errors::Error::Chart(format!("Failed to list jobs: {}", e)));
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

    #[test]
    fn test_query_wedding_help() {
        let mut cmd = Cli::command();
        let query_cmd = cmd
            .find_subcommand_mut("query")
            .expect("query subcommand exists");
        let wedding_cmd = query_cmd
            .find_subcommand_mut("wedding")
            .expect("wedding subcommand exists");
        let _help = wedding_cmd.render_help();
    }

    #[test]
    fn test_query_project_help() {
        let mut cmd = Cli::command();
        let query_cmd = cmd
            .find_subcommand_mut("query")
            .expect("query subcommand exists");
        let project_cmd = query_cmd
            .find_subcommand_mut("project")
            .expect("project subcommand exists");
        let _help = project_cmd.render_help();
    }

    #[test]
    fn test_query_travel_help() {
        let mut cmd = Cli::command();
        let query_cmd = cmd
            .find_subcommand_mut("query")
            .expect("query subcommand exists");
        let travel_cmd = query_cmd
            .find_subcommand_mut("travel")
            .expect("travel subcommand exists");
        let _help = travel_cmd.render_help();
    }

    #[test]
    fn test_job_status_help() {
        let mut cmd = Cli::command();
        let job_cmd = cmd
            .find_subcommand_mut("job")
            .expect("job subcommand exists");
        let status_cmd = job_cmd
            .find_subcommand_mut("status")
            .expect("status subcommand exists");
        let _help = status_cmd.render_help();
    }

    #[test]
    fn test_job_list_help() {
        let mut cmd = Cli::command();
        let job_cmd = cmd
            .find_subcommand_mut("job")
            .expect("job subcommand exists");
        let list_cmd = job_cmd
            .find_subcommand_mut("list")
            .expect("list subcommand exists");
        let _help = list_cmd.render_help();
    }
}
