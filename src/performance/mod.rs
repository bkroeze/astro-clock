//! Performance monitoring and optimization modules

pub mod interpolation;
pub mod memory_monitor;

pub use interpolation::{
    interpolate_latitude, interpolate_longitude, interpolate_position, interpolate_positions,
    InterpolationError,
};

pub use memory_monitor::{
    check_memory_pressure, MemoryMonitor, MemoryPressure, MemoryStats,
    DEFAULT_EVICTION_THRESHOLD_PCT, DEFAULT_HARD_LIMIT_MB, DEFAULT_SOFT_LIMIT_MB,
};
