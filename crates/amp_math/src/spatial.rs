//! Spatial distance caching system for performance optimization.
//!
//! This module provides a distance cache that reduces expensive Vec3::distance_squared
//! calls by caching results with TTL (Time To Live) and spatial indexing via Morton codes.
//!
//! # Examples
//!
//! ```rust
//! use amp_math::spatial::{MortonKey3, DistanceCache, CachedDistance};
//! use glam::Vec3;
//!
//! let mut cache = DistanceCache::new();
//! let camera_pos = Vec3::new(0.0, 0.0, 0.0);
//! let entity_pos = Vec3::new(10.0, 0.0, 0.0);
//!
//! // Cache distance calculation
//! let distance = cache.get_or_compute_distance(camera_pos, entity_pos, 0);
//! assert_eq!(distance, 10.0);
//! ```

use glam::Vec3;
use std::collections::HashMap;

#[cfg(feature = "unstable_hierarchical_world")]
pub mod quadtree;

#[cfg(feature = "unstable_hierarchical_world")]
pub use quadtree::{
    LODLevel, WorldCoord, DETAIL_CHUNK_SIZE, DETAIL_STREAMING_RADIUS, LOCAL_CHUNK_SIZE,
    LOCAL_STREAMING_RADIUS, MACRO_REGION_SIZE, MACRO_STREAMING_RADIUS, MICRO_CHUNK_SIZE,
    MICRO_STREAMING_RADIUS, REGION_SIZE, REGION_STREAMING_RADIUS,
};

/// Morton key wrapper for 3D spatial indexing using u64.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MortonKey3(pub u64);

impl MortonKey3 {
    /// Create a Morton key from 3D position.
    pub fn from_position(pos: Vec3) -> Self {
        Self(crate::morton::Morton3D::encode(pos))
    }

    /// Create a Morton key from raw u64 value.
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }

    /// Get the raw u64 value.
    pub fn raw(&self) -> u64 {
        self.0
    }

    /// Decode the Morton key back to 3D position.
    pub fn to_position(&self) -> Vec3 {
        crate::morton::Morton3D::decode(self.0)
    }

    /// Get the common prefix length between two Morton keys.
    /// Higher values indicate closer spatial proximity.
    pub fn common_prefix_length(&self, other: &Self) -> u32 {
        crate::morton::Morton3D::common_prefix_length(self.0, other.0)
    }
}

/// Cached distance entry with TTL and spatial information.
#[derive(Debug, Clone)]
pub struct CachedDistance {
    /// Cached distance value
    pub distance: f32,
    /// Frame number when this entry was created
    pub frame: u32,
    /// Morton key for spatial locality
    pub morton_key: MortonKey3,
    /// Camera position when distance was calculated
    pub camera_pos: Vec3,
    /// Entity position when distance was calculated
    pub entity_pos: Vec3,
}

impl CachedDistance {
    /// Create a new cached distance entry.
    pub fn new(distance: f32, frame: u32, camera_pos: Vec3, entity_pos: Vec3) -> Self {
        Self {
            distance,
            frame,
            morton_key: MortonKey3::from_position(entity_pos),
            camera_pos,
            entity_pos,
        }
    }

    /// Check if this cache entry is still valid (within TTL).
    pub fn is_valid(&self, current_frame: u32, ttl_frames: u32) -> bool {
        current_frame.saturating_sub(self.frame) < ttl_frames
    }

    /// Check if the cached distance is still accurate for the given positions.
    /// Returns true if positions haven't changed significantly.
    pub fn is_position_accurate(&self, camera_pos: Vec3, entity_pos: Vec3, tolerance: f32) -> bool {
        let camera_delta = (self.camera_pos - camera_pos).length();
        let entity_delta = (self.entity_pos - entity_pos).length();
        camera_delta < tolerance && entity_delta < tolerance
    }
}

/// High-performance distance cache with TTL and spatial indexing.
///
/// This cache reduces expensive Vec3::distance_squared calls by:
/// - Caching recent distance calculations with 5-frame TTL
/// - Using Morton codes for spatial locality
/// - Maintaining a capacity limit of 2048 entries
/// - Providing sub-centimeter accuracy (<1cm float32 error)
#[derive(Debug)]
pub struct DistanceCache {
    /// Cache storage using entity ID as key
    cache: HashMap<u32, CachedDistance>,
    /// Maximum number of entries to maintain
    capacity: usize,
    /// TTL in frames (5 frames as per Oracle spec)
    ttl_frames: u32,
    /// Tolerance for position accuracy checks (1cm)
    position_tolerance: f32,
    /// Statistics
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
}

impl Default for DistanceCache {
    fn default() -> Self {
        Self::new()
    }
}

impl DistanceCache {
    /// Create a new distance cache with default settings.
    pub fn new() -> Self {
        Self {
            cache: HashMap::with_capacity(2048),
            capacity: 2048,
            ttl_frames: 5,
            position_tolerance: 0.01, // 1cm tolerance
            hits: 0,
            misses: 0,
            evictions: 0,
        }
    }

    /// Create a new distance cache with custom settings.
    pub fn with_capacity_and_ttl(capacity: usize, ttl_frames: u32) -> Self {
        Self {
            cache: HashMap::with_capacity(capacity),
            capacity,
            ttl_frames,
            position_tolerance: 0.01,
            hits: 0,
            misses: 0,
            evictions: 0,
        }
    }

    /// Get cached distance or compute and cache it.
    pub fn get_or_compute_distance(
        &mut self,
        camera_pos: Vec3,
        entity_pos: Vec3,
        entity_id: u32,
    ) -> f32 {
        self.get_or_compute_distance_with_frame(camera_pos, entity_pos, entity_id, 0)
    }

    /// Get cached distance or compute and cache it with frame tracking.
    pub fn get_or_compute_distance_with_frame(
        &mut self,
        camera_pos: Vec3,
        entity_pos: Vec3,
        entity_id: u32,
        current_frame: u32,
    ) -> f32 {
        // Check cache first
        if let Some(cached) = self.cache.get(&entity_id) {
            if cached.is_valid(current_frame, self.ttl_frames)
                && cached.is_position_accurate(camera_pos, entity_pos, self.position_tolerance)
            {
                self.hits += 1;
                return cached.distance;
            }
        }

        // Cache miss - compute distance
        self.misses += 1;
        let distance = camera_pos.distance(entity_pos);

        // Store in cache
        self.insert_cached_distance(entity_id, distance, current_frame, camera_pos, entity_pos);

        distance
    }

    /// Insert a pre-computed distance into the cache.
    pub fn insert_cached_distance(
        &mut self,
        entity_id: u32,
        distance: f32,
        frame: u32,
        camera_pos: Vec3,
        entity_pos: Vec3,
    ) {
        // Check capacity and evict if needed
        if self.cache.len() >= self.capacity && !self.cache.contains_key(&entity_id) {
            self.evict_oldest();
        }

        let cached_distance = CachedDistance::new(distance, frame, camera_pos, entity_pos);
        self.cache.insert(entity_id, cached_distance);
    }

    /// Remove expired entries from the cache.
    pub fn cleanup_expired(&mut self, current_frame: u32) {
        let ttl = self.ttl_frames;
        let initial_len = self.cache.len();

        self.cache
            .retain(|_, cached| cached.is_valid(current_frame, ttl));

        let evicted = initial_len - self.cache.len();
        self.evictions += evicted as u64;
    }

    /// Get cache statistics.
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits,
            misses: self.misses,
            evictions: self.evictions,
            size: self.cache.len(),
            capacity: self.capacity,
            hit_rate: if self.hits + self.misses > 0 {
                self.hits as f32 / (self.hits + self.misses) as f32
            } else {
                0.0
            },
        }
    }

    /// Clear all cache entries.
    pub fn clear(&mut self) {
        self.cache.clear();
        self.hits = 0;
        self.misses = 0;
        self.evictions = 0;
    }

    /// Get the number of cached entries.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Evict the oldest entry (simple FIFO eviction).
    fn evict_oldest(&mut self) {
        if let Some(oldest_key) = self.find_oldest_entry() {
            self.cache.remove(&oldest_key);
            self.evictions += 1;
        }
    }

    /// Find the oldest entry in the cache.
    fn find_oldest_entry(&self) -> Option<u32> {
        self.cache
            .iter()
            .min_by_key(|(_, cached)| cached.frame)
            .map(|(key, _)| *key)
    }
}

/// Cache statistics for monitoring performance.
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub size: usize,
    pub capacity: usize,
    pub hit_rate: f32,
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DistanceCache Stats: {}/{} entries, {:.1}% hit rate, {} hits, {} misses, {} evictions",
            self.size,
            self.capacity,
            self.hit_rate * 100.0,
            self.hits,
            self.misses,
            self.evictions
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_morton_key_creation() {
        let pos = Vec3::new(100.0, 200.0, 300.0);
        let key = MortonKey3::from_position(pos);
        let decoded = key.to_position();

        // Should be close to original position
        assert!((decoded - pos).length() < 1.0);
    }

    #[test]
    fn test_morton_key_raw_conversion() {
        let raw_value = 0x123456789abcdef0;
        let key = MortonKey3::from_raw(raw_value);
        assert_eq!(key.raw(), raw_value);
    }

    #[test]
    fn test_morton_key_common_prefix() {
        let pos1 = Vec3::new(100.0, 100.0, 100.0);
        let pos2 = Vec3::new(101.0, 101.0, 101.0);
        let pos3 = Vec3::new(1000.0, 1000.0, 1000.0);

        let key1 = MortonKey3::from_position(pos1);
        let key2 = MortonKey3::from_position(pos2);
        let key3 = MortonKey3::from_position(pos3);

        // Nearby positions should have longer common prefix
        let close_prefix = key1.common_prefix_length(&key2);
        let far_prefix = key1.common_prefix_length(&key3);

        assert!(close_prefix > far_prefix);
    }

    #[test]
    fn test_cached_distance_validity() {
        let camera_pos = Vec3::new(0.0, 0.0, 0.0);
        let entity_pos = Vec3::new(10.0, 0.0, 0.0);
        let distance = 10.0;
        let frame = 100;

        let cached = CachedDistance::new(distance, frame, camera_pos, entity_pos);

        // Should be valid within TTL
        assert!(cached.is_valid(104, 5));
        // Should be invalid beyond TTL
        assert!(!cached.is_valid(106, 5));
    }

    #[test]
    fn test_cached_distance_position_accuracy() {
        let camera_pos = Vec3::new(0.0, 0.0, 0.0);
        let entity_pos = Vec3::new(10.0, 0.0, 0.0);
        let distance = 10.0;
        let frame = 100;

        let cached = CachedDistance::new(distance, frame, camera_pos, entity_pos);

        // Should be accurate for same positions
        assert!(cached.is_position_accurate(camera_pos, entity_pos, 0.01));

        // Should be accurate for small changes
        let moved_entity = Vec3::new(10.005, 0.0, 0.0);
        assert!(cached.is_position_accurate(camera_pos, moved_entity, 0.01));

        // Should be inaccurate for large changes
        let far_entity = Vec3::new(15.0, 0.0, 0.0);
        assert!(!cached.is_position_accurate(camera_pos, far_entity, 0.01));
    }

    #[test]
    fn test_distance_cache_basic_operations() {
        let mut cache = DistanceCache::new();
        let camera_pos = Vec3::new(0.0, 0.0, 0.0);
        let entity_pos = Vec3::new(10.0, 0.0, 0.0);
        let entity_id = 1;

        // First call should be a miss
        let distance1 = cache.get_or_compute_distance(camera_pos, entity_pos, entity_id);
        assert_eq!(distance1, 10.0);
        assert_eq!(cache.stats().misses, 1);
        assert_eq!(cache.stats().hits, 0);

        // Second call should be a hit
        let distance2 = cache.get_or_compute_distance(camera_pos, entity_pos, entity_id);
        assert_eq!(distance2, 10.0);
        assert_eq!(cache.stats().misses, 1);
        assert_eq!(cache.stats().hits, 1);
    }

    #[test]
    fn test_distance_cache_frame_based_ttl() {
        let mut cache = DistanceCache::new();
        let camera_pos = Vec3::new(0.0, 0.0, 0.0);
        let entity_pos = Vec3::new(10.0, 0.0, 0.0);
        let entity_id = 1;

        // Cache at frame 0
        cache.get_or_compute_distance_with_frame(camera_pos, entity_pos, entity_id, 0);
        assert_eq!(cache.stats().misses, 1);

        // Hit at frame 4 (within TTL)
        cache.get_or_compute_distance_with_frame(camera_pos, entity_pos, entity_id, 4);
        assert_eq!(cache.stats().hits, 1);

        // Miss at frame 5 (beyond TTL)
        cache.get_or_compute_distance_with_frame(camera_pos, entity_pos, entity_id, 5);
        assert_eq!(cache.stats().misses, 2);
    }

    #[test]
    fn test_distance_cache_capacity_eviction() {
        let mut cache = DistanceCache::with_capacity_and_ttl(2, 5);
        let camera_pos = Vec3::new(0.0, 0.0, 0.0);

        // Fill cache to capacity
        for i in 0..2 {
            let entity_pos = Vec3::new(i as f32 * 10.0, 0.0, 0.0);
            cache.get_or_compute_distance_with_frame(camera_pos, entity_pos, i, 0);
        }

        assert_eq!(cache.len(), 2);
        assert_eq!(cache.stats().evictions, 0);

        // Adding one more should trigger eviction
        let entity_pos = Vec3::new(20.0, 0.0, 0.0);
        cache.get_or_compute_distance_with_frame(camera_pos, entity_pos, 2, 0);

        assert_eq!(cache.len(), 2);
        assert_eq!(cache.stats().evictions, 1);
    }

    #[test]
    fn test_distance_cache_cleanup_expired() {
        let mut cache = DistanceCache::new();
        let camera_pos = Vec3::new(0.0, 0.0, 0.0);

        // Add entries at different frames
        for i in 0..5 {
            let entity_pos = Vec3::new(i as f32 * 10.0, 0.0, 0.0);
            cache.get_or_compute_distance_with_frame(camera_pos, entity_pos, i, i);
        }

        assert_eq!(cache.len(), 5);

        // Cleanup at frame 10 (TTL=5, so only entries from frame 5+ should remain)
        cache.cleanup_expired(10);

        assert_eq!(cache.len(), 0); // All entries should be expired
        assert_eq!(cache.stats().evictions, 5);
    }

    #[test]
    fn test_distance_cache_position_tolerance() {
        let mut cache = DistanceCache::new();
        let camera_pos = Vec3::new(0.0, 0.0, 0.0);
        let entity_pos = Vec3::new(10.0, 0.0, 0.0);
        let entity_id = 1;

        // Cache initial position
        cache.get_or_compute_distance_with_frame(camera_pos, entity_pos, entity_id, 0);

        // Small movement should still hit cache
        let small_move = Vec3::new(10.005, 0.0, 0.0);
        cache.get_or_compute_distance_with_frame(camera_pos, small_move, entity_id, 1);
        assert_eq!(cache.stats().hits, 1);

        // Large movement should miss cache
        let large_move = Vec3::new(15.0, 0.0, 0.0);
        cache.get_or_compute_distance_with_frame(camera_pos, large_move, entity_id, 2);
        assert_eq!(cache.stats().misses, 2);
    }

    #[test]
    fn test_distance_cache_accuracy() {
        let mut cache = DistanceCache::new();
        let camera_pos = Vec3::new(0.0, 0.0, 0.0);
        let entity_pos = Vec3::new(3.0, 4.0, 0.0); // 3-4-5 triangle
        let entity_id = 1;

        let distance = cache.get_or_compute_distance(camera_pos, entity_pos, entity_id);
        assert!((distance - 5.0).abs() < 0.01); // Should be 5.0 with <1cm error
    }

    #[test]
    fn test_cache_stats_display() {
        let mut cache = DistanceCache::new();
        let camera_pos = Vec3::new(0.0, 0.0, 0.0);
        let entity_pos = Vec3::new(10.0, 0.0, 0.0);
        let entity_id = 1;

        // Generate some stats
        cache.get_or_compute_distance(camera_pos, entity_pos, entity_id);
        cache.get_or_compute_distance(camera_pos, entity_pos, entity_id);

        let stats = cache.stats();
        let display = format!("{stats}");
        assert!(display.contains("hit rate"));
        assert!(display.contains("50.0%")); // 1 hit out of 2 total
    }

    #[test]
    fn test_distance_cache_clear() {
        let mut cache = DistanceCache::new();
        let camera_pos = Vec3::new(0.0, 0.0, 0.0);
        let entity_pos = Vec3::new(10.0, 0.0, 0.0);
        let entity_id = 1;

        cache.get_or_compute_distance(camera_pos, entity_pos, entity_id);
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.stats().misses, 1);

        cache.clear();
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.stats().misses, 0);
        assert_eq!(cache.stats().hits, 0);
    }
}
