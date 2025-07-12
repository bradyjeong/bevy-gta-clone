//! Tests for vehicle systems and components

use super::components::*;

use bevy::prelude::*;
use config_core::{AudioConfig, ConfigLoader};
use tempfile::TempDir;

#[test]
fn test_vehicle_component_default() {
    let vehicle = Vehicle::default();
    assert_eq!(vehicle.mass, 1500.0);
    assert_eq!(vehicle.dimensions, Vec3::new(4.5, 2.0, 1.8));
    assert_eq!(vehicle.center_of_mass, Vec3::new(0.0, -0.5, 0.0));
    assert_eq!(vehicle.drag_coefficient, 0.3);
    assert_eq!(vehicle.frontal_area, 2.5);
}

#[test]
fn test_physics_vehicle_component() {
    let vehicle = PhysicsVehicle;
    assert_eq!(vehicle, PhysicsVehicle);
}

#[test]
fn test_engine_component_default() {
    let engine = Engine::default();
    assert_eq!(engine.rpm, 0.0);
    assert_eq!(engine.throttle, 0.0);
    assert_eq!(engine.torque, 0.0);
    assert_eq!(engine.max_rpm, 7000.0);
    assert_eq!(engine.max_torque, 300.0);
    assert_eq!(engine.idle_rpm, 800.0);
    assert_eq!(engine.engine_braking, 0.3);
    assert_eq!(engine.fuel_consumption, 15.0);
    assert_eq!(engine.torque_curve.len(), 7);
}

#[test]
fn test_engine_torque_curve() {
    let engine = Engine::default();

    // Check that torque curve has expected values
    assert_eq!(engine.torque_curve[0], (0.0, 0.0)); // Start
    assert_eq!(engine.torque_curve[1], (800.0, 120.0)); // Idle
    assert_eq!(engine.torque_curve[2], (1500.0, 250.0)); // Low-end
    assert_eq!(engine.torque_curve[3], (3000.0, 300.0)); // Peak
    assert_eq!(engine.torque_curve[6], (7000.0, 150.0)); // Redline
}

#[test]
fn test_transmission_component_default() {
    let transmission = Transmission::default();
    assert_eq!(transmission.current_gear, 1);
    assert_eq!(transmission.final_drive_ratio, 4.1);
    assert_eq!(transmission.gear_ratios.len(), 8);
    assert_eq!(transmission.gear_ratios[0], -3.5); // Reverse
    assert_eq!(transmission.gear_ratios[1], 0.0); // Neutral
    assert_eq!(transmission.gear_ratios[2], 3.5); // 1st gear
}

#[test]
fn test_suspension_component_default() {
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
fn test_drivetrain_component_default() {
    let drivetrain = Drivetrain::default();
    assert_eq!(drivetrain.drive_type, 1); // RWD
    assert_eq!(drivetrain.differential_ratio, 3.73);
    assert_eq!(drivetrain.power_split, 0.5); // 50/50 for AWD
    assert_eq!(drivetrain.efficiency, 0.85);
}

#[test]
fn test_steering_component_default() {
    let steering = Steering::default();
    assert_eq!(steering.angle, 0.0);
    assert_eq!(steering.max_angle, std::f32::consts::FRAC_PI_4); // 45 degrees
    assert_eq!(steering.steering_rate, 3.0);
    assert_eq!(steering.return_force, 10.0);
    assert_eq!(steering.wheelbase, 2.7);
    assert_eq!(steering.track_width, 1.5);
}

#[test]
fn test_brakes_component_default() {
    let brakes = Brakes::default();
    assert_eq!(brakes.brake_input, 0.0);
    assert_eq!(brakes.max_brake_torque, 2000.0);
    assert_eq!(brakes.brake_bias, 0.7); // 70% front
    assert!(brakes.abs_enabled);
    assert_eq!(brakes.abs_threshold, 0.15); // 15% slip
}

#[test]
fn test_vehicle_input_component_default() {
    let input = PhysicsVehicleInput::default();
    assert_eq!(input.throttle, 0.0);
    assert_eq!(input.brake, 0.0);
    assert_eq!(input.steering, 0.0);
    assert_eq!(input.handbrake, 0.0);
    assert_eq!(input.smoothing, 0.15);
    assert_eq!(input.deadzone, 0.05);
}

#[test]
fn test_wheel_physics_component_default() {
    let wheel = WheelPhysics::default();
    assert_eq!(wheel.radius, 0.35);
    assert_eq!(wheel.mass, 20.0);
    assert_eq!(wheel.angular_velocity, 0.0);
    assert_eq!(wheel.motor_torque, 0.0);
    assert_eq!(wheel.brake_torque, 0.0);
    assert_eq!(wheel.slip_ratio, 0.0);
    assert_eq!(wheel.lateral_force, 0.0);
    assert_eq!(wheel.longitudinal_force, 0.0);
    assert!(!wheel.ground_contact);
    assert_eq!(wheel.friction_coefficient, 0.8);
}

#[test]
fn test_wheel_component_default() {
    let wheel = Wheel::default();
    assert_eq!(wheel.radius, 0.3);
    assert_eq!(wheel.width, 0.225);
    assert_eq!(wheel.friction_coefficient, 0.8);
    assert_eq!(wheel.rotation_angle, 0.0);
    assert_eq!(wheel.position, Vec3::ZERO);
    assert!(!wheel.is_steered);
    assert!(!wheel.is_driven);
}

#[test]
fn test_vehicle_input_component_default_simple() {
    let input = VehicleInput::default();
    assert_eq!(input.throttle, 0.0);
    assert_eq!(input.brake, 0.0);
    assert_eq!(input.steering, 0.0);
    assert_eq!(input.handbrake, 0.0);
}

#[test]
fn test_vehicle_audio_uses_config() {
    let audio = VehicleAudio::default();

    // Should load from config or use fallback values
    assert!(audio.engine_volume >= 0.0 && audio.engine_volume <= 1.0);
    assert!(audio.tire_screech_volume >= 0.0 && audio.tire_screech_volume <= 1.0);
}

#[test]
fn test_vehicle_audio_with_custom_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("audio.ron");

    // Write custom audio config
    std::fs::write(
        &config_path,
        r#"(
            vehicle: (
                engine_sound_enabled: false,
                default_engine_volume: 0.3,
                tire_screech_enabled: true,
                default_tire_screech_volume: 0.7,
            ),
        )"#,
    )
    .unwrap();

    // Test that config would be loaded (simulated)
    let loader = ConfigLoader {
        search_paths: vec![temp_dir.path().to_path_buf()],
    };

    let config: AudioConfig = loader.load_with_merge().unwrap();
    assert!(!config.vehicle.engine_sound_enabled);
    assert_eq!(config.vehicle.default_engine_volume, 0.3);
    assert!(config.vehicle.tire_screech_enabled);
    assert_eq!(config.vehicle.default_tire_screech_volume, 0.7);
}

#[test]
fn test_car_config_component_default() {
    let config = CarConfig::default();
    assert_eq!(config.car_type, "sedan");
    assert_eq!(config.performance_tier, 3);
    assert_eq!(config.fuel_capacity, 60.0);
    assert_eq!(config.fuel_level, 60.0);
}

#[test]
fn test_component_serialization() {
    // Test Engine serialization
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
fn test_transmission_serialization() {
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
fn test_suspension_serialization() {
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
fn test_physics_vehicle_input_serialization() {
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

#[test]
fn test_wheel_constraints() {
    let wheel = Wheel::default();

    // Validate wheel constraints
    assert!(wheel.radius > 0.0);
    assert!(wheel.width > 0.0);
    assert!(wheel.friction_coefficient >= 0.0);
}

#[test]
fn test_engine_constraints() {
    let engine = Engine::default();

    // Validate engine constraints
    assert!(engine.max_rpm > 0.0);
    assert!(engine.max_torque > 0.0);
    assert!(engine.idle_rpm >= 0.0);
    assert!(engine.idle_rpm < engine.max_rpm);
    assert!(engine.engine_braking >= 0.0);
    assert!(engine.fuel_consumption > 0.0);
}

#[test]
fn test_transmission_gear_ratios() {
    let transmission = Transmission::default();

    // First gear should be reverse (negative)
    assert!(transmission.gear_ratios[0] < 0.0);
    // Second gear should be neutral (zero)
    assert_eq!(transmission.gear_ratios[1], 0.0);
    // Subsequent gears should be positive and decreasing
    for i in 2..transmission.gear_ratios.len() - 1 {
        assert!(transmission.gear_ratios[i] > 0.0);
        assert!(transmission.gear_ratios[i] > transmission.gear_ratios[i + 1]);
    }
}

#[test]
fn test_suspension_physics_constraints() {
    let suspension = Suspension::default();

    // Validate physics constraints
    assert!(suspension.spring_stiffness > 0.0);
    assert!(suspension.damper_damping > 0.0);
    assert!(suspension.max_compression > 0.0);
    assert!(suspension.max_extension > 0.0);
    assert!(suspension.rest_length > 0.0);
    assert!(suspension.travel > 0.0);
    assert!(suspension.anti_roll_bar_stiffness >= 0.0);
}

#[test]
fn test_steering_angle_constraints() {
    let steering = Steering::default();

    // Validate steering constraints
    assert!(steering.max_angle > 0.0);
    assert!(steering.max_angle <= std::f32::consts::PI); // Should not exceed 180 degrees
    assert!(steering.steering_rate > 0.0);
    assert!(steering.wheelbase > 0.0);
    assert!(steering.track_width > 0.0);
}

#[test]
fn test_brakes_physics_constraints() {
    let brakes = Brakes::default();

    // Validate brake constraints
    assert!(brakes.max_brake_torque > 0.0);
    assert!(brakes.brake_bias >= 0.0 && brakes.brake_bias <= 1.0);
    assert!(brakes.abs_threshold > 0.0 && brakes.abs_threshold < 1.0);
}

#[test]
fn test_vehicle_input_bounds() {
    let input = PhysicsVehicleInput::default();

    // Input values should be in expected ranges
    assert!(input.throttle >= 0.0 && input.throttle <= 1.0);
    assert!(input.brake >= 0.0 && input.brake <= 1.0);
    assert!(input.steering >= -1.0 && input.steering <= 1.0);
    assert!(input.handbrake >= 0.0 && input.handbrake <= 1.0);
    assert!(input.smoothing >= 0.0 && input.smoothing <= 1.0);
    assert!(input.deadzone >= 0.0 && input.deadzone <= 1.0);
}

#[test]
fn test_vehicle_mass_constraints() {
    let vehicle = Vehicle::default();

    // Vehicle should have positive mass and reasonable dimensions
    assert!(vehicle.mass > 0.0);
    assert!(vehicle.dimensions.x > 0.0);
    assert!(vehicle.dimensions.y > 0.0);
    assert!(vehicle.dimensions.z > 0.0);
    assert!(vehicle.drag_coefficient >= 0.0);
    assert!(vehicle.frontal_area > 0.0);
}

#[test]
fn test_drivetrain_constraints() {
    let drivetrain = Drivetrain::default();

    // Validate drivetrain constraints
    assert!(drivetrain.drive_type <= 2); // 0=FWD, 1=RWD, 2=AWD
    assert!(drivetrain.differential_ratio > 0.0);
    assert!(drivetrain.power_split >= 0.0 && drivetrain.power_split <= 1.0);
    assert!(drivetrain.efficiency > 0.0 && drivetrain.efficiency <= 1.0);
}

#[test]
fn test_wheel_physics_constraints() {
    let wheel = WheelPhysics::default();

    // Validate wheel physics constraints
    assert!(wheel.radius > 0.0);
    assert!(wheel.mass > 0.0);
    assert!(wheel.friction_coefficient >= 0.0);
}

#[test]
fn test_car_config_constraints() {
    let config = CarConfig::default();

    // Validate car config constraints
    assert!(!config.car_type.is_empty());
    assert!(config.performance_tier >= 1 && config.performance_tier <= 5);
    assert!(config.fuel_capacity > 0.0);
    assert!(config.fuel_level >= 0.0 && config.fuel_level <= config.fuel_capacity);
}
