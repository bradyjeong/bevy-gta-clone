//! Simple GPU culling system implementation
//!
//! Feature-flagged GPU culling with fallback to CPU implementation.
//! Implements Oracle P3a specification with simplified architecture.

use crate::{
    ExtractedInstance,
    culling::{CameraProjectionConfig, CullingConfig},
};
use bevy::prelude::*;

/// Simple GPU culling plugin with feature flag support
pub struct GpuCullingSimplePlugin;

impl Plugin for GpuCullingSimplePlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "gpu_culling")]
        {
            info!("GPU culling feature enabled - initializing GPU systems");
            app.add_systems(PostUpdate, gpu_culling_system);
        }

        #[cfg(not(feature = "gpu_culling"))]
        {
            info!("GPU culling feature disabled - using CPU fallback");
            app.add_systems(PostUpdate, cpu_fallback_culling_system);
        }
    }
}

/// GPU culling system (simplified for feature validation)
#[cfg(feature = "gpu_culling")]
fn gpu_culling_system(
    culling_config: Res<CullingConfig>,
    mut instances: Query<&mut ExtractedInstance>,
) {
    // For now, use CPU culling logic but mark as GPU path
    for mut instance in instances.iter_mut() {
        instance.update_visibility(culling_config.max_distance);
    }

    // Log GPU path usage
    let count = instances.iter().count();
    if count > 0 {
        trace!("GPU culling processed {} instances", count);
    }
}

/// CPU fallback culling system
#[cfg(not(feature = "gpu_culling"))]
fn cpu_fallback_culling_system(
    culling_config: Res<CullingConfig>,
    projection_config: Res<CameraProjectionConfig>,
    cameras: Query<(&Camera, &GlobalTransform, Option<&Projection>)>,
    mut instances: Query<&mut ExtractedInstance>,
) {
    // Use existing CPU culling logic
    crate::culling::distance_culling_system(culling_config, instances.reborrow());

    // Use the frustum culling logic but need to adapt signature
    if let Some((_camera, camera_transform, camera_projection)) = cameras.iter().next() {
        let view = camera_transform.compute_matrix().inverse();
        let projection = if let Some(Projection::Perspective(persp)) = camera_projection {
            Mat4::perspective_lh(persp.fov, persp.aspect_ratio, persp.near, persp.far)
        } else {
            Mat4::perspective_lh(
                projection_config.fov,
                projection_config.aspect_ratio,
                projection_config.near,
                projection_config.far,
            )
        };

        let view_proj = projection * view;
        let frustum_planes = crate::culling::extract_frustum_planes(view_proj);

        for mut instance in instances.iter_mut() {
            if !instance.visible {
                continue;
            }

            let position = instance.transform.w_axis.truncate();
            let radius = 1.0; // Default radius since we don't have Cullable component

            // Test sphere against all 6 frustum planes
            let mut inside_frustum = true;
            for plane in &frustum_planes {
                let distance = plane.xyz().dot(position) + plane.w;
                if distance < -radius {
                    inside_frustum = false;
                    break;
                }
            }

            instance.visible = inside_frustum;
        }
    }

    // Log CPU path usage
    let count = instances.iter().count();
    if count > 0 {
        trace!("CPU culling processed {} instances", count);
    }
}

/// Re-exports for convenience
pub mod prelude {
    pub use crate::gpu_culling_simple::GpuCullingSimplePlugin;
}
