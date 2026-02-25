-- Schema Verification Script
-- Run this after migrations to verify all schema objects were created correctly
-- Usage: psql $PG_URL -f migrations/verify_schema.sql

\echo '================================================================================'
\echo 'ASTRO CLOCK DATABASE SCHEMA VERIFICATION'
\echo '================================================================================'
\echo ''

-- ============================================================================
-- 1. CHECK TIMESCALEDB EXTENSION
-- ============================================================================
\echo '--- 1. TimescaleDB Extension ---'
SELECT 
    extname AS extension_name,
    extversion AS version,
    extnamespace::regnamespace AS schema
FROM pg_extension 
WHERE extname = 'timescaledb';

\echo ''

-- ============================================================================
-- 2. VERIFY ALL TABLES EXIST
-- ============================================================================
\echo '--- 2. Table Verification ---'
SELECT 
    schemaname,
    tablename,
    tableowner
FROM pg_tables 
WHERE schemaname = 'public' 
AND tablename IN (
    'planet_positions',
    'aspects', 
    'lunar_conditions',
    'house_cusps',
    'locations',
    'aspect_summaries',
    'charts'
)
ORDER BY tablename;

-- Count of expected tables
\echo ''
\echo 'Expected tables: 7 (planet_positions, aspects, lunar_conditions, house_cusps, locations, aspect_summaries, charts)'
SELECT COUNT(*) AS tables_found 
FROM pg_tables 
WHERE schemaname = 'public' 
AND tablename IN (
    'planet_positions', 'aspects', 'lunar_conditions', 
    'house_cusps', 'locations', 'aspect_summaries', 'charts'
);

\echo ''

-- ============================================================================
-- 3. VERIFY HYPERTABLES
-- ============================================================================
\echo '--- 3. Hypertable Verification ---'
SELECT 
    hypertable_name,
    chunk_time_interval,
    num_dimensions,
    num_chunks
FROM timescaledb_information.hypertables
ORDER BY hypertable_name;

-- Verify chunk_time_interval is 1 day for all hypertables
\echo ''
\echo 'Expected: All hypertables should have 1-day chunk intervals'
SELECT 
    hypertable_name,
    CASE 
        WHEN chunk_time_interval = INTERVAL '1 day' THEN 'OK - 1 day'
        ELSE 'WARNING - Not 1 day: ' || chunk_time_interval::text
    END AS chunk_interval_check
FROM timescaledb_information.hypertables;

\echo ''

-- ============================================================================
-- 4. VERIFY INDEXES
-- ============================================================================
\echo '--- 4. Index Verification ---'
SELECT 
    schemaname,
    tablename,
    indexname,
    indexdef
FROM pg_indexes 
WHERE schemaname = 'public'
AND tablename IN (
    'planet_positions',
    'aspects',
    'lunar_conditions', 
    'house_cusps',
    'locations',
    'aspect_summaries'
)
ORDER BY tablename, indexname;

-- Count indexes per table
\echo ''
\echo 'Index counts per table:'
SELECT 
    tablename,
    COUNT(*) AS index_count
FROM pg_indexes 
WHERE schemaname = 'public'
AND tablename IN (
    'planet_positions', 'aspects', 'lunar_conditions',
    'house_cusps', 'locations', 'aspect_summaries'
)
GROUP BY tablename
ORDER BY tablename;

\echo ''

-- ============================================================================
-- 5. VERIFY FOREIGN KEYS
-- ============================================================================
\echo '--- 5. Foreign Key Verification ---'
SELECT
    tc.constraint_name,
    tc.table_name,
    kcu.column_name,
    ccu.table_name AS foreign_table_name,
    ccu.column_name AS foreign_column_name
FROM information_schema.table_constraints AS tc
JOIN information_schema.key_column_usage AS kcu
    ON tc.constraint_name = kcu.constraint_name
    AND tc.table_schema = kcu.table_schema
JOIN information_schema.constraint_column_usage AS ccu
    ON ccu.constraint_name = tc.constraint_name
    AND ccu.table_schema = tc.table_schema
WHERE tc.constraint_type = 'FOREIGN KEY'
AND tc.table_schema = 'public';

\echo ''

-- ============================================================================
-- 6. ROW COUNT SANITY CHECKS
-- ============================================================================
\echo '--- 6. Row Count Verification ---'
SELECT 'planet_positions' AS table_name, COUNT(*) AS row_count FROM planet_positions
UNION ALL
SELECT 'aspects' AS table_name, COUNT(*) AS row_count FROM aspects
UNION ALL
SELECT 'lunar_conditions' AS table_name, COUNT(*) AS row_count FROM lunar_conditions
UNION ALL
SELECT 'house_cusps' AS table_name, COUNT(*) AS row_count FROM house_cusps
UNION ALL
SELECT 'locations' AS table_name, COUNT(*) AS row_count FROM locations
UNION ALL
SELECT 'aspect_summaries' AS table_name, COUNT(*) AS row_count FROM aspect_summaries
UNION ALL
SELECT 'charts' AS table_name, COUNT(*) AS row_count FROM charts
ORDER BY table_name;

\echo ''
\echo 'Expected: All tables should have 0 rows (fresh database ready for data loading)'

\echo ''

-- ============================================================================
-- 7. TABLE STRUCTURE VERIFICATION
-- ============================================================================
\echo '--- 7. Column Verification (sample: planet_positions) ---'
SELECT 
    column_name,
    data_type,
    is_nullable,
    column_default
FROM information_schema.columns
WHERE table_name = 'planet_positions'
AND table_schema = 'public'
ORDER BY ordinal_position;

\echo ''

-- ============================================================================
-- 8. VERIFICATION SUMMARY
-- ============================================================================
\echo '================================================================================'
\echo 'VERIFICATION SUMMARY'
\echo '================================================================================'

-- Check counts
SELECT 
    (SELECT COUNT(*) FROM timescaledb_information.hypertables) AS hypertable_count,
    (SELECT COUNT(*) FROM pg_indexes WHERE schemaname = 'public' 
     AND tablename IN ('planet_positions', 'aspects', 'lunar_conditions', 
                       'house_cusps', 'locations', 'aspect_summaries')) AS index_count,
    (SELECT COUNT(*) FROM information_schema.table_constraints 
     WHERE constraint_type = 'FOREIGN KEY' AND table_schema = 'public') AS fk_count;

\echo ''
\echo 'Expected results:'
\echo '  - Hypertables: 4 (planet_positions, aspects, lunar_conditions, house_cusps)'
\echo '  - Indexes: 12+ (various composite and partial indexes)'
\echo '  - Foreign Keys: 1 (house_cusps.location_id -> locations.id)'
\echo ''
\echo '================================================================================'
\echo 'VERIFICATION COMPLETE'
\echo '================================================================================'
