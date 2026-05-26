use chrono::NaiveDate;
use lru::LruCache;
use rust_decimal::Decimal;
use sqlx::Row;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;
use tokio::sync::RwLock;

use crate::performance::{MemoryMonitor, MemoryPressure};

use super::chunk::{CACHE_SIZE_CHUNKS, ChunkData, ChunkKey};
use super::chunk_generator::{ChunkGenerator, ChunkGeneratorError};
use super::pool::DatabasePool;
use super::schema;

/// Errors that can occur during chunk manager operations
#[derive(Error, Debug)]
pub enum ChunkManagerError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Cache error: {0}")]
    Cache(String),
    #[error("Chunk not found: {0}")]
    NotFound(NaiveDate),
    #[error("Chunk conversion error: {0}")]
    Conversion(String),
    #[error("Generation error: {0}")]
    Generation(String),
}

impl From<ChunkGeneratorError> for ChunkManagerError {
    fn from(err: ChunkGeneratorError) -> Self {
        ChunkManagerError::Generation(err.to_string())
    }
}

/// Configuration for chunk manager behavior
#[derive(Debug, Clone)]
pub struct ChunkManagerConfig {
    /// Enable background pre-fetching of adjacent chunks
    pub enable_pre_fetching: bool,
    /// Number of days to pre-fetch on each side (default: 1)
    pub pre_fetch_range: i64,
    /// Soft memory limit in MB (default: 30)
    pub memory_soft_limit_mb: usize,
    /// Hard memory limit in MB (default: 50)
    pub memory_hard_limit_mb: usize,
    /// Eviction threshold percentage of hard limit (default: 90)
    pub memory_eviction_threshold_pct: f64,
    /// Enable memory-aware eviction (default: true)
    pub enable_memory_monitoring: bool,
}

impl Default for ChunkManagerConfig {
    fn default() -> Self {
        Self {
            enable_pre_fetching: true,
            pre_fetch_range: 1,
            memory_soft_limit_mb: 30,
            memory_hard_limit_mb: 50,
            memory_eviction_threshold_pct: 90.0,
            enable_memory_monitoring: true,
        }
    }
}

/// Statistics for monitoring chunk manager performance
#[derive(Debug, Default)]
pub struct ChunkManagerStats {
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
    pub db_loads: AtomicU64,
    pub generations: AtomicU64,
    pub pre_fetches: AtomicU64,
}

impl ChunkManagerStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_db_load(&self) {
        self.db_loads.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_generation(&self) {
        self.generations.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_pre_fetch(&self) {
        self.pre_fetches.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> ChunkManagerStatsSnapshot {
        ChunkManagerStatsSnapshot {
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
            db_loads: self.db_loads.load(Ordering::Relaxed),
            generations: self.generations.load(Ordering::Relaxed),
            pre_fetches: self.pre_fetches.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug)]
pub struct ChunkManagerStatsSnapshot {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub db_loads: u64,
    pub generations: u64,
    pub pre_fetches: u64,
}

impl ChunkManagerStatsSnapshot {
    pub fn total_requests(&self) -> u64 {
        self.cache_hits + self.cache_misses
    }

    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.total_requests();
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }
}

/// Manages chunk data with LRU caching, database fallback, and Swiss Ephemeris generation
///
/// Provides thread-safe access to astrological data chunks with a three-tier
/// lookup strategy: memory cache first, then database, then Swiss Ephemeris generation.
#[derive(Clone)]
pub struct ChunkManager {
    /// LRU cache for chunk data, protected by RwLock for thread-safe access
    cache: Arc<RwLock<LruCache<ChunkKey, Arc<ChunkData>>>>,
    /// Database connection pool for loading chunks on cache miss
    db_pool: DatabasePool,
    /// Generator for creating chunks from Swiss Ephemeris when not in database
    generator: ChunkGenerator,
    /// Configuration for chunk manager behavior
    config: ChunkManagerConfig,
    /// Statistics for monitoring performance
    stats: Arc<ChunkManagerStats>,
    /// Memory monitor for cache eviction decisions
    memory_monitor: Arc<RwLock<MemoryMonitor>>,
}

impl ChunkManager {
    /// Create a new ChunkManager with the given database pool
    ///
    /// Initializes an empty LRU cache with capacity for CACHE_SIZE_CHUNKS (30) chunks
    /// and a ChunkGenerator for on-demand data generation.
    pub fn new(db_pool: DatabasePool) -> Self {
        Self::with_config(db_pool, ChunkManagerConfig::default())
    }

    /// Create a new ChunkManager with custom configuration
    pub fn with_config(db_pool: DatabasePool, config: ChunkManagerConfig) -> Self {
        let cache = Arc::new(RwLock::new(LruCache::new(
            NonZeroUsize::new(CACHE_SIZE_CHUNKS).unwrap(),
        )));
        let generator = ChunkGenerator::new(db_pool.clone());
        let memory_monitor = Arc::new(RwLock::new(MemoryMonitor::new(
            config.memory_soft_limit_mb,
            config.memory_hard_limit_mb,
            config.memory_eviction_threshold_pct,
        )));
        Self {
            cache,
            db_pool,
            generator,
            config,
            stats: Arc::new(ChunkManagerStats::new()),
            memory_monitor,
        }
    }

    /// Get current statistics snapshot
    pub fn stats(&self) -> ChunkManagerStatsSnapshot {
        self.stats.snapshot()
    }

    /// Get a chunk for the given date, using cache-first lookup with generation fallback
    ///
    /// # Arguments
    /// * `date` - The date to retrieve chunk data for
    ///
    /// # Returns
    /// * `Ok(Arc<ChunkData>)` - The chunk data from cache, database, or fresh generation
    /// * `Err(ChunkManagerError)` - If all lookup strategies fail
    ///
    /// # Lookup Strategy
    /// 1. Check cache first (O(1) lookup)
    /// 2. On cache miss, load from database
    /// 3. If not in database, generate from Swiss Ephemeris
    /// 4. Save generated data to database in background
    /// 5. Populate cache with loaded/generated data for future queries
    pub async fn get_chunk(&self, date: NaiveDate) -> Result<Arc<ChunkData>, ChunkManagerError> {
        let key = ChunkKey::new(date);

        // 1. Check cache first
        {
            let mut cache = self.cache.write().await;
            if let Some(data) = cache.get(&key) {
                // Cache hit - record stats and trigger pre-fetching
                self.stats.record_cache_hit();
                let self_arc = Arc::new(self.clone());
                self_arc.pre_fetch_adjacent_chunks(date);
                return Ok(Arc::clone(data));
            }
        }

        // Cache miss - record stats and check memory pressure
        self.stats.record_cache_miss();
        self.evict_if_needed().await;

        // 2. Try to load from database
        match self.load_chunk_from_db(date).await {
            Ok(chunk_data) => {
                self.stats.record_db_load();
                let chunk_arc = Arc::new(chunk_data);
                {
                    let mut cache = self.cache.write().await;
                    cache.put(key, Arc::clone(&chunk_arc));
                }

                // Trigger pre-fetching after successful database load
                let self_arc = Arc::new(self.clone());
                self_arc.pre_fetch_adjacent_chunks(date);

                return Ok(chunk_arc);
            }
            Err(ChunkManagerError::NotFound(_)) => {
                // Database doesn't have this chunk, generate it
                tracing::info!(
                    "Chunk not found in database, generating from Swiss Ephemeris: {}",
                    date
                );
            }
            Err(e) => {
                // Other database error, log and try generation
                tracing::warn!(
                    "Database error loading chunk, falling back to generation: {}",
                    e
                );
            }
        }

        // 3. Generate from Swiss Ephemeris
        let chunk_data = self
            .generator
            .generate_chunk(date)
            .await
            .map_err(|e| ChunkManagerError::Generation(e.to_string()))?;

        self.stats.record_generation();

        // 4. Save to database (fire-and-forget, don't fail if save fails)
        let chunk_for_save = chunk_data.clone();
        let generator = ChunkGenerator::new(self.db_pool.clone());
        tokio::spawn(async move {
            if let Err(e) = generator.save_chunk_to_db(&chunk_for_save).await {
                tracing::warn!("Failed to save generated chunk to database: {}", e);
            } else {
                tracing::info!(
                    "Successfully saved chunk to database: {}",
                    chunk_for_save.date
                );
            }
        });

        // 5. Add to cache
        let chunk_arc = Arc::new(chunk_data);
        {
            let mut cache = self.cache.write().await;
            cache.put(key, Arc::clone(&chunk_arc));
        }

        // 6. Trigger pre-fetching after generation
        let self_arc = Arc::new(self.clone());
        self_arc.pre_fetch_adjacent_chunks(date);

        Ok(chunk_arc)
    }

    /// Load chunk data from the database
    ///
    /// Queries all three hypertables (planet_positions, aspects, lunar_conditions)
    /// and converts the results to compact in-memory representations.
    async fn load_chunk_from_db(&self, date: NaiveDate) -> Result<ChunkData, ChunkManagerError> {
        let start_time = date
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| ChunkManagerError::Conversion(format!("Invalid date: {}", date)))?
            .and_utc();
        let end_time = date
            .and_hms_opt(23, 59, 59)
            .ok_or_else(|| ChunkManagerError::Conversion(format!("Invalid date: {}", date)))?
            .and_utc();

        // Load planet positions using sqlx::query and manual mapping
        // Using f64 for database values and converting to Decimal
        let rows = sqlx::query(
            r#"
            SELECT time, body_id, longitude, latitude, distance, speed_lon, retrograde, zodiac_sign
            FROM planet_positions
            WHERE time >= $1 AND time <= $2
            ORDER BY time, body_id
            "#,
        )
        .bind(start_time)
        .bind(end_time)
        .fetch_all(self.db_pool.pool())
        .await?;

        let mut positions = Vec::with_capacity(rows.len());
        for row in rows {
            let longitude: f64 = row.try_get(2)?;
            let latitude: f64 = row.try_get(3)?;
            let distance: f64 = row.try_get(4)?;
            let speed_lon: f64 = row.try_get(5)?;

            positions.push(schema::PlanetPosition {
                time: row.try_get(0)?,
                body_id: row.try_get(1)?,
                longitude: Decimal::from_f64_retain(longitude)
                    .ok_or_else(|| ChunkManagerError::Conversion("longitude".to_string()))?,
                latitude: Decimal::from_f64_retain(latitude)
                    .ok_or_else(|| ChunkManagerError::Conversion("latitude".to_string()))?,
                distance: Decimal::from_f64_retain(distance)
                    .ok_or_else(|| ChunkManagerError::Conversion("distance".to_string()))?,
                speed_lon: Decimal::from_f64_retain(speed_lon)
                    .ok_or_else(|| ChunkManagerError::Conversion("speed_lon".to_string()))?,
                retrograde: row.try_get(6)?,
                zodiac_sign: row.try_get(7)?,
            });
        }

        // Load aspects
        let rows = sqlx::query(
            r#"
            SELECT time, body1_id, body2_id, aspect_type, orb, applying
            FROM aspects
            WHERE time >= $1 AND time <= $2
            ORDER BY time
            "#,
        )
        .bind(start_time)
        .bind(end_time)
        .fetch_all(self.db_pool.pool())
        .await?;

        let mut aspects = Vec::with_capacity(rows.len());
        for row in rows {
            let orb: f64 = row.try_get(4)?;

            aspects.push(schema::Aspect {
                time: row.try_get(0)?,
                body1_id: row.try_get(1)?,
                body2_id: row.try_get(2)?,
                aspect_type: row.try_get(3)?,
                orb: Decimal::from_f64_retain(orb)
                    .ok_or_else(|| ChunkManagerError::Conversion("orb".to_string()))?,
                applying: row.try_get(5)?,
            });
        }

        // Load lunar conditions
        let rows = sqlx::query(
            r#"
            SELECT time, moon_phase, moon_sign, moon_phase_angle, moon_illumination,
                   is_void_of_course, voc_start, voc_end
            FROM lunar_conditions
            WHERE time >= $1 AND time <= $2
            ORDER BY time
            "#,
        )
        .bind(start_time)
        .bind(end_time)
        .fetch_all(self.db_pool.pool())
        .await?;

        let mut lunar = Vec::with_capacity(rows.len());
        for row in rows {
            let moon_phase_angle: f64 = row.try_get(3)?;
            let moon_illumination: f64 = row.try_get(4)?;

            lunar.push(schema::LunarCondition {
                time: row.try_get(0)?,
                moon_phase: row.try_get(1)?,
                moon_sign: row.try_get(2)?,
                moon_phase_angle: Decimal::from_f64_retain(moon_phase_angle)
                    .ok_or_else(|| ChunkManagerError::Conversion("moon_phase_angle".to_string()))?,
                moon_illumination: Decimal::from_f64_retain(moon_illumination).ok_or_else(
                    || ChunkManagerError::Conversion("moon_illumination".to_string()),
                )?,
                is_void_of_course: row.try_get(5)?,
                voc_start: row.try_get(6)?,
                voc_end: row.try_get(7)?,
            });
        }

        // Convert to compact format
        ChunkData::from_schema_data(date, &positions, &aspects, &lunar)
            .map_err(|e| ChunkManagerError::Conversion(e.to_string()))
    }

    /// Get current cache statistics
    ///
    /// Returns (current_size, capacity) tuple showing how many chunks
    /// are currently cached vs the maximum capacity.
    pub async fn cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.read().await;
        (cache.len(), cache.cap().get())
    }

    /// Clear all entries from the cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Check if a chunk for the given date is currently in cache
    pub async fn is_cached(&self, date: NaiveDate) -> bool {
        let key = ChunkKey::new(date);
        let cache = self.cache.read().await;
        cache.contains(&key)
    }

    /// Get the number of chunks currently in cache
    pub async fn cache_size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }

    /// Peek at a cached chunk without affecting LRU order
    ///
    /// Useful for diagnostics and monitoring without disturbing the cache eviction order.
    pub async fn peek_cached(&self, date: NaiveDate) -> Option<Arc<ChunkData>> {
        let key = ChunkKey::new(date);
        let cache = self.cache.read().await;
        cache.peek(&key).map(Arc::clone)
    }

    /// Pre-fetch adjacent chunks (previous and next day) in background
    ///
    /// This is a fire-and-forget operation - failures are silent and don't
    /// affect the main query flow. Pre-fetched chunks populate the cache
    /// for faster subsequent queries.
    fn pre_fetch_adjacent_chunks(self: Arc<Self>, center_date: NaiveDate) {
        if !self.config.enable_pre_fetching {
            return;
        }

        use chrono::Duration;

        // Spawn a new task for pre-fetching to avoid blocking
        tokio::spawn(async move {
            // Pre-fetch range on each side
            for offset in 1..=self.config.pre_fetch_range {
                let prev_date = center_date - Duration::days(offset);
                let next_date = center_date + Duration::days(offset);

                // Check and fetch previous day
                let self_clone = Arc::clone(&self);
                let is_prev_cached = self_clone.is_cached(prev_date).await;
                if !is_prev_cached {
                    let self_fetch = Arc::clone(&self);
                    if self_fetch.get_chunk(prev_date).await.is_ok() {
                        self.stats.record_pre_fetch();
                        tracing::debug!("Pre-fetched previous day: {}", prev_date);
                    }
                }

                // Check and fetch next day
                let self_clone = Arc::clone(&self);
                let is_next_cached = self_clone.is_cached(next_date).await;
                if !is_next_cached {
                    let self_fetch = Arc::clone(&self);
                    if self_fetch.get_chunk(next_date).await.is_ok() {
                        self.stats.record_pre_fetch();
                        tracing::debug!("Pre-fetched next day: {}", next_date);
                    }
                }
            }
        });
    }

    /// Get multiple chunks for a date range efficiently
    ///
    /// This method loads all chunks in the range and triggers pre-fetching
    /// for optimal cache utilization.
    pub async fn get_chunk_range(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<Arc<ChunkData>>, ChunkManagerError> {
        use chrono::Duration;

        let mut chunks = Vec::new();
        let mut current = start_date;

        while current <= end_date {
            let chunk = self.get_chunk(current).await?;
            chunks.push(chunk);
            current += Duration::days(1);
        }

        Ok(chunks)
    }

    /// Evict cache entries if memory pressure is high
    ///
    /// This method checks memory pressure and evicts cache entries
    /// when pressure reaches High or Critical levels.
    async fn evict_if_needed(&self) {
        if !self.config.enable_memory_monitoring {
            return;
        }

        let mut monitor = self.memory_monitor.write().await;
        let pressure = monitor.check_memory_pressure();
        drop(monitor); // Release lock before cache operations

        match pressure {
            MemoryPressure::Critical => {
                // Evict 50% of cache
                let mut cache = self.cache.write().await;
                let target_size = cache.len() / 2;
                while cache.len() > target_size {
                    cache.pop_lru();
                }
                tracing::error!(
                    "Critical memory pressure: evicted to {} chunks",
                    cache.len()
                );
            }
            MemoryPressure::High => {
                // Evict 25% of cache
                let mut cache = self.cache.write().await;
                let target_size = cache.len() * 3 / 4;
                while cache.len() > target_size {
                    cache.pop_lru();
                }
                tracing::warn!("High memory pressure: evicted to {} chunks", cache.len());
            }
            MemoryPressure::Elevated => {
                tracing::info!("Memory usage elevated (above soft limit)");
            }
            MemoryPressure::Normal => {
                // No action needed
            }
        }
    }

    /// Get current memory stats and pressure level
    ///
    /// Returns (current_memory_mb, pressure_level) tuple
    pub async fn memory_stats(&self) -> (usize, MemoryPressure) {
        let mut monitor = self.memory_monitor.write().await;
        let current_mb = monitor.current_memory_mb();
        let pressure = monitor.check_memory_pressure();
        (current_mb, pressure)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a database connection and are integration tests
    // They should be run with: cargo test --features db -- --test-threads=1

    #[test]
    fn test_chunk_manager_error_display() {
        let err = ChunkManagerError::NotFound(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        assert!(err.to_string().contains("2024-01-01"));
    }

    #[test]
    fn test_chunk_manager_error_from_sqlx() {
        // This just verifies the From impl exists
        fn assert_from_sqlx<T: From<sqlx::Error>>() {}
        assert_from_sqlx::<ChunkManagerError>();
    }
}
