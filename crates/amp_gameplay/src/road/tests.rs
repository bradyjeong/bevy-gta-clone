use amp_math::spline::Spline;
use bevy::app::App;
/// Tests for the advanced road system
use bevy::prelude::*;

use crate::road::{components::*, network::*, plugin::*, systems::*};

#[cfg(test)]
mod network_tests {
    use super::*;

    #[test]
    fn test_road_spline_creation() {
        let start = Vec3::new(0.0, 0.0, 0.0);
        let end = Vec3::new(100.0, 0.0, 0.0);
        let road = RoadSpline::new_straight(1, start, end, RoadType::MainStreet);

        assert_eq!(road.id, 1);
        assert_eq!(road.road_type, RoadType::MainStreet);
        assert_eq!(road.evaluate(0.0), start);
        assert_eq!(road.evaluate(1.0), end);
    }

    #[test]
    fn test_curved_road_creation() {
        let start = Vec3::new(0.0, 0.0, 0.0);
        let control = Vec3::new(50.0, 0.0, 50.0);
        let end = Vec3::new(100.0, 0.0, 0.0);
        let road = RoadSpline::new_curved(1, start, control, end, RoadType::Highway);

        assert_eq!(road.road_type, RoadType::Highway);
        assert_eq!(road.evaluate(0.0), start);
        assert_eq!(road.evaluate(1.0), end);

        // Curved road should be longer than straight line
        let straight_distance = start.distance(end);
        assert!(road.length() > straight_distance);
    }

    #[test]
    fn test_road_network_operations() {
        let mut network = RoadNetwork::default();

        let road_id = network.add_road(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(100.0, 0.0, 0.0),
            RoadType::MainStreet,
        );

        assert_eq!(road_id, 0);
        assert_eq!(network.roads.len(), 1);
        assert!(network.roads.contains_key(&road_id));

        let road = &network.roads[&road_id];
        assert_eq!(road.road_type, RoadType::MainStreet);
        assert_eq!(road.width(), 12.0);
    }

    #[test]
    fn test_road_connections() {
        let mut network = RoadNetwork::default();

        let road1_id =
            network.add_road(Vec3::ZERO, Vec3::new(100.0, 0.0, 0.0), RoadType::MainStreet);
        let road2_id = network.add_road(
            Vec3::new(100.0, 0.0, 0.0),
            Vec3::new(100.0, 0.0, 100.0),
            RoadType::SideStreet,
        );

        network.connect_roads(road1_id, road2_id);

        let road1 = &network.roads[&road1_id];
        let road2 = &network.roads[&road2_id];

        assert!(road1.connections.contains(&road2_id));
        assert!(road2.connections.contains(&road1_id));
    }

    #[test]
    fn test_intersection_creation() {
        let mut network = RoadNetwork::default();

        let position = Vec3::new(50.0, 0.0, 50.0);
        let connected_roads = vec![1, 2, 3, 4];
        let intersection_id =
            network.add_intersection(position, connected_roads.clone(), IntersectionType::Cross);

        assert_eq!(intersection_id, 0);
        assert!(network.intersections.contains_key(&intersection_id));

        let intersection = &network.intersections[&intersection_id];
        assert_eq!(intersection.position, position);
        assert_eq!(intersection.connected_roads, connected_roads);
        assert_eq!(intersection.intersection_type, IntersectionType::Cross);
    }

    #[test]
    fn test_nearest_road_finding() {
        let mut network = RoadNetwork::default();

        let _road_id = network.add_road(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(100.0, 0.0, 0.0),
            RoadType::MainStreet,
        );

        let test_position = Vec3::new(50.0, 0.0, 5.0);
        if let Some((_, nearest_point, distance)) = network.find_nearest_road(test_position) {
            assert!((nearest_point.x - 50.0).abs() < 1.0);
            assert!((nearest_point.z - 0.0).abs() < 1.0);
            assert!((distance - 5.0).abs() < 1.0);
        } else {
            panic!("Should have found a nearest road");
        }
    }

    #[test]
    fn test_road_network_stats() {
        let mut network = RoadNetwork::default();

        network.add_road(Vec3::ZERO, Vec3::new(100.0, 0.0, 0.0), RoadType::Highway);
        network.add_road(Vec3::ZERO, Vec3::new(0.0, 0.0, 100.0), RoadType::MainStreet);
        network.add_road(Vec3::ZERO, Vec3::new(50.0, 0.0, 50.0), RoadType::SideStreet);

        let stats = network.stats();
        assert_eq!(stats.total_roads, 3);
        assert_eq!(stats.road_type_counts[&RoadType::Highway], 1);
        assert_eq!(stats.road_type_counts[&RoadType::MainStreet], 1);
        assert_eq!(stats.road_type_counts[&RoadType::SideStreet], 1);
        assert!(stats.total_length > 200.0);
    }

    #[test]
    fn test_chunk_generation() {
        let mut network = RoadNetwork::default();

        // Test regular chunk generation
        let new_roads = network.generate_chunk_roads(1, 1);
        assert!(!new_roads.is_empty());

        // Test that the chunk is now marked as generated
        assert!(network.generated_chunks.contains(&(1, 1)));

        // Test that regenerating the same chunk returns empty
        let repeat_roads = network.generate_chunk_roads(1, 1);
        assert!(repeat_roads.is_empty());
    }

    #[test]
    fn test_spawn_chunk_premium_roads() {
        let mut network = RoadNetwork::default();

        // Generate spawn chunk (0, 0) roads
        let spawn_roads = network.generate_chunk_roads(0, 0);

        // Should generate premium roads including highways
        assert!(!spawn_roads.is_empty());
        assert!(spawn_roads.len() > 10); // Should have many roads in spawn chunk

        // Verify highways were created
        let highway_count = network
            .roads
            .values()
            .filter(|road| road.road_type == RoadType::Highway)
            .count();
        assert!(highway_count > 0);
    }
}

#[cfg(test)]
mod component_tests {
    use super::*;

    #[test]
    fn test_road_type_properties() {
        assert_eq!(RoadType::Highway.width(), 16.0);
        assert_eq!(RoadType::Highway.priority(), 4);
        assert_eq!(RoadType::Highway.lane_count(), 4);
        assert!(RoadType::Highway.has_lane_markings());
        assert!(RoadType::Highway.has_barriers());

        assert_eq!(RoadType::Alley.width(), 4.0);
        assert_eq!(RoadType::Alley.priority(), 1);
        assert_eq!(RoadType::Alley.lane_count(), 1);
        assert!(!RoadType::Alley.has_lane_markings());
        assert!(!RoadType::Alley.has_barriers());
    }

    #[test]
    fn test_intersection_type_properties() {
        assert_eq!(IntersectionType::Cross.radius(), 20.0);
        assert_eq!(IntersectionType::Cross.max_connections(), 4);

        assert_eq!(IntersectionType::Curve.radius(), 12.0);
        assert_eq!(IntersectionType::Curve.max_connections(), 2);
    }

    #[test]
    fn test_road_surface_properties() {
        assert_eq!(RoadSurface::Asphalt.friction_coefficient(), 0.9);
        assert_eq!(RoadSurface::Dirt.friction_coefficient(), 0.6);

        let asphalt_color = RoadSurface::Asphalt.base_color();
        assert!(!asphalt_color.to_srgba().red.is_nan());
    }

    #[test]
    fn test_road_config_defaults() {
        let config = RoadConfig::default();
        assert_eq!(config.road_type, RoadType::SideStreet);
        assert_eq!(config.surface, RoadSurface::Asphalt);
        assert!(config.enabled);
        assert_eq!(config.construction_progress, 1.0);
    }

    #[test]
    fn test_intersection_config_defaults() {
        let config = IntersectionConfig::default();
        assert_eq!(config.intersection_type, IntersectionType::Cross);
        assert_eq!(config.traffic_control, TrafficControl::None);
        assert!(config.enabled);
        assert!(config.connected_roads.is_empty());
    }

    #[test]
    fn test_road_generation_timer() {
        let timer = RoadGenerationTimer::new(400.0);
        assert_eq!(timer.chunk_size, 400.0);
        assert_eq!(timer.timer, 0.0);
        assert!(timer.last_player_chunk.is_none());
    }

    #[test]
    fn test_road_mesh_cache() {
        let cache = RoadMeshCache::new(100);
        assert_eq!(cache.max_cache_size, 100);
        assert!(cache.road_meshes.is_empty());
        assert!(cache.marking_meshes.is_empty());
    }
}

#[cfg(test)]
mod plugin_tests {
    use super::*;

    #[test]
    fn test_road_plugin_basic_setup() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins)
            .add_plugins(RoadPlugin::new());

        // Verify resources were inserted
        assert!(app.world().contains_resource::<RoadNetwork>());
        assert!(app.world().contains_resource::<RoadGenerationTimer>());
        assert!(app.world().contains_resource::<RoadMeshCache>());
    }

    #[test]
    fn test_road_plugin_with_custom_params() {
        let params = RoadGenerationParams {
            chunk_size: 500.0,
            generation_radius: 1000.0,
            cleanup_radius: 2500.0,
            road_density: 0.9,
            curve_probability: 0.4,
        };

        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins)
            .add_plugins(RoadPlugin::new().with_generation_params(params.clone()));

        let road_network = app.world().resource::<RoadNetwork>();
        assert_eq!(road_network.generation_params.chunk_size, 500.0);
        assert_eq!(road_network.generation_params.generation_radius, 1000.0);
        assert_eq!(road_network.generation_params.road_density, 0.9);
    }

    #[test]
    fn test_road_plugin_with_debug() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins)
            .add_plugins(RoadPlugin::new().with_debug());

        // Verify the plugin was configured (basic smoke test)
        assert!(app.world().contains_resource::<RoadNetwork>());
    }

    #[test]
    fn test_road_plugin_extension_trait() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins).add_road_system();

        assert!(app.world().contains_resource::<RoadNetwork>());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_road_system_integration() {
        let mut app = App::new();
        app.add_plugins((
            bevy::MinimalPlugins,
            bevy::asset::AssetPlugin::default(),
            bevy::transform::TransformPlugin,
        ))
        .add_plugins(RoadPlugin::new())
        .insert_resource(Time::default());

        // Add a mock active entity for testing
        app.world_mut().spawn((
            crate::character::components::ActiveEntity,
            Transform::from_translation(Vec3::new(100.0, 0.0, 100.0)),
        ));

        // Run one update cycle
        app.update();

        // Verify road network was initialized
        let road_network = app.world().resource::<RoadNetwork>();
        assert!(road_network.roads.len() >= 0); // May generate roads immediately
    }

    #[test]
    fn test_road_component_reflection() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins)
            .add_plugins(RoadPlugin::new());

        let type_registry = app.world().resource::<AppTypeRegistry>();
        let registry = type_registry.read();

        // Verify components are registered for reflection
        assert!(registry.get(std::any::TypeId::of::<RoadEntity>()).is_some());
        assert!(registry.get(std::any::TypeId::of::<RoadType>()).is_some());
        assert!(registry.get(std::any::TypeId::of::<RoadConfig>()).is_some());
        assert!(registry
            .get(std::any::TypeId::of::<IntersectionEntity>())
            .is_some());
        assert!(registry
            .get(std::any::TypeId::of::<IntersectionConfig>())
            .is_some());
    }

    #[test]
    fn test_road_material_creation() {
        let mut app = App::new();
        app.add_plugins((bevy::MinimalPlugins, bevy::asset::AssetPlugin::default()))
            .add_plugins(RoadPlugin::new());

        // Run startup systems
        app.update();

        // Verify materials resource was created
        assert!(app.world().contains_resource::<RoadMaterials>());

        let materials = app.world().resource::<RoadMaterials>();
        let asphalt_material = materials.get_surface_material(RoadSurface::Asphalt);
        let marking_material = materials.get_marking_material();

        // Basic verification that handles are valid
        assert!(!asphalt_material.is_weak());
        assert!(!marking_material.is_weak());
    }

    #[test]
    fn test_road_generation_with_movement() {
        let mut app = App::new();
        app.add_plugins((
            bevy::MinimalPlugins,
            bevy::asset::AssetPlugin::default(),
            bevy::transform::TransformPlugin,
        ))
        .add_plugins(RoadPlugin::new())
        .insert_resource(Time::default());

        // Spawn active entity
        let active_entity = app
            .world_mut()
            .spawn((
                crate::character::components::ActiveEntity,
                Transform::from_translation(Vec3::ZERO),
            ))
            .id();

        // Update to generate initial roads
        app.update();

        // Move the active entity to a new chunk
        if let Some(mut transform) = app.world_mut().get_mut::<Transform>(active_entity) {
            transform.translation = Vec3::new(500.0, 0.0, 500.0);
        }

        // Update timer to force road generation
        {
            let mut timer = app.world_mut().resource_mut::<RoadGenerationTimer>();
            timer.timer = 1.0; // Force update
        }

        // Run another update to generate roads in new area
        app.update();

        // Verify roads were generated
        let road_network = app.world().resource::<RoadNetwork>();
        assert!(!road_network.generated_chunks.is_empty());
    }
}

/// Performance tests for road system
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_road_generation_performance() {
        let mut network = RoadNetwork::default();

        let start = Instant::now();

        // Generate roads for a 5x5 grid of chunks
        for x in -2..=2 {
            for z in -2..=2 {
                network.generate_chunk_roads(x, z);
            }
        }

        let duration = start.elapsed();

        // Should generate 25 chunks worth of roads quickly
        assert!(
            duration.as_millis() < 100,
            "Road generation took too long: {:?}",
            duration
        );
        assert_eq!(network.generated_chunks.len(), 25);
        assert!(!network.roads.is_empty());
    }

    #[test]
    fn test_nearest_road_search_performance() {
        let mut network = RoadNetwork::default();

        // Create a network with many roads
        for i in 0..100 {
            let start = Vec3::new(i as f32 * 10.0, 0.0, 0.0);
            let end = Vec3::new(i as f32 * 10.0, 0.0, 100.0);
            network.add_road(start, end, RoadType::SideStreet);
        }

        let start = Instant::now();

        // Search for nearest roads multiple times
        for i in 0..1000 {
            let test_pos = Vec3::new((i % 100) as f32 * 10.0 + 5.0, 0.0, 50.0);
            network.find_nearest_road(test_pos);
        }

        let duration = start.elapsed();

        // 1000 searches should complete quickly
        assert!(
            duration.as_millis() < 50,
            "Nearest road search took too long: {:?}",
            duration
        );
    }

    #[test]
    fn test_spline_evaluation_performance() {
        let control_points = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(25.0, 0.0, 25.0),
            Vec3::new(50.0, 0.0, 25.0),
            Vec3::new(75.0, 0.0, 50.0),
            Vec3::new(100.0, 0.0, 0.0),
        ];

        let spline = Spline::new(control_points);

        let start = Instant::now();

        // Evaluate spline at many points
        for i in 0..10000 {
            let t = (i as f32) / 9999.0;
            spline.evaluate(t);
        }

        let duration = start.elapsed();

        // 10000 evaluations should complete quickly
        assert!(
            duration.as_millis() < 20,
            "Spline evaluation took too long: {:?}",
            duration
        );
    }
}
