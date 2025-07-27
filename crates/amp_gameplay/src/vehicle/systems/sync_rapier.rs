//! Synchronization with Rapier physics
//!
//! Handles bidirectional sync between vehicle physics and Rapier3D.

use crate::vehicle::components::*;
use crate::vehicle::resources::*;
// Fix: Use amp_gameplay components, not amp_physics components
// This ensures the query matches entities created by VehicleBundle
use amp_physics::{InterpolatedTransform, PhysicsTime};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// Synchronize vehicle physics with Rapier rigid bodies
#[allow(clippy::type_complexity)]
pub fn sync_vehicle_physics(
    mut vehicle_query: Query<
        (
            &mut InterpolatedTransform,
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
    physics_time: Res<PhysicsTime>,
) {
    #[cfg(feature = "perf_trace")]
    let _span = tracing::trace_span!(
        "sync_vehicle_physics",
        vehicle_count = vehicle_query.iter().count()
    )
    .entered();
    for (
        mut interpolated_transform,
        vehicle,
        engine,
        steering,
        suspension,
        rigid_body,
        velocity,
        external_force,
    ) in vehicle_query.iter_mut()
    {
        // Work with the physics transform (current state)
        let mut physics_transform = interpolated_transform.current;

        // Apply vehicle physics to Rapier components
        if let (Some(_rb), Some(mut vel), Some(mut force)) = (rigid_body, velocity, external_force)
        {
            apply_vehicle_forces(
                &mut physics_transform,
                vehicle,
                engine,
                steering,
                suspension,
                &mut vel,
                &mut force,
                &physics_config,
                &input_state,
                physics_time.fixed_timestep,
            );
        }

        // Physics interpolation is now handled by the physics plugin pipeline
        // Remove duplicate transform updates to fix rubber-band artifacts
    }
}

/// Apply calculated vehicle forces to Rapier physics
#[allow(clippy::too_many_arguments)]
fn apply_vehicle_forces(
    transform: &mut Transform,
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
    // Calculate individual forces
    let engine_force = calculate_engine_force(engine, velocity, delta_time);
    let drag_force = calculate_drag_force(vehicle, velocity, physics_config);
    let rolling_resistance = calculate_rolling_resistance(vehicle, velocity, physics_config);
    let brake_force = calculate_brake_force(input_state, velocity, vehicle.mass);

    // Use vehicle's actual forward direction (Bevy uses -Z as forward for identity rotation)
    let forward_dir = transform.forward();

    // Apply forces in proper directions
    let engine_vec = forward_dir * engine_force;

    // Apply drag and rolling resistance opposite to velocity direction
    let velocity_dir = if velocity.linvel.length() > 0.1 {
        velocity.linvel.normalize()
    } else {
        Vec3::ZERO
    };

    let drag_vec = -velocity_dir * drag_force;
    let rolling_vec = -velocity_dir * rolling_resistance;
    let brake_vec = -forward_dir * brake_force;

    // Combine all forces
    let total_force_vec = engine_vec + drag_vec + rolling_vec + brake_vec;

    // Apply with tighter safety clamping for stability
    let clamped_force_vec = total_force_vec.clamp_length_max(3000.0); // Reduced from 10000.0
    external_force.force += clamped_force_vec;

    // Apply steering torque with tighter safety clamping
    if steering.angle.abs() > 0.001 {
        let steering_torque = calculate_steering_torque(steering, velocity, vehicle.mass);
        let clamped_torque = steering_torque.clamp(-200.0, 200.0); // Reduced from 1000.0 to 200.0
        external_force.torque += Vec3::Y * clamped_torque;
    }
}

/// Calculate engine force based on RPM and throttle
fn calculate_engine_force(engine: &Engine, velocity: &Velocity, _delta_time: f32) -> f32 {
    // Simple engine force calculation
    // TODO: Implement more sophisticated engine modeling
    let speed = velocity.linvel.length();
    let power = engine.max_torque * engine.throttle;

    if speed > 0.1 {
        (power / speed).min(1500.0) // Reduced from 5000.0 for stability
    } else {
        power * 10.0 // Reduced from 50.0 to prevent physics instability
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
        physics_config.rolling_resistance * vehicle.mass * 9.81
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
        // Reduced steering sensitivity for stability
        lateral_force * steering.wheelbase * 0.1 // Reduced from 0.5 to 0.1
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

/// Sprint 9 optimization: Enable sleeping for idle vehicles to reduce CPU usage
pub fn manage_vehicle_sleeping(
    mut vehicle_query: Query<
        (Entity, &Velocity, &VehicleInput, Option<&mut Sleeping>),
        With<Vehicle>,
    >,
    mut commands: Commands,
    _time: Res<Time>,
) {
    #[cfg(feature = "tracy")]
    let _span = tracy_client::span!("manage_vehicle_sleeping");

    let mut sleeping_vehicles = 0;
    let mut active_vehicles = 0;

    for (entity, velocity, input, sleeping) in vehicle_query.iter_mut() {
        let is_static = velocity.linvel.length() < 0.1
            && velocity.angvel.length() < 0.1
            && input.throttle.abs() < 0.01
            && input.brake < 0.01
            && input.steering.abs() < 0.01;

        if is_static {
            // Enable sleeping for static vehicles to save CPU
            if sleeping.is_none() {
                commands.entity(entity).insert(Sleeping::disabled());
                sleeping_vehicles += 1;
            }
        } else {
            // Wake up vehicles that have input or movement
            if let Some(mut sleep_state) = sleeping {
                if sleep_state.sleeping {
                    *sleep_state = Sleeping::disabled();
                    active_vehicles += 1;
                }
            }
        }
    }

    #[cfg(feature = "tracy")]
    {
        tracy_client::plot!("sleeping_vehicles", sleeping_vehicles as f64);
        tracy_client::plot!("active_vehicles", active_vehicles as f64);
    }
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
            .insert(ColliderMassProperties::Density(1.0))
            // Sprint 9 optimization: Enable sleeping for static vehicles to save CPU
            .insert(Sleeping::disabled()) // Start awake, will sleep automatically when static
            .insert(Ccd::enabled()); // Enable continuous collision detection for better physics
    }
}
