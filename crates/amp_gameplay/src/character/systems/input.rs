//! Character input handling systems
//!
//! Handles keyboard and mouse input for character movement and camera control.

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};

use crate::character::components::*;
use crate::character::resources::CharacterInputState;

/// Handle character input from keyboard
pub fn handle_character_input(
    mut input_state: ResMut<CharacterInputState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<
        &mut CharacterInput,
        (
            With<Player>,
            Without<crate::interaction::components::InVehicle>,
        ),
    >,
) {
    // Force debug log to ensure this system is running
    static mut CALL_COUNT: u32 = 0;
    unsafe {
        CALL_COUNT += 1;
        if CALL_COUNT % 60 == 0 {
            // Every 60 calls (about once per second)
            warn!(
                "🎮 Input system running - call #{}, player entities: {}",
                CALL_COUNT,
                query.iter().count()
            );
        }
    }

    // Debug: Check if we have any player entities with input
    let player_count = query.iter().count();
    if player_count == 0 {
        // Only log this occasionally to avoid spam
        static mut LAST_LOG_TIME: f32 = 0.0;
        static mut DELTA_ACCUMULATOR: f32 = 0.0;
        unsafe {
            DELTA_ACCUMULATOR += 0.016; // Assume ~16ms frame time
            if DELTA_ACCUMULATOR - LAST_LOG_TIME > 3.0 {
                warn!("No player entities found with CharacterInput component!");
                LAST_LOG_TIME = DELTA_ACCUMULATOR;
            }
        }
    }

    // Update input state from keyboard
    input_state.movement = Vec2::ZERO;
    input_state.rotation = 0.0;

    // Debug: Check if any keys are pressed
    let pressed_keys: Vec<_> = keyboard.get_pressed().collect();
    if !pressed_keys.is_empty() {
        warn!("🎹 Keys currently pressed: {:?}", pressed_keys);
    }

    // Test specific keys directly
    if keyboard.pressed(KeyCode::Space) {
        warn!("🚀 SPACE key detected!");
    }
    if keyboard.pressed(KeyCode::Escape) {
        warn!("🔄 ESCAPE key detected!");
    }

    if keyboard.pressed(KeyCode::KeyW) {
        input_state.movement.y += 1.0;
        warn!("🚶 W key pressed - forward movement");
    }
    if keyboard.pressed(KeyCode::KeyS) {
        input_state.movement.y -= 1.0;
        warn!("🚶 S key pressed - backward movement");
    }
    if keyboard.pressed(KeyCode::KeyA) {
        input_state.rotation += 1.0;
        warn!("🔄 A key pressed - left rotation");
    }
    if keyboard.pressed(KeyCode::KeyD) {
        input_state.rotation -= 1.0;
        warn!("🔄 D key pressed - right rotation");
    }

    // Debug: Log if we have any movement or rotation
    if input_state.movement.length() > 0.0 {
        info!("📍 Movement calculated: {:?}", input_state.movement);
    }
    if input_state.rotation.abs() > 0.0 {
        info!("🔄 Rotation calculated: {:.2}", input_state.rotation);
    }

    input_state.jump = keyboard.just_pressed(KeyCode::Space);
    input_state.sprint = keyboard.pressed(KeyCode::ShiftLeft);
    input_state.context_action = keyboard.just_pressed(KeyCode::KeyF);

    // Update character input components
    for mut character_input in query.iter_mut() {
        // Movement only uses Y axis for forward/backward, X should be 0.0
        character_input.move_2d = Vec2::new(0.0, input_state.movement.y);
        // Use new yaw field for A/D rotation intent (-1, 0, +1)
        character_input.yaw = input_state.rotation;
        character_input.look_delta = Vec2::ZERO; // No mouse look for now
        character_input.jump = input_state.jump;
        character_input.sprint = input_state.sprint;
        character_input.context_action = input_state.context_action;
        character_input.interact = input_state.context_action; // Map to both for compatibility
        character_input.crouch = false; // Default to false for now

        // Debug logging when input is applied
        if input_state.movement.length() > 0.0 || input_state.rotation.abs() > 0.0 {
            info!(
                "⚡ Applied input to character: movement={:?}, yaw={:.2}, sprint={}",
                character_input.move_2d, character_input.yaw, character_input.sprint
            );
        }
    }
}

/// System to initialize cursor mode (mouse control disabled)
pub fn setup_character_input(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = windows.single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true; // Keep cursor visible since no mouse control
        window.focused = true; // Ensure window is focused to receive input
        info!("✅ Character input setup completed - window focused and cursor ungrabbed");
    } else {
        warn!("❌ Failed to setup character input - no primary window found");
    }
}
