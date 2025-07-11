//! Vehicle input handling systems

use crate::vehicle::components::*;
use amp_physics::components::VehicleInput as PhysicsInput;
use bevy::prelude::*;

/// Handle vehicle input and sync to physics components
pub fn handle_vehicle_input(
    mut query: Query<(&mut VehicleInput, &mut PhysicsInput), With<Vehicle>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    for (mut gameplay_input, mut physics_input) in query.iter_mut() {
        // Handle keyboard input
        let throttle =
            if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW) {
                1.0
            } else {
                0.0
            };

        let brake = if keyboard_input.pressed(KeyCode::ArrowDown)
            || keyboard_input.pressed(KeyCode::KeyS)
        {
            1.0
        } else {
            0.0
        };

        let steering = if keyboard_input.pressed(KeyCode::ArrowLeft)
            || keyboard_input.pressed(KeyCode::KeyA)
        {
            -1.0
        } else if keyboard_input.pressed(KeyCode::ArrowRight)
            || keyboard_input.pressed(KeyCode::KeyD)
        {
            1.0
        } else {
            0.0
        };

        let handbrake = if keyboard_input.pressed(KeyCode::Space) {
            1.0
        } else {
            0.0
        };

        // Update gameplay input
        gameplay_input.throttle = throttle;
        gameplay_input.brake = brake;
        gameplay_input.steering = steering;
        gameplay_input.handbrake = handbrake;

        // Sync to physics input
        physics_input.throttle = throttle;
        physics_input.brake = brake;
        physics_input.steering = steering;
        physics_input.handbrake = handbrake;
    }
}
