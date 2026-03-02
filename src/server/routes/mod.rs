//! Server API routes module
//!
//! Provides HTTP endpoints for job management and data loading operations.
//! All routes are versioned under /api/v1/.
//!
//! ## Available Routes
//! - `jobs`: Load data and job status endpoints
//! - `queries`: Named query endpoints (wedding, project, travel)

pub mod jobs;

// Re-export job-related handlers for convenient access
pub use jobs::{get_job_handler, load_handler};

#[cfg(feature = "db")]
// Re-export query-related handlers
pub use queries::{query_handler, wedding_query_handler, project_query_handler, travel_query_handler};

#[cfg(feature = "db")]
pub mod queries;
