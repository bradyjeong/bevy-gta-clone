//! Character movement systems
//!
//! Handles physics-based character movement, gravity, jumping, and ground detection.

use bevy::prelude::*;
use bevy::time::Fixed;

#[cfg(feature = "rapier3d_030")]
use bevy_rapier3d::prelude::*;

use crate::character::components::*;

/// Handle player movement with physics-based character controller
#[cfg(feature = "rapier3d_030")]
pub fn player_movement(
    time: Res<Time<Fixed>>,
    mut query: Query<
        (
            &CharacterInput,
            &Speed,
            &mut KinematicCharacterController,
            &mut CharacterController,
            &Transform,
            &Grounded,
        ),
        With<Player>,
    >,
) {
    for (input, speed, mut kinematic_controller, mut controller, _transform, grounded) in
        query.iter_mut()
    {
        if input.movement.length() > 0.0 {
            // Calculate movement direction relative to camera/world
            let movement_dir = Vec3::new(input.movement.x, 0.0, -input.movement.y);

            // Apply sprint multiplier
            let move_speed = if input.sprint {
                speed.walk * speed.sprint_multiplier
            } else {
                speed.walk
            };

            // Calculate desired movement
            let desired_translation = movement_dir * move_speed * time.delta_secs();

            // Apply movement through kinematic character controller
            kinematic_controller.translation = Some(desired_translation);
        } else {
            kinematic_controller.translation = None;
        }

        // Handle jumping
        if input.jump && grounded.is_grounded && controller.vertical_velocity <= 0.0 {
            controller.vertical_velocity = speed.jump_force;
        }
    }
}

/// Fallback movement system for non-physics characters
#[cfg(not(feature = "rapier3d_030"))]
pub fn player_movement(
    time: Res<Time<Fixed>>,
    mut query: Query<
        (
            &CharacterInput,
            &Speed,
            &mut Transform,
            &mut CharacterController,
            &Grounded,
        ),
        With<Player>,
    >,
) {
    for (input, speed, mut transform, mut controller, grounded) in query.iter_mut() {
        if input.movement.length() > 0.0 {
            // Calculate movement direction
            let movement_dir = Vec3::new(input.movement.x, 0.0, -input.movement.y);

            // Apply sprint multiplier
            let move_speed = if input.sprint {
                speed.walk * speed.sprint_multiplier
            } else {
                speed.walk
            };

            // Apply movement
            transform.translation += movement_dir * move_speed * time.delta_secs();
        }

        // Handle jumping
        if input.jump && grounded.is_grounded && controller.vertical_velocity <= 0.0 {
            controller.vertical_velocity = speed.jump_force;
        }
    }
}

/// Apply gravity to character controller
pub fn apply_gravity(
    time: Res<Time<Fixed>>,
    mut query: Query<&mut CharacterController, With<Player>>,
) {
    for mut controller in query.iter_mut() {
        // Apply gravity
        controller.vertical_velocity += controller.gravity * time.delta_secs();

        // Clamp to max fall speed
        if controller.vertical_velocity < controller.max_fall_speed {
            controller.vertical_velocity = controller.max_fall_speed;
        }
    }
}

/// Handle jumping physics
#[cfg(feature = "rapier3d_030")]
pub fn handle_jumping(
    time: Res<Time<Fixed>>,
    mut query: Query<
        (
            &mut KinematicCharacterController,
            &mut CharacterController,
            &Grounded,
        ),
        With<Player>,
    >,
) {
    for (mut kinematic_controller, mut controller, grounded) in query.iter_mut() {
        // Apply vertical movement from gravity and jumping
        let vertical_movement = controller.vertical_velocity * time.delta_secs();

        if let Some(ref mut translation) = kinematic_controller.translation {
            translation.y += vertical_movement;
        } else {
            kinematic_controller.translation = Some(Vec3::new(0.0, vertical_movement, 0.0));
        }

        // Reset vertical velocity if grounded and falling
        if grounded.is_grounded && controller.vertical_velocity <= 0.0 {
            controller.vertical_velocity = 0.0;
        }
    }
}

/// Fallback jumping system for non-physics characters
#[cfg(not(feature = "rapier3d_030"))]
pub fn handle_jumping(
    time: Res<Time<Fixed>>,
    mut query: Query<(&mut Transform, &mut CharacterController, &Grounded), With<Player>>,
) {
    for (mut transform, mut controller, grounded) in query.iter_mut() {
        // Apply vertical movement from gravity and jumping
        transform.translation.y += controller.vertical_velocity * time.delta_secs();

        // Reset vertical velocity if grounded and falling
        if grounded.is_grounded && controller.vertical_velocity <= 0.0 {
            controller.vertical_velocity = 0.0;
            // Snap to ground
            transform.translation.y = grounded.ground_distance + 0.9; // Half character height
        }
    }
}

/// Update grounded state through ground detection
#[cfg(feature = "rapier3d_030")]
pub fn update_grounded_state(
    mut query: Query<
        (
            &Transform,
            &mut Grounded,
            &CharacterController,
            &KinematicCharacterControllerOutput,
        ),
        With<Player>,
    >,
) {
    for (transform, mut grounded, _controller, output) in query.iter_mut() {
        // Use Rapier's grounded detection from character controller output
        grounded.is_grounded = output.grounded;

        if output.grounded {
            grounded.ground_distance = transform.translation.y;
            grounded.ground_normal = Vec3::Y; // Simplified - could get from collision
        } else {
            grounded.ground_distance = f32::INFINITY;
            grounded.ground_normal = Vec3::Y;
        }
    }
}

/// Fallback ground detection for non-physics characters
#[cfg(not(feature = "rapier3d_030"))]
pub fn update_grounded_state(
    mut query: Query<(&Transform, &mut Grounded, &CharacterController), With<Player>>,
) {
    for (transform, mut grounded, _controller) in query.iter_mut() {
        // Simple ground detection - check if character is near y=0
        let ground_level = 0.9; // Character height / 2
        grounded.is_grounded = transform.translation.y <= ground_level + 0.1;
        grounded.ground_distance = ground_level;
        grounded.ground_normal = Vec3::Y;
    }
}
