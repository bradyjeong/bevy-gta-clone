//! Simple camera following system
//!
//! Provides a smooth third-person camera that follows the player character.

use crate::SmoothCamera;
use amp_gameplay::character::components::Player;
use amp_physics::interpolation::InterpolatedTransform;
use amp_physics::PhysicsSets;
use bevy::prelude::*;

pub struct SmoothCameraPlugin;

impl Plugin for SmoothCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, camera_follow.after(PhysicsSets::Interpolate));
    }
}

/// System that makes the camera follow the player smoothly
pub fn camera_follow(
    time: Res<Time>,
    mut camera_query: Query<(&mut Transform, &SmoothCamera), (With<Camera3d>, Without<Player>)>,
    player_query: Query<&InterpolatedTransform, (With<Player>, Without<Camera3d>)>,
) {
    for (mut camera_transform, smooth_camera) in camera_query.iter_mut() {
        if let Ok(interpolated_transform) = player_query.single() {
            let player_pos = interpolated_transform.visual.translation;
            let player_forward = interpolated_transform.visual.forward();

            // Calculate desired camera position
            let desired_position = player_pos - *player_forward * smooth_camera.follow_distance
                + Vec3::Y * smooth_camera.follow_height;

            // Exponential damping - frame rate independent
            let damping_factor = 1.0 - (-smooth_camera.damping_rate * time.delta_secs()).exp();
            let new_position = camera_transform
                .translation
                .lerp(desired_position, damping_factor);

            // Update camera position and look-at
            camera_transform.translation = new_position;
            camera_transform.look_at(player_pos + Vec3::Y * 1.0, Vec3::Y);
        }
    }
}

/// Stub for orbit camera (not implemented)
pub fn camera_orbit() {
    // Not implemented - could add mouse orbit control here
}
