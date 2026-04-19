-- Composite index for cursor-based pagination using (created_at, id) tuple comparison
CREATE INDEX idx_jobs_cursor_pagination ON jobs(created_at DESC, id DESC);
