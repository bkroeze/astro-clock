use chrono::{NaiveDate, Utc};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;
use tokio::sync::RwLock;

use super::chunk::{ChunkData, ChunkKey, CACHE_SIZE_CHUNKS};
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
}

/// Manages chunk data with LRU caching and database fallback
///
/// Provides thread-safe access to astrological data chunks with a two-tier
/// lookup strategy: memory cache first, then database on miss.
#[derive(Debug)]
pub struct ChunkManager {
    /// LRU cache for chunk data, protected by RwLock for thread-safe access
    cache: RwLock<LruCache<ChunkKey, Arc<ChunkData>>>,
    /// Database connection pool for loading chunks on cache miss
    db_pool: DatabasePool,
}

impl ChunkManager {
    /// Create a new ChunkManager with the given database pool
    ///
    /// Initializes an empty LRU cache with capacity for CACHE_SIZE_CHUNKS (30) chunks.
    pub fn new(db_pool: DatabasePool) -> Self {
        let cache = RwLock::new(LruCache::new(
            NonZeroUsize::new(CACHE_SIZE_CHUNKS).unwrap(),
        ));
        Self { cache, db_pool }
    }

    /// Get a chunk for the given date, using cache-first lookup
    ///
    /// # Arguments
    /// * `date` - The date to retrieve chunk data for
    ///
    /// # Returns
    /// * `Ok(Arc<ChunkData>)` - The chunk data, either from cache or freshly loaded
    /// * `Err(ChunkManagerError)` - If database query fails or data cannot be converted
    ///
    /// # Lookup Strategy
    /// 1. Check cache first (O(1) lookup)
    /// 2. On cache miss, load from database
    /// 3. Populate cache with loaded data for future queries
    pub async fn get_chunk(
        &self,
        date: NaiveDate,
    ) -> Result<Arc<ChunkData>, ChunkManagerError> {
        let key = ChunkKey::new(date);

        // 1. Check cache first
        {
            let mut cache = self.cache.write().await;
            if let Some(data) = cache.get(&key) {
                return Ok(Arc::clone(data));
            }
        }

        // 2. Cache miss - load from database
        let chunk_data = self.load_chunk_from_db(date).await?;
        let chunk_arc = Arc::new(chunk_data);

        // 3. Populate cache
        {
            let mut cache = self.cache.write().await;
            cache.put(key, Arc::clone(&chunk_arc));
        }

        Ok(chunk_arc)
    }

    /// Load chunk data from the database
    ///
    /// Queries all three hypertables (planet_positions, aspects, lunar_conditions)
    /// and converts the results to compact in-memory representations.
    async fn load_chunk_from_db(
        &self,
        date: NaiveDate,
    ) -> Result<ChunkData, ChunkManagerError> {
        let start_time = date
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| {
                ChunkManagerError::Conversion(format!("Invalid date: {}", date))
            })?
            .and_utc();
        let end_time = date
            .and_hms_opt(23, 59, 59)
            .ok_or_else(|| {
                ChunkManagerError::Conversion(format!("Invalid date: {}", date))
            })?
            .and_utc();

        // Load planet positions
        let positions = sqlx::query_as::<_, schema::PlanetPosition>(
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

        // Load aspects
        let aspects = sqlx::query_as::<_, schema::Aspect>(
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

        // Load lunar conditions
        let lunar = sqlx::query_as::<_, schema::LunarCondition>(
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
