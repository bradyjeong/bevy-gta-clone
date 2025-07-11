//! Vehicle components for ECS
//!
//! Contains all vehicle-related components that define vehicle behavior,
//! physics properties, and state.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Main vehicle component
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Vehicle {
    /// Vehicle mass in kg
    pub mass: f32,
    /// Vehicle dimensions (length, width, height)
    pub dimensions: Vec3,
    /// Center of mass offset from transform
    pub center_of_mass: Vec3,
    /// Drag coefficient
    pub drag_coefficient: f32,
    /// Frontal area for aerodynamic calculations
    pub frontal_area: f32,
}

impl Default for Vehicle {
    fn default() -> Self {
        Self {
            mass: 1500.0,                         // kg
            dimensions: Vec3::new(4.5, 2.0, 1.8), // meters
            center_of_mass: Vec3::new(0.0, -0.5, 0.0),
            drag_coefficient: 0.3,
            frontal_area: 2.5, // m²
        }
    }
}

// VehicleEngine removed - use amp_physics::components::Engine instead

// VehicleSuspension removed - use amp_physics::components::Suspension instead

// VehicleSteering removed - use amp_physics::components::Steering instead

/// Wheel component for individual wheels
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Wheel {
    /// Wheel radius in meters
    pub radius: f32,
    /// Wheel width in meters
    pub width: f32,
    /// Tire friction coefficient
    pub friction_coefficient: f32,
    /// Current rotation angle
    pub rotation_angle: f32,
    /// Wheel position relative to vehicle center
    pub position: Vec3,
    /// Whether this wheel is steered
    pub is_steered: bool,
    /// Whether this wheel is driven
    pub is_driven: bool,
}

impl Default for Wheel {
    fn default() -> Self {
        Self {
            radius: 0.3,  // m
            width: 0.225, // m
            friction_coefficient: 0.8,
            rotation_angle: 0.0,
            position: Vec3::ZERO,
            is_steered: false,
            is_driven: false,
        }
    }
}

// VehiclePhysics removed - use amp_physics components instead

/// Vehicle input component
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct VehicleInput {
    /// Throttle input (0.0 to 1.0)
    pub throttle: f32,
    /// Brake input (0.0 to 1.0)
    pub brake: f32,
    /// Steering input (-1.0 to 1.0)
    pub steering: f32,
    /// Handbrake input (0.0 to 1.0)
    pub handbrake: f32,
}

impl Default for VehicleInput {
    fn default() -> Self {
        Self {
            throttle: 0.0,
            brake: 0.0,
            steering: 0.0,
            handbrake: 0.0,
        }
    }
}

/// Vehicle audio component
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct VehicleAudio {
    /// Engine sound enabled
    pub engine_sound_enabled: bool,
    /// Current engine sound volume
    pub engine_volume: f32,
    /// Tire screech sound enabled
    pub tire_screech_enabled: bool,
    /// Current tire screech volume
    pub tire_screech_volume: f32,
}

impl Default for VehicleAudio {
    fn default() -> Self {
        Self {
            engine_sound_enabled: true,
            engine_volume: 0.5,
            tire_screech_enabled: true,
            tire_screech_volume: 0.3,
        }
    }
}

/// Car-specific configuration component
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct CarConfig {
    /// Car type (sedan, suv, sports, etc.)
    pub car_type: String,
    /// Performance tier (1-5)
    pub performance_tier: u8,
    /// Fuel capacity in liters
    pub fuel_capacity: f32,
    /// Current fuel level
    pub fuel_level: f32,
}

impl Default for CarConfig {
    fn default() -> Self {
        Self {
            car_type: "sedan".to_string(),
            performance_tier: 3,
            fuel_capacity: 60.0,
            fuel_level: 60.0,
        }
    }
}
