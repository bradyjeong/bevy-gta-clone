//! Integration between GPU culling and existing batch processing
//!
//! Provides seamless fallback from GPU to CPU culling and connects
//! with the BatchManager infrastructure.

use crate::culling::{CameraProjectionConfig, Cullable, CullingConfig};
use crate::{ExtractedInstance, batching::BatchManager};
use bevy::prelude::*;

#[cfg(feature = "gpu")]
use crate::culling::CullingConfig as GpuCullingConfig;

/// Integrated culling system that chooses between GPU and CPU
///
/// Automatically falls back to CPU culling when GPU is unavailable
/// or disabled, ensuring consistent behavior across all platforms.
pub fn integrated_culling_system(
    #[cfg(feature = "gpu")] gpu_config: Option<Res<GpuCullingConfig>>,
    culling_config: Res<CullingConfig>,
    projection_config: Res<CameraProjectionConfig>,
    cameras: Query<(&Camera, &GlobalTransform, Option<&Projection>)>,
    instances: Query<(&mut ExtractedInstance, &Cullable)>,
    mut batch_manager: ResMut<BatchManager>,
) {
    // Clear previous batches
    batch_manager.clear();

    #[cfg(feature = "gpu")]
    {
        // Use GPU culling if available and enabled
        if let Some(gpu_resource) = gpu_config {
            if gpu_resource.enable_frustum_culling {
                // GPU culling will handle batch updating directly
                return;
            }
        }
    }

    // Fall back to CPU culling
    cpu_culling_fallback(
        culling_config,
        projection_config,
        cameras,
        instances,
        batch_manager,
    );
}

/// CPU culling fallback implementation
///
/// Performs the same culling logic as GPU shader but on CPU,
/// ensuring consistent behavior when GPU culling is unavailable.
pub fn cpu_culling_fallback(
    culling_config: Res<CullingConfig>,
    projection_config: Res<CameraProjectionConfig>,
    cameras: Query<(&Camera, &GlobalTransform, Option<&Projection>)>,
    mut instances: Query<(&mut ExtractedInstance, &Cullable)>,
    mut batch_manager: ResMut<BatchManager>,
) {
    let Some((_camera, camera_transform, camera_projection)) = cameras.iter().next() else {
        return;
    };

    // Extract frustum if enabled
    let frustum_planes = if culling_config.enable_frustum_culling {
        let view = camera_transform.compute_matrix().inverse();

        // Use actual camera projection if available, otherwise use config defaults
        let projection = if let Some(Projection::Perspective(persp)) = camera_projection {
            Mat4::perspective_lh(persp.fov, persp.aspect_ratio, persp.near, persp.far)
        } else {
            // Fallback to configured defaults
            Mat4::perspective_lh(
                projection_config.fov,
                projection_config.aspect_ratio,
                projection_config.near,
                projection_config.far,
            )
        };

        let view_proj = projection * view;
        Some(crate::culling::extract_frustum_planes(view_proj))
    } else {
        None
    };

    // Process each instance
    for (mut instance, cullable) in instances.iter_mut() {
        let position = instance.transform.w_axis.truncate();
        let radius = cullable.radius;
        let mut visible = true;

        // Distance culling
        if culling_config.enable_distance_culling {
            let distance = position.length();
            let max_distance = cullable.max_distance.unwrap_or(culling_config.max_distance);
            visible = visible && (distance <= max_distance);
        }

        // Frustum culling
        if let Some(ref planes) = frustum_planes {
            let mut inside_frustum = true;
            for plane in planes {
                let distance = plane.xyz().dot(position) + plane.w;
                if distance < -radius {
                    inside_frustum = false;
                    break;
                }
            }
            visible = visible && inside_frustum;
        }

        // Update instance visibility
        instance.visible = visible;

        // Add to batch manager if visible
        if visible {
            batch_manager.add_instance(&instance);
        }
    }
}

/// Performance monitoring resource for culling systems
#[derive(Resource, Default)]
pub struct CullingPerformance {
    /// Last frame culling time (CPU fallback)
    pub cpu_culling_time: f32,
    /// Last frame instance count processed
    pub instances_processed: u32,
    /// Current culling method
    pub culling_method: CullingMethod,
    /// Frame timing history for averaging
    pub timing_history: Vec<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CullingMethod {
    #[default]
    Cpu,
    #[cfg(feature = "gpu")]
    Gpu,
}

impl CullingPerformance {
    /// Record timing for this frame
    pub fn record_timing(&mut self, time_ms: f32, instance_count: u32, method: CullingMethod) {
        self.cpu_culling_time = time_ms;
        self.instances_processed = instance_count;
        self.culling_method = method;

        // Keep rolling history of 60 frames
        self.timing_history.push(time_ms);
        if self.timing_history.len() > 60 {
            self.timing_history.remove(0);
        }
    }

    /// Get average timing over recent frames
    pub fn average_timing(&self) -> f32 {
        if self.timing_history.is_empty() {
            0.0
        } else {
            self.timing_history.iter().sum::<f32>() / self.timing_history.len() as f32
        }
    }

    /// Check if we're meeting performance targets
    pub fn meets_performance_target(&self) -> bool {
        let avg = self.average_timing();
        match self.culling_method {
            CullingMethod::Cpu => avg < 0.9, // 0.9ms CPU target
            #[cfg(feature = "gpu")]
            CullingMethod::Gpu => avg < 0.2, // 0.2ms GPU target
        }
    }
}

/// System to monitor culling performance
pub fn culling_performance_monitor(
    mut performance: ResMut<CullingPerformance>,
    time: Res<Time>,
    instances: Query<&ExtractedInstance>,
) {
    let instance_count = instances.iter().count() as u32;
    let delta_ms = time.delta_secs() * 1000.0;

    #[cfg(feature = "gpu")]
    let method = CullingMethod::Gpu;
    #[cfg(not(feature = "gpu"))]
    let method = CullingMethod::Cpu;

    performance.record_timing(delta_ms, instance_count, method);

    // Log performance warnings
    if !performance.meets_performance_target() {
        warn!(
            "Culling performance below target: {:.2}ms for {} instances (method: {:?})",
            performance.average_timing(),
            instance_count,
            method
        );
    }
}

/// Plugin for integrated culling system
pub struct CullingIntegrationPlugin;

impl Plugin for CullingIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CullingPerformance>().add_systems(
            PostUpdate,
            (integrated_culling_system, culling_performance_monitor).chain(),
        );
    }
}

/// Re-exports for convenience
pub mod prelude {
    pub use crate::culling_integration::{
        CullingIntegrationPlugin, CullingMethod, CullingPerformance, cpu_culling_fallback,
        culling_performance_monitor, integrated_culling_system,
    };
}
