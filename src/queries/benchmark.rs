use chrono::{Duration, Utc};
use tracing::{info, warn};

use crate::database::pool::DatabasePool;
use crate::queries::types::{AspectCriteria, VoCCriteria, WeddingCriteria};
use crate::queries::{find_exact_aspects, find_voc_periods, find_wedding_dates};

/// Performance benchmark results
#[derive(Debug)]
pub struct BenchmarkResults {
    pub wedding_query_60day_ms: u64,
    pub voc_query_60day_ms: u64,
    pub aspect_query_60day_ms: u64,
    pub all_passed: bool,
}

/// Run query performance benchmarks
///
/// Verifies QUERY-05: All queries complete in <100ms for 60-day ranges
pub async fn run_query_benchmarks(pool: &DatabasePool) -> BenchmarkResults {
    info!("Starting query performance benchmarks...");

    let today = Utc::now().date_naive();
    let start_date = today;
    let end_date = today + Duration::days(60);

    // Benchmark wedding query (QUERY-01 + QUERY-05)
    let wedding_criteria = WeddingCriteria {
        start_date,
        end_date,
        min_venus_aspects: 2,
        limit: 10,
    };

    let wedding_result = match find_wedding_dates(pool, &wedding_criteria).await {
        Ok(result) => {
            info!(
                "Wedding query: {} results in {}ms (target: <100ms)",
                result.data.len(),
                result.execution_time_ms
            );
            result.execution_time_ms
        }
        Err(e) => {
            warn!("Wedding query failed: {}", e);
            u64::MAX
        }
    };

    // Benchmark VoC query (QUERY-02)
    let voc_criteria = VoCCriteria {
        start_date,
        end_date,
        min_duration: None,
    };

    let voc_result = match find_voc_periods(pool, &voc_criteria).await {
        Ok(result) => {
            info!(
                "VoC query: {} results in {}ms (target: <100ms)",
                result.data.len(),
                result.execution_time_ms
            );
            result.execution_time_ms
        }
        Err(e) => {
            warn!("VoC query failed: {}", e);
            u64::MAX
        }
    };

    // Benchmark aspect query (QUERY-04)
    let aspect_criteria = AspectCriteria {
        start_date,
        end_date,
        orb_threshold: rust_decimal::Decimal::from(1),
        aspect_types: None,
        body_pairs: None,
    };

    let aspect_result = match find_exact_aspects(pool, &aspect_criteria).await {
        Ok(result) => {
            info!(
                "Aspect query: {} results in {}ms (target: <100ms)",
                result.data.len(),
                result.execution_time_ms
            );
            result.execution_time_ms
        }
        Err(e) => {
            warn!("Aspect query failed: {}", e);
            u64::MAX
        }
    };

    let target_ms = 100;
    let all_passed = wedding_result < target_ms
        && voc_result < target_ms
        && aspect_result < target_ms;

    let results = BenchmarkResults {
        wedding_query_60day_ms: wedding_result,
        voc_query_60day_ms: voc_result,
        aspect_query_60day_ms: aspect_result,
        all_passed,
    };

    if all_passed {
        info!("All query benchmarks passed! (<100ms for 60-day ranges)");
    } else {
        warn!("Some query benchmarks exceeded 100ms target");
        if wedding_result >= target_ms {
            warn!("  - Wedding query: {}ms", wedding_result);
        }
        if voc_result >= target_ms {
            warn!("  - VoC query: {}ms", voc_result);
        }
        if aspect_result >= target_ms {
            warn!("  - Aspect query: {}ms", aspect_result);
        }
    }

    results
}

/// Quick check if wedding query meets 51× speedup target
///
/// Baseline: 2.3s, Target: 45ms (51× faster)
pub async fn check_wedding_query_speedup(pool: &DatabasePool) -> bool {
    let today = Utc::now().date_naive();
    let criteria = WeddingCriteria {
        start_date: today,
        end_date: today + Duration::days(60),
        min_venus_aspects: 2,
        limit: 10,
    };

    match find_wedding_dates(pool, &criteria).await {
        Ok(result) => {
            let baseline_ms = 2300.0; // 2.3 seconds
            let actual_ms = result.execution_time_ms as f64;
            let speedup = baseline_ms / actual_ms;

            info!(
                "Wedding query speedup: {:.1}× ({}ms vs {}ms baseline)",
                speedup, actual_ms, baseline_ms
            );

            speedup >= 51.0
        }
        Err(e) => {
            warn!("Speedup check failed: {}", e);
            false
        }
    }
}

/// Run a single benchmark for a specific query type
pub async fn benchmark_single_query(
    pool: &DatabasePool,
    query_name: &str,
    days: i64,
) -> Option<u64> {
    let today = Utc::now().date_naive();
    let start_date = today;
    let end_date = today + Duration::days(days);

    match query_name {
        "wedding" => {
            let criteria = WeddingCriteria {
                start_date,
                end_date,
                min_venus_aspects: 2,
                limit: 10,
            };
            find_wedding_dates(pool, &criteria)
                .await
                .ok()
                .map(|r| r.execution_time_ms)
        }
        "voc" => {
            let criteria = VoCCriteria {
                start_date,
                end_date,
                min_duration: None,
            };
            find_voc_periods(pool, &criteria)
                .await
                .ok()
                .map(|r| r.execution_time_ms)
        }
        "aspect" => {
            let criteria = AspectCriteria {
                start_date,
                end_date,
                orb_threshold: rust_decimal::Decimal::from(1),
                aspect_types: None,
                body_pairs: None,
            };
            find_exact_aspects(pool, &criteria)
                .await
                .ok()
                .map(|r| r.execution_time_ms)
        }
        _ => None,
    }
}
