//! Serializable versions of game components and types
//!
//! This module provides serialization-safe versions of all game components
//! that need to be persisted, along with conversion functions.

use bevy::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::character::components::Player;
use crate::interaction::components::{InVehicle, PlayerState, VehicleInteraction};
use crate::vehicle::components::{Brakes, Engine, Steering, Suspension, Transmission};
use crate::vehicle::components::{CarConfig, Vehicle, VehicleInput};

/// Serializable version of Bevy Transform
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableTransform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4], // Quaternion as [x, y, z, w]
    pub scale: [f32; 3],
}

impl From<Transform> for SerializableTransform {
    fn from(transform: Transform) -> Self {
        Self {
            translation: transform.translation.to_array(),
            rotation: [
                transform.rotation.x,
                transform.rotation.y,
                transform.rotation.z,
                transform.rotation.w,
            ],
            scale: transform.scale.to_array(),
        }
    }
}

impl From<SerializableTransform> for Transform {
    fn from(serializable: SerializableTransform) -> Transform {
        Transform {
            translation: Vec3::from_array(serializable.translation),
            rotation: Quat::from_xyzw(
                serializable.rotation[0],
                serializable.rotation[1],
                serializable.rotation[2],
                serializable.rotation[3],
            ),
            scale: Vec3::from_array(serializable.scale),
        }
    }
}

/// Serializable version of Rapier3D Velocity
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableVelocity {
    pub linear: [f32; 3],
    pub angular: [f32; 3],
}

impl From<bevy_rapier3d::prelude::Velocity> for SerializableVelocity {
    fn from(velocity: bevy_rapier3d::prelude::Velocity) -> Self {
        Self {
            linear: velocity.linvel.to_array(),
            angular: velocity.angvel.to_array(),
        }
    }
}

impl From<SerializableVelocity> for bevy_rapier3d::prelude::Velocity {
    fn from(serializable: SerializableVelocity) -> bevy_rapier3d::prelude::Velocity {
        bevy_rapier3d::prelude::Velocity {
            linvel: Vec3::from_array(serializable.linear),
            angvel: Vec3::from_array(serializable.angular),
        }
    }
}

/// Serializable player data
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializablePlayer {
    pub entity_id: u32,
    pub transform: SerializableTransform,
    pub velocity: SerializableVelocity,
    pub player_state: PlayerState,
    pub in_vehicle: Option<u32>,
    pub health: f32,
}

/// Serializable vehicle data
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableVehicle {
    pub entity_id: u32,
    pub transform: SerializableTransform,
    pub velocity: SerializableVelocity,
    pub vehicle_config: SerializableVehicleConfig,
    pub input_state: SerializableVehicleInput,
    pub engine_data: Option<SerializableEngine>,
    pub transmission_data: Option<SerializableTransmission>,
    pub suspension_data: Option<SerializableSuspension>,
    pub steering_data: Option<SerializableSteering>,
    pub brakes_data: Option<SerializableBrakes>,
    pub car_config: Option<SerializableCarConfig>,
    pub occupied: bool,
    pub occupant: Option<u32>,
}

/// Serializable vehicle configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableVehicleConfig {
    pub mass: f32,
    pub dimensions: [f32; 3],
    pub center_of_mass: [f32; 3],
    pub drag_coefficient: f32,
    pub frontal_area: f32,
}

impl From<Vehicle> for SerializableVehicleConfig {
    fn from(vehicle: Vehicle) -> Self {
        Self {
            mass: vehicle.mass,
            dimensions: vehicle.dimensions.to_array(),
            center_of_mass: vehicle.center_of_mass.to_array(),
            drag_coefficient: vehicle.drag_coefficient,
            frontal_area: vehicle.frontal_area,
        }
    }
}

impl From<SerializableVehicleConfig> for Vehicle {
    fn from(serializable: SerializableVehicleConfig) -> Vehicle {
        Vehicle {
            mass: serializable.mass,
            dimensions: Vec3::from_array(serializable.dimensions),
            center_of_mass: Vec3::from_array(serializable.center_of_mass),
            drag_coefficient: serializable.drag_coefficient,
            frontal_area: serializable.frontal_area,
        }
    }
}

/// Serializable vehicle input state
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableVehicleInput {
    pub throttle: f32,
    pub brake: f32,
    pub steering: f32,
    pub handbrake: f32,
}

impl From<VehicleInput> for SerializableVehicleInput {
    fn from(input: VehicleInput) -> Self {
        Self {
            throttle: input.throttle,
            brake: input.brake,
            steering: input.steering,
            handbrake: input.handbrake,
        }
    }
}

impl From<SerializableVehicleInput> for VehicleInput {
    fn from(serializable: SerializableVehicleInput) -> VehicleInput {
        VehicleInput {
            throttle: serializable.throttle,
            brake: serializable.brake,
            steering: serializable.steering,
            handbrake: serializable.handbrake,
        }
    }
}

/// Serializable engine data
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableEngine {
    pub rpm: f32,
    pub throttle: f32,
    pub torque: f32,
    pub max_rpm: f32,
    pub max_torque: f32,
    pub idle_rpm: f32,
    pub engine_braking: f32,
    pub fuel_consumption: f32,
    pub torque_curve: Vec<(f32, f32)>,
}

impl From<Engine> for SerializableEngine {
    fn from(engine: Engine) -> Self {
        Self {
            rpm: engine.rpm,
            throttle: engine.throttle,
            torque: engine.torque,
            max_rpm: engine.max_rpm,
            max_torque: engine.max_torque,
            idle_rpm: engine.idle_rpm,
            engine_braking: engine.engine_braking,
            fuel_consumption: engine.fuel_consumption,
            torque_curve: engine.torque_curve,
        }
    }
}

impl From<SerializableEngine> for Engine {
    fn from(serializable: SerializableEngine) -> Engine {
        Engine {
            rpm: serializable.rpm,
            throttle: serializable.throttle,
            torque: serializable.torque,
            max_rpm: serializable.max_rpm,
            max_torque: serializable.max_torque,
            idle_rpm: serializable.idle_rpm,
            engine_braking: serializable.engine_braking,
            fuel_consumption: serializable.fuel_consumption,
            torque_curve: serializable.torque_curve,
        }
    }
}

/// Serializable transmission data
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableTransmission {
    pub gear_ratios: Vec<f32>,
    pub current_gear: i32,
    pub final_drive_ratio: f32,
}

impl From<Transmission> for SerializableTransmission {
    fn from(transmission: Transmission) -> Self {
        Self {
            gear_ratios: transmission.gear_ratios,
            current_gear: transmission.current_gear,
            final_drive_ratio: transmission.final_drive_ratio,
        }
    }
}

impl From<SerializableTransmission> for Transmission {
    fn from(serializable: SerializableTransmission) -> Transmission {
        Transmission {
            gear_ratios: serializable.gear_ratios,
            current_gear: serializable.current_gear,
            final_drive_ratio: serializable.final_drive_ratio,
        }
    }
}

/// Serializable suspension data
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableSuspension {
    pub spring_stiffness: f32,
    pub damper_damping: f32,
    pub max_compression: f32,
    pub max_extension: f32,
    pub rest_length: f32,
    pub anti_roll_bar_stiffness: f32,
    pub travel: f32,
}

impl From<Suspension> for SerializableSuspension {
    fn from(suspension: Suspension) -> Self {
        Self {
            spring_stiffness: suspension.spring_stiffness,
            damper_damping: suspension.damper_damping,
            max_compression: suspension.max_compression,
            max_extension: suspension.max_extension,
            rest_length: suspension.rest_length,
            anti_roll_bar_stiffness: suspension.anti_roll_bar_stiffness,
            travel: suspension.travel,
        }
    }
}

impl From<SerializableSuspension> for Suspension {
    fn from(serializable: SerializableSuspension) -> Suspension {
        Suspension {
            spring_stiffness: serializable.spring_stiffness,
            damper_damping: serializable.damper_damping,
            max_compression: serializable.max_compression,
            max_extension: serializable.max_extension,
            rest_length: serializable.rest_length,
            anti_roll_bar_stiffness: serializable.anti_roll_bar_stiffness,
            travel: serializable.travel,
        }
    }
}

/// Serializable steering data
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableSteering {
    pub angle: f32,
    pub max_angle: f32,
    pub steering_rate: f32,
    pub return_force: f32,
    pub wheelbase: f32,
    pub track_width: f32,
}

impl From<Steering> for SerializableSteering {
    fn from(steering: Steering) -> Self {
        Self {
            angle: steering.angle,
            max_angle: steering.max_angle,
            steering_rate: steering.steering_rate,
            return_force: steering.return_force,
            wheelbase: steering.wheelbase,
            track_width: steering.track_width,
        }
    }
}

impl From<SerializableSteering> for Steering {
    fn from(serializable: SerializableSteering) -> Steering {
        Steering {
            angle: serializable.angle,
            max_angle: serializable.max_angle,
            steering_rate: serializable.steering_rate,
            return_force: serializable.return_force,
            wheelbase: serializable.wheelbase,
            track_width: serializable.track_width,
        }
    }
}

/// Serializable brakes data
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableBrakes {
    pub brake_input: f32,
    pub max_brake_torque: f32,
    pub brake_bias: f32,
    pub abs_enabled: bool,
    pub abs_threshold: f32,
}

impl From<Brakes> for SerializableBrakes {
    fn from(brakes: Brakes) -> Self {
        Self {
            brake_input: brakes.brake_input,
            max_brake_torque: brakes.max_brake_torque,
            brake_bias: brakes.brake_bias,
            abs_enabled: brakes.abs_enabled,
            abs_threshold: brakes.abs_threshold,
        }
    }
}

impl From<SerializableBrakes> for Brakes {
    fn from(serializable: SerializableBrakes) -> Brakes {
        Brakes {
            brake_input: serializable.brake_input,
            max_brake_torque: serializable.max_brake_torque,
            brake_bias: serializable.brake_bias,
            abs_enabled: serializable.abs_enabled,
            abs_threshold: serializable.abs_threshold,
        }
    }
}

/// Serializable car configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableCarConfig {
    pub car_type: String,
    pub performance_tier: u8,
    pub fuel_capacity: f32,
    pub fuel_level: f32,
}

impl From<CarConfig> for SerializableCarConfig {
    fn from(config: CarConfig) -> Self {
        Self {
            car_type: config.car_type,
            performance_tier: config.performance_tier,
            fuel_capacity: config.fuel_capacity,
            fuel_level: config.fuel_level,
        }
    }
}

impl From<SerializableCarConfig> for CarConfig {
    fn from(serializable: SerializableCarConfig) -> CarConfig {
        CarConfig {
            car_type: serializable.car_type,
            performance_tier: serializable.performance_tier,
            fuel_capacity: serializable.fuel_capacity,
            fuel_level: serializable.fuel_level,
        }
    }
}

/// Main save game state container
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SaveGameState {
    pub version: u32,
    pub timestamp: DateTime<Utc>,
    pub player_state: PlayerState,
    pub active_entity_id: Option<u32>,
    pub player: SerializablePlayer,
    pub vehicles: Vec<SerializableVehicle>,
    pub world_seed: Option<u64>,
    pub play_time: f64,
    pub metadata: SaveMetadata,
}

/// Additional save game metadata
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SaveMetadata {
    pub save_name: String,
    pub level_name: String,
    pub difficulty: String,
    pub achievements: Vec<String>,
    pub statistics: GameStatistics,
}

/// Game statistics
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameStatistics {
    pub distance_traveled: f32,
    pub vehicles_driven: u32,
    pub time_in_vehicles: f32,
    pub missions_completed: u32,
}

impl Default for SaveMetadata {
    fn default() -> Self {
        Self {
            save_name: "Quicksave".to_string(),
            level_name: "City".to_string(),
            difficulty: "Normal".to_string(),
            achievements: Vec::new(),
            statistics: GameStatistics::default(),
        }
    }
}

impl Default for GameStatistics {
    fn default() -> Self {
        Self {
            distance_traveled: 0.0,
            vehicles_driven: 0,
            time_in_vehicles: 0.0,
            missions_completed: 0,
        }
    }
}

/// Save version constant
pub const SAVE_VERSION: u32 = 1;

impl SaveGameState {
    /// Validate the save game state for consistency
    pub fn validate(&self) -> Result<(), String> {
        // Version compatibility check
        if self.version > SAVE_VERSION {
            return Err(format!(
                "Save version {} is too new (current: {})",
                self.version, SAVE_VERSION
            ));
        }

        // Check that active entity exists
        if let Some(active_id) = self.active_entity_id {
            let found = self.player.entity_id == active_id
                || self.vehicles.iter().any(|v| v.entity_id == active_id);
            if !found {
                return Err("ActiveEntity reference not found in saved entities".to_string());
            }
        }

        // Validate player state consistency
        match self.player_state {
            PlayerState::Walking => {
                if self.player.in_vehicle.is_some() {
                    return Err("Walking state but player is in vehicle".to_string());
                }
            }
            PlayerState::Driving => {
                if self.player.in_vehicle.is_none() {
                    return Err("Driving state but player not in vehicle".to_string());
                }
                // Check that the vehicle exists
                if let Some(vehicle_id) = self.player.in_vehicle {
                    let vehicle_exists = self.vehicles.iter().any(|v| v.entity_id == vehicle_id);
                    if !vehicle_exists {
                        return Err("Player in vehicle but vehicle not found".to_string());
                    }
                }
            }
        }

        // Physics bounds validation
        let max_position = 10000.0;
        let max_velocity = 1000.0;

        // Check player bounds
        let player_pos = &self.player.transform.translation;
        if player_pos[0].abs() > max_position
            || player_pos[1].abs() > max_position
            || player_pos[2].abs() > max_position
        {
            return Err("Invalid player position detected".to_string());
        }

        let player_vel = &self.player.velocity.linear;
        if player_vel[0].abs() > max_velocity
            || player_vel[1].abs() > max_velocity
            || player_vel[2].abs() > max_velocity
        {
            return Err("Invalid player velocity detected".to_string());
        }

        // Check vehicle bounds
        for vehicle in &self.vehicles {
            let pos = &vehicle.transform.translation;
            if pos[0].abs() > max_position
                || pos[1].abs() > max_position
                || pos[2].abs() > max_position
            {
                return Err("Invalid vehicle position detected".to_string());
            }

            let vel = &vehicle.velocity.linear;
            if vel[0].abs() > max_velocity
                || vel[1].abs() > max_velocity
                || vel[2].abs() > max_velocity
            {
                return Err("Invalid vehicle velocity detected".to_string());
            }
        }

        Ok(())
    }
}
