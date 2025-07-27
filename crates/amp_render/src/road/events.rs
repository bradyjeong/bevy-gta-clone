//! Event types for road mesh generation communication
//!
//! This module defines shared events for loose coupling between
//! gameplay and render crates as specified by Oracle's architecture.

use super::async_mesh_generation::{MeshData, RoadGenerationType};
use super::mesh_generation::{MarkingParams, RoadMeshParams};
use amp_math::spline::Spline;
use bevy::prelude::*;

/// Event requesting road mesh generation
#[derive(Event, Debug, Clone)]
pub struct RoadMeshRequest {
    /// Unique road identifier
    pub road_id: u32,
    /// Entity that will receive the mesh
    pub road_entity: Entity,
    /// Road spline data for mesh generation
    pub spline: Spline,
    /// Road mesh generation parameters
    pub road_params: RoadMeshParams,
    /// Lane marking parameters
    pub marking_params: MarkingParams,
    /// Number of lanes
    pub lane_count: u32,
    /// Priority for generation queue (higher = sooner)
    pub priority: f32,
    /// Type of road generation
    pub generation_type: RoadGenerationType,
}

impl RoadMeshRequest {
    /// Create a new road mesh request
    pub fn new(
        road_id: u32,
        road_entity: Entity,
        spline: Spline,
        road_params: RoadMeshParams,
        marking_params: MarkingParams,
        lane_count: u32,
        priority: f32,
        generation_type: RoadGenerationType,
    ) -> Self {
        Self {
            road_id,
            road_entity,
            spline,
            road_params,
            marking_params,
            lane_count,
            priority,
            generation_type,
        }
    }

    /// Create a standard road mesh request
    pub fn standard(
        road_id: u32,
        road_entity: Entity,
        spline: Spline,
        width: f32,
        lane_count: u32,
    ) -> Self {
        Self::new(
            road_id,
            road_entity,
            spline,
            RoadMeshParams {
                width,
                segments: 20,
                uv_scale: 1.0,
                smooth_normals: true,
            },
            MarkingParams::default(),
            lane_count,
            1.0, // Standard priority
            RoadGenerationType::Standard,
        )
    }

    /// Create a highway road mesh request
    pub fn highway(
        road_id: u32,
        road_entity: Entity,
        spline: Spline,
        lanes_per_direction: u32,
    ) -> Self {
        let total_lanes = lanes_per_direction * 2;
        let highway_width = total_lanes as f32 * 3.5; // 3.5m per highway lane

        Self::new(
            road_id,
            road_entity,
            spline,
            RoadMeshParams {
                width: highway_width,
                segments: 30,
                uv_scale: 2.0,
                smooth_normals: true,
            },
            MarkingParams::default(),
            total_lanes,
            2.0, // High priority for highways
            RoadGenerationType::Highway {
                lanes_per_direction,
            },
        )
    }

    /// Create an intersection road mesh request
    pub fn intersection(
        road_id: u32,
        road_entity: Entity,
        center: Vec3,
        radius: f32,
        intersection_type: super::mesh_generation::IntersectionType,
    ) -> Self {
        let spline = Spline::circle(center, radius);

        Self::new(
            road_id,
            road_entity,
            spline,
            RoadMeshParams {
                width: radius * 2.0,
                segments: 32,
                uv_scale: 1.0,
                smooth_normals: true,
            },
            MarkingParams::default(),
            4,   // Assume 4 lanes for intersections
            3.0, // Highest priority for intersections
            RoadGenerationType::Intersection {
                intersection_type,
                radius,
            },
        )
    }
}

/// Event signaling that road mesh generation is complete
#[derive(Event, Debug, Clone)]
pub struct RoadMeshReady {
    /// Unique road identifier
    pub road_id: u32,
    /// Entity that should receive the mesh
    pub road_entity: Entity,
    /// Generated main road surface mesh
    pub main_mesh: MeshData,
    /// Generated lane marking meshes
    pub marking_meshes: Vec<MeshData>,
    /// Time taken to generate mesh (seconds)
    pub generation_time: f32,
    /// Optional error message if generation failed
    pub error: Option<String>,
}

impl RoadMeshReady {
    /// Create a successful road mesh ready event
    pub fn success(
        road_id: u32,
        road_entity: Entity,
        main_mesh: MeshData,
        marking_meshes: Vec<MeshData>,
        generation_time: f32,
    ) -> Self {
        Self {
            road_id,
            road_entity,
            main_mesh,
            marking_meshes,
            generation_time,
            error: None,
        }
    }

    /// Create a failed road mesh ready event
    pub fn failure(road_id: u32, road_entity: Entity, error: String, generation_time: f32) -> Self {
        Self {
            road_id,
            road_entity,
            main_mesh: super::async_mesh_generation::MeshData {
                vertices: Vec::new(),
                normals: Vec::new(),
                uvs: Vec::new(),
                indices: Vec::new(),
                primitive_topology: bevy::render::mesh::PrimitiveTopology::TriangleList,
            },
            marking_meshes: Vec::new(),
            generation_time,
            error: Some(error),
        }
    }

    /// Check if mesh generation was successful
    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }

    /// Check if mesh generation failed
    pub fn is_failure(&self) -> bool {
        self.error.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::entity::Entity;

    #[test]
    fn test_road_mesh_request_creation() {
        let spline = Spline::linear(Vec3::ZERO, Vec3::new(100.0, 0.0, 0.0));
        let entity = Entity::from_raw(42);

        let request = RoadMeshRequest::standard(123, entity, spline, 8.0, 2);

        assert_eq!(request.road_id, 123);
        assert_eq!(request.road_entity, entity);
        assert_eq!(request.road_params.width, 8.0);
        assert_eq!(request.lane_count, 2);
        assert_eq!(request.priority, 1.0);
        assert!(matches!(
            request.generation_type,
            RoadGenerationType::Standard
        ));
    }

    #[test]
    fn test_highway_mesh_request() {
        let spline = Spline::linear(Vec3::ZERO, Vec3::new(500.0, 0.0, 0.0));
        let entity = Entity::from_raw(24);

        let request = RoadMeshRequest::highway(456, entity, spline, 3);

        assert_eq!(request.road_id, 456);
        assert_eq!(request.lane_count, 6); // 3 lanes per direction
        assert_eq!(request.road_params.width, 21.0); // 6 * 3.5m
        assert_eq!(request.priority, 2.0);
        assert!(matches!(
            request.generation_type,
            RoadGenerationType::Highway {
                lanes_per_direction: 3
            }
        ));
    }

    #[test]
    fn test_intersection_mesh_request() {
        let entity = Entity::from_raw(36);
        let center = Vec3::new(50.0, 0.0, 50.0);
        let radius = 15.0;

        let request = RoadMeshRequest::intersection(
            789,
            entity,
            center,
            radius,
            super::super::mesh_generation::IntersectionType::Cross,
        );

        assert_eq!(request.road_id, 789);
        assert_eq!(request.road_params.width, 30.0); // radius * 2
        assert_eq!(request.priority, 3.0);
        assert!(matches!(
            request.generation_type,
            RoadGenerationType::Intersection { radius: 15.0, .. }
        ));
    }

    #[test]
    fn test_road_mesh_ready_success() {
        let entity = Entity::from_raw(12);
        let main_mesh = MeshData {
            vertices: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
            primitive_topology: bevy::render::mesh::PrimitiveTopology::TriangleList,
        };
        let markings = vec![MeshData {
            vertices: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
            primitive_topology: bevy::render::mesh::PrimitiveTopology::TriangleList,
        }];

        let ready = RoadMeshReady::success(101, entity, main_mesh, markings, 0.05);

        assert_eq!(ready.road_id, 101);
        assert_eq!(ready.road_entity, entity);
        assert_eq!(ready.generation_time, 0.05);
        assert!(ready.is_success());
        assert!(!ready.is_failure());
        assert!(ready.error.is_none());
    }

    #[test]
    fn test_road_mesh_ready_failure() {
        let entity = Entity::from_raw(99);
        let error_msg = "Failed to generate mesh".to_string();

        let ready = RoadMeshReady::failure(202, entity, error_msg.clone(), 0.02);

        assert_eq!(ready.road_id, 202);
        assert_eq!(ready.road_entity, entity);
        assert_eq!(ready.generation_time, 0.02);
        assert!(!ready.is_success());
        assert!(ready.is_failure());
        assert_eq!(ready.error, Some(error_msg));
    }

    #[test]
    fn test_empty_mesh_data() {
        let mesh_data = MeshData {
            vertices: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
            primitive_topology: bevy::render::mesh::PrimitiveTopology::TriangleList,
        };

        assert!(mesh_data.vertices.is_empty());
        assert!(mesh_data.normals.is_empty());
        assert!(mesh_data.uvs.is_empty());
        assert!(mesh_data.indices.is_empty());
        assert_eq!(
            mesh_data.primitive_topology,
            bevy::render::mesh::PrimitiveTopology::TriangleList
        );
    }
}
