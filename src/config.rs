use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to parse configuration: {0}")]
    ParseError(String),
    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartConfig {
    pub width: u32,
    pub height: u32,
    pub background_color: String,
    pub text_color: String,
    pub house_color: String,
    pub planet_color: String,
    pub zodiac_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub enabled: bool,
    pub url: String,
    pub table_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_chart_config")]
    pub chart: ChartConfig,
    #[serde(default = "default_server_config")]
    pub server: ServerConfig,
    #[serde(default = "default_database_config")]
    pub database: DatabaseConfig,
}

fn default_chart_config() -> ChartConfig {
    ChartConfig {
        width: 800,
        height: 800,
        background_color: "#000000".to_string(),
        text_color: "#ffffff".to_string(),
        house_color: "#333333".to_string(),
        planet_color: "#ffcc00".to_string(),
        zodiac_color: "#444444".to_string(),
    }
}

fn default_server_config() -> ServerConfig {
    ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 3000,
        cors_origins: vec!["http://localhost:3000".to_string()],
    }
}

fn default_database_config() -> DatabaseConfig {
    DatabaseConfig {
        enabled: false,
        url: "postgresql://user:password@localhost/astro".to_string(),
        table_name: "charts".to_string(),
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            chart: default_chart_config(),
            server: default_server_config(),
            database: default_database_config(),
        }
    }
}

impl AppConfig {
    pub fn from_args() -> Result<Self, ConfigError> {
        // For now, return default config
        Ok(Self::default())
    }
}

pub type Result<T, E = ConfigError> = std::result::Result<T, E>;
