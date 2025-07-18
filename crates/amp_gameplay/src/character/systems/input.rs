//! Character input handling systems
//!
//! Handles keyboard and mouse input for character movement and camera control.

use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};

use crate::character::components::*;
use crate::character::resources::CharacterInputState;

/// Handle character input from keyboard and mouse
pub fn handle_character_input(
    mut input_state: ResMut<CharacterInputState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: Query<&mut CharacterInput, With<Player>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    // Update input state from keyboard
    input_state.movement = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        input_state.movement.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        input_state.movement.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        input_state.movement.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        input_state.movement.x += 1.0;
    }

    // Normalize movement vector to prevent faster diagonal movement
    if input_state.movement.length() > 1.0 {
        input_state.movement = input_state.movement.normalize();
    }

    input_state.jump = keyboard.just_pressed(KeyCode::Space);
    input_state.sprint = keyboard.pressed(KeyCode::ShiftLeft);
    input_state.context_action = keyboard.just_pressed(KeyCode::KeyF);

    // Handle mouse input for camera
    input_state.mouse_delta = Vec2::ZERO;
    for event in mouse_motion_events.read() {
        input_state.mouse_delta += event.delta;
    }

    // Handle cursor grab toggle
    if keyboard.just_pressed(KeyCode::Escape) {
        if let Ok(mut window) = windows.single_mut() {
            match window.cursor_options.grab_mode {
                CursorGrabMode::None => {
                    window.cursor_options.grab_mode = CursorGrabMode::Locked;
                    window.cursor_options.visible = false;
                }
                _ => {
                    window.cursor_options.grab_mode = CursorGrabMode::None;
                    window.cursor_options.visible = true;
                }
            }
        }
    }

    // Update character input components
    for mut character_input in query.iter_mut() {
        character_input.movement = input_state.movement;
        character_input.jump = input_state.jump;
        character_input.sprint = input_state.sprint;
        character_input.context_action = input_state.context_action;
    }
}

/// System to initialize cursor grab mode for character control
pub fn setup_character_input(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = windows.single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }
}
