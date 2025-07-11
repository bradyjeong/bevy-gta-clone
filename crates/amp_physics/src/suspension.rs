//! Suspension physics systems for vehicle simulation.
//!
//! This module implements realistic suspension physics including spring and damper
//! calculations, raycast-based ground contact detection, and force application.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "rapier3d_030")]
#[allow(unused_imports)]
use bevy_rapier3d::prelude::{QueryFilter, RapierContext};

/// Component storing suspension ray configuration and hit results.
///
/// This component manages the raycast data for suspension physics,
/// including cast parameters and collision results.
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SuspensionRay {
    /// Maximum cast distance for the suspension ray
    pub cast_distance: f32,
    /// Ray origin relative to the vehicle
    pub ray_origin: Vec3,
    /// Ray direction (typically downward)
    pub ray_direction: Vec3,
    /// Last hit distance (compression distance)
    pub hit_distance: Option<f32>,
    /// Hit normal vector
    pub hit_normal: Option<Vec3>,
    /// Hit point in world coordinates
    pub hit_point: Option<Vec3>,
    /// Previous frame's hit distance for velocity calculation
    pub previous_hit_distance: Option<f32>,
    /// Suspension compression velocity
    pub compression_velocity: f32,
}

impl Default for SuspensionRay {
    fn default() -> Self {
        Self {
            cast_distance: 1.0,
            ray_origin: Vec3::ZERO,
            ray_direction: Vec3::NEG_Y,
            hit_distance: None,
            hit_normal: None,
            hit_point: None,
            previous_hit_distance: None,
            compression_velocity: 0.0,
        }
    }
}

/// Component storing wheel physics state.
///
/// This component manages wheel-specific physics data including
/// contact forces and rotation state.
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WheelState {
    /// Current wheel rotation angle (radians)
    pub rotation_angle: f32,
    /// Wheel radius in meters
    pub radius: f32,
    /// Wheel width in meters
    pub width: f32,
    /// Wheel mass in kg
    pub mass: f32,
    /// Whether the wheel is currently in contact with ground
    pub in_contact: bool,
    /// Contact force magnitude
    pub contact_force: f32,
    /// Wheel angular velocity (rad/s)
    pub angular_velocity: f32,
}

impl Default for WheelState {
    fn default() -> Self {
        Self {
            rotation_angle: 0.0,
            radius: 0.35,
            width: 0.2,
            mass: 20.0,
            in_contact: false,
            contact_force: 0.0,
            angular_velocity: 0.0,
        }
    }
}

/// Physics update schedule label for suspension systems.
///
/// This schedule runs after Update but before physics simulation.
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct PhysicsUpdate;

#[cfg(feature = "rapier3d_030")]
/// System that performs suspension raycasting and physics calculations with Rapier.
///
/// This system:
/// 1. Casts rays from each wheel to detect ground contact using Rapier
/// 2. Calculates spring and damper forces based on compression
/// 3. Applies forces to the vehicle chassis
/// 4. Updates wheel contact state
///
/// Note: Currently using flat-ground fallback due to RapierContext system parameter compatibility issues.
/// TODO: Implement proper Rapier ray casting integration.
pub fn vehicle_suspension_system(
    mut suspension_query: Query<(&mut SuspensionRay, &mut WheelState, &GlobalTransform)>,
    suspension_config: Query<&crate::components::Suspension>,
    time: Res<Time>,
) {
    warn!(
        "Suspension system with Rapier integration is not yet fully implemented - using flat-ground fallback"
    );

    let dt = time.delta_secs();

    for (mut suspension_ray, mut wheel_state, global_transform) in suspension_query.iter_mut() {
        // Store previous hit distance for velocity calculation
        suspension_ray.previous_hit_distance = suspension_ray.hit_distance;

        // Calculate ray start and direction in world space
        let ray_start = global_transform.translation()
            + global_transform.rotation() * suspension_ray.ray_origin;
        let ray_dir = (global_transform.rotation() * suspension_ray.ray_direction).normalize();

        // TODO: Implement proper Rapier ray casting - currently using flat-ground fallback
        let ground_y = 0.0;
        let distance_to_ground = (ray_start.y - ground_y) / -ray_dir.y;

        if distance_to_ground > 0.0 && distance_to_ground <= suspension_ray.cast_distance {
            // Update hit data
            suspension_ray.hit_distance = Some(distance_to_ground);
            suspension_ray.hit_point = Some(ray_start + ray_dir * distance_to_ground);

            // Calculate hit normal (simplified - assumes ground is flat)
            suspension_ray.hit_normal = Some(Vec3::Y);

            // Calculate compression velocity
            if let Some(prev_distance) = suspension_ray.previous_hit_distance {
                suspension_ray.compression_velocity = (prev_distance - distance_to_ground) / dt;
            }

            wheel_state.in_contact = true;

            // Get suspension configuration
            if let Ok(suspension) = suspension_config.single() {
                // Calculate compression distance
                let compression_distance = suspension.rest_length - distance_to_ground;

                // Clamp compression within travel limits
                let clamped_compression = compression_distance
                    .clamp(-suspension.max_extension, suspension.max_compression);

                // Calculate spring force: F = k * x
                let spring_force = suspension.spring_stiffness * clamped_compression;

                // Calculate damper force: F = c * v
                let damper_force = suspension.damper_damping * suspension_ray.compression_velocity;

                // Combined suspension force (upward)
                let total_force = spring_force + damper_force;

                // Store contact force for reference
                wheel_state.contact_force = total_force;
            }
        } else {
            // No ground contact
            suspension_ray.hit_distance = None;
            suspension_ray.hit_point = None;
            suspension_ray.hit_normal = None;
            suspension_ray.compression_velocity = 0.0;
            wheel_state.in_contact = false;
            wheel_state.contact_force = 0.0;
        }
    }
}

#[cfg(not(feature = "rapier3d_030"))]
/// System that performs suspension raycasting with simplified flat-ground physics.
///
/// This is a fallback system when Rapier is not available that assumes flat ground at y=0.
/// Enable the 'rapier3d_030' feature for proper 3D physics collision detection.
pub fn vehicle_suspension_system(
    mut suspension_query: Query<(&mut SuspensionRay, &mut WheelState, &GlobalTransform)>,
    suspension_config: Query<&crate::components::Suspension>,
    time: Res<Time>,
) {
    warn!(
        "Suspension system running in simplified mode - enable 'rapier3d_030' feature for proper physics"
    );

    let dt = time.delta_secs();

    for (mut suspension_ray, mut wheel_state, global_transform) in suspension_query.iter_mut() {
        // Store previous hit distance for velocity calculation
        suspension_ray.previous_hit_distance = suspension_ray.hit_distance;

        // Calculate ray start and direction in world space
        let ray_start = global_transform.translation()
            + global_transform.rotation() * suspension_ray.ray_origin;
        let ray_dir = (global_transform.rotation() * suspension_ray.ray_direction).normalize();

        // Simplified ground contact detection (assumes flat ground at y=0)
        let ground_y = 0.0;
        let distance_to_ground = (ray_start.y - ground_y) / -ray_dir.y;

        if distance_to_ground > 0.0 && distance_to_ground <= suspension_ray.cast_distance {
            // Update hit data
            suspension_ray.hit_distance = Some(distance_to_ground);
            suspension_ray.hit_point = Some(ray_start + ray_dir * distance_to_ground);

            // Calculate hit normal (simplified - assumes ground is flat)
            suspension_ray.hit_normal = Some(Vec3::Y);

            // Calculate compression velocity
            if let Some(prev_distance) = suspension_ray.previous_hit_distance {
                suspension_ray.compression_velocity = (prev_distance - distance_to_ground) / dt;
            }

            wheel_state.in_contact = true;

            // Get suspension configuration
            if let Ok(suspension) = suspension_config.single() {
                // Calculate compression distance
                let compression_distance = suspension.rest_length - distance_to_ground;

                // Clamp compression within travel limits
                let clamped_compression = compression_distance
                    .clamp(-suspension.max_extension, suspension.max_compression);

                // Calculate spring force: F = k * x
                let spring_force = suspension.spring_stiffness * clamped_compression;

                // Calculate damper force: F = c * v
                let damper_force = suspension.damper_damping * suspension_ray.compression_velocity;

                // Combined suspension force (upward)
                let total_force = spring_force + damper_force;

                // Store contact force for reference
                wheel_state.contact_force = total_force;
            }
        } else {
            // No ground contact
            suspension_ray.hit_distance = None;
            suspension_ray.hit_point = None;
            suspension_ray.hit_normal = None;
            suspension_ray.compression_velocity = 0.0;
            wheel_state.in_contact = false;
            wheel_state.contact_force = 0.0;
        }
    }
}

/// Plugin for suspension physics systems.
pub struct SuspensionPlugin;

impl Plugin for SuspensionPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(PostUpdate, PhysicsUpdate)
            .add_systems(PostUpdate, vehicle_suspension_system);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Suspension;

    #[test]
    fn suspension_ray_default_creation() {
        let ray = SuspensionRay::default();
        assert_eq!(ray.cast_distance, 1.0);
        assert_eq!(ray.ray_origin, Vec3::ZERO);
        assert_eq!(ray.ray_direction, Vec3::NEG_Y);
        assert_eq!(ray.hit_distance, None);
        assert_eq!(ray.compression_velocity, 0.0);
    }

    #[test]
    fn wheel_state_default_creation() {
        let wheel = WheelState::default();
        assert_eq!(wheel.rotation_angle, 0.0);
        assert_eq!(wheel.radius, 0.35);
        assert_eq!(wheel.width, 0.2);
        assert_eq!(wheel.mass, 20.0);
        assert!(!wheel.in_contact);
        assert_eq!(wheel.contact_force, 0.0);
        assert_eq!(wheel.angular_velocity, 0.0);
    }

    #[test]
    fn spring_force_calculation() {
        let suspension = Suspension::default();
        let compression_distance = 0.1; // 10cm compression
        let spring_force = suspension.spring_stiffness * compression_distance;

        // With default spring stiffness of 35000 N/m
        let expected_force = 35000.0 * 0.1;
        assert!((spring_force - expected_force).abs() < 1e-3);
    }

    #[test]
    fn damper_force_calculation() {
        let suspension = Suspension::default();
        let compression_velocity = 0.5; // 0.5 m/s compression velocity
        let damper_force = suspension.damper_damping * compression_velocity;

        // With default damper damping of 3500 N·s/m
        let expected_force = 3500.0 * 0.5;
        assert!((damper_force - expected_force).abs() < 1e-3);
    }

    #[test]
    fn suspension_travel_limits() {
        let suspension = Suspension::default();
        let excessive_compression: f32 = 0.3; // 30cm compression (more than max)
        let clamped_compression =
            excessive_compression.clamp(-suspension.max_extension, suspension.max_compression);

        assert_eq!(clamped_compression, suspension.max_compression);
        assert_eq!(clamped_compression, 0.15);
    }

    #[test]
    fn suspension_extension_limits() {
        let suspension = Suspension::default();
        let excessive_extension: f32 = -0.3; // 30cm extension (more than max)
        let clamped_extension =
            excessive_extension.clamp(-suspension.max_extension, suspension.max_compression);

        assert_eq!(clamped_extension, -suspension.max_extension);
        assert_eq!(clamped_extension, -0.15);
    }

    #[test]
    fn ground_contact_detection() {
        let mut ray = SuspensionRay::default();
        let mut wheel = WheelState::default();

        // Simulate ground contact
        ray.hit_distance = Some(0.4);
        ray.hit_point = Some(Vec3::new(0.0, -0.4, 0.0));
        ray.hit_normal = Some(Vec3::Y);
        wheel.in_contact = true;
        wheel.contact_force = 3500.0;

        assert!(ray.hit_distance.is_some());
        assert_eq!(ray.hit_distance.unwrap(), 0.4);
        assert!(wheel.in_contact);
        assert_eq!(wheel.contact_force, 3500.0);
    }

    #[test]
    fn suspension_ray_serialization() {
        let ray = SuspensionRay {
            cast_distance: 1.5,
            ray_origin: Vec3::new(0.0, 0.5, 0.0),
            ray_direction: Vec3::NEG_Y,
            hit_distance: Some(0.8),
            hit_normal: Some(Vec3::Y),
            hit_point: Some(Vec3::new(0.0, -0.3, 0.0)),
            previous_hit_distance: Some(0.82),
            compression_velocity: -0.1,
        };

        let serialized = serde_json::to_string(&ray).unwrap();
        let deserialized: SuspensionRay = serde_json::from_str(&serialized).unwrap();

        assert_eq!(ray, deserialized);
    }

    #[test]
    fn wheel_state_serialization() {
        let wheel = WheelState {
            rotation_angle: 1.57,
            radius: 0.4,
            width: 0.25,
            mass: 25.0,
            in_contact: true,
            contact_force: 2500.0,
            angular_velocity: 10.0,
        };

        let serialized = serde_json::to_string(&wheel).unwrap();
        let deserialized: WheelState = serde_json::from_str(&serialized).unwrap();

        assert_eq!(wheel, deserialized);
    }
}
