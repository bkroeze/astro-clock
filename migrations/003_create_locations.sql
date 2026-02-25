-- Migration 003: Create normalized locations table and refactor house_cusps
-- Requirement: DB-03

-- ============================================================================
-- LOCATIONS TABLE
-- ============================================================================
-- Normalized location data to reduce storage by ~60% for repeated locations
-- Location IDs are referenced by house_cusps instead of storing lat/lon per row

CREATE TABLE locations (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100), -- Optional location name (e.g., "New York, NY")
    latitude DECIMAL(8,4) NOT NULL CHECK (latitude >= -90 AND latitude <= 90),
    longitude DECIMAL(8,4) NOT NULL CHECK (longitude >= -180 AND longitude <= 180),
    timezone VARCHAR(50), -- IANA timezone name (e.g., "America/New_York")
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE (latitude, longitude)
);

COMMENT ON TABLE locations IS 'Normalized geographic locations for house cusp calculations';
COMMENT ON COLUMN locations.latitude IS 'Decimal degrees, -90 to 90';
COMMENT ON COLUMN locations.longitude IS 'Decimal degrees, -180 to 180';
COMMENT ON COLUMN locations.timezone IS 'IANA timezone identifier (e.g., America/New_York)';

-- Index for coordinate lookups
CREATE INDEX idx_locations_coordinates 
ON locations (latitude, longitude);

-- ============================================================================
-- HOUSE_CUSPS FOREIGN KEY CONSTRAINT
-- ============================================================================
-- Add foreign key constraint to link house_cusps to locations
-- Note: This assumes house_cusps was created in migration 001

-- First, ensure all existing location_id values reference valid locations
-- or set them to NULL temporarily
-- (In a fresh database, there will be no existing data)

-- Add the foreign key constraint
ALTER TABLE house_cusps 
ADD CONSTRAINT fk_house_cusps_location 
FOREIGN KEY (location_id) 
REFERENCES locations(id) 
ON DELETE CASCADE;

-- Add comment documenting the relationship
COMMENT ON COLUMN house_cusps.location_id IS 'Foreign key to locations(id) - normalized location data';

-- ============================================================================
-- STORAGE OPTIMIZATION NOTES
-- ============================================================================
-- Before normalization (storing lat/lon in every house_cusps row):
--   - 8 bytes per coordinate (DOUBLE PRECISION) × 2 = 16 bytes per row
--   - 1,440 rows/day × 16 bytes = 23KB/day per location
--   - 100 locations × 23KB = 2.3MB/day
--
-- After normalization (using location_id):
--   - 4 bytes per location_id (INTEGER)
--   - locations table: ~100 rows × 50 bytes = 5KB total
--   - 1,440 rows/day × 4 bytes = 5.8KB/day per location
--   - 100 locations × 5.8KB + 5KB = 585KB/day
--
-- Savings: ~75% reduction in storage for house cusps data
--
-- Additional benefits:
--   - Faster JOINs on integer location_id vs composite lat/lon
--   - Easier location management (update name/timezone in one place)
--   - Enforced data integrity via foreign key constraint
--   - Simpler queries (JOIN locations vs passing lat/lon everywhere)

-- ============================================================================
-- EXAMPLE USAGE
-- ============================================================================
-- Insert a location:
--   INSERT INTO locations (name, latitude, longitude, timezone)
--   VALUES ('New York, NY', 40.7128, -74.0060, 'America/New_York')
--   RETURNING id; -- Returns location_id for use in house_cusps
--
-- Query house cusps with location info:
--   SELECT hc.*, l.name, l.latitude, l.longitude
--   FROM house_cusps hc
--   JOIN locations l ON hc.location_id = l.id
--   WHERE hc.time BETWEEN '2024-01-01' AND '2024-01-02'
--   AND l.latitude = 40.7128 AND l.longitude = -74.0060;
