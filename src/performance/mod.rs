//! Performance monitoring and optimization modules

#[cfg(feature = "db")]
pub mod benchmark;
#[cfg(feature = "db")]
pub mod interpolation;
pub mod memory_monitor;

#[cfg(feature = "db")]
pub use benchmark::{
    AlertLevel, BenchmarkConfig, BenchmarkRun, BenchmarkRunner, check_performance_regression,
    run_scheduled_benchmarks,
};

#[cfg(feature = "db")]
pub use interpolation::{
    InterpolationError, interpolate_latitude, interpolate_longitude, interpolate_position,
    interpolate_positions,
};

pub use memory_monitor::{
    DEFAULT_EVICTION_THRESHOLD_PCT, DEFAULT_HARD_LIMIT_MB, DEFAULT_SOFT_LIMIT_MB, MemoryMonitor,
    MemoryPressure, MemoryStats, check_memory_pressure,
};
