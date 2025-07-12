//! Simplified GPU culling system interface
//!
//! Provides the data structures and configuration for Oracle's GPU culling
//! system while avoiding complex Bevy render pipeline integration for now.

use bevy::prelude::*;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};

/// GPU instance data layout (80 bytes, aligned for GPU efficiency)
///
/// Optimized struct for WGSL compute shader consumption with
/// aligned memory layout for maximum GPU throughput.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuInstance {
    /// Model matrix (64 bytes)
    pub model_matrix: [[f32; 4]; 4],
    /// Bounding sphere center (12 bytes)
    pub center: [f32; 3],
    /// Bounding sphere radius (4 bytes)  
    pub radius: f32,
    /// Batch identifier (4 bytes)
    pub batch_id: u32,
    /// Padding to reach 80-byte alignment
    pub _padding: [u32; 3],
}

impl GpuInstance {
    /// Create GPU instance from transform and batch data
    pub fn new(transform: Mat4, center: Vec3, radius: f32, batch_id: u32) -> Self {
        Self {
            model_matrix: transform.to_cols_array_2d(),
            center: center.to_array(),
            radius,
            batch_id,
            _padding: [0; 3],
        }
    }
}

/// GPU draw indirect parameters (std430 layout)
///
/// Compatible with DrawIndirect commands for GPU-driven rendering
/// with atomic instance count updates from compute shader.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct DrawIndirect {
    /// Number of vertices in mesh
    pub vertex_count: u32,
    /// Number of visible instances (updated by compute shader)
    pub instance_count: u32,
    /// First vertex index
    pub first_vertex: u32,
    /// Base instance offset
    pub base_instance: u32,
}

impl DrawIndirect {
    pub fn new(vertex_count: u32, base_instance: u32) -> Self {
        Self {
            vertex_count,
            instance_count: 0,
            first_vertex: 0,
            base_instance,
        }
    }
}

/// Frustum planes for GPU culling (96 bytes)
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuFrustum {
    /// 6 frustum planes (left, right, bottom, top, near, far)
    pub planes: [[f32; 4]; 6],
}

impl GpuFrustum {
    /// Extract frustum planes from view-projection matrix
    pub fn from_view_projection(view_proj: Mat4) -> Self {
        let m = view_proj.to_cols_array_2d();

        Self {
            planes: [
                // Left plane
                [
                    m[3][0] + m[0][0],
                    m[3][1] + m[0][1],
                    m[3][2] + m[0][2],
                    m[3][3] + m[0][3],
                ],
                // Right plane
                [
                    m[3][0] - m[0][0],
                    m[3][1] - m[0][1],
                    m[3][2] - m[0][2],
                    m[3][3] - m[0][3],
                ],
                // Bottom plane
                [
                    m[3][0] + m[1][0],
                    m[3][1] + m[1][1],
                    m[3][2] + m[1][2],
                    m[3][3] + m[1][3],
                ],
                // Top plane
                [
                    m[3][0] - m[1][0],
                    m[3][1] - m[1][1],
                    m[3][2] - m[1][2],
                    m[3][3] - m[1][3],
                ],
                // Near plane
                [
                    m[3][0] + m[2][0],
                    m[3][1] + m[2][1],
                    m[3][2] + m[2][2],
                    m[3][3] + m[2][3],
                ],
                // Far plane
                [
                    m[3][0] - m[2][0],
                    m[3][1] - m[2][1],
                    m[3][2] - m[2][2],
                    m[3][3] - m[2][3],
                ],
            ],
        }
    }
}

/// GPU culling uniforms
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CullingUniforms {
    /// Frustum data
    pub frustum: GpuFrustum,
    /// Maximum culling distance
    pub max_distance: f32,
    /// Instance count for bounds checking
    pub instance_count: u32,
    /// Batch count for processing
    pub batch_count: u32,
    /// Enable flags (frustum, distance culling)
    pub enable_flags: u32,
}

/// Configuration for GPU culling
#[derive(Clone, Resource)]
pub struct GpuCullingConfig {
    /// Enable GPU culling (vs CPU fallback)
    pub enable_gpu_culling: bool,
    /// Maximum instances per dispatch
    pub max_instances: u32,
    /// Enable frustum culling
    pub enable_frustum_culling: bool,
    /// Enable distance culling
    pub enable_distance_culling: bool,
    /// Maximum render distance
    pub max_distance: f32,
}

impl Default for GpuCullingConfig {
    fn default() -> Self {
        Self {
            enable_gpu_culling: false, // Default to CPU fallback for simplicity
            max_instances: 10000,
            enable_frustum_culling: true,
            enable_distance_culling: true,
            max_distance: 1000.0,
        }
    }
}

/// Mock GPU culling system that currently falls back to CPU
///
/// This provides the interface for Oracle's GPU culling system
/// while implementation details can be added in future iterations.
pub fn gpu_culling_system(
    config: Res<GpuCullingConfig>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    instances: Query<(&crate::ExtractedInstance, &crate::culling::Cullable)>,
) {
    if !config.enable_gpu_culling {
        // Fall back to CPU culling (handled by culling_integration)
        return;
    }

    // Get camera data for frustum extraction
    let Some((_camera, camera_transform)) = cameras.iter().next() else {
        return;
    };

    // Extract frustum from camera
    let view = camera_transform.compute_matrix().inverse();
    let projection = Mat4::perspective_lh(
        std::f32::consts::FRAC_PI_4, // Default FOV
        16.0 / 9.0,                  // Default aspect ratio
        0.1,                         // Near
        1000.0,                      // Far
    );
    let view_proj = projection * view;
    let _frustum = GpuFrustum::from_view_projection(view_proj);

    // Collect instances into GPU format
    let mut gpu_instances = Vec::new();
    let mut current_batch_id = 0u32;

    for (instance, cullable) in instances.iter() {
        if !instance.visible {
            continue;
        }

        let center = instance.transform.w_axis.truncate();
        let gpu_instance = GpuInstance::new(
            instance.transform,
            center,
            cullable.radius,
            current_batch_id,
        );
        gpu_instances.push(gpu_instance);
        current_batch_id += 1;
    }

    if gpu_instances.is_empty() {
        return;
    }

    // TODO: Actual GPU dispatch would happen here
    info!(
        "GPU culling mock: {} instances processed (target: <0.2ms)",
        gpu_instances.len()
    );
}

/// Plugin for GPU culling system
pub struct GpuCullingPlugin;

impl Plugin for GpuCullingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GpuCullingConfig>().add_systems(
            PostUpdate,
            gpu_culling_system.before(crate::culling_integration::integrated_culling_system),
        );
    }
}

/// Re-exports for convenience
pub mod prelude {
    pub use crate::gpu_culling_simple::{
        CullingUniforms, DrawIndirect, GpuCullingConfig, GpuCullingPlugin, GpuFrustum, GpuInstance,
        gpu_culling_system,
    };
}
