use chrono::NaiveDate;
use lru::LruCache;
use rust_decimal::Decimal;
use sqlx::Row;
use std::num::NonZeroUsize;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

use super::chunk::{ChunkData, ChunkKey, CACHE_SIZE_CHUNKS};
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

/// Manages chunk data with LRU caching, database fallback, and Swiss Ephemeris generation
///
/// Provides thread-safe access to astrological data chunks with a three-tier
/// lookup strategy: memory cache first, then database, then Swiss Ephemeris generation.
#[derive(Debug)]
pub struct ChunkManager {
    /// LRU cache for chunk data, protected by RwLock for thread-safe access
    cache: RwLock<LruCache<ChunkKey, Arc<ChunkData>>>,
    /// Database connection pool for loading chunks on cache miss
    db_pool: DatabasePool,
    /// Generator for creating chunks from Swiss Ephemeris when not in database
    generator: ChunkGenerator,
}

impl ChunkManager {
    /// Create a new ChunkManager with the given database pool
    ///
    /// Initializes an empty LRU cache with capacity for CACHE_SIZE_CHUNKS (30) chunks
    /// and a ChunkGenerator for on-demand data generation.
    pub fn new(db_pool: DatabasePool) -> Self {
        let cache = RwLock::new(LruCache::new(
            NonZeroUsize::new(CACHE_SIZE_CHUNKS).unwrap(),
        ));
        let generator = ChunkGenerator::new(db_pool.clone());
        Self {
            cache,
            db_pool,
            generator,
        }
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

        // 2. Try to load from database
        match self.load_chunk_from_db(date).await {
            Ok(chunk_data) => {
                let chunk_arc = Arc::new(chunk_data);
                {
                    let mut cache = self.cache.write().await;
                    cache.put(key, Arc::clone(&chunk_arc));
                }
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
                tracing::warn!("Database error loading chunk, falling back to generation: {}", e);
            }
        }

        // 3. Generate from Swiss Ephemeris
        let chunk_data = self
            .generator
            .generate_chunk(date)
            .await
            .map_err(|e| ChunkManagerError::Generation(e.to_string()))?;

        // 4. Save to database (fire-and-forget, don't fail if save fails)
        let chunk_for_save = chunk_data.clone();
        let generator = ChunkGenerator::new(self.db_pool.clone());
        tokio::spawn(async move {
            if let Err(e) = generator.save_chunk_to_db(&chunk_for_save).await {
                tracing::warn!("Failed to save generated chunk to database: {}", e);
            } else {
                tracing::info!("Successfully saved chunk to database: {}", chunk_for_save.date);
            }
        });

        // 5. Add to cache and return
        let chunk_arc = Arc::new(chunk_data);
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
                moon_illumination: Decimal::from_f64_retain(moon_illumination)
                    .ok_or_else(|| ChunkManagerError::Conversion("moon_illumination".to_string()))?,
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
