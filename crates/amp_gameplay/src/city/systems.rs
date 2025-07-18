//! City generation and management systems

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};
// Remove gameplay_factory dependency to avoid circular dependency
use crate::character::components::Player;
use crate::city::components::*;
use crate::city::layout::*;
use crate::city::render_assets::*;
use crate::city::resources::*;
// Direct spawning system without prefab factory dependency
use amp_render::prelude::*;
// Note: Using direct world position based streaming instead of amp_engine dependency to avoid circular dependency

/// Initialize city configuration and resources
pub fn city_setup(
    _commands: Commands,
    mut city_config: ResMut<CityConfig>,
    mut city_layout: ResMut<CityLayout>,
    mut city_prefabs: ResMut<CityPrefabs>,
) {
    info!("Initializing city generation system");

    // Set up city configuration for massive urban environment
    *city_config = CityConfig {
        grid_size: IVec2::new(200, 200), // Massive 200x200 grid
        tile_size: 20.0,                 // 20 meter tiles
        seed: 42,
        building_density: 0.7, // 70% building density for dense urban feel
        street_width: 8.0,
        building_height_range: (8.0, 120.0), // 8m to 120m for varied skyline
        building_size_range: (6.0, 25.0),    // 6m to 25m building footprints
        varied_heights: true,                // Enable f430bc6-style height variation
        city_center: Vec2::new(100.0, 100.0), // Center of the grid
        city_center_radius: 30.0,            // Large dense area around center
        intersection_frequency: 8,           // Intersection every 8 tiles
    };

    // Initialize empty layout - will be populated by layout generation
    *city_layout = CityLayout::default();

    // Initialize prefab registry
    *city_prefabs = CityPrefabs::default();

    info!(
        "City system initialized with grid size: {:?}",
        city_config.grid_size
    );
}

/// Load city layout from configuration or generate procedurally
pub fn load_city_layout(city_config: Res<CityConfig>, mut city_layout: ResMut<CityLayout>) {
    info!("Generating city layout");

    // Generate layout using the layout generator
    let generator = CityLayoutGenerator::new(city_config.clone());
    let layout = generator.generate_layout();

    *city_layout = layout;

    info!(
        "City layout generated with {} buildings, {} streets, {} intersections",
        city_layout.buildings.len(),
        city_layout.streets.len(),
        city_layout.intersections.len()
    );
}

/// Register all city prefabs with the prefab factory
pub fn register_city_prefabs_system(_commands: Commands, mut city_prefabs: ResMut<CityPrefabs>) {
    info!("Initializing city prefabs");

    // Register building prefab names with IDs
    city_prefabs.register_prefab("building", "building_residential".to_string());
    city_prefabs.register_prefab("building", "building_commercial".to_string());
    city_prefabs.register_prefab("building", "building_industrial".to_string());
    city_prefabs.register_prefab("building", "building_infrastructure".to_string());
    city_prefabs.register_prefab("building", "building_skyscraper".to_string());
    city_prefabs.register_prefab("building", "building_shop".to_string());

    // Register street prefab names with IDs
    city_prefabs.register_prefab("street", "street_main".to_string());
    city_prefabs.register_prefab("street", "street_side".to_string());
    city_prefabs.register_prefab("street", "street_secondary".to_string());
    city_prefabs.register_prefab("street", "street_alley".to_string());

    // Register intersection prefab names with IDs
    city_prefabs.register_prefab("intersection", "intersection_fourway".to_string());
    city_prefabs.register_prefab("intersection", "intersection_threeway".to_string());
    city_prefabs.register_prefab("intersection", "intersection_corner".to_string());

    info!(
        "Registered {} building prefabs, {} street prefabs, {} intersection prefabs",
        city_prefabs.buildings.len(),
        city_prefabs.streets.len(),
        city_prefabs.intersections.len()
    );
}

/// Generate the city grid based on layout - DISABLED for streaming
pub fn generate_city_grid(
    _commands: Commands,
    _city_config: Res<CityConfig>,
    _city_layout: Res<CityLayout>,
    _city_prefabs: Res<CityPrefabs>,
    _city_render_assets: Res<CityRenderAssets>,
) {
    info!("City grid generation disabled - using sector-based streaming instead");
}

/// Spawn city infrastructure (lights, signs, etc.)
/// Phase 3.D: Now spawns only DeferredLight components for the light budget system
pub fn spawn_city_infrastructure(
    mut commands: Commands,
    city_config: Res<CityConfig>,
    city_layout: Res<CityLayout>,
) {
    info!("Spawning city infrastructure with light budget system");

    let mut deferred_lights_spawned = 0;

    // Add street lights at intersections as DeferredLight components
    for (grid_pos, _intersection_spec) in &city_layout.intersections {
        let world_pos = city_config.grid_to_world(*grid_pos);

        // Spawn 4 street lights around intersection
        let light_positions = [
            Vec3::new(world_pos.x + 6.0, 8.0, world_pos.z + 6.0),
            Vec3::new(world_pos.x - 6.0, 8.0, world_pos.z + 6.0),
            Vec3::new(world_pos.x + 6.0, 8.0, world_pos.z - 6.0),
            Vec3::new(world_pos.x - 6.0, 8.0, world_pos.z - 6.0),
        ];

        for light_pos in light_positions {
            commands.spawn((
                DeferredLight {
                    intensity: 5000.0,
                    color: Color::srgb(1.0, 0.9, 0.7),
                    range: 25.0,
                    last_distance: f32::INFINITY,
                    is_active: false,
                    light_type: LightType::StreetLight,
                },
                Transform::from_translation(light_pos),
                GlobalTransform::default(),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
                Name::new("Deferred Street Light"),
            ));
            deferred_lights_spawned += 1;
        }
    }

    info!(
        "City infrastructure spawned: {} deferred lights (will be activated dynamically)",
        deferred_lights_spawned
    );
}

/// Add physics colliders to city entities
pub fn add_city_colliders(
    mut commands: Commands,
    city_config: Res<CityConfig>,
    _city_layout: Res<CityLayout>,
    buildings: Query<(Entity, &Transform, &Building)>,
    streets: Query<(Entity, &Transform, &Street)>,
) {
    info!("Adding city colliders");

    let mut building_colliders = 0;
    let mut street_colliders = 0;

    // Add colliders to buildings
    for (entity, _transform, building) in buildings.iter() {
        let collider_size = Vec3::new(
            building.size.x * 0.5,
            building.height * 0.5,
            building.size.y * 0.5,
        );

        commands.entity(entity).insert((
            RigidBody::Fixed,
            Collider::cuboid(collider_size.x, collider_size.y, collider_size.z),
            ColliderMarker {
                collider_type: ColliderType::Building,
                parent: Some(entity),
            },
        ));

        building_colliders += 1;
    }

    // Add colliders to streets
    for (entity, _transform, street) in streets.iter() {
        let collider_size = Vec3::new(
            street.width * 0.5,
            0.1, // Thin surface collider
            city_config.tile_size * 0.5,
        );

        commands.entity(entity).insert((
            RigidBody::Fixed,
            Collider::cuboid(collider_size.x, collider_size.y, collider_size.z),
            ColliderMarker {
                collider_type: ColliderType::Street,
                parent: Some(entity),
            },
        ));

        street_colliders += 1;
    }

    info!(
        "City colliders added: {} building colliders, {} street colliders",
        building_colliders, street_colliders
    );
}

/// Update city tile states
pub fn update_city_tiles(
    _query: Query<&mut CityTile>,
    buildings: Query<&Transform, (With<Building>, Changed<Transform>)>,
) {
    // Update tile occupancy based on building positions
    for _transform in buildings.iter() {
        // Update logic for dynamic tile states
        // This system can be expanded for dynamic city changes
    }
}

/// City cleanup system for removing entities outside streaming range - DISABLED for streaming
pub fn city_cleanup(
    _commands: Commands,
    _buildings: Query<(Entity, &Transform), With<Building>>,
    _camera_query: Query<&Transform, (With<Camera>, Without<Building>)>,
    _city_config: Res<CityConfig>,
) {
    // City cleanup now handled by sector-based streaming
}

// Phase 3.C: Sector-based streaming systems

/// Maximum entities to spawn per frame to maintain performance
const MAX_SPAWNS_PER_FRAME: usize = 50;

/// Counter for building IDs across sectors  
static BUILDING_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Helper function to spawn a single building entity
fn spawn_one_building(
    commands: &mut Commands,
    grid_pos: IVec2,
    building_spec: &BuildingSpec,
    city_config: &CityConfig,
    city_render_assets: &CityRenderAssets,
) -> Entity {
    let world_pos = city_config.grid_to_world(grid_pos);

    // Calculate proper scaling
    let scale = Vec3::new(
        building_spec.size.x / 10.0, // Normalize to base size
        building_spec.height / 15.0, // Normalize to base height
        building_spec.size.y / 10.0,
    );

    // Calculate cullable radius based on scaled dimensions
    let max_dimension = scale.max_element();
    let cullable_radius = max_dimension * 1.5; // 1.5x for safety margin

    // Create building entity with BatchKey and Cullable for instanced rendering
    let building_id = BUILDING_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;

    commands
        .spawn((
            Building {
                building_type: building_spec.building_type,
                height: building_spec.height,
                size: building_spec.size,
                id: building_id,
            },
            Transform::from_translation(world_pos).with_scale(scale),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            BatchKey::new(
                &city_render_assets.cube_mesh,
                &city_render_assets.building_material,
            ),
            Cullable::new(cullable_radius),
            Name::new(format!("Building_{}", building_id)),
        ))
        .id()
}

/// Helper function to spawn a single street entity
fn spawn_one_street(
    commands: &mut Commands,
    grid_pos: IVec2,
    street_spec: &StreetSpec,
    city_config: &CityConfig,
    city_render_assets: &CityRenderAssets,
) -> Entity {
    let world_pos = city_config.grid_to_world(grid_pos);

    // Calculate proper scaling
    let scale = Vec3::new(
        street_spec.width / 8.0, // Normalize to base width
        1.0,
        city_config.tile_size / 8.0,
    );

    // Calculate cullable radius based on scaled dimensions
    let max_dimension = scale.max_element();
    let cullable_radius = max_dimension * 1.5; // 1.5x for safety margin

    commands
        .spawn((
            Street {
                street_type: street_spec.street_type,
                width: street_spec.width,
                direction: street_spec.direction,
                has_sidewalks: street_spec.has_sidewalks,
            },
            Transform::from_translation(world_pos).with_scale(scale),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            BatchKey::new(
                &city_render_assets.plane_mesh,
                &city_render_assets.street_material,
            ),
            Cullable::new(cullable_radius),
            Name::new(format!("Street_{:?}", grid_pos)),
        ))
        .id()
}

/// Helper function to spawn a single intersection entity
fn spawn_one_intersection(
    commands: &mut Commands,
    grid_pos: IVec2,
    intersection_spec: &IntersectionSpec,
    city_config: &CityConfig,
    city_render_assets: &CityRenderAssets,
) -> Entity {
    let world_pos = city_config.grid_to_world(grid_pos);

    // Calculate proper scaling
    let scale = Vec3::new(
        intersection_spec.size / 15.0, // Normalize to base size
        1.0,
        intersection_spec.size / 15.0,
    );

    // Calculate cullable radius based on scaled dimensions
    let max_dimension = scale.max_element();
    let cullable_radius = max_dimension * 1.5; // 1.5x for safety margin

    commands
        .spawn((
            Intersection {
                intersection_type: intersection_spec.intersection_type,
                size: intersection_spec.size,
                has_traffic_lights: intersection_spec.has_traffic_lights,
            },
            Transform::from_translation(world_pos).with_scale(scale),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            BatchKey::new(
                &city_render_assets.plane_mesh,
                &city_render_assets.intersection_material,
            ),
            Cullable::new(cullable_radius),
            Name::new(format!("Intersection_{:?}", grid_pos)),
        ))
        .id()
}

/// Simple radius-based city streaming system - spawns entities around player within radius
pub fn spawn_city_radius(
    mut commands: Commands,
    city_config: Res<CityConfig>,
    city_layout: Res<CityLayout>,
    city_render_assets: Res<CityRenderAssets>,
    player_query: Query<&Transform, With<Player>>,
    existing_buildings: Query<&Transform, With<Building>>,
) {
    const SPAWN_RADIUS: f32 = 512.0; // 512m radius matching Oracle's spec
    const MAX_BUILDINGS_IN_WORLD: usize = 1000; // Limit to prevent spawning too many entities

    // Get player position
    let player_pos = match player_query.single() {
        Ok(transform) => transform.translation,
        Err(_) => return, // No player found
    };

    // Skip if we already have too many buildings
    if existing_buildings.iter().count() >= MAX_BUILDINGS_IN_WORLD {
        return;
    }

    let mut entities_spawned = 0;

    // Check all buildings in layout and spawn those within radius that don't exist yet
    for (grid_pos, building_spec) in city_layout.buildings.iter() {
        if entities_spawned >= MAX_SPAWNS_PER_FRAME {
            break;
        }

        let world_pos = city_config.grid_to_world(*grid_pos);
        let distance = player_pos.distance(world_pos);

        // Only spawn if within radius
        if distance <= SPAWN_RADIUS {
            // Check if building already exists at this position
            let already_exists = existing_buildings
                .iter()
                .any(|existing_transform| existing_transform.translation.distance(world_pos) < 1.0);

            if !already_exists {
                spawn_one_building(
                    &mut commands,
                    *grid_pos,
                    building_spec,
                    &city_config,
                    &city_render_assets,
                );
                entities_spawned += 1;
            }
        }
    }

    // Spawn streets within radius
    for (grid_pos, street_spec) in city_layout.streets.iter() {
        if entities_spawned >= MAX_SPAWNS_PER_FRAME {
            break;
        }

        let world_pos = city_config.grid_to_world(*grid_pos);
        let distance = player_pos.distance(world_pos);

        if distance <= SPAWN_RADIUS {
            spawn_one_street(
                &mut commands,
                *grid_pos,
                street_spec,
                &city_config,
                &city_render_assets,
            );
            entities_spawned += 1;
        }
    }

    // Spawn intersections within radius
    for (grid_pos, intersection_spec) in city_layout.intersections.iter() {
        if entities_spawned >= MAX_SPAWNS_PER_FRAME {
            break;
        }

        let world_pos = city_config.grid_to_world(*grid_pos);
        let distance = player_pos.distance(world_pos);

        if distance <= SPAWN_RADIUS {
            spawn_one_intersection(
                &mut commands,
                *grid_pos,
                intersection_spec,
                &city_config,
                &city_render_assets,
            );
            entities_spawned += 1;
        }
    }

    if entities_spawned > 0 {
        info!(
            "Spawned {} entities this frame around player",
            entities_spawned
        );
    }
}

// Phase 3.D: Light Budget System

/// Maximum number of active point lights allowed at once
const MAX_ACTIVE_LIGHTS: usize = 200;

/// Distance within which lights are activated
const LIGHT_ACTIVATION_DISTANCE: f32 = 120.0;

/// Distance beyond which buildings get emissive materials instead of point lights
const EMISSIVE_BUILDING_DISTANCE: f32 = 200.0;

/// Update light activity system - manages the light budget by dynamically activating/deactivating lights
pub fn update_light_activity(
    mut commands: Commands,
    mut deferred_lights: Query<(Entity, &mut DeferredLight, &Transform), Without<Player>>,
    player_query: Query<&Transform, With<Player>>,
    existing_point_lights: Query<Entity, With<PointLight>>,
    _city_render_assets: Res<CityRenderAssets>,
    buildings: Query<(Entity, &Transform), (With<Building>, Without<DeferredLight>)>,
) {
    // Get player position
    let player_pos = match player_query.single() {
        Ok(transform) => transform.translation,
        Err(_) => return, // No player found
    };

    // Count currently active lights
    let current_active_lights = existing_point_lights.iter().count();

    // Calculate distances and collect light data without references
    let mut light_priorities: Vec<(Entity, f32, DeferredLight)> = Vec::new();

    for (entity, mut deferred_light, transform) in deferred_lights.iter_mut() {
        let distance = player_pos.distance(transform.translation);
        deferred_light.last_distance = distance;

        // Collect lights that should be considered for activation
        if distance <= LIGHT_ACTIVATION_DISTANCE {
            light_priorities.push((entity, distance, deferred_light.clone()));
        } else if deferred_light.is_active {
            // Deactivate lights that are too far
            commands.entity(entity).remove::<PointLight>();
            deferred_light.is_active = false;
        }
    }

    // Sort by distance (closest first)
    light_priorities.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    // Activate closest lights up to the budget
    let mut lights_to_activate = 0;
    let available_slots = MAX_ACTIVE_LIGHTS.saturating_sub(current_active_lights);

    for (entity, _distance, deferred_light) in light_priorities.iter().take(available_slots) {
        if !deferred_light.is_active {
            // Activate this light
            commands.entity(*entity).insert(PointLight {
                intensity: deferred_light.intensity,
                color: deferred_light.color,
                range: deferred_light.range,
                ..default()
            });
            lights_to_activate += 1;
        }
    }

    // Update building materials based on distance - emissive for distant buildings
    // Note: Material switching would require a different approach since Handle<StandardMaterial>
    // is not a Component. This would need to be implemented through BatchKey updates
    // or mesh component replacement for now we'll skip this optimization.
    for (_building_entity, building_transform) in buildings.iter() {
        let distance = player_pos.distance(building_transform.translation);

        if distance > EMISSIVE_BUILDING_DISTANCE {
            // TODO: Switch to emissive material for distant buildings
            // This requires updating the BatchKey or mesh components
            // For now, we rely on the light budget system to reduce lighting cost

            // Future implementation could use:
            // commands.entity(building_entity).insert(BatchKey::new(
            //     &city_render_assets.cube_mesh,
            //     &city_render_assets.building_emissive_material,
            // ));
        }
    }

    // Mark activated lights as active
    for (entity, _distance, _deferred_light) in light_priorities.iter().take(available_slots) {
        if let Ok((_, mut deferred_light, _)) = deferred_lights.get_mut(*entity) {
            if !deferred_light.is_active {
                deferred_light.is_active = true;
            }
        }
    }

    if lights_to_activate > 0 {
        info!(
            "Activated {} lights, {} total active",
            lights_to_activate,
            current_active_lights + lights_to_activate
        );
    }
}
