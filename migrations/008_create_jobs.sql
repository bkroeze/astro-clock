-- Migration 008: Create jobs table for job queue
-- Requirements: JOB-01, JOB-04, JOB-05

CREATE TABLE IF NOT EXISTS jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_type TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('pending', 'in_process', 'complete', 'failed')),
    payload JSONB NOT NULL DEFAULT '{}',
    result JSONB,
    error JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    worker_id TEXT  -- Optional: identifies which worker claimed the job
);

-- Indexes for common query patterns
CREATE INDEX idx_jobs_status_created ON jobs(status, created_at);
CREATE INDEX idx_jobs_status_type ON jobs(status, job_type);
CREATE INDEX idx_jobs_created_at ON jobs(created_at DESC);

-- Comments for documentation
COMMENT ON TABLE jobs IS 'Job queue for tracking async operations';
COMMENT ON COLUMN jobs.status IS 'pending → in_process → (complete | failed)';
COMMENT ON COLUMN jobs.payload IS 'Job parameters as JSON';
COMMENT ON COLUMN jobs.result IS 'Job result data as JSON (null until complete)';
COMMENT ON COLUMN jobs.error IS 'Error details as JSON if job failed';
