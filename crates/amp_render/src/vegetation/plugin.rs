//! Vegetation LOD plugin for amp_render

use super::{components::*, systems::*};
use bevy::prelude::*;

/// Plugin for vegetation Level of Detail (LOD) rendering system
///
/// This plugin provides GPU-accelerated vegetation rendering with adaptive LOD
/// to maintain 60+ FPS while preserving visual quality.
#[derive(Default)]
pub struct VegetationLODPlugin;

impl Plugin for VegetationLODPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register reflection types for components
            .register_type::<VegetationDetailLevel>()
            .register_type::<VegetationLOD>()
            .register_type::<VegetationMeshLOD>()
            .register_type::<VegetationBillboard>()
            .register_type::<VegetationGpuResources>()
            // Add resources (FrameCounter is already provided by distance_cache plugin)
            .insert_resource(VegetationLODStats::default())
            .insert_resource(VegetationGpuMemoryTracker::default())
            // Startup systems - run once at startup
            .add_systems(Startup, vegetation_billboard_mesh_generator)
            // Update systems - run every frame with proper ordering
            .add_systems(
                Update,
                (
                    // GPU resource initialization - must run first
                    vegetation_gpu_resource_init_system
                        .in_set(amp_core::system_ordering::VegetationLODSystemSet::Init),
                    vegetation_lod_system
                        .in_set(amp_core::system_ordering::VegetationLODSystemSet::Update),
                    vegetation_billboard_system
                        .in_set(amp_core::system_ordering::VegetationLODSystemSet::Billboard),
                    vegetation_lod_batching_system
                        .in_set(amp_core::system_ordering::VegetationLODSystemSet::Batch),
                    // GPU resource cleanup systems - critical for VRAM leak prevention
                    vegetation_gpu_cleanup_system
                        .in_set(amp_core::system_ordering::VegetationLODSystemSet::Cleanup),
                    vegetation_component_cleanup_system
                        .in_set(amp_core::system_ordering::VegetationLODSystemSet::Cleanup),
                ),
            )
            // Performance monitoring and adaptive systems - run less frequently
            .add_systems(
                FixedUpdate,
                (
                    adaptive_vegetation_lod_system,
                    vegetation_lod_performance_monitor,
                    vegetation_memory_budget_system,
                    vegetation_gpu_diagnostics_system,
                ),
            )
            // App exit cleanup - prevents VRAM leaks on shutdown
            .add_systems(Last, vegetation_app_exit_cleanup_system);
    }
}

// System sets are now defined in amp_engine::system_ordering
// This ensures consistent ordering across all systems per Oracle's requirements
