//! Persistence plugin for the amp_gameplay crate
//!
//! This plugin provides save/load functionality for the game state,
//! integrating with the Bevy app and registering all necessary systems.

use bevy::prelude::*;

use super::{load_system::*, save_system::*, serializable::*};

/// Plugin that adds persistence functionality to the game
pub struct PersistencePlugin;

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        app
            // Register resources
            .init_resource::<LoadState>()
            .init_resource::<GameStatisticsTracker>()
            // Register types for reflection (useful for debugging)
            .register_type::<LoadState>()
            .register_type::<GameStatisticsTracker>()
            // Add systems
            .add_systems(
                Update,
                (
                    save_game_system,
                    load_game_system,
                    update_statistics_system,
                    quick_save_system,
                    quick_load_system,
                )
                    .chain(), // Ensure save/load don't run simultaneously
            );
    }
}

/// Plugin configuration for persistence settings
#[derive(Debug, Clone, Resource, Reflect)]
#[reflect(Resource)]
pub struct PersistenceConfig {
    /// Maximum number of backup saves to keep
    pub max_backups: usize,
    /// Save directory path
    pub save_directory: String,
    /// Enable auto-save functionality
    pub auto_save_enabled: bool,
    /// Auto-save interval in seconds
    pub auto_save_interval: f32,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            max_backups: 3,
            save_directory: "saves".to_string(),
            auto_save_enabled: false,
            auto_save_interval: 300.0, // 5 minutes
        }
    }
}

/// Extended persistence plugin with configuration
pub struct ConfigurablePersistencePlugin {
    pub config: PersistenceConfig,
}

impl ConfigurablePersistencePlugin {
    pub fn new(config: PersistenceConfig) -> Self {
        Self { config }
    }
}

impl Plugin for ConfigurablePersistencePlugin {
    fn build(&self, app: &mut App) {
        app
            // Insert configuration as resource
            .insert_resource(self.config.clone())
            // Register resources
            .init_resource::<LoadState>()
            .init_resource::<GameStatisticsTracker>()
            // Register types for reflection
            .register_type::<LoadState>()
            .register_type::<GameStatisticsTracker>()
            .register_type::<PersistenceConfig>()
            // Add systems
            .add_systems(
                Update,
                (
                    save_game_system,
                    load_game_system,
                    update_statistics_system,
                    quick_save_system,
                    quick_load_system,
                )
                    .chain(),
            );

        // Add auto-save system if enabled (simplified for now)
        if self.config.auto_save_enabled {
            app.insert_resource(AutoSaveTimer::new(self.config.auto_save_interval));
            // Note: Auto-save system disabled for now due to timer condition complexity
            // app.add_systems(Update, auto_save_system);
        }
    }
}

/// Resource to track auto-save timing
#[derive(Resource)]
struct AutoSaveTimer {
    timer: Timer,
}

impl AutoSaveTimer {
    fn new(interval: f32) -> Self {
        Self {
            timer: Timer::from_seconds(interval, TimerMode::Repeating),
        }
    }
}

/// Condition to check if auto-save timer has elapsed
fn auto_save_timer_elapsed(mut timer: ResMut<AutoSaveTimer>, time: Res<Time>) -> bool {
    timer.timer.tick(time.delta()).just_finished()
}

/// Auto-save system that runs periodically
fn auto_save_system(
    time: Res<Time>,
    statistics: Res<GameStatisticsTracker>,
    player_query: Query<
        (
            Entity,
            &Transform,
            &bevy_rapier3d::prelude::Velocity,
            &crate::interaction::components::PlayerState,
            Option<&crate::interaction::components::InVehicle>,
        ),
        With<crate::character::components::Player>,
    >,
    vehicle_query: Query<
        (
            Entity,
            &Transform,
            &bevy_rapier3d::prelude::Velocity,
            &crate::vehicle::components::Vehicle,
            &crate::vehicle::components::VehicleInput,
            Option<&crate::vehicle::components::Engine>,
            Option<&crate::vehicle::components::Transmission>,
            Option<&crate::vehicle::components::Suspension>,
            Option<&crate::vehicle::components::Steering>,
            Option<&crate::vehicle::components::Brakes>,
            Option<&crate::vehicle::components::CarConfig>,
            Option<&crate::interaction::components::VehicleInteraction>,
        ),
        With<crate::vehicle::components::PhysicsVehicle>,
    >,
) {
    info!("Auto-save triggered");

    // Get player data
    let Ok((player_entity, player_transform, player_velocity, player_state, in_vehicle)) =
        player_query.get_single()
    else {
        warn!("Failed to find player for auto-save operation");
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

    // Determine active entity
    let active_entity_id = match player_state {
        crate::interaction::components::PlayerState::Walking => Some(player_entity.index()),
        crate::interaction::components::PlayerState::Driving => {
            in_vehicle.map(|iv| iv.vehicle_entity.index())
        }
    };

    // Create save metadata
    let metadata = SaveMetadata {
        save_name: format!("AutoSave_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S")),
        level_name: "City".to_string(),
        difficulty: "Normal".to_string(),
        achievements: Vec::new(),
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
        timestamp: chrono::Utc::now(),
        player_state: *player_state,
        active_entity_id,
        player: serializable_player,
        vehicles,
        world_seed: None,
        play_time: time.elapsed_secs_f64(),
        metadata,
    };

    // Validate and save
    if let Err(err) = save_state.validate() {
        error!("Auto-save validation failed: {}", err);
        return;
    }

    // Save to auto-save slot
    if let Err(err) = super::save_system::create_named_save("autosave", &save_state) {
        error!("Auto-save failed: {}", err);
    } else {
        info!("Auto-save completed successfully");
    }
}
