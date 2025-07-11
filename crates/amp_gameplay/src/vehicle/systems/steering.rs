//! Vehicle steering physics
//!
//! Handles steering input, wheel angles, and turning dynamics.

use crate::vehicle::components::*;
use crate::vehicle::resources::*;
use bevy::prelude::*;

/// Update steering physics for all vehicles
pub fn update_steering(
    mut query: Query<(&mut Steering, &Transform), With<Vehicle>>,
    input_state: Res<VehicleInputState>,
    time: Res<Time>,
) {
    for (mut steering, _transform) in query.iter_mut() {
        // Update steering based on input
        update_steering_physics(&mut steering, &input_state, time.delta_secs());
    }
}

/// Calculate steering physics for a single vehicle
fn update_steering_physics(
    steering: &mut Steering,
    input_state: &VehicleInputState,
    delta_time: f32,
) {
    // Calculate target steering angle
    let target_angle = input_state.steering * steering.max_angle;

    // Smooth steering transition
    let angle_diff = target_angle - steering.angle;
    let max_change = steering.steering_rate * delta_time;

    steering.angle += angle_diff.clamp(-max_change, max_change);
    steering.angle = steering
        .angle
        .clamp(-steering.max_angle, steering.max_angle);
}

/// Calculate Ackermann steering geometry for realistic wheel angles
fn calculate_ackermann_angles(steering: &Steering, track_width: f32) -> (f32, f32) {
    if steering.angle.abs() < 0.001 {
        return (0.0, 0.0);
    }

    let wheelbase = steering.wheelbase;
    let half_track = track_width * 0.5;

    // Calculate inner and outer wheel angles using Ackermann geometry
    let turning_radius = wheelbase / steering.angle.tan();

    let inner_radius = (turning_radius - half_track).abs();
    let outer_radius = (turning_radius + half_track).abs();

    let inner_angle = (wheelbase / inner_radius).atan();
    let outer_angle = (wheelbase / outer_radius).atan();

    // Determine which wheel is inner/outer based on steering direction
    if steering.angle > 0.0 {
        // Steering left: left wheel is inner, right wheel is outer
        (inner_angle, outer_angle)
    } else {
        // Steering right: right wheel is inner, left wheel is outer
        (outer_angle, inner_angle)
    }
}

/// Apply steering angles to individual wheels
pub fn apply_steering_to_wheels(
    mut wheel_query: Query<(&mut Wheel, &Transform), With<Wheel>>,
    steering_query: Query<&Steering, With<Vehicle>>,
) {
    for steering in steering_query.iter() {
        let track_width = 1.6; // TODO: Get from vehicle configuration
        let (left_angle, right_angle) = calculate_ackermann_angles(steering, track_width);

        for (mut wheel, _transform) in wheel_query.iter_mut() {
            if wheel.is_steered {
                // Determine if this is left or right wheel based on position
                if wheel.position.x > 0.0 {
                    // Right wheel
                    wheel.rotation_angle = right_angle;
                } else {
                    // Left wheel
                    wheel.rotation_angle = left_angle;
                }
            }
        }
    }
}

/// Calculate turning circle radius
fn calculate_turning_radius(steering: &Steering) -> f32 {
    if steering.angle.abs() < 0.001 {
        return f32::INFINITY;
    }

    steering.wheelbase / steering.angle.tan()
}

/// Debug information for steering
#[allow(dead_code)]
fn debug_steering_info(steering: &Steering, input_steering: f32) -> String {
    format!(
        "Steering: {:.1}°, Input: {:.1}%, Radius: {:.1}m",
        steering.angle.to_degrees(),
        input_steering * 100.0,
        calculate_turning_radius(steering)
    )
}
