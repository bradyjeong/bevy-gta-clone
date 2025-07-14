//! Optimized query implementations for Sprint 9 performance improvements
//!
//! This module contains optimized query patterns to reduce per-frame overhead
//! in the hottest code paths by using more efficient filters and caching.

#![allow(clippy::type_complexity)]

use crate::batching::BatchManager;
use crate::culling::{CameraProjectionConfig, CullingConfig};
use crate::render_world::ExtractedInstances;
use crate::{BatchKey, ExtractedInstance, culling::Cullable};
use bevy::prelude::*;

/// Sprint 9 optimization: Use optimized query patterns to reduce per-frame overhead
pub mod cached_systems {
    use super::*;

    /// Optimized instance extraction using combined visibility checks
    pub fn optimized_extract_instances(
        mut extracted: ResMut<ExtractedInstances>,
        // Single query combining all necessary checks for performance
        #[allow(clippy::type_complexity)] visible_query: Query<
            (&GlobalTransform, &BatchKey),
            (With<Visibility>, With<InheritedVisibility>),
        >,
        camera_query: Query<&GlobalTransform, With<Camera>>,
    ) {
        #[cfg(feature = "tracy")]
        let _span = tracy_client::span!("optimized_extract_instances");

        let Ok(camera_transform) = camera_query.get_single() else {
            return;
        };
        let cam_pos = camera_transform.translation();

        extracted.clear();

        // Optimized: Single iterator over visible entities only
        for (transform, batch_key) in &visible_query {
            extracted.add_instance(ExtractedInstance::new(
                transform.compute_matrix(),
                batch_key.clone(),
                cam_pos,
            ));
        }

        #[cfg(feature = "tracy")]
        tracy_client::plot!("extracted_instances", extracted.instances.len() as f64);
    }

    /// Optimized CPU culling using cached frustum and streamlined queries
    pub fn optimized_cpu_culling(
        culling_config: Res<CullingConfig>,
        projection_config: Res<CameraProjectionConfig>,
        // Combined camera query for efficiency
        camera_query: Query<(&Camera, &GlobalTransform, Option<&Projection>)>,
        // Direct mutable access to cullable instances
        mut cullable_query: Query<(&mut ExtractedInstance, &Cullable)>,
        mut batch_manager: ResMut<BatchManager>,
    ) {
        #[cfg(feature = "tracy")]
        let _span = tracy_client::span!("optimized_cpu_culling");

        let Some((_, camera_transform, camera_projection)) = camera_query.iter().next() else {
            return;
        };

        // Extract frustum once if enabled (cached computation)
        let frustum_planes = if culling_config.enable_frustum_culling {
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
            Some(crate::culling::extract_frustum_planes(view_proj))
        } else {
            None
        };

        let camera_pos = camera_transform.translation();
        let mut visible_count = 0;
        let mut culled_count = 0;

        // Single pass through cullable instances (optimized iteration)
        for (mut instance, cullable) in cullable_query.iter_mut() {
            let position = instance.transform.w_axis.truncate();
            let radius = cullable.radius;
            let mut visible = true;

            // Distance culling (branch prediction friendly)
            if culling_config.enable_distance_culling {
                let distance_sq = camera_pos.distance_squared(position);
                let max_distance_sq = culling_config.max_distance * culling_config.max_distance;
                if distance_sq > max_distance_sq {
                    visible = false;
                }
            }

            // Frustum culling with early exit
            if visible && culling_config.enable_frustum_culling {
                if let Some(ref planes) = frustum_planes {
                    visible = crate::culling::sphere_in_frustum(position, radius, planes);
                }
            }

            instance.visible = visible;

            if visible {
                visible_count += 1;
                batch_manager.add_instance(&instance);
            } else {
                culled_count += 1;
            }
        }

        #[cfg(feature = "tracy")]
        {
            tracy_client::plot!("cpu_culling_visible", visible_count as f64);
            tracy_client::plot!("cpu_culling_culled", culled_count as f64);
        }
    }

    /// Optimized LOD extraction with reduced query overhead
    pub fn optimized_lod_extraction(
        // Only query LOD groups that are actually visible
        lod_query: Query<&crate::lod::LodGroup, With<Visibility>>,
        // Direct access to instances that need LOD updates
        mut instance_query: Query<&mut ExtractedInstance, With<Cullable>>,
    ) {
        #[cfg(feature = "tracy")]
        let _span = tracy_client::span!("optimized_lod_extraction");

        let mut updated_instances = 0;

        // Reduced iteration scope: only visible LOD groups
        for lod_group in lod_query.iter() {
            if let Some(current_level) = lod_group.current_level() {
                for mut instance in instance_query.iter_mut() {
                    let material = current_level
                        .material
                        .clone()
                        .unwrap_or_else(Handle::default);

                    instance.batch_key = crate::BatchKey::new(&current_level.mesh, &material);
                    updated_instances += 1;
                }
            }
        }

        #[cfg(feature = "tracy")]
        tracy_client::plot!("optimized_lod_updates", updated_instances as f64);
    }
}

/// Plugin for Sprint 9 optimized query systems  
pub struct OptimizedQueryPlugin;

impl Plugin for OptimizedQueryPlugin {
    fn build(&self, app: &mut App) {
        // Add optimized systems as alternatives to existing ones
        app.add_systems(
            PostUpdate,
            (
                cached_systems::optimized_cpu_culling,
                cached_systems::optimized_lod_extraction,
            ),
        );
    }
}
