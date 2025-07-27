//! Camera switching system for vehicle interaction

use crate::interaction::{components::*, events::*};
use crate::prelude::{Player, Vehicle};
use amp_physics::InterpolatedTransform;
use bevy::prelude::*;

/// Switch camera rigs based on vehicle enter/exit
pub fn switch_camera_rigs(
    mut commands: Commands,
    mut vehicle_enter_events: EventReader<VehicleEnterEvent>,
    mut vehicle_exit_events: EventReader<VehicleExitEvent>,
    mut camera_query: Query<(Entity, &mut Transform), With<Camera>>,
    player_query: Query<&InterpolatedTransform, (With<Player>, Without<Camera>)>,
    vehicle_query: Query<&Transform, (With<Vehicle>, Without<Camera>, Without<Player>)>,
) {
    // Handle vehicle enter - switch to vehicle camera
    for event in vehicle_enter_events.read() {
        if let Ok((camera_entity, mut camera_transform)) = camera_query.single_mut() {
            if let Ok(vehicle_transform) = vehicle_query.get(event.vehicle_entity) {
                // Add vehicle camera rig
                commands.entity(camera_entity).insert(VehicleCameraRig {
                    target_entity: event.vehicle_entity,
                    follow_distance: 8.0,
                    follow_height: 4.0,
                    follow_damping: 2.0,
                    look_ahead_distance: 5.0,
                    camera_mode: VehicleCameraMode::ThirdPerson,
                });

                // Remove character camera rig if present
                commands
                    .entity(camera_entity)
                    .remove::<CharacterCameraRig>();

                // Initial camera position
                let camera_offset = Vec3::new(0.0, 4.0, 8.0);
                camera_transform.translation = vehicle_transform.translation + camera_offset;
                camera_transform.look_at(vehicle_transform.translation, Vec3::Y);

                info!("Switched to vehicle camera mode");
            }
        }
    }

    // Handle vehicle exit - switch to character camera
    for event in vehicle_exit_events.read() {
        if let Ok((camera_entity, mut camera_transform)) = camera_query.single_mut() {
            if let Ok(interpolated_transform) = player_query.get(event.player_entity) {
                // Add character camera rig
                commands.entity(camera_entity).insert(CharacterCameraRig {
                    target_entity: event.player_entity,
                    follow_distance: 5.0,
                    follow_height: 2.0,
                    follow_damping: 4.0,
                    look_sensitivity: 1.0,
                    camera_mode: CharacterCameraMode::ThirdPerson,
                });

                // Remove vehicle camera rig
                commands.entity(camera_entity).remove::<VehicleCameraRig>();

                // Initial camera position
                let camera_offset = Vec3::new(0.0, 2.0, 5.0);
                camera_transform.translation =
                    interpolated_transform.visual.translation + camera_offset;
                camera_transform.look_at(interpolated_transform.visual.translation, Vec3::Y);

                info!("Switched to character camera mode");
            }
        }
    }
}

/// Update vehicle camera following
pub fn update_vehicle_camera(
    time: Res<Time>,
    mut camera_query: Query<(&mut Transform, &VehicleCameraRig), (With<Camera>, Without<Vehicle>)>,
    vehicle_query: Query<&Transform, (With<Vehicle>, Without<Camera>)>,
) {
    for (mut camera_transform, camera_rig) in camera_query.iter_mut() {
        if let Ok(vehicle_transform) = vehicle_query.get(camera_rig.target_entity) {
            // Calculate desired camera position
            let vehicle_forward = vehicle_transform.forward();
            let desired_position = vehicle_transform.translation
                - vehicle_forward * camera_rig.follow_distance
                + Vec3::Y * camera_rig.follow_height;

            // Smooth camera movement
            let damping_factor = 1.0 - (-camera_rig.follow_damping * time.delta_secs()).exp();
            camera_transform.translation = camera_transform
                .translation
                .lerp(desired_position, damping_factor);

            // Look at vehicle with some look-ahead
            let look_target =
                vehicle_transform.translation + vehicle_forward * camera_rig.look_ahead_distance;
            camera_transform.look_at(look_target, Vec3::Y);
        }
    }
}

/// Update character camera following
pub fn update_character_camera(
    time: Res<Time>,
    mut camera_query: Query<(&mut Transform, &CharacterCameraRig), (With<Camera>, Without<Player>)>,
    character_query: Query<&InterpolatedTransform, (With<Player>, Without<Camera>)>,
) {
    for (mut camera_transform, camera_rig) in camera_query.iter_mut() {
        if let Ok(interpolated_transform) = character_query.get(camera_rig.target_entity) {
            // Calculate desired camera position
            let character_forward = interpolated_transform.visual.forward();
            let desired_position = interpolated_transform.visual.translation
                - character_forward * camera_rig.follow_distance
                + Vec3::Y * camera_rig.follow_height;

            // Smooth camera movement
            let damping_factor = 1.0 - (-camera_rig.follow_damping * time.delta_secs()).exp();
            camera_transform.translation = camera_transform
                .translation
                .lerp(desired_position, damping_factor);

            // Look at character
            camera_transform.look_at(interpolated_transform.visual.translation, Vec3::Y);
        }
    }
}
