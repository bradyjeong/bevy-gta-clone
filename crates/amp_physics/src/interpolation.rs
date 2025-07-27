//! Physics-visual transform interpolation system.
//!
//! This module provides smooth visual interpolation between physics steps
//! to eliminate visual stuttering and jitter in gameplay.

use bevy::prelude::*;

use crate::time::PhysicsTime;

/// Component that stores interpolated transform states for smooth rendering.
///
/// This component maintains both the physics transform (authoritative) and
/// visual transform (interpolated) to provide smooth motion between physics steps.
#[derive(Component, Debug, Clone)]
#[cfg_attr(feature = "inspector", derive(bevy::reflect::Reflect))]
pub struct InterpolatedTransform {
    /// Previous physics transform state
    pub previous: Transform,
    /// Current physics transform state (authoritative)
    pub current: Transform,
    /// Visual transform state (interpolated for rendering)
    pub visual: Transform,
    /// Whether interpolation is enabled for this entity
    pub enabled: bool,
}

impl Default for InterpolatedTransform {
    fn default() -> Self {
        Self {
            previous: Transform::IDENTITY,
            current: Transform::IDENTITY,
            visual: Transform::IDENTITY,
            enabled: true,
        }
    }
}

impl InterpolatedTransform {
    /// Create a new InterpolatedTransform from a base transform.
    pub fn new(transform: Transform) -> Self {
        Self {
            previous: transform,
            current: transform,
            visual: transform,
            enabled: true,
        }
    }

    /// Create with interpolation disabled.
    pub fn new_disabled(transform: Transform) -> Self {
        Self {
            previous: transform,
            current: transform,
            visual: transform,
            enabled: false,
        }
    }

    /// Update the physics transform (should be called during physics steps).
    pub fn update_physics(&mut self, new_transform: Transform) {
        self.previous = self.current;
        self.current = new_transform;
    }

    /// Calculate interpolated visual transform based on alpha.
    pub fn interpolate(&mut self, alpha: f32) {
        if !self.enabled {
            self.visual = self.current;
            return;
        }

        let alpha = alpha.clamp(0.0, 1.0);

        // Interpolate translation
        self.visual.translation = self
            .previous
            .translation
            .lerp(self.current.translation, alpha);

        // Slerp rotation for smooth rotation interpolation
        self.visual.rotation = self.previous.rotation.slerp(self.current.rotation, alpha);

        // Lerp scale (though scale changes are rare in physics)
        self.visual.scale = self.previous.scale.lerp(self.current.scale, alpha);
    }

    /// Force visual transform to match current physics transform.
    pub fn snap_visual_to_current(&mut self) {
        self.visual = self.current;
    }

    /// Reset all transforms to the given value.
    pub fn reset(&mut self, transform: Transform) {
        self.previous = transform;
        self.current = transform;
        self.visual = transform;
    }
}

/// System that updates visual transforms based on physics interpolation alpha.
///
/// This system runs every frame and interpolates visual transforms between
/// the previous and current physics states for smooth rendering.
pub fn interpolate_transforms(
    physics_time: Res<PhysicsTime>,
    mut query: Query<(&mut InterpolatedTransform, &mut Transform)>,
) {
    let alpha = physics_time.interpolation_alpha;

    for (mut interpolated, mut transform) in query.iter_mut() {
        interpolated.interpolate(alpha);
        *transform = interpolated.visual;
    }
}

/// System that prepares physics updates by storing current transform as previous.
///
/// This system should run before physics simulation to ensure we have
/// the correct previous state for interpolation.
pub fn prepare_physics_interpolation(mut query: Query<(&mut InterpolatedTransform, &Transform)>) {
    for (mut interpolated, transform) in query.iter_mut() {
        interpolated.update_physics(*transform);
    }
}

/// System that syncs physics transforms to InterpolatedTransform current state.
///
/// This system should run after physics simulation to update the current
/// physics state for interpolation calculations.
pub fn sync_physics_transforms(
    mut query: Query<(&mut InterpolatedTransform, &Transform), Changed<Transform>>,
) {
    for (mut interpolated, transform) in query.iter_mut() {
        interpolated.current = *transform;
    }
}

/// Bundle for entities that need interpolated transforms.
#[derive(Bundle)]
pub struct InterpolatedTransformBundle {
    pub interpolated_transform: InterpolatedTransform,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for InterpolatedTransformBundle {
    fn default() -> Self {
        Self {
            interpolated_transform: InterpolatedTransform::default(),
            transform: Transform::IDENTITY,
            global_transform: GlobalTransform::IDENTITY,
        }
    }
}

impl InterpolatedTransformBundle {
    /// Create a new bundle with the given transform.
    pub fn new(transform: Transform) -> Self {
        Self {
            interpolated_transform: InterpolatedTransform::new(transform),
            transform,
            global_transform: GlobalTransform::from(transform),
        }
    }

    /// Create a new bundle with interpolation disabled.
    pub fn new_disabled(transform: Transform) -> Self {
        Self {
            interpolated_transform: InterpolatedTransform::new_disabled(transform),
            transform,
            global_transform: GlobalTransform::from(transform),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolated_transform_creation() {
        let transform = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let interpolated = InterpolatedTransform::new(transform);

        assert_eq!(interpolated.previous, transform);
        assert_eq!(interpolated.current, transform);
        assert_eq!(interpolated.visual, transform);
        assert!(interpolated.enabled);
    }

    #[test]
    fn interpolated_transform_disabled() {
        let transform = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let interpolated = InterpolatedTransform::new_disabled(transform);

        assert!(!interpolated.enabled);
    }

    #[test]
    fn interpolated_transform_update_physics() {
        let initial = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let new = Transform::from_translation(Vec3::new(4.0, 5.0, 6.0));

        let mut interpolated = InterpolatedTransform::new(initial);
        interpolated.update_physics(new);

        assert_eq!(interpolated.previous, initial);
        assert_eq!(interpolated.current, new);
    }

    #[test]
    fn interpolated_transform_interpolation() {
        let start = Transform::from_translation(Vec3::new(0.0, 0.0, 0.0));
        let end = Transform::from_translation(Vec3::new(10.0, 10.0, 10.0));

        let mut interpolated = InterpolatedTransform::new(start);
        interpolated.update_physics(end);

        // Test 50% interpolation
        interpolated.interpolate(0.5);
        let expected = Vec3::new(5.0, 5.0, 5.0);
        assert!((interpolated.visual.translation - expected).length() < 0.001);
    }

    #[test]
    fn interpolated_transform_disabled_interpolation() {
        let start = Transform::from_translation(Vec3::new(0.0, 0.0, 0.0));
        let end = Transform::from_translation(Vec3::new(10.0, 10.0, 10.0));

        let mut interpolated = InterpolatedTransform::new_disabled(start);
        interpolated.update_physics(end);
        interpolated.interpolate(0.5);

        // Should use current transform, not interpolated
        assert_eq!(interpolated.visual.translation, end.translation);
    }

    #[test]
    fn interpolated_transform_rotation() {
        let start = Transform::from_rotation(Quat::IDENTITY);
        let end = Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::PI));

        let mut interpolated = InterpolatedTransform::new(start);
        interpolated.update_physics(end);
        interpolated.interpolate(0.5);

        // Should be halfway rotated
        let expected_angle = std::f32::consts::PI * 0.5;
        let actual_angle = interpolated.visual.rotation.to_euler(EulerRot::XYZ).1;
        assert!((actual_angle - expected_angle).abs() < 0.001);
    }

    #[test]
    fn interpolated_transform_reset() {
        let initial = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let new = Transform::from_translation(Vec3::new(4.0, 5.0, 6.0));
        let reset = Transform::from_translation(Vec3::new(7.0, 8.0, 9.0));

        let mut interpolated = InterpolatedTransform::new(initial);
        interpolated.update_physics(new);
        interpolated.reset(reset);

        assert_eq!(interpolated.previous, reset);
        assert_eq!(interpolated.current, reset);
        assert_eq!(interpolated.visual, reset);
    }

    #[test]
    fn interpolated_transform_clamped_alpha() {
        let start = Transform::from_translation(Vec3::new(0.0, 0.0, 0.0));
        let end = Transform::from_translation(Vec3::new(10.0, 10.0, 10.0));

        let mut interpolated = InterpolatedTransform::new(start);
        interpolated.update_physics(end);

        // Test clamping behavior
        interpolated.interpolate(-0.5); // Should clamp to 0.0
        assert_eq!(interpolated.visual.translation, start.translation);

        interpolated.interpolate(1.5); // Should clamp to 1.0
        assert_eq!(interpolated.visual.translation, end.translation);
    }
}
