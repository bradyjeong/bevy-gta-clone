//! Save game system implementation
//!
//! Provides functionality to save the current game state including player data,
//! vehicle states, world state, and statistics to disk.

use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;
use chrono::Utc;
use std::fs;
use std::path::Path;

use crate::character::components::Player;
use crate::interaction::components::{InVehicle, PlayerState, VehicleInteraction};
use crate::vehicle::components::{Brakes, Engine, Steering, Suspension, Transmission};
use crate::vehicle::components::{CarConfig, PhysicsVehicle, Vehicle, VehicleInput};

use super::serializable::*;

/// Resource to track game statistics for save files
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct GameStatisticsTracker {
    pub distance_traveled: f32,
    pub vehicles_driven: u32,
    pub time_in_vehicles: f32,
    pub missions_completed: u32,
    pub last_vehicle_position: Option<Vec3>,
}

impl Default for GameStatisticsTracker {
    fn default() -> Self {
        Self {
            distance_traveled: 0.0,
            vehicles_driven: 0,
            time_in_vehicles: 0.0,
            missions_completed: 0,
            last_vehicle_position: None,
        }
    }
}

/// Maximum number of backup saves to keep
const MAX_BACKUPS: usize = 3;

/// System to handle save game functionality
pub fn save_game_system(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    statistics: Res<GameStatisticsTracker>,
    player_query: Query<
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
) {
    // Trigger save on F5 key
    if !input.just_pressed(KeyCode::F5) {
        return;
    }

    info!("Starting save operation...");

    // Get player data
    let Ok((player_entity, player_transform, player_velocity, player_state, in_vehicle)) =
        player_query.single()
    else {
        error!("Failed to find player for save operation");
        return;
    };

    // Create serializable player data
    let serializable_player = SerializablePlayer {
        entity_id: player_entity.index(),
        transform: (*player_transform).into(),
        velocity: (*player_velocity).into(),
        player_state: *player_state,
        in_vehicle: in_vehicle.map(|iv| iv.vehicle_entity.index()),
        health: 100.0, // TODO: Get from health component when implemented
    };

    // Collect all vehicles
    let mut vehicles = Vec::new();
    for (
        entity,
        transform,
        velocity,
        vehicle_config,
        input_state,
        engine,
        transmission,
        suspension,
        steering,
        brakes,
        car_config,
        interaction,
    ) in vehicle_query.iter()
    {
        let serializable_vehicle = SerializableVehicle {
            entity_id: entity.index(),
            transform: (*transform).into(),
            velocity: (*velocity).into(),
            vehicle_config: vehicle_config.clone().into(),
            input_state: input_state.clone().into(),
            engine_data: engine.map(|e| e.clone().into()),
            transmission_data: transmission.map(|t| t.clone().into()),
            suspension_data: suspension.map(|s| s.clone().into()),
            steering_data: steering.map(|st| st.clone().into()),
            brakes_data: brakes.map(|b| b.clone().into()),
            car_config: car_config.map(|c| c.clone().into()),
            occupied: interaction.map_or(false, |i| i.occupied),
            occupant: interaction.and_then(|i| i.occupant.map(|e| e.index())),
        };
        vehicles.push(serializable_vehicle);
    }

    // Determine active entity (player or vehicle they're in)
    let active_entity_id = match player_state {
        PlayerState::Walking => Some(player_entity.index()),
        PlayerState::Driving => in_vehicle.map(|iv| iv.vehicle_entity.index()),
    };

    // Create save metadata
    let metadata = SaveMetadata {
        save_name: format!("Save_{}", Utc::now().format("%Y%m%d_%H%M%S")),
        level_name: "City".to_string(),
        difficulty: "Normal".to_string(),
        achievements: Vec::new(), // TODO: Implement achievement system
        statistics: GameStatistics {
            distance_traveled: statistics.distance_traveled,
            vehicles_driven: statistics.vehicles_driven,
            time_in_vehicles: statistics.time_in_vehicles,
            missions_completed: statistics.missions_completed,
        },
    };

    // Create save state
    let save_state = SaveGameState {
        version: SAVE_VERSION,
        timestamp: Utc::now(),
        player_state: *player_state,
        active_entity_id,
        player: serializable_player,
        vehicles,
        world_seed: None, // TODO: Add world generation seed if needed
        play_time: time.elapsed_secs_f64(),
        metadata,
    };

    // Validate save state
    if let Err(err) = save_state.validate() {
        error!("Save validation failed: {}", err);
        return;
    }

    // Create saves directory
    if let Err(err) = fs::create_dir_all("saves") {
        error!("Failed to create saves directory: {}", err);
        return;
    }

    // Backup existing saves
    backup_saves();

    // Serialize and save
    let ron_string = match ron::to_string(&save_state) {
        Ok(s) => s,
        Err(err) => {
            error!("Failed to serialize save state: {}", err);
            return;
        }
    };

    let save_path = "saves/savegame.ron";
    if let Err(err) = fs::write(save_path, ron_string) {
        error!("Failed to write save file: {}", err);
        return;
    }

    info!("Game saved successfully to {}", save_path);
    info!(
        "Active entity: {:?}, Player state: {:?}",
        active_entity_id, player_state
    );
    info!(
        "Saved {} vehicles and {} statistics",
        save_state.vehicles.len(),
        save_state.metadata.statistics.missions_completed
    );
}

/// System to update game statistics for saves
pub fn update_statistics_system(
    time: Res<Time>,
    mut statistics: ResMut<GameStatisticsTracker>,
    player_query: Query<(&Transform, &PlayerState, Option<&InVehicle>), With<Player>>,
    vehicle_query: Query<&Transform, (With<PhysicsVehicle>, Without<Player>)>,
) {
    let Ok((player_transform, player_state, in_vehicle)) = player_query.single() else {
        return;
    };

    // Track time in vehicles
    if matches!(player_state, PlayerState::Driving) {
        statistics.time_in_vehicles += time.delta_secs();
    }

    // Track distance traveled when in a vehicle
    if let Some(in_vehicle) = in_vehicle {
        if let Ok(vehicle_transform) = vehicle_query.get(in_vehicle.vehicle_entity) {
            if let Some(last_pos) = statistics.last_vehicle_position {
                let distance = last_pos.distance(vehicle_transform.translation);
                statistics.distance_traveled += distance;
            }
            statistics.last_vehicle_position = Some(vehicle_transform.translation);
        }
    } else {
        statistics.last_vehicle_position = None;
    }
}

/// Create backups of existing save files
fn backup_saves() {
    let save_path = Path::new("saves/savegame.ron");
    if !save_path.exists() {
        return;
    }

    // Shift existing backups
    for i in (1..MAX_BACKUPS).rev() {
        let old_backup = format!("saves/savegame.backup.{}.ron", i);
        let new_backup = format!("saves/savegame.backup.{}.ron", i + 1);
        let _ = fs::rename(&old_backup, &new_backup);
    }

    // Create new backup
    if let Err(err) = fs::copy(save_path, "saves/savegame.backup.1.ron") {
        warn!("Failed to create backup: {}", err);
    } else {
        info!("Created backup of existing save");
    }
}

/// Create a named save file
pub fn create_named_save(
    save_name: &str,
    save_state: &SaveGameState,
) -> Result<String, Box<dyn std::error::Error>> {
    // Create saves directory
    fs::create_dir_all("saves")?;

    // Serialize save state
    let ron_string = ron::to_string(save_state)?;

    // Create safe filename
    let safe_name = save_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();

    let save_path = format!("saves/{}.ron", safe_name);
    fs::write(&save_path, ron_string)?;

    info!("Named save created: {}", save_path);
    Ok(save_path)
}

/// List all available save files
pub fn list_save_files() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let saves_dir = Path::new("saves");
    if !saves_dir.exists() {
        return Ok(Vec::new());
    }

    let mut save_files = Vec::new();
    for entry in fs::read_dir(saves_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("ron") {
            if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                save_files.push(filename.to_string());
            }
        }
    }

    save_files.sort();
    Ok(save_files)
}
