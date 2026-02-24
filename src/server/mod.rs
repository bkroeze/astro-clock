use axum::{extract::Query, response::{IntoResponse, Response}};
use axum::routing::get;
use axum::Json;
use axum::http::StatusCode;
use axum::body::Body;
use std::net::SocketAddr;

use crate::chart::{ChartCalculator, ChartConfig, GeoPos, HouseSystem};
use crate::SwissEphChartCalculator;
use crate::errors::Error;

pub struct Server {
    host: String,
    port: u16,
}

impl Server {
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }

    pub async fn run(self) -> Result<(), Error> {
        let addr: SocketAddr = format!("{}:{}", self.host, self.port)
            .parse()
            .map_err(|e: std::net::AddrParseError| Error::HttpServer(e.to_string()))?;

        let app = axum::Router::new()
            .route("/health", get(health_handler))
            .route("/chart", get(chart_handler));

        tracing::info!("Starting HTTP server on {}", addr);
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e: std::io::Error| Error::HttpServer(e.to_string()))?;
        axum::serve(listener, app)
            .await
            .map_err(|e: std::io::Error| Error::HttpServer(e.to_string()))?;

        Ok(())
    }
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "healthy"}))
}

#[derive(serde::Deserialize)]
struct ChartParams {
    lat: Option<f64>,
    lon: Option<f64>,
    time: Option<String>,
}

#[derive(serde::Deserialize)]
struct ChartQuery {
    lat: Option<f64>,
    lon: Option<f64>,
    time: Option<String>,
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
