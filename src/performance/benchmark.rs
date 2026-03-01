//! Automated benchmark runner with regression detection and alerting

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use tracing::{error, info, warn};

use crate::database::pool::DatabasePool;
use crate::performance::{MemoryMonitor, MemoryPressure};
use crate::queries::benchmark::run_query_benchmarks;

/// Configuration for benchmark runs
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Baseline wedding query time in ms (2.3s = 2300ms)
    pub wedding_baseline_ms: u64,
    /// Target speedup multiplier (51×)
    pub speedup_target: f64,
    /// Degradation warning threshold (%)
    pub degradation_warn_pct: f64,
    /// Degradation error threshold (%)
    pub degradation_error_pct: f64,
    /// Target query time in ms (45ms for 51× speedup)
    pub target_query_ms: u64,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            wedding_baseline_ms: 2300,
            speedup_target: 51.0,
            degradation_warn_pct: 20.0,
            degradation_error_pct: 50.0,
            target_query_ms: 45,
        }
    }
}

/// Alert levels for performance degradation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertLevel {
    /// No degradation detected
    None,
    /// Minor degradation (≥20%)
    Warn,
    /// Severe degradation (≥50%)
    Error,
}

impl AlertLevel {
    /// Convert to string representation for database storage
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertLevel::None => "NONE",
            AlertLevel::Warn => "WARN",
            AlertLevel::Error => "ERROR",
        }
    }
}

/// Results from a single benchmark run
#[derive(Debug)]
pub struct BenchmarkRun {
    /// When the benchmark was run
    pub run_at: DateTime<Utc>,
    /// Wedding query execution time in milliseconds
    pub wedding_query_ms: u64,
    /// VoC query execution time in milliseconds
    pub voc_query_ms: u64,
    /// Aspect query execution time in milliseconds
    pub aspect_query_ms: u64,
    /// Calculated speedup vs baseline
    pub wedding_speedup: f64,
    /// Whether the 51× speedup target was met
    pub speedup_target_met: bool,
    /// Memory usage at time of benchmark (MB)
    pub memory_used_mb: usize,
    /// Memory pressure level at time of benchmark
    pub memory_pressure: MemoryPressure,
    /// Degradation percentage if above target
    pub degradation_pct: Option<f64>,
    /// Alert level based on degradation
    pub alert_level: AlertLevel,
}

/// Automated benchmark runner with regression detection
pub struct BenchmarkRunner {
    pool: DatabasePool,
    config: BenchmarkConfig,
    memory_monitor: MemoryMonitor,
}

impl BenchmarkRunner {
    /// Create a new benchmark runner
    pub fn new(pool: DatabasePool, config: BenchmarkConfig) -> Self {
        let memory_monitor = MemoryMonitor::new(30, 50, 90.0);
        Self {
            pool,
            config,
            memory_monitor,
        }
    }

    /// Run benchmarks and store results
    pub async fn run(&mut self) -> Result<BenchmarkRun, sqlx::Error> {
        let run_at = Utc::now();

        // Get memory stats before running
        let memory_used_mb = self.memory_monitor.current_memory_mb();
        let memory_pressure = self.memory_monitor.check_memory_pressure();

        // Run query benchmarks
        let results = run_query_benchmarks(&self.pool).await;

        // Calculate speedup
        let wedding_speedup =
            self.config.wedding_baseline_ms as f64 / results.wedding_query_60day_ms as f64;
        let speedup_target_met = wedding_speedup >= self.config.speedup_target;

        // Calculate degradation
        let (degradation_pct, alert_level) =
            if results.wedding_query_60day_ms > self.config.target_query_ms {
                let pct = ((results.wedding_query_60day_ms - self.config.target_query_ms) as f64
                    / self.config.target_query_ms as f64)
                    * 100.0;
                let level = if pct >= self.config.degradation_error_pct {
                    AlertLevel::Error
                } else if pct >= self.config.degradation_warn_pct {
                    AlertLevel::Warn
                } else {
                    AlertLevel::None
                };
                (Some(pct), level)
            } else {
                (None, AlertLevel::None)
            };

        let run = BenchmarkRun {
            run_at,
            wedding_query_ms: results.wedding_query_60day_ms,
            voc_query_ms: results.voc_query_60day_ms,
            aspect_query_ms: results.aspect_query_60day_ms,
            wedding_speedup,
            speedup_target_met,
            memory_used_mb,
            memory_pressure,
            degradation_pct,
            alert_level,
        };

        // Store results
        self.store_results(&run).await?;

        // Log alerts
        match alert_level {
            AlertLevel::Error => {
                error!(
                    "PERFORMANCE REGRESSION: Wedding query {}ms (target: {}ms), degradation: {:.1}%",
                    run.wedding_query_ms,
                    self.config.target_query_ms,
                    degradation_pct.unwrap_or(0.0)
                );
            }
            AlertLevel::Warn => {
                warn!(
                    "Performance degradation: Wedding query {}ms (target: {}ms), degradation: {:.1}%",
                    run.wedding_query_ms,
                    self.config.target_query_ms,
                    degradation_pct.unwrap_or(0.0)
                );
            }
            AlertLevel::None => {
                info!(
                    "Benchmark complete: {:.1}× speedup ({}ms), memory: {}MB {:?}",
                    run.wedding_speedup, run.wedding_query_ms, run.memory_used_mb, run.memory_pressure
                );
            }
        }

        Ok(run)
    }

    /// Store benchmark results in database
    async fn store_results(&self, run: &BenchmarkRun) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO benchmark_results (
                run_at, wedding_query_ms, voc_query_ms, aspect_query_ms,
                wedding_speedup, speedup_target_met, memory_used_mb, memory_pressure,
                baseline_wedding_ms, degradation_pct, alert_level
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(run.run_at)
        .bind(run.wedding_query_ms as i64)
        .bind(run.voc_query_ms as i64)
        .bind(run.aspect_query_ms as i64)
        .bind(Decimal::from_f64_retain(run.wedding_speedup).unwrap_or_default())
        .bind(run.speedup_target_met)
        .bind(run.memory_used_mb as i64)
        .bind(format!("{:?}", run.memory_pressure))
        .bind(self.config.wedding_baseline_ms as i64)
        .bind(run.degradation_pct.map(|d| Decimal::from_f64_retain(d).unwrap_or_default()))
        .bind(run.alert_level.as_str())
        .execute(self.pool.pool())
        .await?;

        Ok(())
    }
}

/// Run benchmarks with default configuration
pub async fn run_scheduled_benchmarks(pool: &DatabasePool) -> Result<BenchmarkRun, sqlx::Error> {
    let mut runner = BenchmarkRunner::new(pool.clone(), BenchmarkConfig::default());
    runner.run().await
}

/// Check for performance regression against baseline
pub async fn check_performance_regression(
    pool: &DatabasePool,
    baseline_ms: u64,
) -> Result<Option<(f64, AlertLevel)>, sqlx::Error> {
    let mut runner = BenchmarkRunner::new(pool.clone(), BenchmarkConfig::default());
    let run = runner.run().await?;

    let speedup = baseline_ms as f64 / run.wedding_query_ms as f64;
    let target_speedup = 51.0;

    if speedup < target_speedup {
        let degradation_pct = ((target_speedup - speedup) / target_speedup) * 100.0;
        let level = if degradation_pct >= 50.0 {
            AlertLevel::Error
        } else if degradation_pct >= 20.0 {
            AlertLevel::Warn
        } else {
            AlertLevel::None
        };
        Ok(Some((degradation_pct, level)))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_config_default() {
        let config = BenchmarkConfig::default();
        assert_eq!(config.wedding_baseline_ms, 2300);
        assert_eq!(config.speedup_target, 51.0);
        assert_eq!(config.degradation_warn_pct, 20.0);
        assert_eq!(config.degradation_error_pct, 50.0);
        assert_eq!(config.target_query_ms, 45);
    }

    #[test]
    fn test_alert_level_as_str() {
        assert_eq!(AlertLevel::None.as_str(), "NONE");
        assert_eq!(AlertLevel::Warn.as_str(), "WARN");
        assert_eq!(AlertLevel::Error.as_str(), "ERROR");
    }
}
