//! Load game system implementation
//!
//! Provides functionality to load game state from disk and restore the
//! complete game world including player data, vehicles, and world state.

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::collections::HashMap;
use std::fs;

use crate::character::components::Player;
// use crate::character::bundles::PlayerBundle;
use crate::vehicle::components::{CarConfig, PhysicsVehicle, Vehicle, VehicleInput};
// use crate::vehicle::bundles::{VehicleBundle, CarBundle};
use crate::interaction::components::{InVehicle, PlayerState, VehicleInteraction};
use crate::vehicle::components::{Brakes, Engine, Steering, Suspension, Transmission};

use super::save_system::GameStatisticsTracker;
use super::serializable::*;

/// Resource to track entity mapping during load operations
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct LoadState {
    pub entity_mapping: HashMap<u32, Entity>,
    pub pending_load: bool,
}

/// System to handle load game functionality
pub fn load_game_system(
    input: Res<ButtonInput<KeyCode>>,
    mut load_state: ResMut<LoadState>,
    mut commands: Commands,
    mut statistics: ResMut<GameStatisticsTracker>,
    // Queries for cleanup
    player_query: Query<Entity, With<Player>>,
    vehicle_query: Query<Entity, With<PhysicsVehicle>>,
    asset_server: Res<AssetServer>,
) {
    // Trigger load on F9 key
    if !input.just_pressed(KeyCode::F9) {
        return;
    }

    info!("Starting load operation...");

    // Load save file
    let save_data = match load_save_file("saves/savegame.ron") {
        Ok(data) => data,
        Err(err) => {
            error!("Failed to load save file: {}", err);
            return;
        }
    };

    // Validate loaded data
    if let Err(err) = save_data.validate() {
        error!("Loaded save validation failed: {}", err);
        return;
    }

    // Clear existing entities
    cleanup_existing_entities(&mut commands, &player_query, &vehicle_query);

    // Clear entity mapping
    load_state.entity_mapping.clear();

    // Load player
    let player_entity = spawn_player(&mut commands, &save_data.player, &asset_server);
    load_state
        .entity_mapping
        .insert(save_data.player.entity_id, player_entity);

    // Load vehicles
    for vehicle_data in &save_data.vehicles {
        let vehicle_entity = spawn_vehicle(&mut commands, vehicle_data, &asset_server);
        load_state
            .entity_mapping
            .insert(vehicle_data.entity_id, vehicle_entity);
    }

    // Set up relationships and active entities
    setup_relationships_and_active_entities(
        &mut commands,
        &save_data,
        &load_state.entity_mapping,
        player_entity,
    );

    // Update statistics
    statistics.distance_traveled = save_data.metadata.statistics.distance_traveled;
    statistics.vehicles_driven = save_data.metadata.statistics.vehicles_driven;
    statistics.time_in_vehicles = save_data.metadata.statistics.time_in_vehicles;
    statistics.missions_completed = save_data.metadata.statistics.missions_completed;

    // Post-load validation
    if let Err(err) = validate_post_load(&save_data, &load_state.entity_mapping) {
        error!("Post-load validation failed: {}", err);
        return;
    }

    info!("Game loaded successfully!");
    info!(
        "Loaded state: {:?}, Active entity: {:?}",
        save_data.player_state, save_data.active_entity_id
    );
    info!(
        "Loaded {} vehicles and {} statistics",
        save_data.vehicles.len(),
        save_data.metadata.statistics.missions_completed
    );
}

/// Load save file from disk
pub fn load_save_file(path: &str) -> Result<SaveGameState, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read save file: {}", e))?;

    let save_data: SaveGameState =
        ron::from_str(&content).map_err(|e| format!("Failed to parse save file: {}", e))?;

    Ok(save_data)
}

/// Load a specific named save file
pub fn load_named_save(save_name: &str) -> Result<SaveGameState, String> {
    let path = format!("saves/{}.ron", save_name);
    load_save_file(&path)
}

/// Clean up existing entities before loading
fn cleanup_existing_entities(
    commands: &mut Commands,
    player_query: &Query<Entity, With<Player>>,
    vehicle_query: &Query<Entity, With<PhysicsVehicle>>,
) {
    info!("Cleaning up existing entities...");

    // Despawn all existing players
    for entity in player_query.iter() {
        commands.entity(entity).despawn();
    }

    // Despawn all existing vehicles
    for entity in vehicle_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Spawn player from serialized data
fn spawn_player(
    commands: &mut Commands,
    player_data: &SerializablePlayer,
    asset_server: &Res<AssetServer>,
) -> Entity {
    info!("Spawning player...");

    let transform: Transform = player_data.transform.clone().into();
    let velocity: Velocity = player_data.velocity.clone().into();

    // For now, spawn a basic player with minimal components until bundles are fixed
    let mut entity_commands = commands.spawn((
        Player,
        transform,
        velocity,
        player_data.player_state,
        Name::new("Player"),
    ));

    // Add health component when implemented
    // entity_commands.insert(Health::new(player_data.health));

    let entity = entity_commands.id();
    info!("Player entity spawned: {:?}", entity);
    entity
}

/// Spawn vehicle from serialized data
fn spawn_vehicle(
    commands: &mut Commands,
    vehicle_data: &SerializableVehicle,
    asset_server: &Res<AssetServer>,
) -> Entity {
    info!("Spawning vehicle...");

    let transform: Transform = vehicle_data.transform.clone().into();
    let velocity: Velocity = vehicle_data.velocity.clone().into();
    let vehicle_config: Vehicle = vehicle_data.vehicle_config.clone().into();
    let input_state: VehicleInput = vehicle_data.input_state.clone().into();

    // For now, spawn basic vehicle with minimal components until bundles are fixed
    let mut entity_commands = commands.spawn((
        PhysicsVehicle,
        vehicle_config,
        input_state,
        transform,
        velocity,
        GlobalTransform::default(),
        Visibility::default(),
        Name::new("Vehicle"),
    ));

    // Add car config if available
    if let Some(car_config_data) = &vehicle_data.car_config {
        let car_config: CarConfig = car_config_data.clone().into();
        entity_commands.insert(car_config);
    }

    // Components already inserted above

    // Add optional physics components if they were saved
    if let Some(engine_data) = &vehicle_data.engine_data {
        let engine: Engine = engine_data.clone().into();
        entity_commands.insert(engine);
    }

    if let Some(transmission_data) = &vehicle_data.transmission_data {
        let transmission: Transmission = transmission_data.clone().into();
        entity_commands.insert(transmission);
    }

    if let Some(suspension_data) = &vehicle_data.suspension_data {
        let suspension: Suspension = suspension_data.clone().into();
        entity_commands.insert(suspension);
    }

    if let Some(steering_data) = &vehicle_data.steering_data {
        let steering: Steering = steering_data.clone().into();
        entity_commands.insert(steering);
    }

    if let Some(brakes_data) = &vehicle_data.brakes_data {
        let brakes: Brakes = brakes_data.clone().into();
        entity_commands.insert(brakes);
    }

    // Set up vehicle interaction state
    if vehicle_data.occupied {
        let mut interaction = VehicleInteraction::new(2.0);
        interaction.occupied = true;
        interaction.occupant = vehicle_data.occupant.map(|id| Entity::from_raw(id));
        entity_commands.insert(interaction);
    } else {
        entity_commands.insert(VehicleInteraction::new(2.0));
    }

    let entity = entity_commands.id();
    info!("Vehicle entity spawned: {:?}", entity);
    entity
}

/// Set up relationships and active entities after spawning
fn setup_relationships_and_active_entities(
    commands: &mut Commands,
    save_data: &SaveGameState,
    entity_mapping: &HashMap<u32, Entity>,
    player_entity: Entity,
) {
    info!("Setting up relationships and active entities...");

    // Set up player-vehicle relationships
    if let Some(vehicle_id) = save_data.player.in_vehicle {
        if let Some(&vehicle_entity) = entity_mapping.get(&vehicle_id) {
            // Add InVehicle component to player
            commands
                .entity(player_entity)
                .insert(InVehicle::new(vehicle_entity, 0));

            // Update vehicle interaction state
            if let Some(vehicle_data) = save_data
                .vehicles
                .iter()
                .find(|v| v.entity_id == vehicle_id)
            {
                if vehicle_data.occupied {
                    // Vehicle interaction was already set up in spawn_vehicle
                }
            }

            info!("Player assigned to vehicle: {:?}", vehicle_entity);
        }
    }

    // Validate state consistency and handle special cases
    match save_data.player_state {
        PlayerState::Walking => {
            // Player should be visible and active
            if save_data.active_entity_id == Some(save_data.player.entity_id) {
                // Player is the active entity - this is handled by the game systems
            }
        }
        PlayerState::Driving => {
            // Player should be in a vehicle, vehicle should be active
            if let Some(vehicle_id) = save_data.player.in_vehicle {
                if save_data.active_entity_id == Some(vehicle_id) {
                    // Vehicle is the active entity - this is handled by the game systems
                }
            }
        }
    }
}

/// Validate the loaded state for consistency
fn validate_post_load(
    save_data: &SaveGameState,
    entity_mapping: &HashMap<u32, Entity>,
) -> Result<(), String> {
    info!("Running post-load validation...");

    // Check that all saved entities were recreated
    if !entity_mapping.contains_key(&save_data.player.entity_id) {
        return Err("Player entity not found in mapping".to_string());
    }

    for vehicle in &save_data.vehicles {
        if !entity_mapping.contains_key(&vehicle.entity_id) {
            return Err(format!(
                "Vehicle entity {} not found in mapping",
                vehicle.entity_id
            ));
        }
    }

    // Check active entity consistency
    if let Some(active_id) = save_data.active_entity_id {
        if !entity_mapping.contains_key(&active_id) {
            return Err("Active entity not found in mapping".to_string());
        }
    }

    // Check player state consistency
    match save_data.player_state {
        PlayerState::Walking => {
            if save_data.player.in_vehicle.is_some() {
                return Err("Walking state but player is in vehicle".to_string());
            }
        }
        PlayerState::Driving => {
            if save_data.player.in_vehicle.is_none() {
                return Err("Driving state but player not in vehicle".to_string());
            }
        }
    }

    info!("Post-load validation completed successfully");
    Ok(())
}

/// System to handle quick save functionality
pub fn quick_save_system(
    input: Res<ButtonInput<KeyCode>>,
    save_state_query: Query<
        (
            Entity,
            &Transform,
            &Velocity,
            &PlayerState,
            Option<&InVehicle>,
        ),
        With<Player>,
    >,
    vehicle_query: Query<
        (
            Entity,
            &Transform,
            &Velocity,
            &Vehicle,
            &VehicleInput,
            Option<&Engine>,
            Option<&Transmission>,
            Option<&Suspension>,
            Option<&Steering>,
            Option<&Brakes>,
            Option<&CarConfig>,
            Option<&VehicleInteraction>,
        ),
        With<PhysicsVehicle>,
    >,
    time: Res<Time>,
    statistics: Res<GameStatisticsTracker>,
) {
    if input.just_pressed(KeyCode::F6) {
        info!("Quick save triggered");
        // This could be implemented to create a quick save slot
        // For now, we'll use the regular save system
    }
}

/// System to handle quick load functionality  
pub fn quick_load_system(input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::F7) {
        info!("Quick load triggered");
        // This could be implemented to load from a quick save slot
        // For now, we'll use the regular load system
    }
}
