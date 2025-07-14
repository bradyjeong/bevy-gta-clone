//! Vehicle input handling systems

use crate::vehicle::components::*;
use crate::vehicle::resources::VehicleInputState;
use bevy::prelude::*;

/// Handle vehicle input and sync to physics components
pub fn handle_vehicle_input(
    mut query: Query<(&mut VehicleInput, &mut PhysicsVehicleInput), With<Vehicle>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    for (mut gameplay_input, mut physics_input) in query.iter_mut() {
        // Check if there's keyboard input
        let has_keyboard_input = keyboard_input.pressed(KeyCode::ArrowUp)
            || keyboard_input.pressed(KeyCode::KeyW)
            || keyboard_input.pressed(KeyCode::ArrowDown)
            || keyboard_input.pressed(KeyCode::KeyS)
            || keyboard_input.pressed(KeyCode::ArrowLeft)
            || keyboard_input.pressed(KeyCode::KeyA)
            || keyboard_input.pressed(KeyCode::ArrowRight)
            || keyboard_input.pressed(KeyCode::KeyD)
            || keyboard_input.pressed(KeyCode::Space);

        // Only override input if there's keyboard input
        if has_keyboard_input {
            // Handle keyboard input
            let throttle = if keyboard_input.pressed(KeyCode::ArrowUp)
                || keyboard_input.pressed(KeyCode::KeyW)
            {
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
        }

        // Always sync to physics input
        physics_input.throttle = gameplay_input.throttle;
        physics_input.brake = gameplay_input.brake;
        physics_input.steering = gameplay_input.steering;
        physics_input.handbrake = gameplay_input.handbrake;
    }
}

/// Update VehicleInputState resource from VehicleInput components
/// Oracle's critical fix: Wire gameplay input into physics resource
pub fn update_input_state_from_components(
    mut input_state: ResMut<VehicleInputState>,
    input_query: Query<&VehicleInput, With<Vehicle>>,
) {
    // Take the first vehicle's input (support multiple vehicles)
    if let Some(vehicle_input) = input_query.iter().next() {
        input_state.throttle = vehicle_input.throttle;
        input_state.brake = vehicle_input.brake;
        input_state.steering = vehicle_input.steering;
        input_state.handbrake = vehicle_input.handbrake > 0.0;
        // Keep existing gear logic
    }
}
