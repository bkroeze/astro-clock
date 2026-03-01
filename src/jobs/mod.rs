//! Job system for async operations and incremental data loading
//!
//! Provides job queue, state management, and repository patterns
//! for tracking async operations with both sync and async execution modes.

#[cfg(feature = "db")]
pub mod handlers;
pub mod error;
pub mod executor;
pub mod repository;
pub mod types;

// Re-export commonly used types
pub use error::{JobError, JobResult};
pub use executor::{JobExecutor, JobHandler};
pub use repository::{JobRepository, LoadedDaysRepository};
pub use types::{Job, JobStatus, JobType};

// Re-export handlers (only available with db feature)
#[cfg(feature = "db")]
pub use handlers::LoadJobHandler;
