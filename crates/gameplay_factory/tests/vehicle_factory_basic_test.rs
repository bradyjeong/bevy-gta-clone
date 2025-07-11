//! Basic integration tests for VehicleFactory.

use amp_physics::{Engine, Suspension, Transmission, Vehicle};
use bevy::prelude::*;
use config_core::VehicleConfig;
use gameplay_factory::VehicleFactory;

/// Test that VehicleFactory spawns exactly 5 entities (1 chassis + 4 wheels).
#[test]
fn test_vehicle_factory_spawn_entity_count() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let factory = VehicleFactory::new();
    let config = VehicleConfig::default();

    // Count entities before spawning
    let initial_count = app.world().entities().len();

    // Spawn vehicle
    let _chassis_entity = {
        let mut commands = app.world_mut().commands();
        factory.spawn_vehicle(&mut commands, &config).unwrap()
    };

    // Apply commands
    app.world_mut().flush();

    // Count entities after spawning
    let final_count = app.world().entities().len();

    // Should have spawned exactly 5 entities (1 chassis + 4 wheels)
    assert_eq!(final_count - initial_count, 5);
}

/// Test that VehicleFactory spawns a complete vehicle with proper components.
#[test]
fn test_vehicle_factory_spawn_complete_vehicle() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let factory = VehicleFactory::new();
    let config = VehicleConfig::default();

    // Spawn vehicle
    let chassis_entity = {
        let mut commands = app.world_mut().commands();
        factory.spawn_vehicle(&mut commands, &config).unwrap()
    };

    // Apply commands
    app.world_mut().flush();

    // Verify chassis entity exists and has correct components
    let chassis = app.world().get_entity(chassis_entity).unwrap();
    assert!(chassis.contains::<Vehicle>());
    assert!(chassis.contains::<Engine>());
    assert!(chassis.contains::<Transmission>());
    assert!(chassis.contains::<Transform>());
    assert!(chassis.contains::<Name>());

    // Verify parent-child hierarchy
    let children = chassis.get::<Children>();
    assert!(children.is_some());

    let children = children.unwrap();
    assert_eq!(children.len(), 4); // Should have 4 wheel children
}

/// Test that VehicleFactory handles engine configuration correctly.
#[test]
fn test_vehicle_factory_engine_configuration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let factory = VehicleFactory::new();
    let mut config = VehicleConfig::default();
    config.engine.max_rpm = 8000.0;
    config.engine.torque_curve_torque = vec![200.0, 350.0, 450.0, 400.0, 300.0];

    // Spawn vehicle
    let chassis_entity = {
        let mut commands = app.world_mut().commands();
        factory.spawn_vehicle(&mut commands, &config).unwrap()
    };

    // Apply commands
    app.world_mut().flush();

    // Verify engine configuration
    let chassis = app.world().get_entity(chassis_entity).unwrap();
    let engine = chassis.get::<Engine>().unwrap();

    assert_eq!(engine.max_rpm, 8000.0);
    assert_eq!(engine.max_torque, 450.0); // Should be the max from torque curve
    assert_eq!(engine.rpm, 0.0); // Should start at 0
    assert_eq!(engine.throttle, 0.0); // Should start at 0
}

/// Test that VehicleFactory handles transmission configuration correctly.
#[test]
fn test_vehicle_factory_transmission_configuration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let factory = VehicleFactory::new();
    let mut config = VehicleConfig::default();
    config.transmission.final_drive_ratio = 3.8;
    config.transmission.transmission_type = config_core::TransmissionType::Manual;

    // Spawn vehicle
    let chassis_entity = {
        let mut commands = app.world_mut().commands();
        factory.spawn_vehicle(&mut commands, &config).unwrap()
    };

    // Apply commands
    app.world_mut().flush();

    // Verify transmission configuration
    let chassis = app.world().get_entity(chassis_entity).unwrap();
    let transmission = chassis.get::<Transmission>().unwrap();

    assert_eq!(transmission.final_drive_ratio, 3.8);
    assert_eq!(transmission.current_gear, 0); // Should start in neutral (0, not 1)
    assert!(!transmission.gear_ratios.is_empty());

    // Verify gear ratios are from config, not hardcoded
    assert_eq!(transmission.gear_ratios, config.transmission.gear_ratios);
}

/// Test that VehicleFactory applies engine configuration with all fields correctly.
#[test]
fn test_vehicle_factory_engine_config_all_fields() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let factory = VehicleFactory::new();
    let mut config = VehicleConfig::default();
    config.engine.idle_rpm = 900.0;
    config.engine.engine_braking = 0.4;
    config.engine.fuel_consumption = 12.0;

    // Spawn vehicle
    let chassis_entity = {
        let mut commands = app.world_mut().commands();
        factory.spawn_vehicle(&mut commands, &config).unwrap()
    };

    // Apply commands
    app.world_mut().flush();

    // Verify engine configuration has all fields
    let chassis = app.world().get_entity(chassis_entity).unwrap();
    let engine = chassis.get::<Engine>().unwrap();

    assert_eq!(engine.idle_rpm, 900.0);
    assert_eq!(engine.engine_braking, 0.4);
    assert_eq!(engine.fuel_consumption, 12.0);
}

/// Test that VehicleFactory applies suspension configuration correctly.
#[test]
fn test_vehicle_factory_suspension_configuration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let factory = VehicleFactory::new();
    let mut config = VehicleConfig::default();
    config.suspension.spring_stiffness = 40000.0;
    config.suspension.damper_damping = 4000.0;
    config.suspension.travel = 0.25;

    // Spawn vehicle
    let chassis_entity = {
        let mut commands = app.world_mut().commands();
        factory.spawn_vehicle(&mut commands, &config).unwrap()
    };

    // Apply commands
    app.world_mut().flush();

    // Verify suspension configuration
    let chassis = app.world().get_entity(chassis_entity).unwrap();
    let suspension = chassis.get::<Suspension>().unwrap();

    assert_eq!(suspension.spring_stiffness, 40000.0);
    assert_eq!(suspension.damper_damping, 4000.0);
    assert_eq!(suspension.travel, 0.25);
}

/// Test that VehicleFactory applies mass configuration correctly.
#[cfg(feature = "rapier3d_030")]
#[test]
fn test_vehicle_factory_mass_configuration() {
    use amp_physics::rapier::AdditionalMassProperties;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let factory = VehicleFactory::new();
    let config = VehicleConfig {
        mass: 1800.0,
        ..Default::default()
    };

    // Spawn vehicle
    let chassis_entity = {
        let mut commands = app.world_mut().commands();
        factory.spawn_vehicle(&mut commands, &config).unwrap()
    };

    // Apply commands
    app.world_mut().flush();

    // Verify mass configuration
    let chassis = app.world().get_entity(chassis_entity).unwrap();
    let mass_props = chassis.get::<AdditionalMassProperties>().unwrap();

    match mass_props {
        AdditionalMassProperties::Mass(mass) => {
            assert_eq!(*mass, 1800.0);
        }
        _ => panic!("Expected Mass variant"),
    }
}

/// Test that VehicleFactory uses config-driven wheel positioning.
#[test]
fn test_vehicle_factory_wheel_positioning() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let factory = VehicleFactory::new();
    let mut config = VehicleConfig::default();
    config.suspension.travel = 0.25;
    config.wheels[0].radius = 0.35;

    // Spawn vehicle
    let chassis_entity = {
        let mut commands = app.world_mut().commands();
        factory.spawn_vehicle(&mut commands, &config).unwrap()
    };

    // Apply commands
    app.world_mut().flush();

    // Verify wheel positioning uses config values
    let chassis = app.world().get_entity(chassis_entity).unwrap();
    let children = chassis.get::<Children>().unwrap();

    // Check that wheels are positioned based on config
    let expected_y_offset = -(config.suspension.travel + config.wheels[0].radius);

    for child in children.iter() {
        let child_entity = app.world().get_entity(child).unwrap();
        let transform = child_entity.get::<Transform>().unwrap();

        // Y position should be calculated from config
        assert_eq!(transform.translation.y, expected_y_offset);
    }
}
