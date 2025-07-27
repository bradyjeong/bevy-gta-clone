//! Character camera systems
//!
//! Third-person camera with spring-arm following and mouse orbit controls.

use bevy::prelude::*;

use crate::character::components::*;
use crate::character::resources::CharacterInputState;
use amp_physics::{InterpolatedTransform, PhysicsTime};

/// Camera follow system for third-person view
pub fn camera_follow(
    time: Res<Time>,
    physics_time: Res<PhysicsTime>,
    input_state: Res<CharacterInputState>,
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<Player>)>,
    mut character_query: Query<
        (&InterpolatedTransform, &mut CameraTarget),
        (With<Player>, Without<Camera3d>),
    >,
) {
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    let Ok((interpolated_transform, mut camera_target)) = character_query.single_mut() else {
        return;
    };

    // No mouse camera rotation - fixed camera angle
    // Camera will follow character but maintain fixed relative position

    // Calculate target position (interpolated character position + offset)
    let target_position = interpolated_transform.visual.translation + camera_target.offset;

    // Calculate camera position using spherical coordinates
    let yaw = camera_target.rotation.x;
    let pitch = camera_target.rotation.y;

    let camera_offset = Vec3::new(
        yaw.cos() * pitch.cos() * camera_target.distance,
        pitch.sin() * camera_target.distance + camera_target.height_offset,
        yaw.sin() * pitch.cos() * camera_target.distance,
    );

    let desired_camera_position = target_position + camera_offset;

    // Smooth camera movement with exponential damping and physics interpolation
    let damping_rate = camera_target.smoothness * 10.0; // Convert smoothness to damping rate
    let base_damping = 1.0 - (-damping_rate * time.delta_secs()).exp();

    // Apply physics interpolation for sub-frame smoothness
    let interpolated_damping = base_damping * (1.0 + physics_time.interpolation_alpha * 0.5);
    let damping_factor = interpolated_damping.clamp(0.0, 1.0);

    camera_transform.translation = camera_transform
        .translation
        .lerp(desired_camera_position, damping_factor);

    // Always look at the target
    camera_transform.look_at(target_position, Vec3::Y);
}

/// Camera orbit system for mouse-based camera rotation
pub fn camera_orbit(
    _input_state: Res<CharacterInputState>,
    mut _character_query: Query<&mut CameraTarget, With<Player>>,
) {
    // This is handled in camera_follow system to avoid borrow checker issues
    // but could be separated if needed for different update rates
}

/// Setup camera for character following
pub fn setup_character_camera(
    mut commands: Commands,
    character_query: Query<(Entity, &Transform), With<Player>>,
) {
    if let Ok((_character_entity, character_transform)) = character_query.single() {
        // Spawn camera positioned behind and above the character
        let camera_position = character_transform.translation + Vec3::new(0.0, 3.0, 5.0);

        commands.spawn((
            Camera3d::default(),
            Transform::from_translation(camera_position)
                .looking_at(character_transform.translation, Vec3::Y),
        ));
    }
}

/// Camera distance adjustment system (disabled - no mouse control)
pub fn adjust_camera_distance(// Mouse control removed
) {
    // No mouse wheel zoom - fixed camera distance
}
