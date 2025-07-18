//! Character components
//!
//! Components for character entities including movement, collision, and interaction.

use bevy::prelude::*;

/// Marker component for player character
#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
pub struct Player;

/// Character grounded state component
#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
pub struct Grounded {
    /// Whether the character is currently on the ground
    pub is_grounded: bool,
    /// Ground normal vector
    pub ground_normal: Vec3,
    /// Distance to ground
    pub ground_distance: f32,
}

/// Character speed configuration
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Speed {
    /// Walking speed (units per second)
    pub walk: f32,
    /// Sprinting speed multiplier
    pub sprint_multiplier: f32,
    /// Jump force
    pub jump_force: f32,
}

impl Default for Speed {
    fn default() -> Self {
        Self {
            walk: 5.0,
            sprint_multiplier: 2.0,
            jump_force: 10.0,
        }
    }
}

/// Character capsule collider configuration
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct CapsuleCollider {
    /// Capsule radius
    pub radius: f32,
    /// Capsule height
    pub height: f32,
}

impl Default for CapsuleCollider {
    fn default() -> Self {
        Self {
            radius: 0.5,
            height: 1.8,
        }
    }
}

/// Camera target marker for third-person camera
#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
pub struct CameraTarget {
    /// Offset from character center for camera target
    pub offset: Vec3,
    /// Camera distance from target
    pub distance: f32,
    /// Camera height offset
    pub height_offset: f32,
    /// Camera follow smoothness (0.0-1.0)
    pub smoothness: f32,
    /// Mouse sensitivity for camera orbit
    pub mouse_sensitivity: f32,
    /// Current camera rotation angles (yaw, pitch)
    pub rotation: Vec2,
}

impl CameraTarget {
    pub fn new() -> Self {
        Self {
            offset: Vec3::new(0.0, 1.0, 0.0),
            distance: 5.0,
            height_offset: 1.5,
            smoothness: 0.1,
            mouse_sensitivity: 0.002,
            rotation: Vec2::ZERO,
        }
    }
}

/// Character controller component for physics-based movement
#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
pub struct CharacterController {
    /// Gravity acceleration
    pub gravity: f32,
    /// Maximum fall speed
    pub max_fall_speed: f32,
    /// Ground detection ray length
    pub ground_ray_length: f32,
    /// Current vertical velocity
    pub vertical_velocity: f32,
}

impl CharacterController {
    pub fn new() -> Self {
        Self {
            gravity: -20.0,
            max_fall_speed: -30.0,
            ground_ray_length: 2.0,
            vertical_velocity: 0.0,
        }
    }
}

/// Character input component
#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
pub struct CharacterInput {
    /// Movement input vector
    pub movement: Vec2,
    /// Jump input
    pub jump: bool,
    /// Sprint input
    pub sprint: bool,
    /// Context action input
    pub context_action: bool,
}
