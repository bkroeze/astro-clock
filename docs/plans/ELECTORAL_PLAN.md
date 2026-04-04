## Electoral Astrology: Core Concepts

**Electoral astrology** (also called "electional astrology") is about finding optimal timing for events by analyzing planetary positions, aspects, and configurations . Unlike natal astrology which interprets existing charts, electoral astrology works backward from desired outcomes to find auspicious moments.

### Key Techniques in Electoral Astrology:
1. **Planetary Condition & Placement** - Strength by sign/house, dignity, retrograde status
2. **Moon Phases** - Waxing vs waning, void-of-course periods, lunar aspects
3. **Astrological Houses** - House placements for specific life areas
4. **Aspects** - Conjunctions, oppositions, trines, squares, sextiles between planets
5. **Transits** - How current planetary positions relate to natal charts

---

## Database Schema Design

### 1. Core Time-Series Tables (TimescaleDB Hypertables)

```sql
-- Main ephemeris data - one row per planet per timestamp
CREATE TABLE planet_positions (
    time TIMESTAMPTZ NOT NULL,
    julian_day DOUBLE PRECISION NOT NULL,
    body_id SMALLINT NOT NULL,  -- 0=Sun, 1=Moon, 2=Merc, etc.
    longitude DOUBLE PRECISION NOT NULL,  -- 0-360 degrees
    latitude DOUBLE PRECISION,            -- ecliptic latitude
    distance_au DOUBLE PRECISION,         -- distance from Earth
    speed_long DOUBLE PRECISION,          -- daily motion in longitude
    speed_lat DOUBLE PRECISION,           -- daily motion in latitude
    speed_dist DOUBLE PRECISION,          -- daily motion in distance
    is_retrograde BOOLEAN GENERATED ALWAYS AS (speed_long < 0) STORED,
    
    -- Derived astrological data
    zodiac_sign SMALLINT,  -- 0=Aries, 1=Taurus, etc.
    degree_in_sign DOUBLE PRECISION,
    
    PRIMARY KEY (time, body_id)
);

-- Convert to hypertable for time-series optimization
SELECT create_hypertable('planet_positions', 'time', chunk_time_interval => INTERVAL '1 day');

-- Indexes for common queries
CREATE INDEX idx_planet_positions_body_time ON planet_positions (body_id, time DESC);
CREATE INDEX idx_planet_positions_longitude ON planet_positions (longitude);
```

### 2. House Cusps & Angles Table

```sql
CREATE TABLE house_cusps (
    time TIMESTAMPTZ NOT NULL,
    julian_day DOUBLE PRECISION NOT NULL,
    latitude DOUBLE PRECISION NOT NULL,    -- geographic latitude
    longitude DOUBLE PRECISION NOT NULL,   -- geographic longitude
    house_system CHAR(1) NOT NULL,         -- 'P'=Placidus, 'K'=Koch, etc.
    
    -- House cusps 1-12
    cusp_1 DOUBLE PRECISION, cusp_2 DOUBLE PRECISION, cusp_3 DOUBLE PRECISION,
    cusp_4 DOUBLE PRECISION, cusp_5 DOUBLE PRECISION, cusp_6 DOUBLE PRECISION,
    cusp_7 DOUBLE PRECISION, cusp_8 DOUBLE PRECISION, cusp_9 DOUBLE PRECISION,
    cusp_10 DOUBLE PRECISION, cusp_11 DOUBLE PRECISION, cusp_12 DOUBLE PRECISION,
    
    -- Angles
    ascendant DOUBLE PRECISION,
    mc DOUBLE PRECISION,                   -- Medium Coeli
    armc DOUBLE PRECISION,                 -- ARMC
    vertex DOUBLE PRECISION,
    
    PRIMARY KEY (time, latitude, longitude, house_system)
);

SELECT create_hypertable('house_cusps', 'time', chunk_time_interval => INTERVAL '1 day');
```

### 3. Aspects Table (Pre-calculated for fast queries)

```sql
CREATE TABLE aspects (
    time TIMESTAMPTZ NOT NULL,
    julian_day DOUBLE PRECISION NOT NULL,
    body1_id SMALLINT NOT NULL,
    body2_id SMALLINT NOT NULL,
    aspect_type SMALLINT NOT NULL,  -- 0=conj, 1=opp, 2=trine, 3=square, 4=sextile, etc.
    orb DOUBLE PRECISION NOT NULL,   -- exactness in degrees
    is_applying BOOLEAN,             -- approaching or separating
    exact_time TIMESTAMPTZ,          -- when aspect becomes exact
    
    PRIMARY KEY (time, body1_id, body2_id, aspect_type)
);

SELECT create_hypertable('aspects', 'time', chunk_time_interval => INTERVAL '1 day');
CREATE INDEX idx_aspects_type ON aspects (aspect_type, orb);
```

### 4. Moon Phases & Special Conditions

```sql
CREATE TABLE lunar_conditions (
    time TIMESTAMPTZ NOT NULL,
    julian_day DOUBLE PRECISION NOT NULL,
    
    -- Moon phase data
    moon_phase_angle DOUBLE PRECISION,  -- 0-360, 0=new, 180=full
    moon_phase_name VARCHAR(20),        -- "New Moon", "Waxing Crescent", etc.
    moon_illumination DOUBLE PRECISION, -- 0-1 percentage
    
    -- Void of Course
    is_void_of_course BOOLEAN,
    voc_start_time TIMESTAMPTZ,
    voc_end_time TIMESTAMPTZ,
    voc_duration_minutes INTEGER,
    
    -- Lunar sign and house
    moon_sign SMALLINT,
    moon_house SMALLINT,
    
    PRIMARY KEY (time)
);

SELECT create_hypertable('lunar_conditions', 'time', chunk_time_interval => INTERVAL '1 day');
```

### 5. Retrograde Periods (Summary Table)

```sql
CREATE TABLE retrograde_periods (
    body_id SMALLINT NOT NULL,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    start_longitude DOUBLE PRECISION,
    end_longitude DOUBLE PRECISION,
    shadow_start_time TIMESTAMPTZ,  -- pre-retrograde shadow
    shadow_end_time TIMESTAMPTZ,    -- post-retrograde shadow
    
    PRIMARY KEY (body_id, start_time)
);

CREATE INDEX idx_retrograde_active ON retrograde_periods (body_id, start_time, end_time);
```

### 6. Configuration/Pattern Detection

```sql
CREATE TABLE configurations (
    time TIMESTAMPTZ NOT NULL,
    config_type VARCHAR(50) NOT NULL,  -- "Grand Trine", "T-Square", "Grand Cross", etc.
    planets_involved SMALLINT[],        -- array of body_ids
    orbs DOUBLE PRECISION[],            -- array of orbs for each aspect
    strength_score DOUBLE PRECISION,    -- calculated strength 0-100
    
    PRIMARY KEY (time, config_type)
);

SELECT create_hypertable('configurations', 'time', chunk_time_interval => INTERVAL '1 day');
```

---

## Swiss Ephemeris Data Points to Store

Based on the Swiss Ephemeris documentation , `swe_calc_ut()` returns an array `xx[6]` containing:

| Index | Field | Description |
|-------|-------|-------------|
| 0 | Longitude | Ecliptic longitude (0-360°) |
| 1 | Latitude | Ecliptic latitude |
| 2 | Distance | Distance in AU |
| 3 | Speed Long | Daily motion in longitude (deg/day) |
| 4 | Speed Lat | Daily motion in latitude |
| 5 | Speed Dist | Daily motion in distance |

**Key flags you'll use:**
- `SEFLG_SPEED` - Include speed calculations (essential for retrograde detection)
- `SEFLG_EQUATORIAL` - For right ascension/declination if needed
- `SEFLG_TOPOCTR` - For topocentric positions
- `SEFLG_SIDEREAL` - For sidereal zodiac calculations

---

## Data Population Strategy

### 1. **Bulk Calculation Approach**
```rust
// Pseudocode for your Rust app
fn populate_ephemeris_range(start_date: DateTime<Utc>, end_date: DateTime<Utc>) {
    let mut current = start_date;
    let step = Duration::minutes(1); // or 5 min for broader searches
    
    while current < end_date {
        let jd = swe_julday(current.year(), current.month(), current.day(), 
                           current.hour() as f64 + current.minute() as f64 / 60.0);
        
        for body_id in 0..=10 { // Sun through Pluto + Moon
            let (xx, _) = swe_calc_ut(jd, body_id, SEFLG_SPEED);
            // Insert into planet_positions table
        }
        
        current += step;
    }
}
```

### 2. **Aspect Pre-calculation**
Calculate aspects on-the-fly during insertion or as a background job:
- For each timestamp, compare all planet pairs
- Calculate angular separation using `swe_difdegn()`
- Check against aspect orbs (conjunction: 10°, opposition: 10°, trine: 8°, square: 7.8°, sextile: 6°) 

### 3. **House Calculation**
Only calculate for specific locations when needed, or pre-calculate for major cities:
```rust
let (cusps, ascmc) = swe_houses_ex(jd_ut, 0, lat, lon, b'P'); // Placidus
```

---

## Query Patterns for Electoral Astrology

### Find Best Wedding Dates (Venus strong, Moon favorable)
```sql
SELECT 
    time,
    moon_sign,
    venus_longitude,
    (SELECT COUNT(*) FROM aspects a 
     WHERE a.time = pp.time 
     AND a.body1_id = 3 -- Venus
     AND a.aspect_type IN (0, 4) -- Conjunction or Sextile
     AND a.orb < 2) as favorable_venus_aspects
FROM planet_positions pp
WHERE body_id = 1 -- Moon
  AND time BETWEEN '2024-06-01' AND '2024-08-01'
  AND moon_sign IN (1, 3, 4, 6, 7, 9, 11, 12) -- Taurus, Cancer, Leo, Libra, Scorpio, Capricorn, Aquarius, Pisces
  AND NOT EXISTS (
      SELECT 1 FROM lunar_conditions lc 
      WHERE lc.time = pp.time AND lc.is_void_of_course = true
  )
ORDER BY favorable_venus_aspects DESC, time;
```

### Find Grand Trines
```sql
SELECT * FROM configurations 
WHERE config_type = 'Grand Trine'
  AND time BETWEEN '2024-01-01' AND '2024-12-31'
  AND strength_score > 80
ORDER BY strength_score DESC;
```

---

## Recommended Architecture

1. **High-resolution base data**: 1-minute intervals for Moon (fast-moving), 1-hour for outer planets
2. **Continuous aggregates**: Use TimescaleDB's continuous aggregates for daily/weekly summaries
3. **Retrograde caching**: Store retrograde periods in a separate table for quick lookup
4. **Aspect materialization**: Pre-calculate aspects for common orb ranges (0-10°)
5. **House on-demand**: Calculate houses only when specific location queries come in

This schema will let you efficiently search for:
- Void-of-course Moon periods
- Retrograde stations
- Exact aspects and transits
- Planetary ingresses (sign changes)
- Complex configurations (grand trines, T-squares, etc.)
- House placements for specific locations

Would you like me to elaborate on any specific part of this schema or the Rust integration?