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

/// Marker component for vehicle entities.
///
/// This component identifies an entity as a vehicle and enables
/// vehicle-specific physics systems to process it.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct PhysicsVehicle;

impl Default for PhysicsVehicle {
    fn default() -> Self {
        Self
    }
}

/// Engine component containing engine physics parameters.
///
/// This component stores the current state and parameters of a vehicle's engine,
/// including RPM, throttle input, and torque characteristics with realistic torque curves.
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Engine {
    /// Current engine RPM (revolutions per minute)
    pub rpm: f32,
    /// Current throttle input (0.0 to 1.0)
    pub throttle: f32,
    /// Current engine torque output (in Newton-meters)
    pub torque: f32,
    /// Maximum engine RPM
    pub max_rpm: f32,
    /// Maximum engine torque
    pub max_torque: f32,
    /// Idle RPM
    pub idle_rpm: f32,
    /// Engine braking coefficient
    pub engine_braking: f32,
    /// Fuel consumption rate (liters per hour at max power)
    pub fuel_consumption: f32,
    /// Torque curve as (RPM, torque) pairs for realistic engine behavior
    pub torque_curve: Vec<(f32, f32)>,
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            rpm: 0.0,
            throttle: 0.0,
            torque: 0.0,
            max_rpm: 7000.0,
            max_torque: 300.0,
            idle_rpm: 800.0,
            engine_braking: 0.3,
            fuel_consumption: 15.0,
            torque_curve: vec![
                (0.0, 0.0),
                (800.0, 120.0),  // Idle torque
                (1500.0, 250.0), // Low-end torque
                (3000.0, 300.0), // Peak torque
                (4500.0, 280.0), // High-rev torque
                (6000.0, 200.0), // Near redline
                (7000.0, 150.0), // Redline
            ],
        }
    }
}

/// Transmission component containing transmission physics parameters.
///
/// This component stores the current state and parameters of a vehicle's transmission,
/// including gear ratios and current gear selection.
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Transmission {
    /// Gear ratios for each gear (forward gears are positive, reverse is negative)
    pub gear_ratios: Vec<f32>,
    /// Current gear index (0 = neutral, 1+ = forward gears, negative = reverse)
    pub current_gear: i32,
    /// Final drive ratio
    pub final_drive_ratio: f32,
}

impl Default for Transmission {
    fn default() -> Self {
        Self {
            gear_ratios: vec![
                -3.5, // Reverse
                0.0,  // Neutral
                3.5,  // 1st gear
                2.1,  // 2nd gear
                1.4,  // 3rd gear
                1.0,  // 4th gear
                0.8,  // 5th gear
                0.6,  // 6th gear
            ],
            current_gear: 1, // Start in neutral
            final_drive_ratio: 4.1,
        }
    }
}

/// Suspension component containing suspension physics parameters.
///
/// This component stores the suspension configuration for a vehicle,
/// including spring stiffness, damping, and travel parameters.
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Suspension {
    /// Spring stiffness (N/m)
    pub spring_stiffness: f32,
    /// Damper damping coefficient (N·s/m)
    pub damper_damping: f32,
    /// Maximum suspension compression (meters)
    pub max_compression: f32,
    /// Maximum suspension extension (meters)
    pub max_extension: f32,
    /// Rest length of suspension (meters)
    pub rest_length: f32,
    /// Anti-roll bar stiffness (N·m/rad)
    pub anti_roll_bar_stiffness: f32,
    /// Suspension travel (meters)
    pub travel: f32,
}

impl Default for Suspension {
    fn default() -> Self {
        Self {
            spring_stiffness: 35000.0,
            damper_damping: 3500.0,
            max_compression: 0.15,
            max_extension: 0.15,
            rest_length: 0.5,
            anti_roll_bar_stiffness: 15000.0,
            travel: 0.3,
        }
    }
}

/// Drivetrain component containing drivetrain physics parameters.
///
/// This component handles the transmission of power from the engine to the wheels,
/// including gear ratios, differential behavior, and drive type.
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Drivetrain {
    /// Drive type: 0 = FWD, 1 = RWD, 2 = AWD
    pub drive_type: u8,
    /// Differential ratio
    pub differential_ratio: f32,
    /// Power split for AWD (0.0 = full rear, 1.0 = full front)
    pub power_split: f32,
    /// Efficiency of power transmission (0.0 to 1.0)
    pub efficiency: f32,
}

impl Default for Drivetrain {
    fn default() -> Self {
        Self {
            drive_type: 1, // RWD
            differential_ratio: 3.73,
            power_split: 0.5, // 50/50 for AWD
            efficiency: 0.85,
        }
    }
}

/// Steering component containing steering physics parameters.
///
/// This component handles the steering system including Ackermann geometry,
/// steering limits, and return-to-center forces.
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Steering {
    /// Current steering angle in radians
    pub angle: f32,
    /// Maximum steering angle in radians
    pub max_angle: f32,
    /// Steering input rate (radians per second)
    pub steering_rate: f32,
    /// Return-to-center force coefficient
    pub return_force: f32,
    /// Wheelbase for Ackermann geometry
    pub wheelbase: f32,
    /// Track width for Ackermann geometry
    pub track_width: f32,
}

impl Default for Steering {
    fn default() -> Self {
        Self {
            angle: 0.0,
            max_angle: std::f32::consts::FRAC_PI_4, // 45 degrees
            steering_rate: 3.0,                     // 3 rad/s
            return_force: 10.0,
            wheelbase: 2.7,
            track_width: 1.5,
        }
    }
}

/// Braking component containing braking physics parameters.
///
/// This component handles the braking system including brake torque,
/// brake bias, and ABS-like behavior.
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Brakes {
    /// Current brake input (0.0 to 1.0)
    pub brake_input: f32,
    /// Maximum brake torque (Newton-meters)
    pub max_brake_torque: f32,
    /// Brake bias (0.0 = all rear, 1.0 = all front)
    pub brake_bias: f32,
    /// ABS enabled
    pub abs_enabled: bool,
    /// ABS trigger threshold (slip ratio)
    pub abs_threshold: f32,
}

impl Default for Brakes {
    fn default() -> Self {
        Self {
            brake_input: 0.0,
            max_brake_torque: 2000.0,
            brake_bias: 0.7, // 70% front
            abs_enabled: true,
            abs_threshold: 0.15, // 15% slip
        }
    }
}

/// Vehicle input component containing control inputs.
///
/// This component stores the current input state from the player,
/// including throttle, brake, and steering inputs.
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct PhysicsVehicleInput {
    /// Throttle input (0.0 to 1.0)
    pub throttle: f32,
    /// Brake input (0.0 to 1.0)
    pub brake: f32,
    /// Steering input (-1.0 to 1.0)
    pub steering: f32,
    /// Handbrake input (0.0 to 1.0)
    pub handbrake: f32,
    /// Input smoothing factor (0.0 to 1.0)
    pub smoothing: f32,
    /// Input deadzone
    pub deadzone: f32,
}

impl Default for PhysicsVehicleInput {
    fn default() -> Self {
        Self {
            throttle: 0.0,
            brake: 0.0,
            steering: 0.0,
            handbrake: 0.0,
            smoothing: 0.15,
            deadzone: 0.05,
        }
    }
}

/// Wheel physics component containing wheel-specific physics parameters.
///
/// This component stores physics parameters for individual wheels,
/// including slip ratios, forces, and contact information.
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct WheelPhysics {
    /// Wheel radius (meters)
    pub radius: f32,
    /// Wheel mass (kg)
    pub mass: f32,
    /// Wheel angular velocity (rad/s)
    pub angular_velocity: f32,
    /// Applied motor torque (N·m)
    pub motor_torque: f32,
    /// Applied brake torque (N·m)
    pub brake_torque: f32,
    /// Wheel slip ratio
    pub slip_ratio: f32,
    /// Lateral force (N)
    pub lateral_force: f32,
    /// Longitudinal force (N)
    pub longitudinal_force: f32,
    /// Is wheel in contact with ground
    pub ground_contact: bool,
    /// Friction coefficient
    pub friction_coefficient: f32,
}

impl Default for WheelPhysics {
    fn default() -> Self {
        Self {
            radius: 0.35,
            mass: 20.0,
            angular_velocity: 0.0,
            motor_torque: 0.0,
            brake_torque: 0.0,
            slip_ratio: 0.0,
            lateral_force: 0.0,
            longitudinal_force: 0.0,
            ground_contact: false,
            friction_coefficient: 0.8,
        }
    }
}

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
        // Load from config if available, otherwise use fallback values
        if let Ok(config) =
            config_core::ConfigLoader::new().load_with_merge::<config_core::AudioConfig>()
        {
            Self {
                engine_sound_enabled: config.vehicle.engine_sound_enabled,
                engine_volume: config.vehicle.default_engine_volume,
                tire_screech_enabled: config.vehicle.tire_screech_enabled,
                tire_screech_volume: config.vehicle.default_tire_screech_volume,
            }
        } else {
            // Fallback to hardcoded values if config loading fails
            Self {
                engine_sound_enabled: true,
                engine_volume: 0.5,
                tire_screech_enabled: true,
                tire_screech_volume: 0.3,
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physics_vehicle_component_creation() {
        let vehicle = PhysicsVehicle;
        assert_eq!(vehicle, PhysicsVehicle);
    }

    #[test]
    fn engine_component_creation() {
        let engine = Engine::default();
        assert_eq!(engine.rpm, 0.0);
        assert_eq!(engine.throttle, 0.0);
        assert_eq!(engine.torque, 0.0);
        assert_eq!(engine.max_rpm, 7000.0);
        assert_eq!(engine.max_torque, 300.0);
        assert_eq!(engine.idle_rpm, 800.0);
        assert_eq!(engine.engine_braking, 0.3);
        assert_eq!(engine.fuel_consumption, 15.0);
    }

    #[test]
    fn engine_component_serialization() {
        let engine = Engine {
            rpm: 2500.0,
            throttle: 0.5,
            torque: 150.0,
            max_rpm: 7000.0,
            max_torque: 300.0,
            idle_rpm: 800.0,
            engine_braking: 0.3,
            fuel_consumption: 15.0,
            torque_curve: vec![(1000.0, 200.0), (3000.0, 300.0), (5000.0, 250.0)],
        };

        let serialized = serde_json::to_string(&engine).unwrap();
        let deserialized: Engine = serde_json::from_str(&serialized).unwrap();

        assert_eq!(engine, deserialized);
    }

    #[test]
    fn transmission_component_creation() {
        let transmission = Transmission::default();
        assert_eq!(transmission.current_gear, 1);
        assert_eq!(transmission.final_drive_ratio, 4.1);
        assert_eq!(transmission.gear_ratios.len(), 8);
        assert_eq!(transmission.gear_ratios[0], -3.5); // Reverse
        assert_eq!(transmission.gear_ratios[1], 0.0); // Neutral
        assert_eq!(transmission.gear_ratios[2], 3.5); // 1st gear
    }

    #[test]
    fn transmission_component_serialization() {
        let transmission = Transmission {
            gear_ratios: vec![3.5, 2.1, 1.4, 1.0],
            current_gear: 2,
            final_drive_ratio: 4.1,
        };

        let serialized = serde_json::to_string(&transmission).unwrap();
        let deserialized: Transmission = serde_json::from_str(&serialized).unwrap();

        assert_eq!(transmission, deserialized);
    }

    #[test]
    fn suspension_component_creation() {
        let suspension = Suspension::default();
        assert_eq!(suspension.spring_stiffness, 35000.0);
        assert_eq!(suspension.damper_damping, 3500.0);
        assert_eq!(suspension.max_compression, 0.15);
        assert_eq!(suspension.max_extension, 0.15);
        assert_eq!(suspension.rest_length, 0.5);
        assert_eq!(suspension.anti_roll_bar_stiffness, 15000.0);
        assert_eq!(suspension.travel, 0.3);
    }

    #[test]
    fn suspension_component_serialization() {
        let suspension = Suspension {
            spring_stiffness: 40000.0,
            damper_damping: 4000.0,
            max_compression: 0.2,
            max_extension: 0.2,
            rest_length: 0.6,
            anti_roll_bar_stiffness: 20000.0,
            travel: 0.4,
        };

        let serialized = serde_json::to_string(&suspension).unwrap();
        let deserialized: Suspension = serde_json::from_str(&serialized).unwrap();

        assert_eq!(suspension, deserialized);
    }

    #[test]
    fn physics_vehicle_input_component_serialization() {
        let input = PhysicsVehicleInput {
            throttle: 0.5,
            brake: 0.3,
            steering: -0.2,
            handbrake: 0.0,
            smoothing: 0.15,
            deadzone: 0.05,
        };

        let serialized = serde_json::to_string(&input).unwrap();
        let deserialized: PhysicsVehicleInput = serde_json::from_str(&serialized).unwrap();

        assert_eq!(input, deserialized);
    }
}
