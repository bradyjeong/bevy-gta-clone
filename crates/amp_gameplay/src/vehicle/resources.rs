//! Vehicle system resources
//!
//! Global resources for vehicle systems including configuration,
//! physics constants, and shared state.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Global vehicle physics configuration
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct VehiclePhysicsConfig {
    /// Air density at sea level (kg/m³)
    pub air_density: f32,
    /// Tire-ground friction coefficient
    pub ground_friction: f32,
    /// Rolling resistance coefficient
    pub rolling_resistance: f32,
    /// Maximum tire slip angle before losing grip
    pub max_tire_slip: f32,
    /// Downforce coefficient
    pub downforce_coefficient: f32,
}

impl Default for VehiclePhysicsConfig {
    fn default() -> Self {
        Self {
            air_density: 1.225,
            ground_friction: 0.8,
            rolling_resistance: 0.015,
            max_tire_slip: 0.2,
            downforce_coefficient: 0.1,
        }
    }
}

/// Vehicle debug visualization settings
#[derive(Resource, Debug, Clone)]
pub struct VehicleDebugSettings {
    /// Show suspension visualization
    pub show_suspension: bool,
    /// Show force vectors
    pub show_forces: bool,
    /// Show center of mass
    pub show_center_of_mass: bool,
    /// Show wheel contact points
    pub show_wheel_contacts: bool,
    /// Debug line color
    pub debug_color: Color,
}

impl Default for VehicleDebugSettings {
    fn default() -> Self {
        Self {
            show_suspension: false,
            show_forces: false,
            show_center_of_mass: false,
            show_wheel_contacts: false,
            debug_color: Color::srgb(0.0, 1.0, 0.0),
        }
    }
}

/// Vehicle input state resource
#[derive(Resource, Debug, Clone, Default)]
pub struct VehicleInputState {
    /// Throttle input (0.0 to 1.0)
    pub throttle: f32,
    /// Brake input (0.0 to 1.0)
    pub brake: f32,
    /// Steering input (-1.0 to 1.0)
    pub steering: f32,
    /// Handbrake engaged
    pub handbrake: bool,
    /// Gear selection
    pub gear: VehicleGear,
}

/// Vehicle gear states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VehicleGear {
    Park,
    Reverse,
    Neutral,
    Drive,
    Manual(u8),
}

impl Default for VehicleGear {
    fn default() -> Self {
        Self::Park
    }
}

/// Vehicle performance metrics
#[derive(Resource, Debug, Clone, Default)]
pub struct VehicleMetrics {
    /// Current speed in m/s
    pub current_speed: f32,
    /// Maximum recorded speed
    pub max_speed: f32,
    /// Current acceleration in m/s²
    pub acceleration: f32,
    /// Engine load percentage
    pub engine_load: f32,
    /// Fuel consumption rate
    pub fuel_consumption: f32,
    /// Tire wear levels [0.0 to 1.0]
    pub tire_wear: [f32; 4],
}
