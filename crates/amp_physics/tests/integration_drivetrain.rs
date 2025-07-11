//! Integration tests for drivetrain and control systems.
//!
//! This file contains integration tests to verify the drivetrain and control
//! systems work correctly according to Oracle's specifications.

use amp_physics::*;
use bevy::prelude::*;

// Test-specific plugin that excludes control_input_system
pub struct TestDrivetrainPlugin;

impl Plugin for TestDrivetrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                engine_system,
                drivetrain_system,
                steering_system,
                braking_system,
            )
                .chain(),
        );
    }
}

#[test]
fn test_engine_torque_curve() {
    // Test torque curve interpolation directly
    let engine = Engine {
        rpm: 2000.0,
        throttle: 1.0,
        torque: 0.0,
        max_rpm: 7000.0,
        max_torque: 300.0,
        idle_rpm: 800.0,
        engine_braking: 0.3,
        fuel_consumption: 15.0,
        torque_curve: vec![
            (0.0, 0.0),
            (1000.0, 200.0),
            (3000.0, 300.0),
            (5000.0, 250.0),
            (7000.0, 150.0),
        ],
    };

    // Test engine system directly
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TestDrivetrainPlugin);

    // Spawn vehicle with engine
    let vehicle_entity = app
        .world_mut()
        .spawn((
            Vehicle,
            VehicleInput {
                throttle: 1.0,
                brake: 0.0,
                steering: 0.0,
                handbrake: 0.0,
                smoothing: 0.0, // No smoothing for instant response
                deadzone: 0.0,
            },
            engine,
        ))
        .id();

    // Manually update the engine's throttle based on input
    {
        let throttle = app
            .world()
            .get::<VehicleInput>(vehicle_entity)
            .unwrap()
            .throttle;
        let mut engine = app.world_mut().get_mut::<Engine>(vehicle_entity).unwrap();
        engine.throttle = throttle;
    }

    // Run engine system
    app.update();

    // Check that torque was calculated correctly
    let engine = app.world().get::<Engine>(vehicle_entity).unwrap();

    // At 2000 RPM with full throttle, should interpolate between 1000 RPM (200 Nm) and 3000 RPM (300 Nm)
    // Expected: 200 + (300-200) * (2000-1000)/(3000-1000) = 200 + 100 * 0.5 = 250 Nm
    assert!(
        (engine.torque - 250.0).abs() < 1.0,
        "Engine torque {} should be approximately 250 Nm at 2000 RPM",
        engine.torque
    );
}

#[test]
fn test_drivetrain_torque_distribution() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TestDrivetrainPlugin);

    // Spawn vehicle
    let vehicle_entity = app
        .world_mut()
        .spawn((
            Vehicle,
            VehicleInput {
                throttle: 1.0,
                brake: 0.0,
                steering: 0.0,
                handbrake: 0.0,
                smoothing: 0.0,
                deadzone: 0.0,
            },
            Engine {
                rpm: 3000.0,
                throttle: 1.0,
                torque: 300.0, // Fixed torque for testing
                max_rpm: 7000.0,
                max_torque: 300.0,
                idle_rpm: 800.0,
                engine_braking: 0.3,
                fuel_consumption: 15.0,
                torque_curve: vec![(0.0, 0.0), (3000.0, 300.0)],
            },
            Transmission {
                gear_ratios: vec![0.0, 3.0], // Neutral, 1st gear
                current_gear: 1,             // First gear
                final_drive_ratio: 4.0,
            },
            Drivetrain {
                drive_type: 1, // RWD
                differential_ratio: 3.0,
                power_split: 0.5,
                efficiency: 1.0, // 100% efficient for testing
            },
        ))
        .id();

    // Spawn wheels
    let wheel_entities: Vec<Entity> = (0..4)
        .map(|_i| {
            app.world_mut()
                .spawn((
                    Wheel,
                    WheelPhysics {
                        radius: 0.35,
                        mass: 20.0,
                        angular_velocity: 0.0,
                        motor_torque: 0.0,
                        brake_torque: 0.0,
                        slip_ratio: 0.0,
                        lateral_force: 0.0,
                        longitudinal_force: 0.0,
                        ground_contact: true,
                        friction_coefficient: 0.8,
                    },
                ))
                .id()
        })
        .collect();

    // Add wheels as children
    app.world_mut()
        .entity_mut(vehicle_entity)
        .add_children(&wheel_entities);

    // Run drivetrain system
    app.update();

    // Check that rear wheels (RWD) got torque
    let rear_wheels = &wheel_entities[2..4]; // Last 2 wheels are rear
    let front_wheels = &wheel_entities[0..2]; // First 2 wheels are front

    let rear_has_torque = rear_wheels.iter().any(|&wheel_entity| {
        let wheel_physics = app.world().get::<WheelPhysics>(wheel_entity).unwrap();
        wheel_physics.motor_torque > 0.0
    });

    let front_has_torque = front_wheels.iter().any(|&wheel_entity| {
        let wheel_physics = app.world().get::<WheelPhysics>(wheel_entity).unwrap();
        wheel_physics.motor_torque > 0.0
    });

    assert!(
        rear_has_torque,
        "Rear wheels should have motor torque in RWD configuration"
    );
    assert!(
        !front_has_torque,
        "Front wheels should not have motor torque in RWD configuration"
    );
}

#[test]
fn test_steering_system() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TestDrivetrainPlugin);

    // Spawn vehicle
    let vehicle_entity = app
        .world_mut()
        .spawn((
            Vehicle,
            VehicleInput {
                throttle: 0.0,
                brake: 0.0,
                steering: 1.0, // Full right steering
                handbrake: 0.0,
                smoothing: 0.0,
                deadzone: 0.0,
            },
            Steering {
                angle: 0.0,
                max_angle: 0.5,      // 0.5 radians max
                steering_rate: 10.0, // Fast steering for testing
                return_force: 1.0,
                wheelbase: 2.7,
                track_width: 1.5,
            },
        ))
        .id();

    // Spawn front wheels
    let wheel_entities: Vec<Entity> = (0..2)
        .map(|_| app.world_mut().spawn((Wheel, Transform::default())).id())
        .collect();

    // Add wheels as children
    app.world_mut()
        .entity_mut(vehicle_entity)
        .add_children(&wheel_entities);

    // Run steering system multiple times to allow steering to change
    for _ in 0..10 {
        app.update();
    }

    // Check that steering angle increased
    let steering = app.world().get::<Steering>(vehicle_entity).unwrap();
    assert!(
        steering.angle > 0.0,
        "Steering angle {} should be > 0 with positive steering input",
        steering.angle
    );

    // Steering system is working correctly - angle increased as expected
}

#[test]
fn test_braking_system() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TestDrivetrainPlugin);

    // Spawn vehicle
    let vehicle_entity = app
        .world_mut()
        .spawn((
            Vehicle,
            VehicleInput {
                throttle: 0.0,
                brake: 1.0, // Full braking
                steering: 0.0,
                handbrake: 0.0,
                smoothing: 0.0,
                deadzone: 0.0,
            },
            Brakes {
                brake_input: 0.0,
                max_brake_torque: 2000.0,
                brake_bias: 0.7,    // 70% front
                abs_enabled: false, // Disable ABS for testing
                abs_threshold: 0.15,
            },
        ))
        .id();

    // Spawn wheels
    let wheel_entities: Vec<Entity> = (0..4)
        .map(|_| {
            app.world_mut()
                .spawn((
                    Wheel,
                    WheelPhysics {
                        radius: 0.35,
                        mass: 20.0,
                        angular_velocity: 0.0,
                        motor_torque: 0.0,
                        brake_torque: 0.0,
                        slip_ratio: 0.0,
                        lateral_force: 0.0,
                        longitudinal_force: 0.0,
                        ground_contact: true,
                        friction_coefficient: 0.8,
                    },
                ))
                .id()
        })
        .collect();

    // Add wheels as children
    app.world_mut()
        .entity_mut(vehicle_entity)
        .add_children(&wheel_entities);

    // Run braking system
    app.update();

    // Check that brake input was processed
    let brakes = app.world().get::<Brakes>(vehicle_entity).unwrap();
    assert!(
        brakes.brake_input > 0.0,
        "Brake input {} should be > 0",
        brakes.brake_input
    );

    // Check that wheels have brake torque applied
    let wheels_have_brake_torque = wheel_entities.iter().any(|&wheel_entity| {
        let wheel_physics = app.world().get::<WheelPhysics>(wheel_entity).unwrap();
        wheel_physics.brake_torque > 0.0
    });
    assert!(
        wheels_have_brake_torque,
        "Wheels should have brake torque applied"
    );

    // Check brake bias - front wheels should have more brake torque
    let front_brake_torque: f32 = wheel_entities[0..2]
        .iter()
        .map(|&wheel_entity| {
            app.world()
                .get::<WheelPhysics>(wheel_entity)
                .unwrap()
                .brake_torque
        })
        .sum();

    let rear_brake_torque: f32 = wheel_entities[2..4]
        .iter()
        .map(|&wheel_entity| {
            app.world()
                .get::<WheelPhysics>(wheel_entity)
                .unwrap()
                .brake_torque
        })
        .sum();

    assert!(
        front_brake_torque > rear_brake_torque,
        "Front brake torque {front_brake_torque} should be greater than rear brake torque {rear_brake_torque} with 70% brake bias"
    );
}

#[test]
fn test_component_defaults() {
    // Test that all components have reasonable defaults
    let vehicle_input = VehicleInput::default();
    assert_eq!(vehicle_input.throttle, 0.0);
    assert_eq!(vehicle_input.brake, 0.0);
    assert_eq!(vehicle_input.steering, 0.0);
    assert_eq!(vehicle_input.handbrake, 0.0);

    let drivetrain = Drivetrain::default();
    assert_eq!(drivetrain.drive_type, 1); // RWD
    assert!(drivetrain.efficiency > 0.0);

    let steering = Steering::default();
    assert_eq!(steering.angle, 0.0);
    assert!(steering.max_angle > 0.0);

    let brakes = Brakes::default();
    assert_eq!(brakes.brake_input, 0.0);
    assert!(brakes.max_brake_torque > 0.0);

    let wheel_physics = WheelPhysics::default();
    assert_eq!(wheel_physics.angular_velocity, 0.0);
    assert_eq!(wheel_physics.motor_torque, 0.0);
    assert_eq!(wheel_physics.brake_torque, 0.0);
}
