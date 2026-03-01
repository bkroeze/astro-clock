-- Migration 009: Create loaded_days tracking table
-- Requirements: JOB-02

CREATE TABLE IF NOT EXISTS loaded_days (
    date DATE PRIMARY KEY,
    coverage_minutes SMALLINT NOT NULL DEFAULT 0 CHECK (coverage_minutes >= 0 AND coverage_minutes <= 1440),
    loaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    job_id UUID,
    
    CONSTRAINT fk_loaded_days_job
        FOREIGN KEY (job_id) 
        REFERENCES jobs(id) 
        ON DELETE SET NULL
);

-- Indexes for date range queries
CREATE INDEX idx_loaded_days_date ON loaded_days(date);
CREATE INDEX idx_loaded_days_loaded_at ON loaded_days(loaded_at DESC);

-- Comments
COMMENT ON TABLE loaded_days IS 'Tracks which dates have planetary data loaded';
COMMENT ON COLUMN loaded_days.date IS 'Calendar date (YYYY-MM-DD)';
COMMENT ON COLUMN loaded_days.coverage_minutes IS 'How many minutes of data loaded (0-1440)';
COMMENT ON COLUMN loaded_days.job_id IS 'Reference to job that loaded this day';
