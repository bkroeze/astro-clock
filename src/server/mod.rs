use axum::{extract::Query, response::{IntoResponse, Response}};
use axum::routing::{get, post};
use axum::Json;
use axum::http::StatusCode;
use axum::body::Body;
use std::net::SocketAddr;

use crate::chart::{ChartCalculator, ChartConfig, GeoPos, HouseSystem};
use crate::SwissEphChartCalculator;
use crate::errors::Error;

// Import job-related types when database feature is enabled
#[cfg(feature = "db")]
use crate::jobs::{executor::JobExecutor, repository::JobRepository, handlers::{LoadJobHandler, QueryJobHandler}};
#[cfg(feature = "db")]
use crate::server::state::AppState;
#[cfg(feature = "db")]
use crate::server::routes::query_handler;

pub mod state;
pub mod routes;

pub struct Server {
    host: String,
    port: u16,
    #[cfg(feature = "db")]
    database_url: Option<String>,
}

impl Server {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            #[cfg(feature = "db")]
            database_url: std::env::var("DATABASE_URL").ok(),
        }
    }

    #[cfg(feature = "db")]
    pub fn with_database(mut self, url: String) -> Self {
        self.database_url = Some(url);
        self
    }

    pub async fn run(self) -> Result<(), Error> {
        let addr: SocketAddr = format!("{}:{}", self.host, self.port)
            .parse()
            .map_err(|e: std::net::AddrParseError| Error::HttpServer(e.to_string()))?;

        #[cfg(feature = "db")]
        let app = self.build_app_with_db().await?;

        #[cfg(not(feature = "db"))]
        let app = self.build_app_without_db();

        tracing::info!("Starting HTTP server on {}", addr);
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e: std::io::Error| Error::HttpServer(e.to_string()))?;
        axum::serve(listener, app)
            .await
            .map_err(|e: std::io::Error| Error::HttpServer(e.to_string()))?;

        Ok(())
    }

    /// Build router with database-dependent routes
    #[cfg(feature = "db")]
    async fn build_app_with_db(self) -> Result<axum::Router, Error> {
        use sqlx::postgres::PgPoolOptions;

        // Get database URL
        let db_url = self.database_url
            .or_else(|| std::env::var("DATABASE_URL").ok())
            .ok_or_else(|| Error::Config("DATABASE_URL not set".to_string()))?;

        // Create database pool
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(&db_url)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        tracing::info!("Connected to database");

        // Create repository
        let repository = JobRepository::new(pool.clone());

        // Create load job handler
        let load_handler = Arc::new(LoadJobHandler::new(pool.clone()));

        // Create query job handler
        let query_handler_job = Arc::new(QueryJobHandler::new(pool.clone()));

        // Create executor with handlers
        let executor = JobExecutor::new(
            repository,
            vec![load_handler, query_handler_job],
            format!("server-{}", std::process::id()),
        );

        // Create application state
        let app_state = AppState::new(executor, pool);

        // Build router with API routes
        let app = axum::Router::new()
            .route("/health", get(health_handler))
            .route("/chart", get(chart_handler))
            // Job routes (from 06-03)
            .route("/api/v1/load", post(routes::load_handler))
            .route("/api/v1/jobs/:id", get(routes::get_job_handler))
            .route("/api/v1/jobs", get(routes::list_jobs_handler))
            // Query routes (dedicated endpoints)
            .route("/api/v1/query/wedding", post(routes::wedding_query_handler))
            .route("/api/v1/query/project", post(routes::project_query_handler))
            .route("/api/v1/query/travel", post(routes::travel_query_handler))
            // Generic query route (fallback)
            .route("/api/v1/query/:query_name", post(query_handler))
            .with_state(app_state);

        Ok(app)
    }

    /// Build router without database-dependent routes
    #[cfg(not(feature = "db"))]
    fn build_app_without_db(self) -> axum::Router {
        axum::Router::new()
            .route("/health", get(health_handler))
            .route("/chart", get(chart_handler))
    }
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "healthy"}))
}

// #[derive(serde::Deserialize)]
// struct ChartParams {
//     lat: Option<f64>,
//     lon: Option<f64>,
//     time: Option<String>,
// }

#[derive(serde::Deserialize)]
struct ChartQuery {
    lat: Option<f64>,
    lon: Option<f64>,
    time: Option<String>,
    format: Option<String>,
}

async fn chart_handler(Query(query): Query<ChartQuery>) -> Result<Response, StatusCode> {
    let lat = query.lat.unwrap_or(0.0);
    let lon = query.lon.unwrap_or(0.0);
    let _time_str = query.time.clone();

    let config = ChartConfig::new(
        HouseSystem::Placidus,
        GeoPos::new(lat, lon, 0.0),
        2451545.0,
    );

    let calculator: SwissEphChartCalculator = match SwissEphChartCalculator::new(config.clone()) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to create chart calculator: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let chart_data = match calculator.calculate_chart() {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("Failed to calculate chart: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Handle different output formats
    match query.format.as_deref() {
        Some("svg") => {
            use crate::svg_renderer::SvgRenderer;
            let renderer = SvgRenderer::new(800, 800);
            let svg = match renderer.render_chart(&chart_data) {
                Ok(svg_data) => svg_data,
                Err(e) => {
                    tracing::error!("Failed to render SVG: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };
            Ok((
                StatusCode::OK,
                [("Content-Type", "image/svg+xml")],
                Body::from(svg),
            ).into_response())
        }
        _ => {
            // Default to PNG
            let size = crate::renderer::Size::new(800.0, 800.0);
            let mut renderer = match crate::renderer::Renderer::new(size.clone()) {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!("Failed to create renderer: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            match renderer.render_chart(&chart_data) {
                Ok(_) => {},
                Err(e) => {
                    tracing::error!("Failed to render chart: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            let buffer = match renderer.export_png() {
                Ok(png_data) => png_data,
                Err(e) => {
                    tracing::error!("Failed to export PNG: {:?}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            Ok((
                StatusCode::OK,
                [("Content-Type", "image/png")],
                Body::from(buffer),
            ).into_response())
        }
    }
}

// Need Arc for handlers in JobExecutor
#[cfg(feature = "db")]
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = axum::Router::new().route("/health", get(health_handler));

        let response = tower::ServiceBuilder::new()
            .service(app)
            .oneshot(
                http::Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await;

        assert!(response.unwrap().status().is_success());
    }
}
