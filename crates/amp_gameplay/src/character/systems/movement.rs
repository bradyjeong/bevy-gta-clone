//! Character movement systems
//!
//! Handles physics-based character movement, gravity, jumping, and ground detection.

use bevy::prelude::*;
use bevy::time::Fixed;

#[cfg(feature = "rapier3d_030")]
use bevy_rapier3d::prelude::*;

use crate::character::components::Velocity as CharacterVelocity;
use crate::character::components::*;
use amp_physics::{InterpolatedTransform, PhysicsTime};

/// Update velocity components by smoothly steering toward desired values with clamping
pub fn update_velocity_steering(time: Res<Time>, mut query: Query<&mut CharacterVelocity>) {
    for mut velocity in query.iter_mut() {
        let dt = time.delta_secs();

        // Clamp delta time to prevent large frame-time spikes causing oscillation
        let clamped_dt = dt.min(1.0 / 30.0); // Max 30 FPS equivalent

        // Smoothly steer linear velocity toward desired with clamping
        let linear_diff = velocity.desired_linear - velocity.linear;
        let linear_accel_step = linear_diff * velocity.linear_acceleration * clamped_dt;

        // Clamp velocity change to prevent overshooting
        let max_linear_change = velocity.max_velocity_delta * clamped_dt;
        let clamped_linear_step = if linear_accel_step.length() > max_linear_change {
            linear_accel_step.normalize() * max_linear_change.min(linear_diff.length())
        } else {
            linear_accel_step
        };

        velocity.linear += clamped_linear_step;

        // --- snap small velocities to zero to let animation go idle -----------
        if velocity.desired_linear == Vec3::ZERO && velocity.linear.length() < 0.1 {
            velocity.linear = Vec3::ZERO;
        }

        // Smoothly steer angular velocity toward desired with clamping
        let angular_diff = velocity.desired_angular - velocity.angular;
        let angular_accel_step = angular_diff * velocity.angular_acceleration * clamped_dt;

        // Clamp angular velocity change
        let max_angular_change = velocity.max_velocity_delta * clamped_dt;
        let clamped_angular_step = angular_accel_step.clamp(
            -max_angular_change.min(angular_diff.abs()),
            max_angular_change.min(angular_diff.abs()),
        );

        velocity.angular += clamped_angular_step;
    }
}

/// Handle player movement with physics-based character controller
#[cfg(feature = "rapier3d_030")]
pub fn player_movement(
    physics_time: Res<PhysicsTime>,
    mut query: Query<
        (
            &CharacterInput,
            &Speed,
            &mut KinematicCharacterController,
            &mut CharacterController,
            &mut Transform,
            &mut InterpolatedTransform,
            &Grounded,
            &mut CharacterVelocity,
        ),
        With<Player>,
    >,
) {
    let yaw_speed = 4.0; // rad/s - Oracle tuning: More natural turning speed (~230°/s)

    for (
        input,
        speed,
        mut kinematic_controller,
        mut controller,
        mut transform,
        mut interpolated_transform,
        grounded,
        mut velocity,
    ) in query.iter_mut()
    {
        // Handle rotation (yaw) - use yaw field for rotation intent
        if input.yaw.abs() > 0.0 {
            let yaw_delta = input.yaw * yaw_speed * physics_time.fixed_timestep;
            warn!(
                "🔄 PHYSICS: Rotating character by {:.3} radians (yaw: {:.3})",
                yaw_delta, input.yaw
            );
            transform.rotate_y(yaw_delta);
            // Update desired angular velocity for smooth animation system integration
            velocity.desired_angular = input.yaw * yaw_speed;
        } else {
            velocity.desired_angular = 0.0;
        }

        // Handle movement in world space - only forward/backward movement
        if input.move_2d.y.abs() > 0.0 {
            // Get world-space forward direction from character transform
            let forward = transform.forward();
            // Only use Y component for forward/backward movement, no strafing
            let movement_dir = forward * input.move_2d.y;

            // Apply sprint multiplier
            let move_speed = if input.sprint {
                speed.walk * speed.sprint_multiplier
            } else {
                speed.walk
            };

            // Normalize for diagonal movement and ensure world-space direction
            let movement_dir = if movement_dir.length_squared() > 0.0 {
                movement_dir.normalize()
            } else {
                movement_dir
            };

            // Calculate world-space translation for character controller
            let desired_translation = movement_dir * move_speed * physics_time.fixed_timestep;

            // Update desired linear velocity for smooth animation system integration
            velocity.desired_linear = movement_dir * move_speed;

            // Write back the actual horizontal velocity for animation system
            // This ensures animation state machine sees the real movement velocity
            velocity.linear = Vec3::new(
                movement_dir.x * move_speed,
                velocity.linear.y, // Keep existing Y velocity (gravity/jumping)
                movement_dir.z * move_speed,
            );

            // Apply world-space movement through kinematic character controller
            kinematic_controller.translation = Some(desired_translation);
        } else {
            kinematic_controller.translation = None;
            velocity.desired_linear = Vec3::ZERO;
            // Clear horizontal velocity when not moving
            velocity.linear = Vec3::new(0.0, velocity.linear.y, 0.0);
        }

        // Handle jumping
        if input.jump && grounded.is_grounded && controller.vertical_velocity <= 0.0 {
            controller.vertical_velocity = speed.jump_force;
        }

        // Update the interpolated transform to keep it in sync
        interpolated_transform.current = *transform;

        // Debug: Log character world rotation in degrees
        if input.yaw.abs() > 0.0 {
            let (yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
            let (visual_yaw, _, _) = interpolated_transform
                .visual
                .rotation
                .to_euler(EulerRot::YXZ);
            warn!(
                "🌐 Physics rotation: {:.1}°, Visual rotation: {:.1}°",
                yaw.to_degrees(),
                visual_yaw.to_degrees()
            );
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
            &mut CharacterVelocity,
        ),
        With<Player>,
    >,
) {
    let yaw_speed = 4.0; // rad/s - Oracle tuning: More natural turning speed (~230°/s)

    for (input, speed, mut transform, mut controller, grounded, mut velocity) in query.iter_mut() {
        // Handle rotation (yaw) - use yaw field for rotation intent
        if input.yaw.abs() > 0.0 {
            let yaw_delta = input.yaw * yaw_speed * time.delta_secs();
            warn!(
                "🔄 FALLBACK: Rotating character by {:.3} radians (yaw: {:.3})",
                yaw_delta, input.yaw
            );
            transform.rotate_y(yaw_delta);
            // Update desired angular velocity for smooth animation system integration
            velocity.desired_angular = input.yaw * yaw_speed;
        } else {
            velocity.desired_angular = 0.0;
        }

        // Handle movement in world space - only forward/backward movement
        if input.move_2d.y.abs() > 0.0 {
            // Get world-space forward direction from character transform
            let forward = transform.forward();
            // Only use Y component for forward/backward movement, no strafing
            let movement_dir = forward * input.move_2d.y;

            // Apply sprint multiplier
            let move_speed = if input.sprint {
                speed.walk * speed.sprint_multiplier
            } else {
                speed.walk
            };

            // Normalize for diagonal movement and apply world-space translation
            let movement_dir = if movement_dir.length_squared() > 0.0 {
                movement_dir.normalize()
            } else {
                movement_dir
            };

            // Update desired linear velocity for smooth animation system integration
            velocity.desired_linear = movement_dir * move_speed;

            // Write back the actual horizontal velocity for animation system
            // This ensures animation state machine sees the real movement velocity
            velocity.linear = Vec3::new(
                movement_dir.x * move_speed,
                velocity.linear.y, // Keep existing Y velocity (gravity/jumping)
                movement_dir.z * move_speed,
            );

            // Apply world-space movement directly to transform
            transform.translation += movement_dir * move_speed * time.delta_secs();
        } else {
            velocity.desired_linear = Vec3::ZERO;
            // Clear horizontal velocity when not moving
            velocity.linear = Vec3::new(0.0, velocity.linear.y, 0.0);
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

/// Sync velocity from player entity to skeleton entity each frame
///
/// Oracle's Option A: Keep Velocity on player entity (for movement systems)
/// and copy it to skeleton entity (for locomotion state machine)
pub fn sync_velocity_to_skeleton(
    rigs: Query<(&HumanoidRig, &CharacterVelocity), With<Player>>,
    mut skel_vel_query: Query<&mut CharacterVelocity, Without<Player>>,
) {
    for (rig, player_vel) in rigs.iter() {
        if let Ok(mut skel_vel) = skel_vel_query.get_mut(rig.skeleton_entity) {
            *skel_vel = player_vel.clone();
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
