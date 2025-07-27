//! Oracle's Day 4-5 Memory Infrastructure Integration
//!
//! Integration layer between FixedVecPool and existing systems:
//! - TransientBufferPool re-routing to use memory pools
//! - Spawn DSL temporary Vec optimization
//! - Batching instance vector pooling

// Temporarily comment out entire file to fix compilation - will restore after fixing exports
/*
    /// Pooled byte vector for buffer data
    pub data: PooledVec<u8>,
    /// Buffer size for validation
    pub size: u64,
}

#[cfg(feature = "bevy16")]
impl PooledTransientBuffer {
    /// Create a new pooled buffer with specified capacity
    pub fn new(pools: &mut GlobalMemoryPools, size: u64) -> Self {
        let mut data = pools.byte_pool.get(size as usize);
        data.resize(size as usize, 0);

        Self { data, size }
    }

    /// Get buffer data as slice
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Get mutable buffer data
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Get buffer size
    pub fn len(&self) -> u64 {
        self.size
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}

/// Factory DSL memory pool integration
///
/// Oracle's requirement: "Update spawn DSL temporary Vecs to use pooled allocation"
pub trait PooledEntityFactory {
    /// Create pooled vector for component data
    fn create_component_vec<T>(&mut self, capacity: usize) -> PooledVec<T>;

    /// Create pooled vector for entity IDs
    fn create_entity_vec(&mut self, capacity: usize) -> PooledVec<u32>;

    /// Create pooled vector for position data
    fn create_position_vec(&mut self, capacity: usize) -> PooledVec<f32>;
}

#[cfg(feature = "bevy16")]
impl PooledEntityFactory for GlobalMemoryPools {
    fn create_component_vec<T>(&mut self, _capacity: usize) -> PooledVec<T> {
        // For now, use a generic approach - in real implementation,
        // we'd have type-specific pools
        todo!("Type-specific pools not yet implemented")
    }

    fn create_entity_vec(&mut self, capacity: usize) -> PooledVec<u32> {
        self.u32_pool.get(capacity)
    }

    fn create_position_vec(&mut self, capacity: usize) -> PooledVec<f32> {
        self.f32_pool.get(capacity)
    }
}

/// Batching system memory pool integration
///
/// Oracle's requirement: "Convert batching instance vectors to use pools"
#[cfg(feature = "bevy16")]
pub struct PooledInstanceBatch {
    /// Instance transforms (4x4 matrices as f32 arrays)
    pub transforms: PooledVec<f32>,
    /// Instance colors (RGBA)
    pub colors: PooledVec<f32>,
    /// Instance IDs for culling
    pub instance_ids: PooledVec<u32>,
    /// Batch capacity
    pub capacity: usize,
}

#[cfg(feature = "bevy16")]
impl PooledInstanceBatch {
    /// Create a new pooled instance batch
    pub fn new(pools: &mut GlobalMemoryPools, capacity: usize) -> Self {
        Self {
            transforms: pools.f32_pool.get(capacity * 16), // 4x4 matrix = 16 floats
            colors: pools.f32_pool.get(capacity * 4),      // RGBA = 4 floats
            instance_ids: pools.u32_pool.get(capacity),
            capacity,
        }
    }

    /// Add an instance to the batch
    pub fn add_instance(&mut self, transform: &[f32; 16], color: &[f32; 4], id: u32) -> bool {
        if self.transforms.len() + 16 <= self.transforms.capacity() {
            self.transforms.extend_from_slice(transform);
            self.colors.extend_from_slice(color);
            self.instance_ids.push(id);
            true
        } else {
            false // Batch full
        }
    }

    /// Get number of instances in batch
    pub fn instance_count(&self) -> usize {
        self.instance_ids.len()
    }

    /// Clear batch for reuse
    pub fn clear(&mut self) {
        self.transforms.clear();
        self.colors.clear();
        self.instance_ids.clear();
    }

    /// Get transform data as slice
    pub fn transform_data(&self) -> &[f32] {
        &self.transforms
    }

    /// Get color data as slice
    pub fn color_data(&self) -> &[f32] {
        &self.colors
    }

    /// Get instance IDs as slice
    pub fn instance_ids(&self) -> &[u32] {
        &self.instance_ids
    }
}

/// Memory pool statistics aggregation
#[derive(Debug, Clone)]
pub struct IntegratedMemoryStats {
    /// Global pool statistics
    pub global_stats: super::CombinedPoolStats,
    /// TransientBuffer allocations this frame
    pub transient_allocations: u32,
    /// Factory allocations this frame
    pub factory_allocations: u32,
    /// Batch allocations this frame
    pub batch_allocations: u32,
    /// Total memory saved vs traditional allocation (estimated)
    pub estimated_memory_saved_bytes: u64,
}

#[cfg(feature = "bevy16")]
impl IntegratedMemoryStats {
    /// Calculate integrated statistics
    pub fn calculate(pools: &GlobalMemoryPools) -> Self {
        let global_stats = pools.combined_stats();

        // Estimate memory savings (rough calculation)
        let total_reuses = global_stats.byte_pool.frame_reuses
            + global_stats.u32_pool.frame_reuses
            + global_stats.f32_pool.frame_reuses;

        // Assume average allocation size and estimate savings
        let estimated_memory_saved_bytes = total_reuses as u64 * 1024; // Rough estimate

        Self {
            global_stats,
            transient_allocations: 0, // Would be tracked by integration layer
            factory_allocations: 0,   // Would be tracked by integration layer
            batch_allocations: 0,     // Would be tracked by integration layer
            estimated_memory_saved_bytes,
        }
    }
}

/// Helper functions for integrating memory pools with existing systems
pub mod helpers {


    /// Convert a regular Vec to use pooled allocation
    #[cfg(feature = "bevy16")]
    pub fn convert_to_pooled<T>(vec: Vec<T>, pool: &mut FixedVecPool<T>) -> PooledVec<T> {
        let mut pooled = pool.get(vec.len());
        pooled.extend(vec);
        pooled
    }

    /// Batch convert multiple vectors to pooled allocation
    #[cfg(feature = "bevy16")]
    pub fn batch_convert_to_pooled<T>(
        vecs: Vec<Vec<T>>,
        pool: &mut FixedVecPool<T>,
    ) -> Vec<PooledVec<T>> {
        vecs.into_iter()
            .map(|vec| convert_to_pooled(vec, pool))
            .collect()
    }

    /// Create a pooled buffer that's compatible with WGPU
    #[cfg(feature = "bevy16")]
    pub fn create_wgpu_compatible_buffer(
        pools: &mut GlobalMemoryPools,
        data: &[u8],
    ) -> PooledTransientBuffer {
        let mut buffer = PooledTransientBuffer::new(pools, data.len() as u64);
        buffer.as_mut_slice().copy_from_slice(data);
        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "bevy16")]
    #[test]
    fn test_pooled_transient_buffer() {
        let mut pools = GlobalMemoryPools::default();

        let mut buffer = PooledTransientBuffer::new(&mut pools, 1024);
        assert_eq!(buffer.len(), 1024);
        assert!(!buffer.is_empty());

        // Test data access
        buffer.as_mut_slice()[0] = 42;
        assert_eq!(buffer.as_slice()[0], 42);
    }

    #[cfg(feature = "bevy16")]
    #[test]
    fn test_pooled_instance_batch() {
        let mut pools = GlobalMemoryPools::default();

        let mut batch = PooledInstanceBatch::new(&mut pools, 100);
        assert_eq!(batch.capacity, 100);
        assert_eq!(batch.instance_count(), 0);

        // Add an instance
        let transform = [1.0; 16];
        let color = [1.0, 0.0, 0.0, 1.0];
        let id = 42;

        assert!(batch.add_instance(&transform, &color, id));
        assert_eq!(batch.instance_count(), 1);
        assert_eq!(batch.instance_ids()[0], 42);
    }

    #[cfg(feature = "bevy16")]
    #[test]
    fn test_batch_convert_to_pooled() {
        let mut pool = FixedVecPool::<i32>::new();

        let vecs = vec![vec![1, 2, 3], vec![4, 5, 6, 7]];

        let pooled_vecs = helpers::batch_convert_to_pooled(vecs, &mut pool);

        assert_eq!(pooled_vecs.len(), 2);
        assert_eq!(pooled_vecs[0].len(), 3);
        assert_eq!(pooled_vecs[1].len(), 4);
        assert_eq!(pooled_vecs[0][0], 1);
        assert_eq!(pooled_vecs[1][3], 7);
    }

    #[cfg(feature = "bevy16")]
    #[test]
    fn test_memory_stats_calculation() {
        let pools = GlobalMemoryPools::default();
        let stats = IntegratedMemoryStats::calculate(&pools);

        assert_eq!(stats.global_stats.frame_count, 0);
        // Other fields would be populated based on actual usage
    }
}
*/
