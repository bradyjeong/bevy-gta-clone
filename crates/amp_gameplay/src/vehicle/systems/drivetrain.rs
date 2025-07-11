//! Vehicle drivetrain and engine physics
//!
//! Handles engine torque calculations, transmission, and power delivery.

use crate::vehicle::components::*;
use crate::vehicle::resources::*;
use bevy::prelude::*;

/// Update drivetrain physics for all vehicles
pub fn update_drivetrain(
    mut query: Query<(&mut Engine, &Transform), With<Vehicle>>,
    input_state: Res<VehicleInputState>,
    time: Res<Time>,
) {
    for (mut engine, _transform) in query.iter_mut() {
        // Update engine based on throttle input
        update_engine_physics(&mut engine, &input_state, time.delta_secs());
    }
}

/// Calculate engine physics for a single vehicle
fn update_engine_physics(engine: &mut Engine, input_state: &VehicleInputState, delta_time: f32) {
    // Update throttle
    engine.throttle = input_state.throttle;

    // Calculate target RPM based on throttle
    let target_rpm = if engine.throttle > 0.0 {
        engine.idle_rpm + (engine.max_rpm - engine.idle_rpm) * engine.throttle
    } else {
        engine.idle_rpm
    };

    // Smooth RPM transition
    let rpm_change_rate = 5000.0; // RPM per second
    let rpm_diff = target_rpm - engine.rpm;
    let max_change = rpm_change_rate * delta_time;

    engine.rpm += rpm_diff.clamp(-max_change, max_change);
    engine.rpm = engine.rpm.clamp(engine.idle_rpm, engine.max_rpm);
}

/// Calculate engine torque from RPM using torque curve
fn calculate_engine_torque(engine: &Engine) -> f32 {
    if engine.torque_curve.is_empty() {
        return 0.0;
    }

    let rpm = engine.rpm;

    // Find the two closest points on the torque curve
    for i in 0..engine.torque_curve.len() - 1 {
        let (rpm1, torque1) = engine.torque_curve[i];
        let (rpm2, torque2) = engine.torque_curve[i + 1];

        if rpm >= rpm1 && rpm <= rpm2 {
            // Linear interpolation between the two points
            let t = (rpm - rpm1) / (rpm2 - rpm1);
            let torque_multiplier = torque1 + t * (torque2 - torque1);
            return engine.max_torque * torque_multiplier * engine.throttle;
        }
    }

    // If RPM is outside the curve, use the closest endpoint
    if rpm < engine.torque_curve[0].0 {
        engine.max_torque * engine.torque_curve[0].1 * engine.throttle
    } else {
        engine.max_torque * engine.torque_curve.last().unwrap().1 * engine.throttle
    }
}

/// Debug information for drivetrain
#[allow(dead_code)]
fn debug_drivetrain_info(engine: &Engine) -> String {
    format!(
        "RPM: {:.0}, Throttle: {:.1}%, Torque: {:.0} Nm",
        engine.rpm,
        engine.throttle * 100.0,
        calculate_engine_torque(engine)
    )
}
