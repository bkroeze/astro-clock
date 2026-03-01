//! Memory monitoring and pressure detection for cache management

use sysinfo::{get_current_pid, Pid, ProcessRefreshKind, System};
use tracing::{error, info, warn};

/// Default soft memory limit in MB (30MB)
pub const DEFAULT_SOFT_LIMIT_MB: usize = 30;

/// Default hard memory limit in MB (50MB)
pub const DEFAULT_HARD_LIMIT_MB: usize = 50;

/// Default eviction threshold as percentage of hard limit (90%)
pub const DEFAULT_EVICTION_THRESHOLD_PCT: f64 = 90.0;

/// Memory pressure levels for cache management decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryPressure {
    /// Below soft limit - normal operation
    Normal,
    /// Above soft limit (30MB) - consider cache cleanup
    Elevated,
    /// Above 90% of hard limit (45MB) - aggressive eviction
    High,
    /// At or above hard limit (50MB) - critical, must evict
    Critical,
}

impl MemoryPressure {
    /// Returns true if eviction should be performed
    pub fn should_evict(&self) -> bool {
        matches!(self, MemoryPressure::High | MemoryPressure::Critical)
    }

    /// Returns the severity level (higher = more severe)
    pub fn severity(&self) -> u8 {
        match self {
            MemoryPressure::Normal => 0,
            MemoryPressure::Elevated => 1,
            MemoryPressure::High => 2,
            MemoryPressure::Critical => 3,
        }
    }
}

/// Monitors process memory usage and detects pressure levels
pub struct MemoryMonitor {
    soft_limit_mb: usize,
    hard_limit_mb: usize,
    eviction_threshold_pct: f64,
    system: System,
    pid: Pid,
}

impl MemoryMonitor {
    /// Create a new memory monitor with specified limits
    ///
    /// # Arguments
    /// * `soft_limit_mb` - Memory threshold for Elevated pressure
    /// * `hard_limit_mb` - Memory threshold for Critical pressure
    /// * `eviction_threshold_pct` - Percentage of hard limit for High pressure (0-100)
    pub fn new(soft_limit_mb: usize, hard_limit_mb: usize, eviction_threshold_pct: f64) -> Self {
        let mut system = System::new_all();
        let pid = get_current_pid().expect("Failed to get current process PID");

        // Initial refresh to populate data
        system.refresh_memory();
        system.refresh_processes_specifics(ProcessRefreshKind::new().with_memory());

        info!(
            "MemoryMonitor initialized: soft_limit={}MB, hard_limit={}MB, eviction_threshold={}%",
            soft_limit_mb, hard_limit_mb, eviction_threshold_pct
        );

        Self {
            soft_limit_mb,
            hard_limit_mb,
            eviction_threshold_pct,
            system,
            pid,
        }
    }

    /// Create a memory monitor with default configuration
    pub fn with_defaults() -> Self {
        Self::new(
            DEFAULT_SOFT_LIMIT_MB,
            DEFAULT_HARD_LIMIT_MB,
            DEFAULT_EVICTION_THRESHOLD_PCT,
        )
    }

    /// Get current memory usage in MB
    pub fn current_memory_mb(&mut self) -> usize {
        self.refresh_memory_info();

        self.system
            .process(self.pid)
            .map(|p| p.memory() / 1024 / 1024)
            .unwrap_or(0) as usize
    }

    /// Check current memory pressure level
    pub fn check_memory_pressure(&mut self) -> MemoryPressure {
        let current_mb = self.current_memory_mb();
        let hard_threshold =
            (self.hard_limit_mb as f64 * (self.eviction_threshold_pct / 100.0)) as usize;

        let pressure = if current_mb >= self.hard_limit_mb {
            error!(
                "CRITICAL memory pressure: {}MB >= {}MB hard limit",
                current_mb, self.hard_limit_mb
            );
            MemoryPressure::Critical
        } else if current_mb >= hard_threshold {
            warn!(
                "HIGH memory pressure: {}MB >= {}MB ({}% of hard limit)",
                current_mb, hard_threshold, self.eviction_threshold_pct
            );
            MemoryPressure::High
        } else if current_mb >= self.soft_limit_mb {
            info!(
                "ELEVATED memory pressure: {}MB >= {}MB soft limit",
                current_mb, self.soft_limit_mb
            );
            MemoryPressure::Elevated
        } else {
            MemoryPressure::Normal
        };

        pressure
    }

    /// Check if eviction should be performed and return pressure level
    ///
    /// Returns (should_evict, pressure_level) tuple
    pub fn should_evict(&mut self) -> (bool, MemoryPressure) {
        let pressure = self.check_memory_pressure();
        (pressure.should_evict(), pressure)
    }

    /// Get memory statistics
    pub fn memory_stats(&mut self) -> MemoryStats {
        self.refresh_memory_info();

        let current_mb = self.current_memory_mb();
        let hard_threshold =
            (self.hard_limit_mb as f64 * (self.eviction_threshold_pct / 100.0)) as usize;

        MemoryStats {
            current_mb,
            soft_limit_mb: self.soft_limit_mb,
            hard_limit_mb: self.hard_limit_mb,
            eviction_threshold_mb: hard_threshold,
            pressure: self.check_memory_pressure(),
        }
    }

    /// Refresh system memory information
    fn refresh_memory_info(&mut self) {
        self.system.refresh_memory();
        self.system
            .refresh_processes_specifics(ProcessRefreshKind::new().with_memory());
    }
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub current_mb: usize,
    pub soft_limit_mb: usize,
    pub hard_limit_mb: usize,
    pub eviction_threshold_mb: usize,
    pub pressure: MemoryPressure,
}

impl MemoryStats {
    /// Calculate percentage of hard limit used
    pub fn percent_of_hard_limit(&self) -> f64 {
        (self.current_mb as f64 / self.hard_limit_mb as f64) * 100.0
    }

    /// Calculate headroom until hard limit
    pub fn headroom_mb(&self) -> isize {
        self.hard_limit_mb as isize - self.current_mb as isize
    }
}

/// Convenience function to check memory pressure with default limits
pub fn check_memory_pressure() -> MemoryPressure {
    let mut monitor = MemoryMonitor::with_defaults();
    monitor.check_memory_pressure()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_pressure_severity() {
        assert_eq!(MemoryPressure::Normal.severity(), 0);
        assert_eq!(MemoryPressure::Elevated.severity(), 1);
        assert_eq!(MemoryPressure::High.severity(), 2);
        assert_eq!(MemoryPressure::Critical.severity(), 3);
    }

    #[test]
    fn test_memory_pressure_should_evict() {
        assert!(!MemoryPressure::Normal.should_evict());
        assert!(!MemoryPressure::Elevated.should_evict());
        assert!(MemoryPressure::High.should_evict());
        assert!(MemoryPressure::Critical.should_evict());
    }

    #[test]
    fn test_memory_stats_percent() {
        let stats = MemoryStats {
            current_mb: 25,
            soft_limit_mb: 30,
            hard_limit_mb: 50,
            eviction_threshold_mb: 45,
            pressure: MemoryPressure::Normal,
        };
        assert_eq!(stats.percent_of_hard_limit(), 50.0);
    }

    #[test]
    fn test_memory_stats_headroom() {
        let stats = MemoryStats {
            current_mb: 35,
            soft_limit_mb: 30,
            hard_limit_mb: 50,
            eviction_threshold_mb: 45,
            pressure: MemoryPressure::Elevated,
        };
        assert_eq!(stats.headroom_mb(), 15);
    }

    #[test]
    fn test_default_constants() {
        assert_eq!(DEFAULT_SOFT_LIMIT_MB, 30);
        assert_eq!(DEFAULT_HARD_LIMIT_MB, 50);
        assert_eq!(DEFAULT_EVICTION_THRESHOLD_PCT, 90.0);
    }
}
