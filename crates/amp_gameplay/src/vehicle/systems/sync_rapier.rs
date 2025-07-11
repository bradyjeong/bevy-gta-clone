//! Synchronization with Rapier physics
//!
//! Handles bidirectional sync between vehicle physics and Rapier3D.

use crate::vehicle::components::*;
use crate::vehicle::resources::*;
use amp_physics::components::{Engine, Steering, Suspension};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// Synchronize vehicle physics with Rapier rigid bodies
#[allow(clippy::type_complexity)]
pub fn sync_vehicle_physics(
    mut vehicle_query: Query<
        (
            &mut Transform,
            &Vehicle,
            &Engine,
            &Steering,
            &Suspension,
            Option<&mut RigidBody>,
            Option<&mut Velocity>,
            Option<&mut ExternalForce>,
        ),
        With<Vehicle>,
    >,
    physics_config: Res<VehiclePhysicsConfig>,
    input_state: Res<VehicleInputState>,
    time: Res<Time>,
) {
    for (
        mut transform,
        vehicle,
        engine,
        steering,
        suspension,
        rigid_body,
        velocity,
        external_force,
    ) in vehicle_query.iter_mut()
    {
        // Apply vehicle physics to Rapier components
        if let (Some(_rb), Some(mut vel), Some(mut force)) = (rigid_body, velocity, external_force)
        {
            apply_vehicle_forces(
                &mut transform,
                vehicle,
                engine,
                steering,
                suspension,
                &mut vel,
                &mut force,
                &physics_config,
                &input_state,
                time.delta_secs(),
            );
        }
    }
}

/// Apply calculated vehicle forces to Rapier physics
#[allow(clippy::too_many_arguments)]
fn apply_vehicle_forces(
    _transform: &mut Transform,
    vehicle: &Vehicle,
    engine: &Engine,
    steering: &Steering,
    _suspension: &Suspension,
    velocity: &mut Velocity,
    external_force: &mut ExternalForce,
    physics_config: &VehiclePhysicsConfig,
    input_state: &VehicleInputState,
    delta_time: f32,
) {
    // Calculate engine force
    let engine_force = calculate_engine_force(engine, velocity, delta_time);

    // Calculate aerodynamic drag
    let drag_force = calculate_drag_force(vehicle, velocity, physics_config);

    // Calculate rolling resistance
    let rolling_resistance = calculate_rolling_resistance(vehicle, velocity, physics_config);

    // Calculate braking force
    let brake_force = calculate_brake_force(input_state, velocity, vehicle.mass);

    // Apply forces
    let total_force = engine_force - drag_force - rolling_resistance - brake_force;

    // Apply longitudinal force
    let forward_dir = Vec3::Z; // TODO: Get from vehicle transform
    external_force.force += forward_dir * total_force;

    // Apply steering torque
    if steering.angle.abs() > 0.001 {
        let steering_torque = calculate_steering_torque(steering, velocity, vehicle.mass);
        external_force.torque += Vec3::Y * steering_torque;
    }
}

/// Calculate engine force based on RPM and throttle
fn calculate_engine_force(engine: &Engine, velocity: &Velocity, _delta_time: f32) -> f32 {
    // Simple engine force calculation
    // TODO: Implement more sophisticated engine modeling
    let speed = velocity.linvel.length();
    let power = engine.max_torque * engine.throttle;

    if speed > 0.1 {
        power / speed
    } else {
        power * 100.0 // High torque at low speed
    }
}

/// Calculate aerodynamic drag force
fn calculate_drag_force(
    vehicle: &Vehicle,
    velocity: &Velocity,
    physics_config: &VehiclePhysicsConfig,
) -> f32 {
    let speed = velocity.linvel.length();
    0.5 * physics_config.air_density
        * vehicle.drag_coefficient
        * vehicle.frontal_area
        * speed
        * speed
}

/// Calculate rolling resistance force
fn calculate_rolling_resistance(
    vehicle: &Vehicle,
    velocity: &Velocity,
    physics_config: &VehiclePhysicsConfig,
) -> f32 {
    let speed = velocity.linvel.length();
    if speed > 0.1 {
        physics_config.rolling_resistance * vehicle.mass * 9.81 * speed.signum()
    } else {
        0.0
    }
}

/// Calculate braking force
fn calculate_brake_force(input_state: &VehicleInputState, _velocity: &Velocity, mass: f32) -> f32 {
    if input_state.brake > 0.0 {
        let max_brake_force = mass * 8.0; // 8 m/s² deceleration
        input_state.brake * max_brake_force
    } else {
        0.0
    }
}

/// Calculate steering torque for turning
fn calculate_steering_torque(steering: &Steering, velocity: &Velocity, mass: f32) -> f32 {
    let speed = velocity.linvel.length();
    if speed > 0.1 {
        let lateral_force = mass * speed * speed / calculate_turning_radius(steering);
        lateral_force * steering.wheelbase * 0.5
    } else {
        0.0
    }
}

/// Calculate turning radius from steering angle
fn calculate_turning_radius(steering: &Steering) -> f32 {
    if steering.angle.abs() < 0.001 {
        return f32::INFINITY;
    }

    steering.wheelbase / steering.angle.tan()
}

/// Setup Rapier components for a vehicle
pub fn setup_vehicle_rapier_components(
    mut commands: Commands,
    vehicle_query: Query<Entity, (With<Vehicle>, Without<RigidBody>)>,
) {
    for entity in vehicle_query.iter() {
        commands
            .entity(entity)
            .insert(RigidBody::Dynamic)
            .insert(Velocity::default())
            .insert(ExternalForce::default())
            .insert(Collider::cuboid(2.25, 1.0, 0.9)) // Half-extents for typical car
            .insert(Restitution::coefficient(0.1))
            .insert(Friction::coefficient(0.8))
            .insert(ColliderMassProperties::Density(1.0));
    }
}
