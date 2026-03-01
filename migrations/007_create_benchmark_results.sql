-- Create benchmark results table for performance tracking
CREATE TABLE IF NOT EXISTS benchmark_results (
    id BIGSERIAL PRIMARY KEY,
    run_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Query performance metrics (in milliseconds)
    wedding_query_ms BIGINT NOT NULL,
    voc_query_ms BIGINT NOT NULL,
    aspect_query_ms BIGINT NOT NULL,
    
    -- Speedup metrics
    wedding_speedup DECIMAL(8,2) NOT NULL,
    speedup_target_met BOOLEAN NOT NULL,
    
    -- Memory metrics (in MB)
    memory_used_mb BIGINT NOT NULL,
    memory_pressure VARCHAR(20) NOT NULL,
    
    -- Regression detection
    baseline_wedding_ms BIGINT,
    degradation_pct DECIMAL(5,2),
    alert_level VARCHAR(10), -- 'NONE', 'WARN', 'ERROR'
    
    -- Additional metadata
    cache_hit_rate DECIMAL(5,4),
    chunks_cached INTEGER
);

-- Create hypertable for time-series data
SELECT create_hypertable('benchmark_results', 'run_at', if_not_exists => TRUE);

-- Index for querying recent results
CREATE INDEX IF NOT EXISTS idx_benchmark_results_run_at 
ON benchmark_results (run_at DESC);

-- Index for finding regressions
CREATE INDEX IF NOT EXISTS idx_benchmark_results_alert 
ON benchmark_results (alert_level, run_at DESC) 
WHERE alert_level != 'NONE';
