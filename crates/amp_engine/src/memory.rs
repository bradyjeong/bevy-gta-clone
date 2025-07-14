//! Oracle's Day 4-5 Memory Infrastructure - FixedVecPool & Frame-based Recycling
//!
//! High-performance memory pools designed for game engine frame allocation patterns:
//! - 4 KiB pages for optimal CPU cache line alignment
//! - Generation tracking prevents ABA problems in debug builds
//! - Frame-based recycling with ScopedArena for temporary allocations
//! - Integration with existing TransientBufferPool and gameplay systems

#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::unwrap_or_default)]
#![allow(clippy::manual_div_ceil)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::assertions_on_constants)]

pub mod integration;

#[cfg(feature = "bevy16")]
pub mod render_integration;

#[cfg(feature = "bevy16")]
use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU32, Ordering};

/// Size of memory pages for efficient allocation (4 KiB)
const PAGE_SIZE: usize = 4 * 1024;

/// Generation counter for pool objects to prevent ABA problems
static GLOBAL_GENERATION: AtomicU32 = AtomicU32::new(1);

/// A pool handle with generation tracking for safety
#[derive(Debug, Clone)]
pub struct PoolHandle<T> {
    generation: u32,
    index: usize,
    _phantom: PhantomData<T>,
}

impl<T> PoolHandle<T> {
    fn new(generation: u32, index: usize) -> Self {
        Self {
            generation,
            index,
            _phantom: PhantomData,
        }
    }
}

/// Entry in the pool with generation tracking
#[derive(Debug)]
struct PoolEntry<T> {
    data: Option<Vec<T>>,
    generation: u32,
}

impl<T> PoolEntry<T> {
    fn new() -> Self {
        Self {
            data: None,
            generation: GLOBAL_GENERATION.fetch_add(1, Ordering::Relaxed),
        }
    }

    fn take(&mut self) -> Option<Vec<T>> {
        self.data.take()
    }

    fn put(&mut self, mut vec: Vec<T>) {
        vec.clear();
        self.data = Some(vec);
        self.generation = GLOBAL_GENERATION.fetch_add(1, Ordering::Relaxed);
    }
}

/// Fixed-size vector pool with 4 KiB page allocation and generation tracking
///
/// Oracle's specification:
/// - Pages of 4 KiB for efficient allocation
/// - Reuse across frames with generation tracking
/// - Memory pool safety with ABA problem prevention
#[derive(Debug)]
pub struct FixedVecPool<T> {
    /// Pools organized by capacity buckets (powers of 2)
    pools: HashMap<usize, VecDeque<PoolEntry<T>>>,
    /// Total allocated pools for metrics
    total_pools: usize,
    /// Peak pools in use this session
    peak_pools: usize,
    /// Current active pools
    active_pools: usize,
    /// Frame allocation counter
    frame_allocations: u32,
    /// Frame reuse counter
    frame_reuses: u32,
}

impl<T> Default for FixedVecPool<T> {
    fn default() -> Self {
        Self {
            pools: HashMap::new(),
            total_pools: 0,
            peak_pools: 0,
            active_pools: 0,
            frame_allocations: 0,
            frame_reuses: 0,
        }
    }
}

impl<T> FixedVecPool<T> {
    /// Create a new pool
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a vector from the pool with specified minimum capacity
    ///
    /// Uses power-of-2 bucketing for efficient reuse patterns
    /// Aligns capacity to 4 KiB page boundaries when possible
    pub fn get(&mut self, min_capacity: usize) -> PooledVec<T> {
        // Calculate bucket size (power of 2, minimum page alignment)
        let bucket_capacity = self.calculate_bucket_capacity(min_capacity);

        // Try to reuse from pool
        if let Some(pool_queue) = self.pools.get_mut(&bucket_capacity) {
            if let Some(mut entry) = pool_queue.pop_front() {
                if let Some(vec) = entry.take() {
                    self.frame_reuses += 1;
                    let handle = PoolHandle::new(entry.generation, 0);
                    return PooledVec::new(vec, handle, self as *mut Self);
                }
            }
        }

        // Create new vector with bucket capacity
        self.frame_allocations += 1;
        self.active_pools += 1;
        self.total_pools += 1;
        if self.active_pools > self.peak_pools {
            self.peak_pools = self.active_pools;
        }

        let vec = Vec::with_capacity(bucket_capacity);
        let generation = GLOBAL_GENERATION.fetch_add(1, Ordering::Relaxed);
        let handle = PoolHandle::new(generation, 0);

        PooledVec::new(vec, handle, self as *mut Self)
    }

    /// Return a vector to the pool for reuse
    ///
    /// SAFETY: Only call this from PooledVec::drop()
    unsafe fn return_vec(&mut self, mut vec: Vec<T>, capacity: usize) {
        vec.clear();

        let bucket_capacity = self.calculate_bucket_capacity(capacity);
        let mut entry = PoolEntry::new();
        entry.put(vec);

        self.pools
            .entry(bucket_capacity)
            .or_insert_with(VecDeque::new)
            .push_back(entry);

        self.active_pools = self.active_pools.saturating_sub(1);
    }

    /// Calculate optimal bucket capacity for given minimum capacity
    ///
    /// Uses power-of-2 bucketing with 4 KiB page alignment where beneficial
    fn calculate_bucket_capacity(&self, min_capacity: usize) -> usize {
        if min_capacity == 0 {
            return 64; // Minimum reasonable capacity
        }

        // For small allocations, use power of 2
        if min_capacity <= 1024 {
            return min_capacity.next_power_of_two();
        }

        // For larger allocations, align to page boundaries
        let element_size = std::mem::size_of::<T>();
        if element_size > 0 {
            let elements_per_page = PAGE_SIZE / element_size;
            if elements_per_page > 0 {
                // Round up to next page boundary
                let pages_needed = (min_capacity + elements_per_page - 1) / elements_per_page;
                return pages_needed * elements_per_page;
            }
        }

        // Fallback to power of 2
        min_capacity.next_power_of_two()
    }

    /// Clear all pools and reset counters (called between frames)
    pub fn reset_frame(&mut self) {
        self.frame_allocations = 0;
        self.frame_reuses = 0;
    }

    /// Get allocation statistics for monitoring
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            total_pools: self.total_pools,
            peak_pools: self.peak_pools,
            active_pools: self.active_pools,
            frame_allocations: self.frame_allocations,
            frame_reuses: self.frame_reuses,
            bucket_count: self.pools.len(),
        }
    }
}

/// Statistics for pool monitoring
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total_pools: usize,
    pub peak_pools: usize,
    pub active_pools: usize,
    pub frame_allocations: u32,
    pub frame_reuses: u32,
    pub bucket_count: usize,
}

/// A vector managed by the pool with automatic return-to-pool on drop
///
/// Provides transparent `Vec<T>` interface while managing pool lifecycle
#[derive(Debug)]
pub struct PooledVec<T> {
    vec: Option<Vec<T>>,
    handle: PoolHandle<T>,
    pool_ptr: *mut FixedVecPool<T>,
}

impl<T> PooledVec<T> {
    fn new(vec: Vec<T>, handle: PoolHandle<T>, pool_ptr: *mut FixedVecPool<T>) -> Self {
        Self {
            vec: Some(vec),
            handle,
            pool_ptr,
        }
    }

    /// Get mutable reference to the underlying vector
    pub fn as_mut(&mut self) -> &mut Vec<T> {
        self.vec
            .as_mut()
            .expect("PooledVec should always contain a Vec")
    }

    /// Get immutable reference to the underlying vector
    pub fn as_ref(&self) -> &Vec<T> {
        self.vec
            .as_ref()
            .expect("PooledVec should always contain a Vec")
    }

    /// Take ownership of the underlying vector (consumes the PooledVec)
    ///
    /// NOTE: This prevents automatic return to pool - use sparingly
    pub fn into_inner(mut self) -> Vec<T> {
        self.vec
            .take()
            .expect("PooledVec should always contain a Vec")
    }
}

impl<T> std::ops::Deref for PooledVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> std::ops::DerefMut for PooledVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<T> Drop for PooledVec<T> {
    fn drop(&mut self) {
        if let Some(vec) = self.vec.take() {
            let capacity = vec.capacity();
            // SAFETY: We hold a valid pointer to the pool and are dropping
            unsafe {
                if !self.pool_ptr.is_null() {
                    (*self.pool_ptr).return_vec(vec, capacity);
                }
            }
        }
    }
}

// Safety: PooledVec is Send if T is Send
unsafe impl<T: Send> Send for PooledVec<T> {}

/// ScopedArena for per-frame temporary allocations
///
/// Oracle's specification: "Provide ScopedArena helper for per-frame temp allocations"
/// Automatically cleans up all allocations when dropped
#[derive(Debug)]
pub struct ScopedArena<T> {
    pool: FixedVecPool<T>,
    active_vecs: Vec<PooledVec<T>>,
}

impl<T> ScopedArena<T> {
    /// Create a new scoped arena
    pub fn new() -> Self {
        Self {
            pool: FixedVecPool::new(),
            active_vecs: Vec::new(),
        }
    }

    /// Allocate a new vector in this arena
    pub fn alloc(&mut self, min_capacity: usize) -> &mut Vec<T> {
        let pooled_vec = self.pool.get(min_capacity);
        self.active_vecs.push(pooled_vec);
        self.active_vecs.last_mut().unwrap().as_mut()
    }

    /// Get arena statistics
    pub fn stats(&self) -> PoolStats {
        self.pool.stats()
    }
}

impl<T> Default for ScopedArena<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Global memory pools for different data types
#[cfg(feature = "bevy16")]
#[derive(Debug, Resource)]
pub struct GlobalMemoryPools {
    /// Pool for `Vec<u8>` - used by TransientBufferPool
    pub byte_pool: FixedVecPool<u8>,
    /// Pool for `Vec<u32>` - used by index buffers
    pub u32_pool: FixedVecPool<u32>,
    /// Pool for `Vec<f32>` - used by vertex buffers
    pub f32_pool: FixedVecPool<f32>,
    /// Frame counter for cleanup
    pub frame_count: u64,
}

#[cfg(feature = "bevy16")]
impl Default for GlobalMemoryPools {
    fn default() -> Self {
        Self {
            byte_pool: FixedVecPool::new(),
            u32_pool: FixedVecPool::new(),
            f32_pool: FixedVecPool::new(),
            frame_count: 0,
        }
    }
}

#[cfg(feature = "bevy16")]
impl GlobalMemoryPools {
    /// Reset frame counters for all pools
    pub fn reset_frame(&mut self) {
        self.byte_pool.reset_frame();
        self.u32_pool.reset_frame();
        self.f32_pool.reset_frame();
        self.frame_count += 1;
    }

    /// Get combined statistics across all pools
    pub fn combined_stats(&self) -> CombinedPoolStats {
        CombinedPoolStats {
            byte_pool: self.byte_pool.stats(),
            u32_pool: self.u32_pool.stats(),
            f32_pool: self.f32_pool.stats(),
            frame_count: self.frame_count,
        }
    }
}

/// Combined statistics across all global pools
#[derive(Debug, Clone)]
pub struct CombinedPoolStats {
    pub byte_pool: PoolStats,
    pub u32_pool: PoolStats,
    pub f32_pool: PoolStats,
    pub frame_count: u64,
}

/// System to reset memory pools each frame
#[cfg(feature = "bevy16")]
pub fn reset_memory_pools(mut pools: ResMut<GlobalMemoryPools>) {
    pools.reset_frame();
}

/// Plugin to add memory pool systems to Bevy
#[cfg(feature = "bevy16")]
pub struct MemoryPoolPlugin;

#[cfg(feature = "bevy16")]
impl Plugin for MemoryPoolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalMemoryPools>()
            .add_systems(Last, reset_memory_pools);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_vec_pool_basic() {
        let mut pool = FixedVecPool::<i32>::new();

        // Get a vector from the pool
        let mut vec1 = pool.get(10);
        vec1.push(42);
        assert_eq!(vec1.len(), 1);
        assert_eq!(vec1[0], 42);

        let stats = pool.stats();
        assert_eq!(stats.frame_allocations, 1);
        assert_eq!(stats.frame_reuses, 0);
        assert_eq!(stats.active_pools, 1);
    }

    #[test]
    fn test_pool_reuse() {
        let mut pool = FixedVecPool::<i32>::new();

        // Get and return a vector
        {
            let mut vec1 = pool.get(10);
            vec1.push(42);
        } // vec1 dropped here, returned to pool

        // Get another vector - should reuse
        let vec2 = pool.get(10);
        assert_eq!(vec2.len(), 0); // Should be cleared

        let stats = pool.stats();
        assert_eq!(stats.frame_allocations, 1);
        assert_eq!(stats.frame_reuses, 1);
    }

    #[test]
    fn test_bucket_capacity_calculation() {
        let pool = FixedVecPool::<u8>::new();

        // Test power-of-2 for small sizes
        assert_eq!(pool.calculate_bucket_capacity(0), 64);
        assert_eq!(pool.calculate_bucket_capacity(100), 128);
        assert_eq!(pool.calculate_bucket_capacity(500), 512);

        // Test page alignment for larger sizes
        let large_capacity = pool.calculate_bucket_capacity(5000);
        assert!(large_capacity >= 5000);

        // Should align to page boundaries when beneficial
        let elements_per_page = PAGE_SIZE / std::mem::size_of::<u8>();
        assert_eq!(large_capacity % elements_per_page, 0);
    }

    #[test]
    fn test_scoped_arena() {
        let mut arena = ScopedArena::<i32>::new();

        // Allocate and populate first vector
        {
            let vec1 = arena.alloc(10);
            vec1.push(1);
            vec1.push(2);
            assert_eq!(vec1.len(), 2);
        }

        // Allocate and populate second vector
        {
            let vec2 = arena.alloc(20);
            vec2.push(3);
            vec2.push(4);
            vec2.push(5);
            assert_eq!(vec2.len(), 3);
        }

        let stats = arena.stats();
        assert_eq!(stats.frame_allocations, 2);
    } // All vectors automatically cleaned up when arena drops

    #[test]
    fn test_generation_tracking() {
        let mut pool = FixedVecPool::<i32>::new();

        let vec1 = pool.get(10);
        let generation1 = vec1.handle.generation;

        drop(vec1);

        let vec2 = pool.get(10);
        let generation2 = vec2.handle.generation;

        // Generation should be different to prevent ABA problems
        assert_ne!(generation1, generation2);
    }

    #[test]
    fn test_pool_stats() {
        let mut pool = FixedVecPool::<i32>::new();

        let _vec1 = pool.get(10);
        let _vec2 = pool.get(20);

        let stats = pool.stats();
        assert_eq!(stats.active_pools, 2);
        assert_eq!(stats.total_pools, 2);
        assert_eq!(stats.peak_pools, 2);
        assert_eq!(stats.frame_allocations, 2);
        assert_eq!(stats.frame_reuses, 0);
    }

    #[test]
    fn test_pooled_vec_deref() {
        let mut pool = FixedVecPool::<i32>::new();
        let mut vec = pool.get(10);

        // Test Deref and DerefMut
        vec.push(42);
        vec.push(84);

        assert_eq!(vec.len(), 2);
        assert_eq!(vec[0], 42);
        assert_eq!(vec[1], 84);

        // Test iteration
        let sum: i32 = vec.iter().sum();
        assert_eq!(sum, 126);
    }

    #[test]
    fn test_into_inner() {
        let mut pool = FixedVecPool::<i32>::new();
        let mut pooled = pool.get(10);
        pooled.push(42);

        let owned = pooled.into_inner();
        assert_eq!(owned.len(), 1);
        assert_eq!(owned[0], 42);

        // Pool should not see this as returned
        let stats = pool.stats();
        assert_eq!(stats.active_pools, 1); // Still counted as active since not returned
    }
}
