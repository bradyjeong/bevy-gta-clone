//! GPU buffer management for compute culling
//!
//! Manages GPU buffers for instance data, visibility results, and indirect draw commands.

use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::render::renderer::{RenderDevice, RenderQueue};
use wgpu::util::DeviceExt;

use super::{GpuCameraData, GpuCullingParams, GpuInstanceData};

/// Manages GPU buffers for compute culling pipeline with double-buffering
#[derive(Resource)]
pub struct GpuCullingBuffers {
    /// Instance data buffer (read-only in compute shader)
    pub instance_buffer: Buffer,
    /// Camera data uniform buffer
    pub camera_buffer: Buffer,
    /// Culling parameters uniform buffer
    pub params_buffer: Buffer,
    /// Visibility bitset output buffer
    pub visibility_buffer: Buffer,
    /// Visible count output buffer (atomic counter)
    pub count_buffer: Buffer,
    /// Double-buffered staging buffers for async readback (eliminates GPU→CPU sync stalls)
    pub staging_buffers: [Buffer; 2],
    /// Bind group for compute shader
    pub bind_group: Option<BindGroup>,
    /// Current capacity in instances
    pub capacity: u32,
}

impl GpuCullingBuffers {
    /// Create new GPU culling buffers with specified capacity
    pub fn new(device: &RenderDevice, capacity: u32) -> Self {
        // Calculate buffer sizes
        let instance_buffer_size = capacity * std::mem::size_of::<GpuInstanceData>() as u32;
        let camera_buffer_size = std::mem::size_of::<GpuCameraData>() as u64;
        let params_buffer_size = std::mem::size_of::<GpuCullingParams>() as u64;
        let visibility_buffer_size = capacity.div_ceil(32) * 4; // Bitset: 1 bit per instance, packed in u32s
        let count_buffer_size = 4; // Single u32 atomic counter

        // Create instance data buffer
        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("gpu_culling_instance_buffer"),
            size: instance_buffer_size as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create camera uniform buffer
        let camera_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("gpu_culling_camera_buffer"),
            size: camera_buffer_size,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create culling parameters uniform buffer
        let params_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("gpu_culling_params_buffer"),
            size: params_buffer_size,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create visibility bitset buffer
        let visibility_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("gpu_culling_visibility_buffer"),
            size: visibility_buffer_size as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create visible count buffer
        let count_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("gpu_culling_count_buffer"),
            size: count_buffer_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create double-buffered staging buffers for async readback
        let staging_buffers = [
            device.create_buffer(&BufferDescriptor {
                label: Some("gpu_culling_staging_buffer_0"),
                size: count_buffer_size,
                usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            device.create_buffer(&BufferDescriptor {
                label: Some("gpu_culling_staging_buffer_1"),
                size: count_buffer_size,
                usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        ];

        Self {
            instance_buffer,
            camera_buffer,
            params_buffer,
            visibility_buffer,
            count_buffer,
            staging_buffers,
            bind_group: None,
            capacity,
        }
    }

    /// Create bind group for compute shader
    pub fn create_bind_group(&mut self, device: &RenderDevice, layout: &BindGroupLayout) {
        let bind_group = device.create_bind_group(
            "gpu_culling_bind_group",
            layout,
            &BindGroupEntries::sequential((
                // Binding 0: Camera data
                self.camera_buffer.as_entire_binding(),
                // Binding 1: Instance data
                self.instance_buffer.as_entire_binding(),
                // Binding 2: Culling parameters
                self.params_buffer.as_entire_binding(),
                // Binding 3: Visibility bitset
                self.visibility_buffer.as_entire_binding(),
                // Binding 4: Visible count
                self.count_buffer.as_entire_binding(),
            )),
        );

        self.bind_group = Some(bind_group);
    }

    /// Upload instance data to GPU
    pub fn upload_instances(&self, queue: &RenderQueue, instances: &[GpuInstanceData]) {
        let data = bytemuck::cast_slice(instances);
        queue.write_buffer(&self.instance_buffer, 0, data);
    }

    /// Upload camera data to GPU
    pub fn upload_camera_data(&self, queue: &RenderQueue, camera_data: &GpuCameraData) {
        let data = bytemuck::bytes_of(camera_data);
        queue.write_buffer(&self.camera_buffer, 0, data);
    }

    /// Upload culling parameters to GPU
    pub fn upload_params(&self, queue: &RenderQueue, params: &GpuCullingParams) {
        let data = bytemuck::bytes_of(params);
        queue.write_buffer(&self.params_buffer, 0, data);
    }

    /// Clear visibility buffers before culling
    pub fn clear_visibility(&self, encoder: &mut CommandEncoder) {
        // Clear visibility bitset to all zeros
        let visibility_size = self.capacity.div_ceil(32) * 4;
        encoder.clear_buffer(&self.visibility_buffer, 0, Some(visibility_size as u64));

        // Clear visible count to zero
        encoder.clear_buffer(&self.count_buffer, 0, Some(4));
    }

    /// Get staging buffer for non-blocking GPU culling result readback
    ///
    /// Returns the current frame's staging buffer for asynchronous reading.
    /// Uses double-buffering to eliminate GPU→CPU sync stalls by maintaining
    /// two result buffers and swapping between frames.
    pub fn get_result_staging_buffer(&self, frame_index: usize) -> &Buffer {
        &self.staging_buffers[frame_index % 2]
    }

    /// Initiate asynchronous copy of culling results to staging buffer
    ///
    /// This does NOT block the GPU pipeline. The results will be available
    /// in the next frame via async readback without causing frame drops.
    pub fn begin_async_readback(
        &self,
        device: &RenderDevice,
        encoder: &mut CommandEncoder,
        frame_index: usize,
    ) {
        let staging_buffer = &self.staging_buffers[frame_index % 2];
        encoder.copy_buffer_to_buffer(&self.count_buffer, 0, staging_buffer, 0, 4);
    }

    /// Check if buffers can accommodate the given instance count
    pub fn can_accommodate(&self, instance_count: u32) -> bool {
        instance_count <= self.capacity
    }

    /// Resize buffers if needed
    pub fn ensure_capacity(&mut self, device: &RenderDevice, required_capacity: u32) {
        if required_capacity > self.capacity {
            // Recreate buffers with larger capacity
            let new_capacity = required_capacity.next_power_of_two().max(1024);
            *self = Self::new(device, new_capacity);
            info!("Resized GPU culling buffers to capacity: {}", new_capacity);
        }
    }
}

/// GPU culling indirect draw commands
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct IndirectDrawCommand {
    /// Vertex count per instance
    pub vertex_count: u32,
    /// Number of instances to draw (filled by compute shader)
    pub instance_count: u32,
    /// First vertex index
    pub first_vertex: u32,
    /// First instance index
    pub first_instance: u32,
}

impl Default for IndirectDrawCommand {
    fn default() -> Self {
        Self {
            vertex_count: 0,
            instance_count: 0,
            first_vertex: 0,
            first_instance: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indirect_draw_command_size() {
        assert_eq!(std::mem::size_of::<IndirectDrawCommand>(), 16);
        assert_eq!(std::mem::align_of::<IndirectDrawCommand>(), 4);
    }

    #[test]
    fn test_buffer_capacity_calculation() {
        let instance_count = 1000u32;
        let visibility_buffer_size = instance_count.div_ceil(32) * 4;

        // Should be at least 1000/32 = 32 words (128 bytes)
        assert!(visibility_buffer_size >= 128);
        // Should not be more than 33 words (132 bytes)
        assert!(visibility_buffer_size <= 132);
    }
}
