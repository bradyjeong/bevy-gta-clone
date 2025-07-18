//! Character camera systems
//!
//! Third-person camera with spring-arm following and mouse orbit controls.

use bevy::prelude::*;

use crate::character::components::*;
use crate::character::resources::CharacterInputState;

/// Camera follow system for third-person view
pub fn camera_follow(
    time: Res<Time>,
    input_state: Res<CharacterInputState>,
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<Player>)>,
    mut character_query: Query<(&Transform, &mut CameraTarget), (With<Player>, Without<Camera3d>)>,
) {
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    let Ok((character_transform, mut camera_target)) = character_query.single_mut() else {
        return;
    };

    // Update camera rotation from mouse input
    camera_target.rotation.x -= input_state.mouse_delta.x * camera_target.mouse_sensitivity;
    camera_target.rotation.y -= input_state.mouse_delta.y * camera_target.mouse_sensitivity;

    // Clamp pitch to prevent camera flipping
    camera_target.rotation.y = camera_target.rotation.y.clamp(-1.5, 1.5);

    // Calculate target position (character position + offset)
    let target_position = character_transform.translation + camera_target.offset;

    // Calculate camera position using spherical coordinates
    let yaw = camera_target.rotation.x;
    let pitch = camera_target.rotation.y;

    let camera_offset = Vec3::new(
        yaw.cos() * pitch.cos() * camera_target.distance,
        pitch.sin() * camera_target.distance + camera_target.height_offset,
        yaw.sin() * pitch.cos() * camera_target.distance,
    );

    let desired_camera_position = target_position + camera_offset;

    // Smooth camera movement
    let smoothing = camera_target.smoothness;
    camera_transform.translation = camera_transform.translation.lerp(
        desired_camera_position,
        smoothing * time.delta_secs() * 10.0, // Scale for reasonable follow speed
    );

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

/// Camera distance adjustment system (for zoom in/out with scroll wheel)
pub fn adjust_camera_distance(
    mut scroll_events: EventReader<bevy::input::mouse::MouseWheel>,
    mut character_query: Query<&mut CameraTarget, With<Player>>,
) {
    for event in scroll_events.read() {
        for mut camera_target in character_query.iter_mut() {
            camera_target.distance -= event.y * 0.5;
            camera_target.distance = camera_target.distance.clamp(2.0, 20.0);
        }
    }
}
