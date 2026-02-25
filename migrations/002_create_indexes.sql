-- Migration 002: Create composite and partial indexes for query optimization
-- Requirement: DB-02

-- ============================================================================
-- PLANET_POSITIONS INDEXES
-- ============================================================================

-- Composite index for time-range queries by body
-- Used when querying specific bodies over time ranges
CREATE INDEX idx_planet_positions_time_body 
ON planet_positions (time, body_id);

-- Partial index for Moon queries (body_id = 1)
-- Moon is queried most frequently for lunar conditions and wedding queries
CREATE INDEX idx_planet_positions_moon 
ON planet_positions (time) 
WHERE body_id = 1;

-- Composite index for Moon sign queries
-- Supports queries like: "Find all times Moon is in Cancer"
CREATE INDEX idx_planet_positions_moon_sign 
ON planet_positions (body_id, zodiac_sign, time) 
WHERE body_id = 1;

-- Index for retrograde lookups
CREATE INDEX idx_planet_positions_retrograde 
ON planet_positions (body_id, time) 
WHERE retrograde = TRUE;

-- ============================================================================
-- ASPECTS INDEXES
-- ============================================================================

-- Composite index for time-range aspect queries
-- Supports: "Get all aspects for this time range"
CREATE INDEX idx_aspects_time_bodies 
ON aspects (time, body1_id, body2_id);

-- Composite index for filtered aspect queries by type and orb
-- Supports: "Find conjunctions with small orbs"
CREATE INDEX idx_aspects_time_type_orb 
ON aspects (time, aspect_type, orb);

-- Partial index for major aspects only (orb <= 5 degrees)
-- Filters out loose aspects for cleaner result sets
CREATE INDEX idx_aspects_tight_orbs 
ON aspects (time, body1_id, body2_id, aspect_type) 
WHERE orb <= 5.0;

-- ============================================================================
-- LUNAR_CONDITIONS INDEXES
-- ============================================================================

-- Partial index for Void-of-Course queries
-- Critical for wedding/electional astrology queries
CREATE INDEX idx_lunar_voc 
ON lunar_conditions (time, is_void_of_course) 
WHERE is_void_of_course = TRUE;

-- Covering index for VoC checks with Moon sign
-- Includes additional columns to avoid table lookups
CREATE INDEX idx_lunar_voc_covering 
ON lunar_conditions (time, is_void_of_course, moon_sign) 
INCLUDE (moon_phase_angle, moon_illumination);

-- Index for moon phase queries
CREATE INDEX idx_lunar_moon_phase 
ON lunar_conditions (moon_phase, time);

-- Index for moon sign queries (used in wedding queries)
CREATE INDEX idx_lunar_moon_sign 
ON lunar_conditions (moon_sign, time);

-- ============================================================================
-- HOUSE_CUSPS INDEXES
-- ============================================================================

-- Composite index for location-based queries
-- Primary access pattern: "Get house cusps for location X at time Y"
CREATE INDEX idx_house_cusps_location_time 
ON house_cusps (location_id, time);

-- Index for ascendant queries (commonly searched)
CREATE INDEX idx_house_cusps_ascendant 
ON house_cusps (ascendant, time);

-- ============================================================================
-- INDEX SUMMARY
-- ============================================================================
-- Total indexes created: 12
-- 
-- Planet positions: 4 indexes
--   - Time + body (general queries)
--   - Partial: Moon only (frequent Moon queries)
--   - Partial: Moon + sign (sign-based queries)
--   - Partial: Retrograde periods
--
-- Aspects: 3 indexes  
--   - Time + bodies (range queries)
--   - Time + type + orb (filtered queries)
--   - Partial: Tight orbs only (major aspects)
--
-- Lunar conditions: 4 indexes
--   - Partial: VoC only (critical for electional)
--   - Covering: VoC with included columns
--   - Moon phase queries
--   - Moon sign queries
--
-- House cusps: 2 indexes
--   - Location + time (primary access)
--   - Ascendant (chart lookups)
--
-- These indexes support the optimized wedding query pattern:
--   SELECT ... FROM planet_positions pp
--   LEFT JOIN aspect_summaries asum ON pp.time = asum.time
--   WHERE pp.body_id = 1 AND pp.zodiac_sign IN (...)
--   AND NOT EXISTS (SELECT 1 FROM lunar_conditions WHERE ...)
