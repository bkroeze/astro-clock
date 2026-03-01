//! Server API routes module
//!
//! Provides HTTP endpoints for job management and data loading operations.
//! All routes are versioned under /api/v1/.

pub mod jobs;

// Re-export job-related handlers for convenient access
pub use jobs::{get_job_handler, load_handler};
