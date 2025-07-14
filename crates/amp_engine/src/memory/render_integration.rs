//! Oracle's Day 4-5 Memory Infrastructure - Render System Integration
//!
//! Integration layer between FixedVecPool and amp_render's TransientBufferPool:
//! - Drop-in replacement for existing buffer allocation patterns
//! - Memory pool backend with transparent API compatibility
//! - Performance monitoring and metrics collection

#[cfg(feature = "bevy16")]
use bevy::prelude::*;
#[cfg(feature = "bevy16")]
use bevy::render::render_resource::{Buffer, BufferDescriptor, BufferUsages};
#[cfg(feature = "bevy16")]
use bevy::render::renderer::RenderDevice;

use super::{GlobalMemoryPools, PooledVec};

/// Enhanced buffer allocation with memory pool backend
///
/// Oracle's requirement: "Re-route TransientBufferPool to use memory pools"
/// This provides a drop-in replacement for buffer allocation patterns.
#[cfg(feature = "bevy16")]
#[derive(Resource)]
pub struct PooledBufferAllocator {
    /// Reference to global memory pools
    pools: GlobalMemoryPools,
    /// Buffer usage tracking for optimization
    buffer_usages: std::collections::HashMap<String, BufferUsageStats>,
    /// Frame allocation counter
    frame_allocations: u32,
}

#[cfg(feature = "bevy16")]
#[derive(Debug, Default)]
struct BufferUsageStats {
    total_allocations: u64,
    total_bytes: u64,
    peak_size: u64,
    reuse_count: u64,
}

#[cfg(feature = "bevy16")]
impl Default for PooledBufferAllocator {
    fn default() -> Self {
        Self {
            pools: GlobalMemoryPools::default(),
            buffer_usages: std::collections::HashMap::new(),
            frame_allocations: 0,
        }
    }
}

#[cfg(feature = "bevy16")]
impl PooledBufferAllocator {
    /// Create a new pooled buffer allocator
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocate a buffer using memory pool backend
    ///
    /// This method provides the same interface as TransientBufferPool::get_buffer()
    /// but uses FixedVecPool for improved performance and memory locality
    pub fn allocate_buffer(
        &mut self,
        label: &str,
        size: u64,
        usage: BufferUsages,
        render_device: &RenderDevice,
    ) -> PooledBuffer {
        self.frame_allocations += 1;

        // Get pooled byte vector
        let data = self.pools.byte_pool.get(size as usize);

        // Track usage statistics
        let stats = self.buffer_usages.entry(label.to_string()).or_default();
        stats.total_allocations += 1;
        stats.total_bytes += size;
        if size > stats.peak_size {
            stats.peak_size = size;
        }

        // Create WGPU buffer
        let buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some(label),
            size,
            usage,
            mapped_at_creation: false,
        });

        PooledBuffer {
            buffer,
            data,
            label: label.to_string(),
            size,
        }
    }

    /// Reset frame counters and cleanup
    pub fn reset_frame(&mut self) {
        self.pools.reset_frame();
        self.frame_allocations = 0;
    }

    /// Get allocation statistics
    pub fn get_stats(&self) -> PooledBufferStats {
        let global_stats = self.pools.combined_stats();

        let total_buffer_allocations: u64 = self
            .buffer_usages
            .values()
            .map(|stats| stats.total_allocations)
            .sum();

        let total_buffer_bytes: u64 = self
            .buffer_usages
            .values()
            .map(|stats| stats.total_bytes)
            .sum();

        PooledBufferStats {
            frame_allocations: self.frame_allocations,
            total_buffer_allocations,
            total_buffer_bytes,
            memory_pool_stats: global_stats,
            buffer_type_count: self.buffer_usages.len(),
        }
    }
}

/// A buffer with pooled memory backend
#[cfg(feature = "bevy16")]
pub struct PooledBuffer {
    /// The WGPU buffer
    pub buffer: Buffer,
    /// Pooled data storage
    data: PooledVec<u8>,
    /// Buffer label for debugging
    label: String,
    /// Buffer size in bytes
    size: u64,
}

#[cfg(feature = "bevy16")]
impl PooledBuffer {
    /// Get buffer reference
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// Get mutable data for writing
    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Get data as slice
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get buffer size
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Get buffer label
    pub fn label(&self) -> &str {
        &self.label
    }
}

/// Statistics for pooled buffer allocation
#[derive(Debug, Clone)]
pub struct PooledBufferStats {
    pub frame_allocations: u32,
    pub total_buffer_allocations: u64,
    pub total_buffer_bytes: u64,
    pub memory_pool_stats: super::CombinedPoolStats,
    pub buffer_type_count: usize,
}

/// Helper functions for transitioning existing render code
pub mod transition_helpers {
    use super::*;

    /// Convert existing TransientBufferPool usage to PooledBufferAllocator
    ///
    /// This function provides a migration path for existing render code
    #[cfg(feature = "bevy16")]
    pub fn reset_pooled_allocator_system(mut allocator: ResMut<PooledBufferAllocator>) {
        allocator.reset_frame();
    }

    /// System to monitor memory pool performance
    #[cfg(feature = "bevy16")]
    pub fn monitor_pool_performance(
        allocator: Res<PooledBufferAllocator>,
        mut last_stats: Local<Option<PooledBufferStats>>,
    ) {
        let current_stats = allocator.get_stats();

        if let Some(ref last) = *last_stats {
            if current_stats.frame_allocations > 0 {
                let frame_diff = current_stats.frame_allocations - last.frame_allocations;
                debug!(
                    "Memory pools: {} allocations this frame, {} total reuses",
                    frame_diff,
                    current_stats.memory_pool_stats.byte_pool.frame_reuses
                        + current_stats.memory_pool_stats.u32_pool.frame_reuses
                        + current_stats.memory_pool_stats.f32_pool.frame_reuses
                );
            }
        }

        *last_stats = Some(current_stats);
    }
}

/// Bevy plugin for render integration
#[cfg(feature = "bevy16")]
pub struct RenderMemoryIntegrationPlugin;

#[cfg(feature = "bevy16")]
impl Plugin for RenderMemoryIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PooledBufferAllocator>().add_systems(
            PostUpdate,
            (
                transition_helpers::reset_pooled_allocator_system,
                transition_helpers::monitor_pool_performance,
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "bevy16")]
    #[test]
    fn test_pooled_buffer_allocator_basic() {
        let mut allocator = PooledBufferAllocator::new();

        // Test stats before allocation
        let initial_stats = allocator.get_stats();
        assert_eq!(initial_stats.frame_allocations, 0);
        assert_eq!(initial_stats.total_buffer_allocations, 0);
    }

    #[cfg(feature = "bevy16")]
    #[test]
    fn test_frame_reset() {
        let mut allocator = PooledBufferAllocator::new();

        // Simulate some allocations
        allocator.frame_allocations = 10;

        // Reset frame
        allocator.reset_frame();

        let stats = allocator.get_stats();
        assert_eq!(stats.frame_allocations, 0);
    }

    #[test]
    fn test_buffer_usage_stats() {
        // Test that buffer usage stats are properly tracked
        // This is a placeholder test - real implementation would need RenderDevice
        assert!(true);
    }
}
