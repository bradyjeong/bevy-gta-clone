/// Road system implementations for generation, management, and cleanup
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::road::components::*;
use crate::road::network::{RoadNetwork, RoadSpline};
use crate::road::resources::RoadEntityMap;
use amp_render::road::async_mesh_generation::RoadGenerationType;
use amp_render::road::{MarkingParams, RoadMeshParams, RoadMeshRequest};

/// System to generate roads around the active player
pub fn road_generation_system(
    mut commands: Commands,
    mut road_network: ResMut<RoadNetwork>,
    mut road_entity_map: ResMut<RoadEntityMap>,
    mut timer: ResMut<RoadGenerationTimer>,
    mut mesh_request_events: EventWriter<RoadMeshRequest>,
    active_query: Query<&Transform, With<crate::character::components::ActiveEntity>>,
    road_query: Query<(Entity, &Transform), With<RoadEntity>>,
    time: Res<Time>,
) {
    let Ok(active_transform) = active_query.single() else {
        return;
    };

    let active_pos = active_transform.translation;
    timer.timer += time.delta_secs();

    // Calculate current chunk
    let current_chunk = (
        (active_pos.x / timer.chunk_size).floor() as i32,
        (active_pos.z / timer.chunk_size).floor() as i32,
    );

    let chunk_changed = timer.last_player_chunk != Some(current_chunk);
    let should_update = timer.timer >= 0.5 || chunk_changed;

    if !should_update {
        return;
    }

    timer.timer = 0.0;
    timer.last_player_chunk = Some(current_chunk);

    // Clear cache if no roads exist but cache exists
    if road_network.roads.is_empty() && !road_network.generated_chunks.is_empty() {
        road_network.clear_cache();
    }

    let generation_radius = road_network.generation_params.generation_radius;
    let cleanup_radius = road_network.generation_params.cleanup_radius;

    // Clean up distant road entities
    if chunk_changed {
        for (entity, transform) in road_query.iter() {
            let distance = active_pos.distance(transform.translation);
            if distance > cleanup_radius {
                debug!("Cleaning up road entity at distance {}", distance);
                commands.entity(entity).despawn();
            }
        }
    }

    // Generate roads for nearby chunks
    let (chunk_x, chunk_z) = current_chunk;
    let chunk_radius = ((generation_radius / timer.chunk_size).ceil() as i32).max(3);

    for dx in -chunk_radius..=chunk_radius {
        for dz in -chunk_radius..=chunk_radius {
            let check_chunk_x = chunk_x + dx;
            let check_chunk_z = chunk_z + dz;

            let chunk_center = Vec3::new(
                check_chunk_x as f32 * timer.chunk_size,
                0.0,
                check_chunk_z as f32 * timer.chunk_size,
            );

            let distance = active_pos.distance(chunk_center);
            if distance <= generation_radius {
                let new_road_ids = road_network.generate_chunk_roads(check_chunk_x, check_chunk_z);

                // Spawn entities for new roads
                for road_id in new_road_ids {
                    if let Some(road) = road_network.roads.get(&road_id) {
                        spawn_road_entity(
                            &mut commands,
                            &mut road_entity_map,
                            &mut mesh_request_events,
                            road_id,
                            road,
                        );
                    }
                }
            }
        }
    }
}

/// Spawn a road entity with components
fn spawn_road_entity(
    commands: &mut Commands,
    road_entity_map: &mut ResMut<RoadEntityMap>,
    mesh_request_events: &mut EventWriter<RoadMeshRequest>,
    road_id: u32,
    road: &RoadSpline,
) {
    let start_pos = road.evaluate(0.0);

    // Create road entity with basic components
    let road_entity = commands
        .spawn((
            RoadEntity { road_id },
            RoadConfig {
                road_type: road.road_type,
                surface: RoadSurface::Asphalt,
                connections: road.connections.clone(),
                enabled: true,
                construction_progress: 1.0,
            },
            Transform::from_translation(Vec3::new(start_pos.x, 0.0, start_pos.z)),
            GlobalTransform::default(),
            Visibility::default(),
            DynamicRoadContent {
                content_type: RoadContentType::Road,
            },
            // Physics collider for road surface
            RigidBody::Fixed,
            create_road_collider(road),
            // Collision groups - roads only collide with vehicles, not characters
            CollisionGroups::new(Group::GROUP_2, Group::GROUP_1), // Static group, collides with vehicles
        ))
        .id();

    // Register the entity in the road entity map
    road_entity_map.insert_road(road_id, road_entity);

    // Create spline for mesh generation
    let spline = road.spline.clone();

    // Determine mesh parameters based on road type
    let road_params = RoadMeshParams {
        width: road.width(),
        segments: 20,
        uv_scale: 1.0,
        smooth_normals: true,
    };

    // Create mesh request event
    let mesh_request = RoadMeshRequest::new(
        road_id,
        road_entity,
        spline,
        road_params,
        MarkingParams::default(),
        road.road_type.lane_count(),
        1.0, // Standard priority
        RoadGenerationType::Standard,
    );

    // Emit the mesh request event
    mesh_request_events.write(mesh_request);

    info!(
        "Spawned road entity {} for road {} and requested mesh generation",
        road_entity.index(),
        road_id
    );
}

/// Create a physics collider for a road
fn create_road_collider(road: &RoadSpline) -> Collider {
    let width = road.width();
    let length = road.length();

    // Create a thin, flat collider for the road surface
    Collider::cuboid(width * 0.5, 0.02, length * 0.5)
}

/// System to update roads based on player movement and network changes
pub fn road_update_system(
    road_network: Res<RoadNetwork>,
    mut road_query: Query<(&mut RoadConfig, &RoadEntity)>,
) {
    // Update road configurations based on network state
    for (mut config, road_entity) in road_query.iter_mut() {
        if let Some(road) = road_network.roads.get(&road_entity.road_id) {
            // Update connections if they've changed
            if config.connections != road.connections {
                config.connections = road.connections.clone();
            }

            // Update road type if it's changed (rare but possible)
            if config.road_type != road.road_type {
                config.road_type = road.road_type;
            }
        }
    }
}

/// System to manage vehicle positioning relative to roads
pub fn vehicle_road_alignment_system(
    road_network: Res<RoadNetwork>,
    mut vehicle_query: Query<
        &mut Transform,
        (
            With<crate::vehicle::components::Car>,
            Without<crate::character::components::ActiveEntity>,
        ),
    >,
) {
    for mut transform in vehicle_query.iter_mut() {
        // Check if vehicle is significantly off-road
        if !road_network.is_on_road(transform.translation, 3.0) {
            // Find nearest road and gently guide vehicle towards it
            if let Some((_, nearest_point, distance)) =
                road_network.find_nearest_road(transform.translation)
            {
                if distance > 5.0 && distance < 50.0 {
                    // Gently pull vehicle towards nearest road
                    let direction = (nearest_point - transform.translation).normalize_or_zero();
                    transform.translation += direction * 0.1; // Gentle correction
                }
            }
        }
    }
}

/// System to manage NPC positioning relative to roads
pub fn npc_road_alignment_system(
    road_network: Res<RoadNetwork>,
    mut npc_query: Query<
        &mut Transform,
        (
            With<crate::npc::components::NPC>,
            Without<crate::character::components::ActiveEntity>,
            Without<crate::vehicle::components::Car>,
        ),
    >,
) {
    for mut transform in npc_query.iter_mut() {
        // Keep NPCs on sidewalks or roads
        if !road_network.is_on_road(transform.translation, 2.0) {
            if let Some((_, nearest_point, distance)) =
                road_network.find_nearest_road(transform.translation)
            {
                if distance > 3.0 && distance < 20.0 {
                    // Move NPC to sidewalk area (slightly off the road)
                    let to_road = (nearest_point - transform.translation).normalize_or_zero();
                    let sidewalk_pos = nearest_point + to_road.cross(Vec3::Y).normalize() * 2.0;
                    transform.translation = sidewalk_pos;
                }
            }
        }
    }
}

/// System to detect and create intersections dynamically
pub fn intersection_detection_system(
    mut road_network: ResMut<RoadNetwork>,
    mut commands: Commands,
    intersection_query: Query<&IntersectionEntity>,
) {
    let existing_intersections: std::collections::HashSet<_> = intersection_query
        .iter()
        .map(|i| i.intersection_id)
        .collect();

    let mut potential_intersections = Vec::new();

    // Find potential intersection points between roads
    let road_ids: Vec<_> = road_network.roads.keys().cloned().collect();

    for i in 0..road_ids.len() {
        for j in (i + 1)..road_ids.len() {
            let road1_id = road_ids[i];
            let road2_id = road_ids[j];

            if let (Some(road1), Some(road2)) = (
                road_network.roads.get(&road1_id),
                road_network.roads.get(&road2_id),
            ) {
                // Simple intersection detection: check if roads are close at any point
                if let Some(intersection_point) = find_intersection_point(road1, road2) {
                    potential_intersections.push((intersection_point, vec![road1_id, road2_id]));
                }
            }
        }
    }

    // Create intersection entities for new intersections
    for (position, connected_roads) in potential_intersections {
        let intersection_type = determine_intersection_type(&connected_roads);
        let intersection_id =
            road_network.add_intersection(position, connected_roads, intersection_type);

        if !existing_intersections.contains(&intersection_id) {
            spawn_intersection_entity(&mut commands, intersection_id, position);
        }
    }
}

/// Find intersection point between two roads (simplified implementation)
fn find_intersection_point(road1: &RoadSpline, road2: &RoadSpline) -> Option<Vec3> {
    const SAMPLES: usize = 20;
    const INTERSECTION_THRESHOLD: f32 = 5.0;

    for i in 0..SAMPLES {
        let t1 = i as f32 / (SAMPLES - 1) as f32;
        let point1 = road1.evaluate(t1);

        for j in 0..SAMPLES {
            let t2 = j as f32 / (SAMPLES - 1) as f32;
            let point2 = road2.evaluate(t2);

            if point1.distance(point2) < INTERSECTION_THRESHOLD {
                return Some((point1 + point2) * 0.5);
            }
        }
    }

    None
}

/// Determine intersection type based on connected roads
fn determine_intersection_type(connected_roads: &[u32]) -> IntersectionType {
    match connected_roads.len() {
        2 => IntersectionType::Curve,
        3 => IntersectionType::TJunction,
        4 => IntersectionType::Cross,
        _ => IntersectionType::Cross,
    }
}

/// Spawn an intersection entity
fn spawn_intersection_entity(commands: &mut Commands, intersection_id: u32, position: Vec3) {
    commands.spawn((
        IntersectionEntity { intersection_id },
        IntersectionConfig {
            intersection_type: IntersectionType::Cross,
            connected_roads: Vec::new(),
            traffic_control: TrafficControl::None,
            enabled: true,
        },
        Transform::from_translation(position),
        GlobalTransform::default(),
        Visibility::default(),
        DynamicRoadContent {
            content_type: RoadContentType::Intersection,
        },
    ));

    info!("Spawned intersection entity at {:?}", position);
}

/// System to maintain the RoadEntityMap resource
pub fn road_entity_map_maintenance_system(
    mut road_entity_map: ResMut<RoadEntityMap>,
    entity_query: Query<Entity>,
) {
    // Clean up mappings for entities that no longer exist
    let existing_entities: std::collections::HashSet<Entity> = entity_query.iter().collect();
    road_entity_map.cleanup_invalid_entities_with_set(&existing_entities);
}

/// System to clean up orphaned road entities
pub fn road_cleanup_system(
    mut commands: Commands,
    road_network: Res<RoadNetwork>,
    mut road_entity_map: ResMut<RoadEntityMap>,
    road_query: Query<(Entity, &RoadEntity)>,
    intersection_query: Query<(Entity, &IntersectionEntity)>,
) {
    // Remove road entities that no longer exist in the network
    for (entity, road_entity) in road_query.iter() {
        if !road_network.roads.contains_key(&road_entity.road_id) {
            debug!("Cleaning up orphaned road entity {}", road_entity.road_id);
            commands.entity(entity).despawn();
            // Also remove from entity map
            road_entity_map.remove_road(road_entity.road_id);
        }
    }

    // Remove intersection entities that no longer exist in the network
    for (entity, intersection_entity) in intersection_query.iter() {
        if !road_network
            .intersections
            .contains_key(&intersection_entity.intersection_id)
        {
            debug!(
                "Cleaning up orphaned intersection entity {}",
                intersection_entity.intersection_id
            );
            commands.entity(entity).despawn();
            // Also remove from entity map
            road_entity_map.remove_intersection(intersection_entity.intersection_id);
        }
    }
}

/// System to provide road information for debugging
pub fn road_debug_system(
    road_network: Res<RoadNetwork>,
    mut last_stats: Local<Option<String>>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();

    if *timer >= 5.0 {
        *timer = 0.0;

        let stats = road_network.stats();
        let stats_string = format!(
            "Roads: {}, Intersections: {}, Total Length: {:.1}m, Chunks: {}",
            stats.total_roads,
            stats.total_intersections,
            stats.total_length,
            stats.generated_chunks
        );

        if last_stats.as_ref() != Some(&stats_string) {
            info!("Road Network Stats: {}", stats_string);
            *last_stats = Some(stats_string);
        }
    }
}

/// System to handle road mesh ready events and attach meshes to entities
pub fn road_mesh_attachment_system(
    mut commands: Commands,
    mut road_entity_map: ResMut<RoadEntityMap>,
    mut mesh_ready_events: EventReader<amp_render::road::RoadMeshReady>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    road_materials: Option<Res<crate::road::plugin::RoadMaterials>>,
) {
    for ready_event in mesh_ready_events.read() {
        // Look up the road entity
        let Some(road_entity) = road_entity_map.get_road_entity(ready_event.road_id) else {
            warn!(
                "No entity found for road {} in RoadEntityMap",
                ready_event.road_id
            );
            continue;
        };

        if ready_event.is_failure() {
            warn!(
                "Road mesh generation failed for road {}: {}",
                ready_event.road_id,
                ready_event.error.as_ref().unwrap()
            );
            continue;
        }

        // Convert mesh data to Bevy meshes
        let main_mesh = ready_event.main_mesh.clone().to_mesh();
        let main_mesh_handle = meshes.add(main_mesh);

        // Get or create road surface material
        let road_material_handle = if let Some(materials_res) = &road_materials {
            materials_res.get_surface_material(RoadSurface::Asphalt)
        } else {
            // Fallback material creation
            let road_material = StandardMaterial {
                base_color: Color::srgb(0.4, 0.4, 0.4), // Dark gray for asphalt
                metallic: 0.0,
                perceptual_roughness: 0.8,
                ..default()
            };
            materials.add(road_material)
        };

        // Attach mesh to the road entity
        commands.entity(road_entity).insert((
            Mesh3d(main_mesh_handle),
            MeshMaterial3d(road_material_handle),
        ));

        // Get or create marking material
        let marking_material_handle = if let Some(materials_res) = &road_materials {
            materials_res.get_marking_material()
        } else {
            // Fallback marking material
            let marking_material = StandardMaterial {
                base_color: Color::srgb(1.0, 1.0, 1.0), // White for markings
                metallic: 0.0,
                perceptual_roughness: 0.9,
                ..default()
            };
            materials.add(marking_material)
        };

        // Create child entities for lane markings
        let mut marking_entities = Vec::new();
        for (i, marking_mesh_data) in ready_event.marking_meshes.iter().enumerate() {
            let marking_mesh = marking_mesh_data.clone().to_mesh();
            let marking_mesh_handle = meshes.add(marking_mesh);

            let marking_entity = commands
                .spawn((
                    Name::new(format!("RoadMarking_{}_{}", ready_event.road_id, i)),
                    Mesh3d(marking_mesh_handle),
                    MeshMaterial3d(marking_material_handle.clone()),
                    Transform::default(),
                    GlobalTransform::default(),
                    Visibility::default(),
                    DynamicRoadContent {
                        content_type: RoadContentType::Markings,
                    },
                ))
                .id();

            marking_entities.push(marking_entity);
        }

        // Set markings as children of the road entity
        if !marking_entities.is_empty() {
            commands.entity(road_entity).add_children(&marking_entities);
        }

        info!(
            "✅ Attached mesh to road entity {} for road {} ({} markings)",
            road_entity.index(),
            ready_event.road_id,
            ready_event.marking_meshes.len()
        );
    }
}

/// System to handle road network reset commands
pub fn road_reset_system(
    mut road_network: ResMut<RoadNetwork>,
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    road_query: Query<Entity, With<RoadEntity>>,
    intersection_query: Query<Entity, With<IntersectionEntity>>,
) {
    if keyboard.just_pressed(KeyCode::F9) {
        info!("Resetting road network...");

        // Despawn all road entities
        for entity in road_query.iter() {
            commands.entity(entity).despawn();
        }

        // Despawn all intersection entities
        for entity in intersection_query.iter() {
            commands.entity(entity).despawn();
        }

        // Reset the network
        road_network.reset();

        info!("Road network reset complete");
    }
}

/// System set definitions for road systems
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum RoadSystemSet {
    /// Road generation and network management
    Generation,
    /// Road entity updates and maintenance
    Update,
    /// Road cleanup and optimization
    Cleanup,
    /// Road debugging and diagnostics
    Debug,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_road_collider_creation() {
        use crate::road::network::RoadSpline;

        let road = RoadSpline::new_straight(
            1,
            Vec3::ZERO,
            Vec3::new(100.0, 0.0, 0.0),
            RoadType::MainStreet,
        );
        let collider = create_road_collider(&road);

        // Verify collider was created (basic smoke test)
        // More detailed testing would require Rapier setup
        assert!(matches!(collider, Collider::Cuboid { .. }));
    }

    #[test]
    fn test_intersection_type_determination() {
        assert_eq!(
            determine_intersection_type(&[1, 2]),
            IntersectionType::Curve
        );
        assert_eq!(
            determine_intersection_type(&[1, 2, 3]),
            IntersectionType::TJunction
        );
        assert_eq!(
            determine_intersection_type(&[1, 2, 3, 4]),
            IntersectionType::Cross
        );
    }

    #[test]
    fn test_system_set_organization() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);

        // Test that system sets can be configured
        app.configure_sets(
            Update,
            (
                RoadSystemSet::Generation,
                RoadSystemSet::Update,
                RoadSystemSet::Cleanup,
                RoadSystemSet::Debug,
            )
                .chain(),
        );

        // Basic smoke test for system set configuration
        let schedule = app.world_mut().resource::<Schedules>().get(Update).unwrap();
        assert!(schedule.graph().system_sets().count() >= 4);
    }
}
