-- Migration 006: Create TimescaleDB continuous aggregates for multi-resolution storage
-- Requirements: PERF-01, PERF-03
--
-- This migration creates continuous aggregates that downsample planet positions
-- to reduce storage requirements. Outer planets move slowly, so we can store
-- them at lower resolution without losing significant accuracy.
--
-- Resolution strategy:
-- - Moon (body_id=1): 1-minute resolution (fast movement)
-- - Inner planets (Sun=0, Mercury=2, Venus=3, Mars=4): 5-minute resolution
-- - Outer planets (Jupiter=5, Saturn=6, Uranus=7, Neptune=8, Pluto=9): 60-minute resolution

-- ============================================================================
-- 1. CONTINUOUS AGGREGATE: 5-MINUTE RESOLUTION (Inner Planets)
-- ============================================================================
-- Covers Sun (0), Mercury (2), Venus (3), Mars (4)

CREATE MATERIALIZED VIEW IF NOT EXISTS planet_positions_5min
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('5 minutes', time) AS bucket,
    body_id,
    last(longitude, time) AS longitude,
    last(latitude, time) AS latitude,
    last(distance, time) AS distance,
    last(speed_lon, time) AS speed_lon,
    last(retrograde::smallint, time)::boolean AS retrograde,
    last(zodiac_sign, time) AS zodiac_sign
FROM planet_positions
WHERE body_id IN (0, 2, 3, 4)  -- Sun, Mercury, Venus, Mars
GROUP BY bucket, body_id
WITH NO DATA;

-- Add comment for documentation
COMMENT ON MATERIALIZED VIEW planet_positions_5min IS '5-minute resolution positions for inner planets (Sun, Mercury, Venus, Mars). Reduces storage by 5x while maintaining accuracy for fast-moving bodies.';

-- ============================================================================
-- 2. CONTINUOUS AGGREGATE: 60-MINUTE RESOLUTION (Outer Planets)
-- ============================================================================
-- Covers Jupiter (5), Saturn (6), Uranus (7), Neptune (8), Pluto (9)

CREATE MATERIALIZED VIEW IF NOT EXISTS planet_positions_60min
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('60 minutes', time) AS bucket,
    body_id,
    last(longitude, time) AS longitude,
    last(latitude, time) AS latitude,
    last(distance, time) AS distance,
    last(speed_lon, time) AS speed_lon,
    last(retrograde::smallint, time)::boolean AS retrograde,
    last(zodiac_sign, time) AS zodiac_sign
FROM planet_positions
WHERE body_id IN (5, 6, 7, 8, 9)  -- Jupiter, Saturn, Uranus, Neptune, Pluto
GROUP BY bucket, body_id
WITH NO DATA;

-- Add comment for documentation
COMMENT ON MATERIALIZED VIEW planet_positions_60min IS '60-minute resolution positions for outer planets (Jupiter, Saturn, Uranus, Neptune, Pluto). Reduces storage by 60x for slowly-moving bodies.';

-- ============================================================================
-- 3. REFRESH POLICIES
-- ============================================================================
-- Automatic refresh policies keep aggregates up-to-date without manual intervention

-- 5-minute aggregate: Refresh last day of data every hour
-- (Inner planets change rapidly, so we refresh more frequently)
SELECT add_continuous_aggregate_policy('planet_positions_5min',
    start_offset => INTERVAL '1 day',
    end_offset => INTERVAL '1 hour',
    schedule_interval => INTERVAL '1 hour',
    if_not_exists => TRUE
);

-- 60-minute aggregate: Refresh last 7 days of data every hour
-- (Outer planets move slowly, so less frequent refresh is acceptable)
SELECT add_continuous_aggregate_policy('planet_positions_60min',
    start_offset => INTERVAL '7 days',
    end_offset => INTERVAL '1 hour',
    schedule_interval => INTERVAL '1 hour',
    if_not_exists => TRUE
);

-- ============================================================================
-- Migration complete
-- ============================================================================
