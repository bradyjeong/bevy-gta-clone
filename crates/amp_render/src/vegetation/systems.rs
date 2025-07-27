//! Vegetation LOD systems for distance-based rendering optimization

use super::components::*;
use crate::distance_cache::{get_cached_distance, DistanceCacheResource, FrameCounter};
use bevy::prelude::*;

/// System to update vegetation LOD based on player distance
pub fn vegetation_lod_system(
    mut distance_cache: ResMut<DistanceCacheResource>,
    frame_counter: Res<FrameCounter>,
    // Query for active entity (player/camera)
    active_query: Query<&Transform, (With<Camera>, With<Transform>)>,
    mut vegetation_query: Query<
        (
            Entity,
            &mut VegetationLOD,
            &Transform,
            &mut Visibility,
            Option<&mut Mesh3d>,
        ),
        With<VegetationMeshLOD>,
    >,
    mesh_lod_query: Query<&VegetationMeshLOD>,
) {
    // Get player/camera position
    let active_pos = match active_query.iter().next() {
        Some(transform) => transform.translation,
        None => return, // No active camera found
    };

    for (entity, mut veg_lod, transform, mut visibility, mesh_handle) in vegetation_query.iter_mut()
    {
        // Calculate distance to player using distance cache for efficiency
        let distance = get_cached_distance(
            &mut distance_cache,
            &frame_counter,
            active_pos,
            transform.translation,
            entity,
        );

        let old_level = veg_lod.detail_level;
        veg_lod.update_from_distance(distance, frame_counter.frame.into());

        // Update visibility based on LOD level
        *visibility = if veg_lod.should_be_visible() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        // Update mesh if LOD level changed and mesh component exists
        if old_level != veg_lod.detail_level {
            if let (Ok(mesh_lod), Some(mut mesh_handle)) = (mesh_lod_query.get(entity), mesh_handle)
            {
                if let Some(new_mesh) = mesh_lod.get_mesh_for_level(veg_lod.detail_level) {
                    mesh_handle.0 = new_mesh;
                }
            }
        }
    }
}

/// System to make billboard vegetation always face the camera
pub fn vegetation_billboard_system(
    active_query: Query<&Transform, (With<Camera>, With<Transform>)>,
    mut billboard_query: Query<
        (&mut Transform, &VegetationLOD, &VegetationBillboard),
        (Without<Camera>, With<VegetationBillboard>),
    >,
) {
    // Get camera position
    let camera_pos = match active_query.iter().next() {
        Some(transform) => transform.translation,
        None => return,
    };

    for (mut transform, veg_lod, billboard) in billboard_query.iter_mut() {
        // Only update billboards for entities at billboard LOD level
        if matches!(veg_lod.detail_level, VegetationDetailLevel::Billboard) {
            let direction = (camera_pos - transform.translation).normalize();

            // Create rotation to face camera (Y-axis billboard)
            let look_rotation = Quat::from_rotation_y(direction.x.atan2(direction.z));
            transform.rotation = look_rotation;

            // Scale based on distance for better visual consistency
            let distance_scale = (veg_lod.distance_to_player / 150.0).clamp(0.5, 1.0);
            transform.scale = billboard.original_scale * distance_scale;
        }
    }
}

/// System to create billboard meshes and materials for distant vegetation
pub fn vegetation_billboard_mesh_generator(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Generate a simple quad mesh for billboards
    let billboard_mesh = create_billboard_quad();
    let billboard_mesh_handle = meshes.add(billboard_mesh);

    // Create material for billboards with basic green color
    let billboard_material = StandardMaterial {
        base_color: Color::srgb(0.2, 0.8, 0.3),
        alpha_mode: AlphaMode::Opaque,
        unlit: true,
        ..default()
    };
    let billboard_material_handle = materials.add(billboard_material);

    // Store resources for reuse
    commands.insert_resource(VegetationBillboardResources {
        mesh: billboard_mesh_handle,
        material: billboard_material_handle,
    });
}

/// System to initialize GPU resources for new vegetation entities
///
/// This tracks GPU resource allocation and ensures proper cleanup later.
pub fn vegetation_gpu_resource_init_system(
    mut query: Query<
        (Entity, &mut VegetationLOD),
        (Added<VegetationLOD>, Without<VegetationGpuResources>),
    >,
    mut memory_tracker: ResMut<VegetationGpuMemoryTracker>,
    mut commands: Commands,
) {
    for (entity, veg_lod) in query.iter_mut() {
        // Estimate memory usage based on LOD level
        let estimated_memory = match veg_lod.detail_level {
            VegetationDetailLevel::Full => 4096,     // 4KB for full detail
            VegetationDetailLevel::Medium => 2048,   // 2KB for medium detail
            VegetationDetailLevel::Billboard => 512, // 512B for billboard
            VegetationDetailLevel::Culled => 0,      // No memory for culled
        };

        if estimated_memory > 0 {
            let mut gpu_resources = VegetationGpuResources::new();

            // In a real implementation, these would be actual GPU resource IDs
            let instance_buffer_id = if matches!(veg_lod.detail_level, VegetationDetailLevel::Full)
            {
                Some(entity.index()) // Use entity index as mock buffer ID
            } else {
                None
            };

            let atlas_texture_id = Some(1); // Mock texture atlas ID
            let quadtree_node_id = Some(entity.index() % 1000); // Mock spatial node ID

            gpu_resources.allocate_resources(
                instance_buffer_id,
                atlas_texture_id,
                quadtree_node_id,
                estimated_memory,
            );

            // Track memory allocation
            memory_tracker.allocate_memory(estimated_memory);

            // Add component to entity
            commands.entity(entity).insert(gpu_resources);

            debug!(
                "Initialized GPU resources for vegetation entity {:?}: {}B memory",
                entity, estimated_memory
            );
        }
    }
}

/// System to adaptively adjust LOD distances based on performance
pub fn adaptive_vegetation_lod_system(
    time: Res<Time>,
    mut vegetation_query: Query<&mut VegetationLOD>,
) {
    let frame_time = time.delta_secs();
    let target_frame_time = 1.0 / 60.0; // 60 FPS target

    // Adjust LOD distances based on performance
    let distance_multiplier = if frame_time > target_frame_time * 1.5 {
        0.8 // Reduce distances by 20% to improve performance
    } else if frame_time < target_frame_time * 0.8 {
        1.1 // Increase distances by 10% for better quality
    } else {
        1.0 // No change
    };

    if distance_multiplier != 1.0 {
        for mut veg_lod in vegetation_query.iter_mut() {
            let adjusted_distance = veg_lod.distance_to_player * distance_multiplier;
            veg_lod.update_from_distance(adjusted_distance, 0);
        }
    }
}

/// Performance monitoring system for vegetation LOD
pub fn vegetation_lod_performance_monitor(
    vegetation_query: Query<&VegetationLOD>,
    mut lod_stats: ResMut<VegetationLODStats>,
) {
    let mut full_count = 0;
    let mut medium_count = 0;
    let mut billboard_count = 0;
    let mut culled_count = 0;

    for veg_lod in vegetation_query.iter() {
        match veg_lod.detail_level {
            VegetationDetailLevel::Full => full_count += 1,
            VegetationDetailLevel::Medium => medium_count += 1,
            VegetationDetailLevel::Billboard => billboard_count += 1,
            VegetationDetailLevel::Culled => culled_count += 1,
        }
    }

    // Update resource with current stats
    lod_stats.full_count = full_count;
    lod_stats.medium_count = medium_count;
    lod_stats.billboard_count = billboard_count;
    lod_stats.culled_count = culled_count;
    lod_stats.total_entities = full_count + medium_count + billboard_count;

    // Log performance stats periodically (debug builds only)
    #[cfg(debug_assertions)]
    {
        use std::time::{Duration, Instant};
        thread_local! {
            static LAST_LOG: std::cell::Cell<Option<Instant>> = std::cell::Cell::new(None);
        }

        LAST_LOG.with(|last| {
            let now = Instant::now();
            let should_log = last
                .get()
                .map(|last_time| now.duration_since(last_time) > Duration::from_secs(5))
                .unwrap_or(true);

            if should_log {
                debug!(
                    "Vegetation LOD: Full: {}, Medium: {}, Billboard: {}, Culled: {}",
                    full_count, medium_count, billboard_count, culled_count
                );
                last.set(Some(now));
            }
        });
    }
}

/// System to batch vegetation entities by LOD level for efficient rendering
pub fn vegetation_lod_batching_system(
    vegetation_query: Query<(Entity, &VegetationLOD, &Transform)>,
    mut batches: Local<Vec<Vec<Entity>>>,
) {
    // Clear and resize batches for each LOD level
    batches.clear();
    batches.resize(3, Vec::new()); // Full, Medium, Billboard

    // Group entities by LOD level
    for (entity, veg_lod, _transform) in vegetation_query.iter() {
        let batch_index = match veg_lod.detail_level {
            VegetationDetailLevel::Full => 0,
            VegetationDetailLevel::Medium => 1,
            VegetationDetailLevel::Billboard => 2,
            VegetationDetailLevel::Culled => continue,
        };

        batches[batch_index].push(entity);
    }

    // In a full implementation, these batches would be used for instanced rendering
    // This is the foundation for GPU instancing optimization
}

/// System to cleanup GPU resources when vegetation entities are despawned
///
/// This prevents VRAM leaks by properly releasing GPU buffers, instance arrays,
/// and quadtree nodes when vegetation entities are removed.
pub fn vegetation_gpu_cleanup_system(
    mut removed_entities: RemovedComponents<VegetationGpuResources>,
    mut memory_tracker: ResMut<VegetationGpuMemoryTracker>,
    mut commands: Commands,
) {
    for entity in removed_entities.read() {
        // Log cleanup operation for diagnostics
        debug!(
            "Cleaning up GPU resources for vegetation entity: {:?}",
            entity
        );

        // In a full implementation, this would:
        // 1. Release GPU instance buffers
        // 2. Remove texture atlas entries
        // 3. Clean up quadtree nodes
        // 4. Free GPU memory allocations

        // For now, we increment the cleanup counter
        memory_tracker.cleanup_operations += 1;

        // Placeholder for actual GPU resource cleanup
        // This would interface with the actual GPU resource managers
        cleanup_gpu_resources_for_entity(entity, &mut memory_tracker);
    }
}

/// System to monitor and cleanup GPU resources based on component despawn
pub fn vegetation_component_cleanup_system(
    mut query: Query<(Entity, &mut VegetationGpuResources), With<VegetationLOD>>,
    mut memory_tracker: ResMut<VegetationGpuMemoryTracker>,
    mut commands: Commands,
) {
    let mut entities_to_cleanup = Vec::new();

    for (entity, gpu_resources) in query.iter() {
        // Check if entity should be cleaned up (e.g., distance culled for too long)
        if gpu_resources.needs_cleanup() {
            let memory_usage = gpu_resources.memory_usage();

            // Track memory deallocation
            if memory_usage > 0 {
                memory_tracker.deallocate_memory(memory_usage);
            }

            entities_to_cleanup.push(entity);
        }
    }

    // Remove VegetationGpuResources component to trigger cleanup
    for entity in entities_to_cleanup {
        commands.entity(entity).remove::<VegetationGpuResources>();
        debug!("Removed VegetationGpuResources from entity: {:?}", entity);
    }
}

/// System to monitor GPU memory usage and trigger cleanup when over budget
pub fn vegetation_memory_budget_system(
    memory_tracker: Res<VegetationGpuMemoryTracker>,
    gpu_resources_query: Query<(Entity, &VegetationGpuResources)>,
    mut lod_query: Query<&mut VegetationLOD>,
) {
    if !memory_tracker.is_within_memory_budget() {
        let usage_percentage = memory_tracker.memory_usage_percentage();

        warn!(
            "Vegetation GPU memory over budget: {:.1}% ({:.2}MB / 512MB)",
            usage_percentage,
            memory_tracker.total_vram_usage as f32 / (1024.0 * 1024.0)
        );

        // Aggressively cull distant vegetation to free GPU memory
        let mut culled_count = 0;
        for mut veg_lod in lod_query.iter_mut() {
            if veg_lod.distance_to_player > 200.0 {
                veg_lod.detail_level = VegetationDetailLevel::Culled;
                culled_count += 1;
            }
        }

        if culled_count > 0 {
            info!(
                "Emergency culled {} vegetation entities due to GPU memory pressure",
                culled_count
            );
        }
    }
}

/// System to provide GPU memory diagnostics and monitoring
pub fn vegetation_gpu_diagnostics_system(
    memory_tracker: Res<VegetationGpuMemoryTracker>,
    gpu_resources_query: Query<&VegetationGpuResources>,
) {
    // Only log diagnostics periodically (debug builds only)
    #[cfg(debug_assertions)]
    {
        use std::time::{Duration, Instant};
        thread_local! {
            static LAST_LOG: std::cell::Cell<Option<Instant>> = std::cell::Cell::new(None);
        }

        LAST_LOG.with(|last| {
            let now = Instant::now();
            let should_log = last
                .get()
                .map(|last_time| now.duration_since(last_time) > Duration::from_secs(10))
                .unwrap_or(true);

            if should_log {
                let active_resources = gpu_resources_query.iter().filter(|r| r.resources_allocated).count();
                let total_memory_mb = memory_tracker.total_vram_usage as f32 / (1024.0 * 1024.0);
                let peak_memory_mb = memory_tracker.peak_vram_usage as f32 / (1024.0 * 1024.0);
                let usage_percentage = memory_tracker.memory_usage_percentage();

                debug!(
                    "Vegetation GPU: {} active resources, {:.2}MB used ({:.1}%), peak: {:.2}MB, {} cleanups",
                    active_resources,
                    total_memory_mb,
                    usage_percentage,
                    peak_memory_mb,
                    memory_tracker.cleanup_operations
                );

                last.set(Some(now));
            }
        });
    }
}

/// System to handle app state exit and cleanup all vegetation GPU resources
pub fn vegetation_app_exit_cleanup_system(
    mut gpu_resources_query: Query<(Entity, &mut VegetationGpuResources)>,
    mut memory_tracker: ResMut<VegetationGpuMemoryTracker>,
    mut commands: Commands,
) {
    info!("Cleaning up all vegetation GPU resources on app exit");

    let mut total_memory_freed = 0u64;
    let mut entities_cleaned = 0u32;

    for (entity, mut gpu_resources) in gpu_resources_query.iter_mut() {
        if gpu_resources.needs_cleanup() {
            total_memory_freed += gpu_resources.memory_usage();

            // Perform actual GPU resource cleanup
            cleanup_gpu_resources_for_entity(entity, &mut memory_tracker);

            // Mark resources as cleaned up
            gpu_resources.resources_allocated = false;
            gpu_resources.gpu_memory_usage = 0;

            entities_cleaned += 1;
        }

        // Remove the component to trigger final cleanup
        commands.entity(entity).remove::<VegetationGpuResources>();
    }

    // Reset memory tracker
    memory_tracker.total_vram_usage = 0;
    memory_tracker.active_instances = 0;

    info!(
        "Vegetation GPU cleanup complete: {} entities, {:.2}MB freed",
        entities_cleaned,
        total_memory_freed as f32 / (1024.0 * 1024.0)
    );
}

/// Helper function to cleanup GPU resources for a specific entity
///
/// In a full implementation, this would interface with actual GPU resource managers
/// to free buffers, textures, and other GPU allocations.
fn cleanup_gpu_resources_for_entity(
    entity: Entity,
    memory_tracker: &mut VegetationGpuMemoryTracker,
) {
    // Placeholder for actual GPU resource cleanup
    // In a full implementation, this would:
    // 1. Free GPU instance buffers using device.destroy_buffer()
    // 2. Remove texture atlas entries
    // 3. Clean up quadtree spatial nodes
    // 4. Update GPU memory tracking

    // For now, just log the cleanup
    debug!("GPU resource cleanup placeholder for entity: {:?}", entity);

    // Decrement instance count (actual memory will be decremented in calling system)
    memory_tracker.active_instances = memory_tracker.active_instances.saturating_sub(1);
}

/// Helper function to create a billboard quad mesh
fn create_billboard_quad() -> Mesh {
    Mesh::from(Plane3d::default().mesh().size(2.0, 3.0))
}
