use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to parse configuration: {0}")]
    ParseError(String),
    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),
    #[error("IO error: {0}")]
    IoError(String),
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::IoError(err.to_string())
    }
}

impl From<ron::de::Error> for ConfigError {
    fn from(err: ron::de::Error) -> Self {
        ConfigError::ParseError(err.to_string())
    }
}

impl From<ron::error::SpannedError> for ConfigError {
    fn from(err: ron::error::SpannedError) -> Self {
        ConfigError::ParseError(err.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocationConfig {
    #[serde(default)]
    pub latitude: Option<f64>,
    #[serde(default)]
    pub longitude: Option<f64>,
}

impl Default for LocationConfig {
    fn default() -> Self {
        Self {
            latitude: None,
            longitude: None,
        }
    }
}

impl LocationConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if let Some(lat) = self.latitude {
            if lat < -90.0 || lat > 90.0 {
                return Err(ConfigError::InvalidValue(format!(
                    "latitude must be between -90 and 90, got {}",
                    lat
                )));
            }
        }
        if let Some(lon) = self.longitude {
            if lon < -180.0 || lon > 180.0 {
                return Err(ConfigError::InvalidValue(format!(
                    "longitude must be between -180 and 180, got {}",
                    lon
                )));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChartConfig {
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
    #[serde(default = "default_background_color")]
    pub background_color: String,
    #[serde(default = "default_text_color")]
    pub text_color: String,
    #[serde(default = "default_house_color")]
    pub house_color: String,
    #[serde(default = "default_planet_color")]
    pub planet_color: String,
    #[serde(default = "default_zodiac_color")]
    pub zodiac_color: String,
    #[serde(default)]
    pub location: LocationConfig,
    #[serde(default = "default_orb")]
    pub orb: f64,
    #[serde(default = "default_house_system")]
    pub house_system: String,
}

impl ChartConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.width < 100 || self.width > 4096 {
            return Err(ConfigError::InvalidValue(format!(
                "chart width must be 100-4096, got {}",
                self.width
            )));
        }
        if self.height < 100 || self.height > 4096 {
            return Err(ConfigError::InvalidValue(format!(
                "chart height must be 100-4096, got {}",
                self.height
            )));
        }
        if self.orb < 0.0 || self.orb > 15.0 {
            return Err(ConfigError::InvalidValue(format!(
                "aspect orb must be 0-15 degrees, got {}",
                self.orb
            )));
        }
        // Validate house system
        let valid_systems = [
            "Placidus",
            "Koch",
            "Equal",
            "Whole",
            "Porphyry",
            "Regiomontanus",
            "Campanus",
            "Morinus",
            "Alcabitus",
            "Topocentric",
            "Vehlow",
        ];
        if !valid_systems.contains(&self.house_system.as_str()) {
            return Err(ConfigError::InvalidValue(format!(
                "invalid house system: {}. Valid options: {}",
                self.house_system,
                valid_systems.join(", ")
            )));
        }
        self.location.validate()?;
        Ok(())
    }
}

fn default_width() -> u32 {
    800
}

fn default_height() -> u32 {
    800
}

fn default_background_color() -> String {
    "#000000".to_string()
}

fn default_text_color() -> String {
    "#ffffff".to_string()
}

fn default_house_color() -> String {
    "#333333".to_string()
}

fn default_planet_color() -> String {
    "#ffcc00".to_string()
}

fn default_zodiac_color() -> String {
    "#444444".to_string()
}

fn default_orb() -> f64 {
    3.0
}

fn default_house_system() -> String {
    "Placidus".to_string()
}

impl Default for ChartConfig {
    fn default() -> Self {
        Self {
            width: default_width(),
            height: default_height(),
            background_color: default_background_color(),
            text_color: default_text_color(),
            house_color: default_house_color(),
            planet_color: default_planet_color(),
            zodiac_color: default_zodiac_color(),
            location: LocationConfig::default(),
            orb: default_orb(),
            house_system: default_house_system(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_cors_origins")]
    pub cors_origins: Vec<String>,
}

impl ServerConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.port == 0 {
            return Err(ConfigError::InvalidValue(
                "server port cannot be 0".to_string(),
            ));
        }
        if self.host.is_empty() {
            return Err(ConfigError::InvalidValue(
                "server host cannot be empty".to_string(),
            ));
        }
        Ok(())
    }
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    3000
}

fn default_cors_origins() -> Vec<String> {
    vec!["http://localhost:3000".to_string()]
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            cors_origins: default_cors_origins(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DatabaseConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_database_url")]
    pub url: String,
    #[serde(default = "default_table_name")]
    pub table_name: String,
}

impl DatabaseConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.enabled && self.url.is_empty() {
            return Err(ConfigError::InvalidValue(
                "database url required when enabled".to_string(),
            ));
        }
        Ok(())
    }
}

fn default_database_url() -> String {
    "postgresql://user:password@localhost/astro".to_string()
}

fn default_table_name() -> String {
    "charts".to_string()
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            url: default_database_url(),
            table_name: default_table_name(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    #[serde(default)]
    pub chart: ChartConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            chart: ChartConfig::default(),
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let config: Self = ron::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn from_file_or_default<P: AsRef<Path>>(path: Option<P>) -> Result<Self, ConfigError> {
        match path {
            Some(p) => Self::from_file(p),
            None => Ok(Self::default()),
        }
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        self.chart.validate()?;
        self.server.validate()?;
        self.database.validate()?;
        Ok(())
    }
}

pub type Result<T, E = ConfigError> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.chart.width, 800);
        assert_eq!(config.chart.height, 800);
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.server.host, "127.0.0.1");
        assert!(!config.database.enabled);
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        config.chart.width = 50;
        assert!(config.validate().is_err());

        config.chart.width = 800;
        config.chart.height = 5000;
        assert!(config.validate().is_err());

        config.chart.height = 800;
        config.server.port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r##"
            (
                chart: (
                    width: 1024,
                    height: 768,
                    background_color: "#111111",
                    text_color: "#eeeeee",
                    house_color: "#222222",
                    planet_color: "#ff0000",
                    zodiac_color: "#333333",
                ),
                server: (
                    host: "0.0.0.0",
                    port: 8080,
                    cors_origins: ["http://example.com"],
                ),
                database: (
                    enabled: true,
                    url: "postgresql://localhost/astro_test",
                    table_name: "test_charts",
                ),
            )
        "##;
        temp_file.write_all(config_content.as_bytes()).unwrap();

        let config = AppConfig::from_file(temp_file.path()).unwrap();
        assert_eq!(config.chart.width, 1024);
        assert_eq!(config.chart.height, 768);
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8080);
        assert!(config.database.enabled);
        assert_eq!(config.database.table_name, "test_charts");
    }

    #[test]
    fn test_config_from_file_partial() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
            (
                chart: (
                    width: 500,
                    height: 500,
                ),
            )
        "#;
        temp_file.write_all(config_content.as_bytes()).unwrap();

        let config = AppConfig::from_file(temp_file.path()).unwrap();
        assert_eq!(config.chart.width, 500);
        assert_eq!(config.chart.height, 500);
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.database.enabled, false);
    }

    #[test]
    fn test_config_from_file_or_default() {
        let config = AppConfig::from_file_or_default(None::<&str>).unwrap();
        assert_eq!(config.chart.width, 800);
    }

    #[test]
    fn test_config_parse_error() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"invalid ron content here").unwrap();

        let result = AppConfig::from_file(temp_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_chart_config_validate() {
        let valid = ChartConfig::default();
        assert!(valid.validate().is_ok());

        let mut invalid = ChartConfig::default();
        invalid.width = 50;
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_server_config_validate() {
        let valid = ServerConfig::default();
        assert!(valid.validate().is_ok());

        let mut invalid = ServerConfig::default();
        invalid.port = 0;
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_database_config_validate() {
        let valid = DatabaseConfig::default();
        assert!(valid.validate().is_ok());

        let mut invalid = DatabaseConfig::default();
        invalid.enabled = true;
        invalid.url = String::new();
        assert!(invalid.validate().is_err());
    }
}
