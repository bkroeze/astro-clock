//! Server API routes module
//!
//! Provides HTTP endpoints for job management and data loading operations.
//! All routes are versioned under /api/v1/.
//!
//! ## Available Routes
//! - `jobs`: Load data and job status endpoints
//! - `queries`: Named query endpoints (wedding, project, travel)

#[cfg(feature = "db")]
pub mod jobs;

#[cfg(feature = "db")]
// Re-export job-related handlers for convenient access
pub use jobs::{delete_job_handler, get_job_handler, list_jobs_handler, load_handler};

#[cfg(feature = "db")]
// Re-export query-related handlers
pub use queries::{
    project_query_handler, query_handler, travel_query_handler, wedding_query_handler,
};

#[cfg(feature = "db")]
pub mod queries;
