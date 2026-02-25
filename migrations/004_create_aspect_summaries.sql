-- Migration 004: Create aspect_summaries table for pre-aggregated aspect counts
-- Requirement: DB-01

-- ============================================================================
-- ASPECT_SUMMARIES TABLE
-- ============================================================================
-- Pre-aggregated aspect counts per body per time bucket
-- Eliminates correlated subqueries in wedding/electional queries
-- 
-- Performance impact (from implementation_plan.md):
--   Before: 2.3s for 60-day range with correlated subqueries
--   After: 45ms for 60-day range with aspect_summaries JOIN
--   Improvement: 51× faster

CREATE TABLE aspect_summaries (
    time TIMESTAMPTZ NOT NULL,
    body_id SMALLINT NOT NULL CHECK (body_id >= 0 AND body_id <= 9),
    
    -- Individual aspect type counts
    conjunctions SMALLINT DEFAULT 0 CHECK (conjunctions >= 0),
    sextiles SMALLINT DEFAULT 0 CHECK (sextiles >= 0),
    squares SMALLINT DEFAULT 0 CHECK (squares >= 0),
    trines SMALLINT DEFAULT 0 CHECK (trines >= 0),
    oppositions SMALLINT DEFAULT 0 CHECK (oppositions >= 0),
    
    -- Aggregated counts for common query patterns
    total_favorable SMALLINT DEFAULT 0 CHECK (total_favorable >= 0), -- trines + sextiles
    total_challenging SMALLINT DEFAULT 0 CHECK (total_challenging >= 0), -- squares + oppositions
    
    PRIMARY KEY (time, body_id)
);

COMMENT ON TABLE aspect_summaries IS 'Pre-aggregated aspect counts per body per time - populated by Phase 3 Query System';
COMMENT ON COLUMN aspect_summaries.conjunctions IS 'Number of conjunction aspects (0° ± orb) for this body at this time';
COMMENT ON COLUMN aspect_summaries.sextiles IS 'Number of sextile aspects (60° ± orb) for this body at this time';
COMMENT ON COLUMN aspect_summaries.squares IS 'Number of square aspects (90° ± orb) for this body at this time';
COMMENT ON COLUMN aspect_summaries.trines IS 'Number of trine aspects (120° ± orb) for this body at this time';
COMMENT ON COLUMN aspect_summaries.oppositions IS 'Number of opposition aspects (180° ± orb) for this body at this time';
COMMENT ON COLUMN aspect_summaries.total_favorable IS 'Sum of trines and sextiles - "good" aspects';
COMMENT ON COLUMN aspect_summaries.total_challenging IS 'Sum of squares and oppositions - "difficult" aspects';

-- ============================================================================
-- INDEXES
-- ============================================================================

-- Primary access pattern: "Get aspect counts for body X at time Y"
CREATE INDEX idx_aspect_summaries_time_body 
ON aspect_summaries (time, body_id);

-- Index for favorable aspect queries (used in wedding queries)
CREATE INDEX idx_aspect_summaries_favorable 
ON aspect_summaries (time, body_id, total_favorable) 
WHERE total_favorable > 0;

-- Index for challenging aspect queries
CREATE INDEX idx_aspect_summaries_challenging 
ON aspect_summaries (time, body_id, total_challenging) 
WHERE total_challenging > 0;

-- ============================================================================
-- POPULATION NOTES
-- ============================================================================
-- This table is created EMPTY. Population logic will be implemented in Phase 3
-- (Query System) when the query functions are built.
--
-- The table structure supports the optimized query pattern from
-- implementation_plan.md:
--
--   SELECT 
--       pp.time,
--       pp.zodiac_sign as moon_sign,
--       asum.total_favorable as favorable_aspects
--   FROM planet_positions pp
--   LEFT JOIN aspect_summaries asum 
--       ON pp.time = asum.time 
--       AND asum.body_id = 3 -- Venus
--   WHERE pp.body_id = 1 -- Moon
--     AND pp.time BETWEEN '2024-06-01' AND '2024-08-01'
--     AND pp.zodiac_sign IN (1, 3, 4, 6, 7, 9, 11, 12)
--     AND NOT EXISTS (
--         SELECT 1 FROM lunar_conditions lc 
--         WHERE lc.time = pp.time AND lc.is_void_of_course = true
--     )
--   ORDER BY asum.total_favorable DESC NULLS LAST, pp.time;
--
-- Expected population query (to be implemented in Phase 3):
--   INSERT INTO aspect_summaries (time, body_id, conjunctions, sextiles, squares, trines, oppositions)
--   SELECT 
--       time,
--       body1_id as body_id,
--       COUNT(*) FILTER (WHERE aspect_type = 0) as conjunctions,
--       COUNT(*) FILTER (WHERE aspect_type = 1) as sextiles,
--       COUNT(*) FILTER (WHERE aspect_type = 2) as squares,
--       COUNT(*) FILTER (WHERE aspect_type = 3) as trines,
--       COUNT(*) FILTER (WHERE aspect_type = 4) as oppositions
--   FROM aspects
--   WHERE orb <= 10.0 -- Configurable orb threshold
--   GROUP BY time, body1_id;
--
-- Note: This is a simplified example. Actual population will handle both
-- body1_id and body2_id (aspects are bidirectional).

-- ============================================================================
-- STORAGE ESTIMATE
-- ============================================================================
-- Row size: ~30 bytes (time: 8, body_id: 2, 7 counts: 14, padding: 6)
-- Per day: 1,440 minutes × 10 bodies = 14,400 rows
-- Per day storage: 14,400 × 30 bytes = 432 KB
-- Per year storage: 432 KB × 365 = ~158 MB
--
-- Compare to storing raw aspects:
--   Raw aspects: ~50M rows/year × 40 bytes = ~2 GB/year
--   Aspect summaries: ~5.3M rows/year × 30 bytes = ~158 MB/year
--   Savings: 92% reduction in aspect-related storage
