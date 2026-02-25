use thiserror::Error;

/// Errors that can occur during query operations
#[derive(Error, Debug)]
pub enum QueryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Invalid criteria: {0}")]
    InvalidCriteria(String),

    #[error("Query timeout after {0}ms")]
    Timeout(u64),

    #[error("Chunk manager error: {0}")]
    ChunkManager(String),
}

impl From<super::chunk_manager::ChunkManagerError> for QueryError {
    fn from(err: super::chunk_manager::ChunkManagerError) -> Self {
        QueryError::ChunkManager(err.to_string())
    }
}
