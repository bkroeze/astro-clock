-- Migration 001: Create core TimescaleDB hypertables for astrological time-series data
-- Requirements: DB-04, DB-05, DB-06, DB-03

-- Enable TimescaleDB extension (if not already enabled)
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- ============================================================================
-- 1. PLANET_POSITIONS HYPERTABLE
-- ============================================================================
-- Stores 1-minute resolution positions for all celestial bodies
-- Body IDs: 0=Sun, 1=Moon, 2=Mercury, 3=Venus, 4=Mars, 5=Jupiter, 6=Saturn, 7=Uranus, 8=Neptune, 9=Pluto

CREATE TABLE planet_positions (
    time TIMESTAMPTZ NOT NULL,
    body_id SMALLINT NOT NULL CHECK (body_id >= 0 AND body_id <= 9),
    longitude DECIMAL(8,4) NOT NULL CHECK (longitude >= 0 AND longitude < 360),
    latitude DECIMAL(8,4),
    distance DECIMAL(10,6), -- Astronomical Units (AU)
    speed_lon DECIMAL(9,5), -- Daily motion in degrees
    retrograde BOOLEAN DEFAULT FALSE,
    zodiac_sign SMALLINT CHECK (zodiac_sign >= 0 AND zodiac_sign <= 11),
    
    PRIMARY KEY (time, body_id)
);

-- Convert to hypertable with 1-day chunks
SELECT create_hypertable('planet_positions', 'time', 
    chunk_time_interval => INTERVAL '1 day',
    if_not_exists => TRUE
);

-- Add comment for documentation
COMMENT ON TABLE planet_positions IS '1-minute resolution positions for Sun, Moon, and planets (body_id 0-9)';
COMMENT ON COLUMN planet_positions.body_id IS '0=Sun, 1=Moon, 2=Mercury, 3=Venus, 4=Mars, 5=Jupiter, 6=Saturn, 7=Uranus, 8=Neptune, 9=Pluto';
COMMENT ON COLUMN planet_positions.zodiac_sign IS '0=Aries, 1=Taurus, 2=Gemini, 3=Cancer, 4=Leo, 5=Virgo, 6=Libra, 7=Scorpio, 8=Sagittarius, 9=Capricorn, 10=Aquarius, 11=Pisces';

-- ============================================================================
-- 2. ASPECTS HYPERTABLE
-- ============================================================================
-- Stores pre-calculated astrological aspects between bodies
-- Aspect types: 0=Conjunction, 1=Sextile, 2=Square, 3=Trine, 4=Opposition

CREATE TABLE aspects (
    time TIMESTAMPTZ NOT NULL,
    body1_id SMALLINT NOT NULL CHECK (body1_id >= 0 AND body1_id <= 9),
    body2_id SMALLINT NOT NULL CHECK (body2_id >= 0 AND body2_id <= 9),
    aspect_type SMALLINT NOT NULL CHECK (aspect_type >= 0 AND aspect_type <= 4),
    orb DECIMAL(5,2) NOT NULL CHECK (orb >= 0 AND orb <= 180),
    applying BOOLEAN DEFAULT TRUE, -- TRUE if aspect is getting tighter
    
    PRIMARY KEY (time, body1_id, body2_id, aspect_type),
    CONSTRAINT body_ordering CHECK (body1_id < body2_id) -- Prevent duplicates (A-B same as B-A)
);

-- Convert to hypertable with 1-day chunks
SELECT create_hypertable('aspects', 'time', 
    chunk_time_interval => INTERVAL '1 day',
    if_not_exists => TRUE
);

COMMENT ON TABLE aspects IS 'Pre-calculated astrological aspects between celestial bodies';
COMMENT ON COLUMN aspects.aspect_type IS '0=Conjunction (0°), 1=Sextile (60°), 2=Square (90°), 3=Trine (120°), 4=Opposition (180°)';
COMMENT ON COLUMN aspects.orb IS 'Degrees from exact aspect (typically 0-10°)';
COMMENT ON COLUMN aspects.applying IS 'TRUE if the aspect orb is decreasing (getting tighter)';

-- ============================================================================
-- 3. LUNAR_CONDITIONS HYPERTABLE
-- ============================================================================
-- Stores Moon phases and void-of-course periods
-- Moon phases: 0=New, 1=Waxing Crescent, 2=First Quarter, 3=Waxing Gibbous,
--              4=Full, 5=Waning Gibbous, 6=Last Quarter, 7=Waning Crescent

CREATE TABLE lunar_conditions (
    time TIMESTAMPTZ NOT NULL,
    moon_phase SMALLINT CHECK (moon_phase >= 0 AND moon_phase <= 7),
    moon_sign SMALLINT CHECK (moon_sign >= 0 AND moon_sign <= 11),
    moon_phase_angle DECIMAL(7,3) CHECK (moon_phase_angle >= 0 AND moon_phase_angle < 360),
    moon_illumination DECIMAL(5,4) CHECK (moon_illumination >= 0 AND moon_illumination <= 1),
    is_void_of_course BOOLEAN DEFAULT FALSE,
    voc_start TIMESTAMPTZ, -- NULL if not VoC
    voc_end TIMESTAMPTZ,   -- NULL if not VoC
    
    PRIMARY KEY (time)
);

-- Convert to hypertable with 1-day chunks
SELECT create_hypertable('lunar_conditions', 'time', 
    chunk_time_interval => INTERVAL '1 day',
    if_not_exists => TRUE
);

COMMENT ON TABLE lunar_conditions IS 'Moon phases, illumination, and void-of-course periods';
COMMENT ON COLUMN lunar_conditions.moon_phase IS '0=New, 1=Waxing Crescent, 2=First Quarter, 3=Waxing Gibbous, 4=Full, 5=Waning Gibbous, 6=Last Quarter, 7=Waning Crescent';
COMMENT ON COLUMN lunar_conditions.is_void_of_course IS 'TRUE when Moon makes no major aspects before leaving its current sign';

-- ============================================================================
-- 4. HOUSE_CUSPS HYPERTABLE
-- ============================================================================
-- Stores calculated house cusps for specific locations and times
-- House systems: 0=Placidus, 1=Koch, 2=Equal, 3=Whole Sign, 4=Porphyry,
--                5=Regiomontanus, 6=Campanus, 7=Morinus, 8=Topocentric,
--                9=Alcabitius, 10=Azimuthal

CREATE TABLE house_cusps (
    time TIMESTAMPTZ NOT NULL,
    location_id INTEGER, -- Will be FK to locations table (migration 003)
    house_system SMALLINT NOT NULL CHECK (house_system >= 0 AND house_system <= 10),
    cusp_1 DECIMAL(8,4) CHECK (cusp_1 >= 0 AND cusp_1 < 360),
    cusp_2 DECIMAL(8,4) CHECK (cusp_2 >= 0 AND cusp_2 < 360),
    cusp_3 DECIMAL(8,4) CHECK (cusp_3 >= 0 AND cusp_3 < 360),
    cusp_4 DECIMAL(8,4) CHECK (cusp_4 >= 0 AND cusp_4 < 360),
    cusp_5 DECIMAL(8,4) CHECK (cusp_5 >= 0 AND cusp_5 < 360),
    cusp_6 DECIMAL(8,4) CHECK (cusp_6 >= 0 AND cusp_6 < 360),
    cusp_7 DECIMAL(8,4) CHECK (cusp_7 >= 0 AND cusp_7 < 360),
    cusp_8 DECIMAL(8,4) CHECK (cusp_8 >= 0 AND cusp_8 < 360),
    cusp_9 DECIMAL(8,4) CHECK (cusp_9 >= 0 AND cusp_9 < 360),
    cusp_10 DECIMAL(8,4) CHECK (cusp_10 >= 0 AND cusp_10 < 360),
    cusp_11 DECIMAL(8,4) CHECK (cusp_11 >= 0 AND cusp_11 < 360),
    cusp_12 DECIMAL(8,4) CHECK (cusp_12 >= 0 AND cusp_12 < 360),
    ascendant DECIMAL(8,4) CHECK (ascendant >= 0 AND ascendant < 360),
    mc DECIMAL(8,4) CHECK (mc >= 0 AND mc < 360), -- Midheaven
    
    PRIMARY KEY (time, location_id, house_system)
);

-- Convert to hypertable with 1-day chunks
SELECT create_hypertable('house_cusps', 'time', 
    chunk_time_interval => INTERVAL '1 day',
    if_not_exists => TRUE
);

COMMENT ON TABLE house_cusps IS 'Calculated house cusps for specific locations and times';
COMMENT ON COLUMN house_cusps.house_system IS '0=Placidus, 1=Koch, 2=Equal, 3=Whole Sign, 4=Porphyry, 5=Regiomontanus, 6=Campanus, 7=Morinus, 8=Topocentric, 9=Alcabitius, 10=Azimuthal';
COMMENT ON COLUMN house_cusps.location_id IS 'References locations(id) - FK added in migration 003';

-- ============================================================================
-- Migration complete
-- ============================================================================
