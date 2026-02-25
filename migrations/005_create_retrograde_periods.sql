-- Migration 005: Create retrograde_periods table for planetary retrograde tracking
-- Requirement: QUERY-03

CREATE TABLE retrograde_periods (
    id SERIAL PRIMARY KEY,
    body_id SMALLINT NOT NULL CHECK (body_id >= 0 AND body_id <= 9),
    retrograde_start TIMESTAMPTZ NOT NULL,
    retrograde_end TIMESTAMPTZ NOT NULL,
    pre_shadow_start TIMESTAMPTZ,  -- Optional: shadow period before retrograde
    post_shadow_end TIMESTAMPTZ,   -- Optional: shadow period after retrograde
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    CONSTRAINT retrograde_ordering CHECK (retrograde_start < retrograde_end),
    CONSTRAINT pre_shadow_ordering CHECK (pre_shadow_start IS NULL OR pre_shadow_start <= retrograde_start),
    CONSTRAINT post_shadow_ordering CHECK (post_shadow_end IS NULL OR post_shadow_end >= retrograde_end)
);

-- Index for efficient date range queries
CREATE INDEX idx_retrograde_periods_dates ON retrograde_periods(body_id, retrograde_start, retrograde_end);
CREATE INDEX idx_retrograde_periods_body ON retrograde_periods(body_id);

COMMENT ON TABLE retrograde_periods IS 'Planetary retrograde periods with optional shadow periods';
COMMENT ON COLUMN retrograde_periods.body_id IS '0=Sun, 1=Moon, 2=Mercury, 3=Venus, 4=Mars, 5=Jupiter, 6=Saturn, 7=Uranus, 8=Neptune, 9=Pluto';
COMMENT ON COLUMN retrograde_periods.pre_shadow_start IS 'Start of pre-retrograde shadow period (planet enters retrograde zone)';
COMMENT ON COLUMN retrograde_periods.post_shadow_end IS 'End of post-retrograde shadow period (planet leaves retrograde zone)';
