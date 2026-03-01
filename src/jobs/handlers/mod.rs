//! Job handlers for specific job types
//!
//! Each handler implements the `JobHandler` trait for a specific `JobType`,
//! providing the business logic for job execution. Handlers are registered
//! with the `JobExecutor` and called when jobs of their type are processed.
//!
//! # Available Handlers
//!
//! - `LoadJobHandler`: Handles data loading for date ranges
//!
//! # Example
//!
//! ```rust,ignore
//! use astro_clock::jobs::{LoadJobHandler, JobExecutor};
//! use std::sync::Arc;
//!
//! // Create handlers
//! let load_handler = Arc::new(LoadJobHandler::new(pool.clone()));
//!
//! // Register with executor
//! let executor = JobExecutor::new(repository, vec![load_handler], worker_id);
//! ```

pub mod load;

pub use load::LoadJobHandler;
