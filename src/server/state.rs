//! Application state shared across HTTP handlers
//!
//! Provides access to the job executor and database pool
//! via axum's State extractor pattern.

use std::sync::Arc;

use crate::jobs::executor::JobExecutor;

/// Application state shared across all request handlers
#[derive(Clone)]
pub struct AppState {
    /// Job executor for managing job lifecycle
    executor: Arc<JobExecutor>,
    /// Database connection pool for repository creation
    pool: sqlx::Pool<sqlx::Postgres>,
}

impl AppState {
    /// Create new application state with the given executor and pool
    pub fn new(executor: JobExecutor, pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        Self {
            executor: Arc::new(executor),
            pool,
        }
    }

    /// Get a reference to the job executor
    pub fn executor(&self) -> &JobExecutor {
        &self.executor
    }

    /// Get a clone of the connection pool
    ///
    /// This provides access to the database pool for creating repositories
    /// and other database operations in handlers.
    pub fn get_pool(&self) -> sqlx::Pool<sqlx::Postgres> {
        self.pool.clone()
    }
}
