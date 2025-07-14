//! GPU-based compute shader culling implementation
//!
//! This module provides high-performance GPU culling using compute shaders
//! for large-scale entity batching scenarios (100k+ entities).
//!
//! Feature-gated behind `gpu_culling` - falls back to CPU culling when disabled.

#[cfg(feature = "gpu_culling")]
pub mod buffers;

#[cfg(feature = "gpu_culling")]
pub mod compute;

#[cfg(feature = "gpu_culling")]
pub mod real_compute;

#[cfg(feature = "gpu_culling")]
pub mod render_graph_minimal;

#[cfg(feature = "gpu_culling")]
mod integration_test_simple;

#[cfg(feature = "gpu_culling")]
pub mod integration_test;

#[cfg(feature = "gpu_culling")]
pub use compute::*;

#[cfg(feature = "gpu_culling")]
pub use render_graph_minimal as render_graph;

#[cfg(feature = "gpu_culling")]
pub use render_graph_minimal::*;

use bevy::prelude::*;

/// GPU culling configuration and parameters
#[derive(Debug, Clone, Resource)]
pub struct GpuCullingConfig {
    /// Maximum instances per compute shader dispatch
    pub max_instances_per_dispatch: u32,
    /// Workgroup size for compute shader (must match shader)
    pub workgroup_size: u32,
    /// Enable debug output from GPU culling
    pub debug_output: bool,
    /// Enable frustum culling
    pub enable_frustum_culling: bool,
}

impl Default for GpuCullingConfig {
    fn default() -> Self {
        Self {
            max_instances_per_dispatch: 400_000, // Oracle's Phase 3 specification: 400k instances target
            workgroup_size: 64, // Oracle's Phase 3 specification: workgroup size 64
            debug_output: false,
            enable_frustum_culling: true,
        }
    }
}

/// GPU culling statistics for performance monitoring
#[derive(Debug, Clone, Default, Resource)]
pub struct GpuCullingStats {
    /// Total instances processed by GPU culling
    pub instances_processed: u32,
    /// Instances remaining after culling
    pub instances_visible: u32,
    /// GPU culling time in milliseconds
    pub gpu_time_ms: f32,
    /// CPU→GPU transfer time in milliseconds  
    pub upload_time_ms: f32,
    /// GPU→CPU result readback time in milliseconds
    pub readback_time_ms: f32,
}

impl GpuCullingStats {
    /// Calculate culling efficiency (percentage of instances culled)
    pub fn culling_efficiency(&self) -> f32 {
        if self.instances_processed == 0 {
            0.0
        } else {
            1.0 - (self.instances_visible as f32 / self.instances_processed as f32)
        }
    }

    /// Total GPU culling pipeline time
    pub fn total_time_ms(&self) -> f32 {
        self.gpu_time_ms + self.upload_time_ms + self.readback_time_ms
    }
}

/// GPU culling availability check
pub fn is_gpu_culling_available() -> bool {
    cfg!(feature = "gpu_culling")
}

/// GPU culling feature status for runtime queries
pub fn gpu_culling_status() -> &'static str {
    if cfg!(feature = "gpu_culling") {
        "Available"
    } else {
        "Disabled (feature not enabled)"
    }
}

/// Plugin for GPU culling system
pub struct GpuCullingPlugin;

impl Plugin for GpuCullingPlugin {
    fn build(&self, app: &mut App) {
        info!("GPU culling feature enabled");

        #[cfg(feature = "gpu_culling")]
        {
            app.init_resource::<GpuCullingConfig>();
            app.init_resource::<GpuCullingStats>();
            // TODO: Implement proper RenderWorld integration for setup_gpu_culling
            // app.add_systems(Startup, compute::setup_gpu_culling);
            // app.add_systems(PostUpdate, real_compute::run_real_gpu_culling);
        }

        #[cfg(not(feature = "gpu_culling"))]
        {
            warn!("GPU culling plugin added but feature not enabled");
        }
    }
}

/// Re-exports for convenience
pub mod prelude {
    pub use crate::gpu_culling::{
        gpu_culling_status, is_gpu_culling_available, GpuCullingConfig, GpuCullingPlugin,
        GpuCullingStats,
    };

    #[cfg(feature = "gpu_culling")]
    pub use crate::gpu_culling::compute::{
        GpuCameraData, GpuCullingParams, GpuCullingPipeline, GpuCullingResources, GpuInstanceData,
    };

    #[cfg(feature = "gpu_culling")]
    pub use crate::gpu_culling::render_graph_minimal::{
        is_gpu_culling_pipeline_available, GpuCullNode, GpuCullingLabel, GpuCullingPipelinePlugin,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_culling_config_default() {
        let config = GpuCullingConfig::default();
        assert_eq!(config.max_instances_per_dispatch, 400_000); // Oracle's Phase 3 specification
        assert_eq!(config.workgroup_size, 64); // Oracle's Phase 3 specification
        assert!(!config.debug_output);
    }

    #[test]
    fn test_gpu_culling_stats_efficiency() {
        let stats = GpuCullingStats {
            instances_processed: 1000,
            instances_visible: 200,
            ..Default::default()
        };

        assert_eq!(stats.culling_efficiency(), 0.8); // 80% culled

        // Edge case: no instances processed
        let stats_empty = GpuCullingStats {
            instances_processed: 0,
            ..Default::default()
        };
        assert_eq!(stats_empty.culling_efficiency(), 0.0);
    }

    #[test]
    fn test_gpu_culling_stats_total_time() {
        let stats = GpuCullingStats {
            gpu_time_ms: 0.1,
            upload_time_ms: 0.05,
            readback_time_ms: 0.03,
            ..Default::default()
        };

        assert_eq!(stats.total_time_ms(), 0.18);
    }

    #[test]
    fn test_gpu_culling_availability() {
        // This will be true when gpu_culling feature is enabled
        let available = is_gpu_culling_available();
        assert_eq!(available, cfg!(feature = "gpu_culling"));
    }

    #[test]
    fn test_gpu_culling_status() {
        let status = gpu_culling_status();
        if cfg!(feature = "gpu_culling") {
            assert_eq!(status, "Available");
        } else {
            assert_eq!(status, "Disabled (feature not enabled)");
        }
    }
}
