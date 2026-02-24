use crate::ChartError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Swiss Ephemeris error: {0}")]
    SwissEphemeris(String),
    #[error("Rendering error: {0}")]
    Rendering(String),
    #[error("HTTP server error: {0}")]
    HttpServer(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Chart error: {0}")]
    Chart(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<ron::de::Error> for Error {
    fn from(err: ron::de::Error) -> Self {
        Error::Config(err.to_string())
    }
}

impl From<ChartError> for Error {
    fn from(err: ChartError) -> Self {
        match err {
            ChartError::SwissEph(msg) => Error::SwissEphemeris(msg),
            ChartError::InvalidParams(msg) => Error::Config(msg),
            ChartError::MissingEphemerisData => {
                Error::SwissEphemeris("Missing ephemeris data".into())
            }
            ChartError::UnknownPlanet(msg) => Error::SwissEphemeris(msg),
            ChartError::CalculationFailed(msg) => Error::Unknown(msg),
        }
    }
}

impl From<crate::config::ConfigError> for Error {
    fn from(err: crate::config::ConfigError) -> Self {
        Error::Config(err.to_string())
    }
}
