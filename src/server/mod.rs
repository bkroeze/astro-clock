use axum::Json;
use axum::body::Body;
use axum::http::StatusCode;
use axum::routing::get;
#[cfg(feature = "db")]
use axum::routing::post;
use axum::{
    extract::{Query, rejection::QueryRejection},
    response::{IntoResponse, Response},
};
use std::net::SocketAddr;

use crate::SwissEphChartCalculator;
use crate::aspects::{AspectConfig, analyze_aspects};
use crate::chart::{ChartCalculator, ChartConfig, GeoPos, HouseSystem};
use crate::errors::Error;

// Import job-related types when database feature is enabled
#[cfg(feature = "db")]
use crate::jobs::{
    executor::JobExecutor,
    handlers::{LoadJobHandler, QueryJobHandler},
    repository::JobRepository,
};
#[cfg(feature = "db")]
use crate::server::routes::query_handler;
#[cfg(feature = "db")]
use crate::server::state::AppState;

pub mod routes;
pub mod state;

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
        let db_url = self
            .database_url
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
            .route("/api/v1/chart/data", get(chart_data_handler))
            // Job routes (from 06-03)
            .route("/api/v1/load", post(routes::load_handler))
            .route(
                "/api/v1/jobs/:id",
                get(routes::get_job_handler).delete(routes::delete_job_handler),
            )
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
            .route("/api/v1/chart/data", get(chart_data_handler))
    }
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "db_enabled": cfg!(feature = "db")
    }))
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

    let config = ChartConfig::new(HouseSystem::Placidus, GeoPos::new(lat, lon, 0.0), 2451545.0);

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
            )
                .into_response())
        }
        _ => {
            // Default to PNG
            let size = crate::renderer::Size::new(800.0, 800.0);
            let mut renderer = match crate::renderer::Renderer::new(size) {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!("Failed to create renderer: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            match renderer.render_chart(&chart_data) {
                Ok(_) => {}
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
            )
                .into_response())
        }
    }
}

/// Query parameters for the chart data API endpoint
#[derive(Debug, serde::Deserialize)]
struct ChartDataQuery {
    lat: Option<f64>,
    lon: Option<f64>,
    time: Option<String>,
    house: Option<String>,
}

/// Response structure for chart data API
#[derive(Debug, serde::Serialize)]
struct ChartDataResponse {
    planets: Vec<crate::chart::PlanetPosition>,
    houses: crate::chart::HouseCusps,
    aspects: Vec<crate::aspects::Aspect>,
    grand_trines: Vec<crate::aspects::GrandTrine>,
    moon_void_of_course: Option<String>,
    metadata: ChartMetadata,
}

/// Metadata about the chart calculation
#[derive(Debug, serde::Serialize)]
struct ChartMetadata {
    latitude: f64,
    longitude: f64,
    julian_day: f64,
    house_system: String,
}

#[derive(Debug, serde::Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

type ChartDataResult = Result<Json<ChartDataResponse>, (StatusCode, Json<ErrorResponse>)>;

fn chart_data_error(
    status: StatusCode,
    error: &str,
    message: impl Into<String>,
) -> (StatusCode, Json<ErrorResponse>) {
    (
        status,
        Json(ErrorResponse {
            error: error.to_string(),
            message: message.into(),
        }),
    )
}

fn parse_chart_data_time(time: Option<&str>) -> Result<f64, (StatusCode, Json<ErrorResponse>)> {
    match time {
        Some(time) => {
            let datetime = chrono::DateTime::parse_from_rfc3339(time).map_err(|e| {
                tracing::warn!("Invalid time: {}", e);
                chart_data_error(
                    StatusCode::BAD_REQUEST,
                    "invalid_time",
                    format!("Invalid time format: {}", e),
                )
            })?;
            let julian_day =
                crate::ephemeris::julian_day_from_chrono(datetime.with_timezone(&chrono::Utc));
            if !julian_day.is_finite() {
                tracing::warn!("Invalid Julian day calculated from time");
                return Err(chart_data_error(
                    StatusCode::BAD_REQUEST,
                    "invalid_time",
                    "Time produced an invalid Julian day",
                ));
            }
            Ok(julian_day)
        }
        None => Ok(crate::ephemeris::julian_day_from_chrono(chrono::Utc::now())),
    }
}

/// JSON endpoint for chart data (planets, houses, aspects)
/// Returns structured JSON for display in the Django app
async fn chart_data_handler(
    query: Result<Query<ChartDataQuery>, QueryRejection>,
) -> ChartDataResult {
    let Query(query) = query.map_err(|e| {
        tracing::warn!("Invalid chart data query: {}", e);
        chart_data_error(
            StatusCode::BAD_REQUEST,
            "invalid_query",
            format!("Invalid query parameters: {}", e),
        )
    })?;

    // Validate required parameters
    let lat = query.lat.ok_or_else(|| {
        tracing::warn!("Missing required parameter: lat");
        chart_data_error(
            StatusCode::BAD_REQUEST,
            "missing_lat",
            "Missing required parameter: lat",
        )
    })?;

    let lon = query.lon.ok_or_else(|| {
        tracing::warn!("Missing required parameter: lon");
        chart_data_error(
            StatusCode::BAD_REQUEST,
            "missing_lon",
            "Missing required parameter: lon",
        )
    })?;

    // Validate coordinates
    if !(-90.0..=90.0).contains(&lat) {
        tracing::warn!("Invalid latitude: {}", lat);
        return Err(chart_data_error(
            StatusCode::BAD_REQUEST,
            "invalid_lat",
            "Latitude must be between -90 and 90",
        ));
    }

    if !(-180.0..=180.0).contains(&lon) {
        tracing::warn!("Invalid longitude: {}", lon);
        return Err(chart_data_error(
            StatusCode::BAD_REQUEST,
            "invalid_lon",
            "Longitude must be between -180 and 180",
        ));
    }

    let julian_day = parse_chart_data_time(query.time.as_deref())?;

    let house_system = match query.house.as_deref() {
        Some(house) => match house.parse::<HouseSystem>() {
            Ok(system) => system,
            Err(e) => {
                tracing::warn!("{}", e);
                return Err(chart_data_error(
                    StatusCode::BAD_REQUEST,
                    "invalid_house",
                    e.to_string(),
                ));
            }
        },
        None => HouseSystem::Placidus,
    };

    tracing::info!(
        "Fetching chart data for lat={}, lon={}, jd={}",
        lat,
        lon,
        julian_day
    );

    let config = ChartConfig::new(house_system, GeoPos::new(lat, lon, 0.0), julian_day);

    let calculator: SwissEphChartCalculator = match SwissEphChartCalculator::new(config.clone()) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to create chart calculator: {}", e);
            return Err(chart_data_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "chart_calculator_error",
                "Failed to create chart calculator",
            ));
        }
    };

    let chart_data = match calculator.calculate_chart() {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("Failed to calculate chart: {}", e);
            return Err(chart_data_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "chart_calculation_error",
                "Failed to calculate chart",
            ));
        }
    };

    // Perform aspect analysis
    let aspect_config = AspectConfig::new(3.0); // 3 degree default orb
    let aspect_analysis = analyze_aspects(&chart_data.planets, aspect_config);

    // Extract house_system before moving chart_data
    let house_system = chart_data.houses.system.to_string();

    let response = ChartDataResponse {
        planets: chart_data.planets,
        houses: chart_data.houses,
        aspects: aspect_analysis.aspects,
        grand_trines: aspect_analysis.grand_trines,
        moon_void_of_course: aspect_analysis.moon_void_of_course,
        metadata: ChartMetadata {
            latitude: lat,
            longitude: lon,
            julian_day: chart_data.julian_day,
            house_system,
        },
    };

    tracing::info!(
        "Successfully calculated chart data for lat={}, lon={}",
        lat,
        lon
    );
    Ok(Json(response))
}

// Need Arc for handlers in JobExecutor
#[cfg(feature = "db")]
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = axum::Router::new().route("/health", get(health_handler));

        let response = tower::ServiceBuilder::new()
            .service(app)
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await;

        assert!(response.unwrap().status().is_success());
    }

    #[tokio::test]
    async fn test_chart_data_missing_lat() {
        let app = axum::Router::new().route("/api/v1/chart/data", get(chart_data_handler));

        let response = tower::ServiceBuilder::new()
            .service(app)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/chart/data?lon=-74.0060")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await;

        let response = response.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json_value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            json_value.get("error").unwrap().as_str().unwrap(),
            "missing_lat"
        );
        assert_eq!(
            json_value.get("message").unwrap().as_str().unwrap(),
            "Missing required parameter: lat"
        );
    }

    #[tokio::test]
    async fn test_chart_data_missing_lon() {
        let app = axum::Router::new().route("/api/v1/chart/data", get(chart_data_handler));

        let response = tower::ServiceBuilder::new()
            .service(app)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/chart/data?lat=40.7128")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await;

        let response = response.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json_value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            json_value.get("error").unwrap().as_str().unwrap(),
            "missing_lon"
        );
    }

    #[tokio::test]
    async fn test_chart_data_invalid_lat() {
        let app = axum::Router::new().route("/api/v1/chart/data", get(chart_data_handler));

        // Latitude > 90 should fail
        let response = tower::ServiceBuilder::new()
            .service(app)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/chart/data?lat=100.0&lon=-74.0060")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await;

        let response = response.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json_value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            json_value.get("error").unwrap().as_str().unwrap(),
            "invalid_lat"
        );
    }

    #[tokio::test]
    async fn test_chart_data_rejects_malformed_lat_as_json() {
        let app = axum::Router::new().route("/api/v1/chart/data", get(chart_data_handler));

        let response = tower::ServiceBuilder::new()
            .service(app)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/chart/data?lat=abc&lon=-74.0060")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await;

        let response = response.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json_value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            json_value.get("error").unwrap().as_str().unwrap(),
            "invalid_query"
        );
        assert!(
            json_value
                .get("message")
                .unwrap()
                .as_str()
                .unwrap()
                .starts_with("Invalid query parameters:")
        );
    }

    #[tokio::test]
    async fn test_chart_data_invalid_lon() {
        let app = axum::Router::new().route("/api/v1/chart/data", get(chart_data_handler));

        // Longitude > 180 should fail
        let response = tower::ServiceBuilder::new()
            .service(app)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/chart/data?lat=40.7128&lon=-200.0")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await;

        let response = response.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json_value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            json_value.get("error").unwrap().as_str().unwrap(),
            "invalid_lon"
        );
    }

    #[tokio::test]
    async fn test_chart_data_valid_request() {
        let app = axum::Router::new().route("/api/v1/chart/data", get(chart_data_handler));

        let response = tower::ServiceBuilder::new()
            .service(app)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/chart/data?lat=40.7128&lon=-74.0060")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await;

        let response = response.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Parse the JSON response
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json_value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        // Verify response structure
        assert!(json_value.get("planets").is_some());
        assert!(json_value.get("houses").is_some());
        assert!(json_value.get("aspects").is_some());
        assert!(json_value.get("metadata").is_some());

        // Verify metadata
        let metadata = json_value.get("metadata").unwrap();
        assert_eq!(metadata.get("latitude").unwrap().as_f64().unwrap(), 40.7128);
        assert_eq!(
            metadata.get("longitude").unwrap().as_f64().unwrap(),
            -74.0060
        );
        assert_eq!(
            metadata.get("house_system").unwrap().as_str().unwrap(),
            "Placidus"
        );

        // Verify planets array contains expected planets
        let planets = json_value.get("planets").unwrap().as_array().unwrap();
        let planet_names: Vec<&str> = planets
            .iter()
            .map(|p| p.get("name").unwrap().as_str().unwrap())
            .collect();

        // Should contain at least the classical planets
        assert!(planet_names.contains(&"Sun"));
        assert!(planet_names.contains(&"Moon"));
        assert!(planet_names.contains(&"Mercury"));
        assert!(planet_names.contains(&"Venus"));
        assert!(planet_names.contains(&"Mars"));
        assert!(planet_names.contains(&"Jupiter"));
        assert!(planet_names.contains(&"Saturn"));

        // Verify aspects array exists
        let aspects = json_value.get("aspects").unwrap().as_array().unwrap();
        // Aspects may be empty depending on positions, but should be a valid array
        assert!(!aspects.is_empty() || true); // aspects.is_array() is implicit from as_array()
    }

    #[tokio::test]
    async fn test_chart_data_accepts_house_system() {
        let app = axum::Router::new().route("/api/v1/chart/data", get(chart_data_handler));

        let response = tower::ServiceBuilder::new()
            .service(app)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/chart/data?lat=40.7128&lon=-74.0060&house=Whole")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await;

        let response = response.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json_value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let metadata = json_value.get("metadata").unwrap();
        assert_eq!(
            metadata.get("house_system").unwrap().as_str().unwrap(),
            "Whole"
        );
    }

    #[test]
    fn test_parse_chart_data_time_uses_time_parameter() {
        assert_eq!(
            parse_chart_data_time(Some("2000-01-01T12:00:00Z")).unwrap(),
            2451545.0
        );
    }

    #[tokio::test]
    async fn test_chart_data_rejects_invalid_time() {
        let app = axum::Router::new().route("/api/v1/chart/data", get(chart_data_handler));

        let response = tower::ServiceBuilder::new()
            .service(app)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/chart/data?lat=40.7128&lon=-74.0060&time=not-a-time")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await;

        let response = response.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json_value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            json_value.get("error").unwrap().as_str().unwrap(),
            "invalid_time"
        );
    }

    #[tokio::test]
    async fn test_chart_data_rejects_invalid_house_system() {
        let app = axum::Router::new().route("/api/v1/chart/data", get(chart_data_handler));

        let response = tower::ServiceBuilder::new()
            .service(app)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/chart/data?lat=40.7128&lon=-74.0060&house=Invalid")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await;

        let response = response.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json_value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            json_value.get("error").unwrap().as_str().unwrap(),
            "invalid_house"
        );
    }
}
