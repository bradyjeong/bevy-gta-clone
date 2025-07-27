//! Plugin for async road mesh generation and attachment system
//!
//! This plugin implements Oracle's complete road mesh rendering pipeline:
//! 1. Enqueue mesh generation jobs from events  
//! 2. Process async mesh generation
//! 3. Emit completion events
//! 4. Attach meshes to entities
//! 5. Cleanup and diagnostics

use super::async_mesh_generation::*;
use super::events::{RoadMeshReady, RoadMeshRequest};
use super::materials::RoadMaterialLibrary;
use super::mesh_attachment::*;
use super::mesh_generation::IntersectionType;
use bevy::prelude::*;

/// Plugin for async road mesh generation and attachment
#[derive(Default)]
pub struct AsyncRoadMeshPlugin;

impl Plugin for AsyncRoadMeshPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<AsyncRoadMeshManager>()
            .init_resource::<MeshAttachmentStats>()
            // Events
            .add_event::<RoadMeshRequest>()
            .add_event::<RoadMeshReady>()
            // Systems with proper ordering as per Oracle's requirements
            .add_systems(Startup, setup_road_materials)
            .add_systems(
                Update,
                (
                    // 1. Road generation → enqueue jobs
                    enqueue_road_mesh_jobs,
                    // 2. Process async mesh generation
                    async_road_mesh_generation_system,
                    // 3. Process completed → emit events
                    process_completed_road_meshes,
                    // 4. Attach meshes to entities
                    attach_road_meshes_with_stats,
                    // 5. Cleanup and diagnostics
                    cleanup_despawned_road_meshes,
                    async_road_mesh_debug_system,
                    debug_mesh_attachment_stats,
                ),
            );
    }
}

/// Setup system to initialize road materials
fn setup_road_materials(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let road_materials = RoadMaterialLibrary::new(&mut materials);
    commands.insert_resource(road_materials);
}

// The old systems have been replaced by the new event-driven architecture:
// - handle_road_mesh_requests -> enqueue_road_mesh_jobs (in async_mesh_generation.rs)
// - process_completed_road_meshes -> process_completed_road_meshes (in async_mesh_generation.rs)

/// Helper function to create a road generation job
pub fn create_road_generation_job(
    road_id: String,
    road_entity: Entity,
    start: Vec3,
    end: Vec3,
    width: f32,
    lane_count: u32,
    priority: f32,
) -> RoadGenerationJob {
    use amp_math::spline::Spline;

    let spline = Spline::linear(start, end);

    RoadGenerationJob {
        road_id,
        road_entity,
        spline,
        road_params: super::mesh_generation::RoadMeshParams {
            width,
            segments: 20,
            uv_scale: 1.0,
            smooth_normals: true,
        },
        marking_params: super::mesh_generation::MarkingParams::default(),
        lane_count,
        priority,
        generation_type: RoadGenerationType::Standard,
    }
}

/// Helper function to create a highway generation job
pub fn create_highway_generation_job(
    road_id: String,
    road_entity: Entity,
    start: Vec3,
    end: Vec3,
    lanes_per_direction: u32,
    priority: f32,
) -> RoadGenerationJob {
    use amp_math::spline::Spline;

    let spline = Spline::linear(start, end);
    let total_lanes = lanes_per_direction * 2;
    let highway_width = total_lanes as f32 * 3.5; // 3.5m per lane for highways

    RoadGenerationJob {
        road_id,
        road_entity,
        spline,
        road_params: super::mesh_generation::RoadMeshParams {
            width: highway_width,
            segments: 30,  // More segments for highways
            uv_scale: 2.0, // Larger UV scale for highway textures
            smooth_normals: true,
        },
        marking_params: super::mesh_generation::MarkingParams::default(),
        lane_count: total_lanes,
        priority,
        generation_type: RoadGenerationType::Highway {
            lanes_per_direction,
        },
    }
}

/// Helper function to create an intersection generation job
pub fn create_intersection_generation_job(
    road_id: String,
    road_entity: Entity,
    center: Vec3,
    radius: f32,
    intersection_type: super::mesh_generation::IntersectionType,
    priority: f32,
) -> RoadGenerationJob {
    use amp_math::spline::Spline;

    // Create a circular spline for the intersection
    let spline = Spline::circle(center, radius);

    RoadGenerationJob {
        road_id,
        road_entity,
        spline,
        road_params: super::mesh_generation::RoadMeshParams {
            width: radius * 2.0,
            segments: 32, // More segments for smooth curves
            uv_scale: 1.0,
            smooth_normals: true,
        },
        marking_params: super::mesh_generation::MarkingParams::default(),
        lane_count: 4, // Assume 4 lanes for intersections
        priority,
        generation_type: RoadGenerationType::Intersection {
            intersection_type,
            radius,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_road_generation_job_creation() {
        let job = create_road_generation_job(
            "test_road".to_string(),
            Entity::from_raw(42),
            Vec3::ZERO,
            Vec3::new(100.0, 0.0, 0.0),
            8.0,
            2,
            1.0,
        );

        assert_eq!(job.road_id, "test_road");
        assert_eq!(job.lane_count, 2);
        assert_eq!(job.road_params.width, 8.0);
        assert!(matches!(job.generation_type, RoadGenerationType::Standard));
    }

    #[test]
    fn test_highway_generation_job_creation() {
        let job = create_highway_generation_job(
            "test_highway".to_string(),
            Entity::from_raw(24),
            Vec3::ZERO,
            Vec3::new(500.0, 0.0, 0.0),
            3,
            2.0,
        );

        assert_eq!(job.road_id, "test_highway");
        assert_eq!(job.lane_count, 6); // 3 lanes per direction
        assert_eq!(job.road_params.width, 21.0); // 6 * 3.5m
        assert!(matches!(
            job.generation_type,
            RoadGenerationType::Highway {
                lanes_per_direction: 3
            }
        ));
    }

    #[test]
    fn test_intersection_generation_job_creation() {
        let job = create_intersection_generation_job(
            "test_intersection".to_string(),
            Entity::from_raw(99),
            Vec3::new(50.0, 0.0, 50.0),
            15.0,
            IntersectionType::Cross,
            3.0,
        );

        assert_eq!(job.road_id, "test_intersection");
        assert_eq!(job.road_params.width, 30.0); // radius * 2
        assert!(matches!(
            job.generation_type,
            RoadGenerationType::Intersection {
                intersection_type: IntersectionType::Cross,
                radius: 15.0
            }
        ));
    }
}
