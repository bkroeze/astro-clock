//! Performance monitoring and optimization modules

pub mod benchmark;
pub mod interpolation;
pub mod memory_monitor;

pub use benchmark::{
    run_scheduled_benchmarks, check_performance_regression,
    AlertLevel, BenchmarkConfig, BenchmarkRun, BenchmarkRunner,
};

pub use interpolation::{
    interpolate_latitude, interpolate_longitude, interpolate_position, interpolate_positions,
    InterpolationError,
};

pub use memory_monitor::{
    check_memory_pressure, MemoryMonitor, MemoryPressure, MemoryStats,
    DEFAULT_EVICTION_THRESHOLD_PCT, DEFAULT_HARD_LIMIT_MB, DEFAULT_SOFT_LIMIT_MB,
};
