//! Vehicle physics systems for the Amp game engine.
//!
//! This module implements the core vehicle physics systems including
//! drivetrain, steering, braking, and control input processing.

use crate::components::*;
use bevy::prelude::*;

#[cfg(feature = "rapier3d_030")]
use bevy_rapier3d::prelude::*;

/// Engine torque calculation and RPM update system.
///
/// This system updates engine RPM based on throttle input and calculates
/// engine torque using the torque curve interpolation.
pub fn engine_system(mut query: Query<&mut Engine>, time: Res<Time>) {
    for mut engine in query.iter_mut() {
        // Update RPM based on throttle input
        let target_rpm = engine.idle_rpm + (engine.max_rpm - engine.idle_rpm) * engine.throttle;
        let rpm_change_rate = 2000.0; // RPM per second

        if engine.rpm < target_rpm {
            engine.rpm = (engine.rpm + rpm_change_rate * time.delta_secs()).min(target_rpm);
        } else {
            engine.rpm = (engine.rpm - rpm_change_rate * time.delta_secs()).max(target_rpm);
        }

        // Apply engine speed limiter
        engine.rpm = engine.rpm.clamp(0.0, engine.max_rpm);

        // Calculate torque from torque curve
        engine.torque =
            interpolate_torque_curve(&engine.torque_curve, engine.rpm) * engine.throttle;

        // Apply engine braking when throttle is released
        if engine.throttle < 0.1 {
            engine.torque -= engine.engine_braking * engine.rpm / engine.max_rpm * 50.0;
        }
    }
}

/// Interpolates torque from the engine's torque curve at a given RPM.
fn interpolate_torque_curve(torque_curve: &[(f32, f32)], rpm: f32) -> f32 {
    if torque_curve.is_empty() {
        return 0.0;
    }

    if rpm <= torque_curve[0].0 {
        return torque_curve[0].1;
    }

    if rpm >= torque_curve[torque_curve.len() - 1].0 {
        return torque_curve[torque_curve.len() - 1].1;
    }

    // Find the two points to interpolate between
    for i in 0..torque_curve.len() - 1 {
        let (rpm1, torque1) = torque_curve[i];
        let (rpm2, torque2) = torque_curve[i + 1];

        if rpm >= rpm1 && rpm <= rpm2 {
            // Linear interpolation
            let t = (rpm - rpm1) / (rpm2 - rpm1);
            return torque1 + t * (torque2 - torque1);
        }
    }

    0.0
}

/// Drivetrain system that applies engine torque to wheels.
///
/// This system handles the transmission of power from the engine to the wheels
/// through the transmission and drivetrain components.
pub fn drivetrain_system(
    mut vehicles: Query<(Entity, &mut Engine, &Transmission, &Drivetrain), With<Vehicle>>,
    mut wheels: Query<&mut WheelPhysics, With<Wheel>>,
    children: Query<&Children>,
) {
    for (vehicle_entity, engine, transmission, drivetrain) in vehicles.iter_mut() {
        // Calculate wheel torque from engine torque
        let gear_ratio = if transmission.current_gear == 0 {
            0.0 // Neutral
        } else if transmission.current_gear > 0 {
            transmission.gear_ratios[transmission.current_gear as usize]
        } else {
            transmission.gear_ratios[0] // Reverse
        };

        let total_ratio =
            gear_ratio * transmission.final_drive_ratio * drivetrain.differential_ratio;
        let wheel_torque = engine.torque * total_ratio * drivetrain.efficiency;

        // Apply torque to wheels based on drive type
        if let Ok(vehicle_children) = children.get(vehicle_entity) {
            let mut wheel_count = 0;
            let mut driven_wheels = Vec::new();

            // Count wheels and determine driven wheels
            for child in vehicle_children.iter() {
                if let Ok(_wheel_physics) = wheels.get(child) {
                    wheel_count += 1;

                    // Determine if wheel is driven based on drive type
                    let is_front_wheel = wheel_count <= 2; // Assume first two wheels are front
                    let is_driven = match drivetrain.drive_type {
                        0 => is_front_wheel,  // FWD
                        1 => !is_front_wheel, // RWD
                        2 => true,            // AWD
                        _ => false,
                    };

                    if is_driven {
                        driven_wheels.push(child);
                    }
                }
            }

            // Apply torque to driven wheels
            let torque_per_wheel = if driven_wheels.is_empty() {
                0.0
            } else {
                wheel_torque / driven_wheels.len() as f32
            };

            for wheel_entity in driven_wheels {
                if let Ok(mut wheel_physics) = wheels.get_mut(wheel_entity) {
                    wheel_physics.motor_torque = torque_per_wheel;
                }
            }
        }
    }
}

/// Steering system that applies steering input to front wheels.
///
/// This system handles steering input and applies Ackermann geometry
/// for realistic steering behavior.
pub fn steering_system(
    mut vehicles: Query<(Entity, &mut Steering, &VehicleInput), With<Vehicle>>,
    mut wheels: Query<&mut Transform, With<Wheel>>,
    children: Query<&Children>,
    time: Res<Time>,
) {
    for (vehicle_entity, mut steering, input) in vehicles.iter_mut() {
        // Apply steering input with smoothing
        let target_angle = input.steering * steering.max_angle;
        let steering_change = steering.steering_rate * time.delta_secs();

        if steering.angle < target_angle {
            steering.angle = (steering.angle + steering_change).min(target_angle);
        } else {
            steering.angle = (steering.angle - steering_change).max(target_angle);
        }

        // Apply return-to-center force when no input
        if input.steering.abs() < 0.1 {
            let return_force = steering.return_force * time.delta_secs();
            if steering.angle > 0.0 {
                steering.angle = (steering.angle - return_force).max(0.0);
            } else {
                steering.angle = (steering.angle + return_force).min(0.0);
            }
        }

        // Apply Ackermann steering geometry to front wheels
        if let Ok(vehicle_children) = children.get(vehicle_entity) {
            let mut wheel_count = 0;

            for child in vehicle_children.iter() {
                if let Ok(mut wheel_transform) = wheels.get_mut(child) {
                    wheel_count += 1;

                    // Apply steering only to front wheels (first two)
                    if wheel_count <= 2 {
                        let is_left_wheel = wheel_count == 1;
                        let ackermann_angle = calculate_ackermann_angle(
                            steering.angle,
                            steering.wheelbase,
                            steering.track_width,
                            is_left_wheel,
                        );

                        // Apply steering angle to wheel transform
                        wheel_transform.rotation = Quat::from_rotation_y(ackermann_angle);
                    }
                }
            }
        }
    }
}

/// Calculates Ackermann steering angle for a specific wheel.
fn calculate_ackermann_angle(
    steering_angle: f32,
    wheelbase: f32,
    track_width: f32,
    is_left: bool,
) -> f32 {
    if steering_angle.abs() < 0.001 {
        return 0.0;
    }

    let radius = wheelbase / steering_angle.tan();
    let wheel_radius = if is_left {
        if steering_angle > 0.0 {
            radius - track_width / 2.0
        } else {
            radius + track_width / 2.0
        }
    } else if steering_angle > 0.0 {
        radius + track_width / 2.0
    } else {
        radius - track_width / 2.0
    };

    (wheelbase / wheel_radius).atan()
}

/// Braking system that applies brake torque to wheels.
///
/// This system handles brake input and applies brake torque to wheels
/// with brake bias and ABS-like behavior.
pub fn braking_system(
    mut vehicles: Query<(Entity, &mut Brakes, &VehicleInput), With<Vehicle>>,
    mut wheels: Query<&mut WheelPhysics, With<Wheel>>,
    children: Query<&Children>,
) {
    for (vehicle_entity, mut brakes, input) in vehicles.iter_mut() {
        // Update brake input
        brakes.brake_input = input.brake;

        // Calculate brake torque
        let brake_torque = brakes.brake_input * brakes.max_brake_torque;

        // Apply brake torque to wheels based on brake bias
        if let Ok(vehicle_children) = children.get(vehicle_entity) {
            let mut wheel_count = 0;

            for child in vehicle_children.iter() {
                if let Ok(mut wheel_physics) = wheels.get_mut(child) {
                    wheel_count += 1;
                    let is_front_wheel = wheel_count <= 2;

                    // Apply brake bias
                    let wheel_brake_torque = if is_front_wheel {
                        brake_torque * brakes.brake_bias
                    } else {
                        brake_torque * (1.0 - brakes.brake_bias)
                    };

                    // Apply ABS if enabled
                    if brakes.abs_enabled && wheel_physics.slip_ratio.abs() > brakes.abs_threshold {
                        wheel_physics.brake_torque = wheel_brake_torque * 0.5; // Reduce brake force
                    } else {
                        wheel_physics.brake_torque = wheel_brake_torque;
                    }
                }
            }
        }
    }
}

/// Control input system that processes player input.
///
/// This system handles keyboard/gamepad input and applies smoothing
/// and deadzone processing to control inputs.
pub fn control_input_system(
    mut query: Query<&mut VehicleInput>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    for mut vehicle_input in query.iter_mut() {
        // Process throttle input
        let throttle_input = if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
            1.0
        } else {
            0.0
        };

        // Process brake input
        let brake_input = if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
            1.0
        } else {
            0.0
        };

        // Process steering input
        let steering_input = if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
            -1.0
        } else if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
            1.0
        } else {
            0.0
        };

        // Process handbrake input
        let handbrake_input = if input.pressed(KeyCode::Space) {
            1.0
        } else {
            0.0
        };

        // Apply deadzone
        let apply_deadzone = |value: f32, deadzone: f32| {
            if value.abs() < deadzone { 0.0 } else { value }
        };

        let throttle_target = apply_deadzone(throttle_input, vehicle_input.deadzone);
        let brake_target = apply_deadzone(brake_input, vehicle_input.deadzone);
        let steering_target = apply_deadzone(steering_input, vehicle_input.deadzone);
        let handbrake_target = apply_deadzone(handbrake_input, vehicle_input.deadzone);

        // Apply smoothing
        let smoothing_factor = vehicle_input.smoothing;
        let dt = time.delta_secs();

        vehicle_input.throttle = lerp(
            vehicle_input.throttle,
            throttle_target,
            smoothing_factor * dt * 10.0,
        );
        vehicle_input.brake = lerp(
            vehicle_input.brake,
            brake_target,
            smoothing_factor * dt * 10.0,
        );
        vehicle_input.steering = lerp(
            vehicle_input.steering,
            steering_target,
            smoothing_factor * dt * 10.0,
        );
        vehicle_input.handbrake = lerp(
            vehicle_input.handbrake,
            handbrake_target,
            smoothing_factor * dt * 10.0,
        );
    }
}

/// Linear interpolation function.
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

/// Wheel physics system that applies forces to wheels.
///
/// This system calculates slip ratios, applies motor and brake torques,
/// and generates forces for the physics simulation.
#[cfg(feature = "rapier3d_030")]
pub fn wheel_physics_system(
    mut wheels: Query<(&mut WheelPhysics, &mut ExternalForce, &Velocity, &Transform), With<Wheel>>,
    time: Res<Time>,
) {
    for (mut wheel_physics, mut external_force, velocity, transform) in wheels.iter_mut() {
        // Calculate wheel linear velocity
        let wheel_linear_velocity = wheel_physics.angular_velocity * wheel_physics.radius;

        // Calculate vehicle velocity in wheel's local space
        let forward_velocity = velocity.linvel.dot(*transform.forward());

        // Calculate slip ratio
        wheel_physics.slip_ratio = if forward_velocity.abs() > 0.1 {
            (wheel_linear_velocity - forward_velocity) / forward_velocity.abs()
        } else {
            0.0
        };

        // Calculate longitudinal force based on slip ratio
        let max_force = wheel_physics.friction_coefficient * 1000.0; // Simplified normal force
        wheel_physics.longitudinal_force =
            calculate_tire_force(wheel_physics.slip_ratio, max_force);

        // Apply motor torque to wheel angular velocity
        let wheel_inertia = 0.5 * wheel_physics.mass * wheel_physics.radius * wheel_physics.radius;
        let angular_acceleration = wheel_physics.motor_torque / wheel_inertia;
        wheel_physics.angular_velocity += angular_acceleration * time.delta_secs();

        // Apply brake torque
        let brake_acceleration = -wheel_physics.brake_torque / wheel_inertia;
        wheel_physics.angular_velocity += brake_acceleration * time.delta_secs();

        // Apply longitudinal force to vehicle
        let force_vector = *transform.forward() * wheel_physics.longitudinal_force;
        external_force.force += force_vector;

        // Apply lateral force for cornering (simplified)
        let lateral_slip = velocity.linvel.dot(*transform.right());
        wheel_physics.lateral_force = -lateral_slip * wheel_physics.friction_coefficient * 500.0;
        let lateral_force_vector = *transform.right() * wheel_physics.lateral_force;
        external_force.force += lateral_force_vector;
    }
}

/// Calculate tire force based on slip ratio using a simplified Pacejka model.
fn calculate_tire_force(slip_ratio: f32, max_force: f32) -> f32 {
    let peak_slip = 0.15;
    let shape_factor = 10.0;

    if slip_ratio.abs() <= peak_slip {
        // Linear region
        max_force * slip_ratio / peak_slip
    } else {
        // Sliding region
        max_force
            * slip_ratio.signum()
            * (1.0 - (slip_ratio.abs() - peak_slip) / shape_factor).max(0.3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolate_torque_curve() {
        let torque_curve = vec![
            (0.0, 0.0),
            (1000.0, 200.0),
            (3000.0, 300.0),
            (5000.0, 250.0),
        ];

        assert_eq!(interpolate_torque_curve(&torque_curve, 0.0), 0.0);
        assert_eq!(interpolate_torque_curve(&torque_curve, 1000.0), 200.0);
        assert_eq!(interpolate_torque_curve(&torque_curve, 2000.0), 250.0); // Interpolated
        assert_eq!(interpolate_torque_curve(&torque_curve, 3000.0), 300.0);
        assert_eq!(interpolate_torque_curve(&torque_curve, 5000.0), 250.0);
        assert_eq!(interpolate_torque_curve(&torque_curve, 6000.0), 250.0); // Clamped
    }

    #[test]
    fn test_calculate_ackermann_angle() {
        let steering_angle = 0.5; // radians
        let wheelbase = 2.7;
        let track_width = 1.5;

        let left_angle = calculate_ackermann_angle(steering_angle, wheelbase, track_width, true);
        let right_angle = calculate_ackermann_angle(steering_angle, wheelbase, track_width, false);

        // Left wheel should have a larger angle than right wheel when turning right
        assert!(left_angle.abs() > right_angle.abs());
    }

    #[test]
    fn test_calculate_tire_force() {
        let max_force = 1000.0;

        // Linear region
        assert_eq!(calculate_tire_force(0.0, max_force), 0.0);
        assert!((calculate_tire_force(0.15, max_force) - 1000.0).abs() < 0.001);
        assert!((calculate_tire_force(-0.15, max_force) + 1000.0).abs() < 0.001);

        // Sliding region
        let sliding_force = calculate_tire_force(0.3, max_force);
        assert!(sliding_force > 0.0 && sliding_force < max_force);
    }

    #[test]
    fn test_lerp() {
        assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
    }
}
