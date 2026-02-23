use std::fmt;

#[derive(Debug)]
pub enum Error {
    Config(String),
    Io(std::io::Error),
    SwissEphemeris(String),
    Rendering(String),
    HttpServer(String),
    Database(String),
    Unknown(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Config(msg) => write!(f, "Configuration error: {}", msg),
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::SwissEphemeris(msg) => write!(f, "Swiss Ephemeris error: {}", msg),
            Error::Rendering(msg) => write!(f, "Rendering error: {}", msg),
            Error::HttpServer(msg) => write!(f, "HTTP server error: {}", msg),
            Error::Database(msg) => write!(f, "Database error: {}", msg),
            Error::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl From<ron::de::Error> for Error {
    fn from(err: ron::de::Error) -> Self {
        Error::Config(err.to_string())
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Unknown(err.to_string())
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Error::Unknown(err)
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Error::Unknown(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}
