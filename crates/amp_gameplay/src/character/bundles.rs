//! Character bundles
//!
//! Bundle types for easily spawning character entities with all required components.

use bevy::prelude::*;

#[cfg(feature = "rapier3d_030")]
use bevy_rapier3d::prelude::*;

use super::components::*;

/// Complete character bundle with physics
#[derive(Bundle)]
pub struct CharacterBundle {
    /// Player marker
    pub player: Player,
    /// Character speed configuration
    pub speed: Speed,
    /// Grounded state
    pub grounded: Grounded,
    /// Character controller
    pub controller: CharacterController,
    /// Character input
    pub input: CharacterInput,
    /// Camera target configuration
    pub camera_target: CameraTarget,
    /// Capsule collider configuration
    pub capsule: CapsuleCollider,
    /// Transform for position and rotation
    pub transform: Transform,
    /// Global transform (computed automatically)
    pub global_transform: GlobalTransform,
    /// Visibility
    pub visibility: Visibility,
    /// Inherited visibility (computed automatically)
    pub inherited_visibility: InheritedVisibility,
    /// View visibility (computed automatically)
    pub view_visibility: ViewVisibility,
}

impl Default for CharacterBundle {
    fn default() -> Self {
        Self {
            player: Player,
            speed: Speed::default(),
            grounded: Grounded::default(),
            controller: CharacterController::new(),
            input: CharacterInput::default(),
            camera_target: CameraTarget::new(),
            capsule: CapsuleCollider::default(),
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
        }
    }
}

/// Physics character bundle with Rapier integration
#[cfg(feature = "rapier3d_030")]
#[derive(Bundle)]
pub struct PhysicsCharacterBundle {
    /// Base character components
    pub player: Player,
    pub speed: Speed,
    pub grounded: Grounded,
    pub controller: CharacterController,
    pub input: CharacterInput,
    pub camera_target: CameraTarget,
    pub capsule: CapsuleCollider,

    /// Transform components
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,

    /// Rapier rigid body (kinematic for character controller)
    pub rigid_body: RigidBody,

    /// Rapier collider for capsule shape
    pub collider: Collider,

    /// Kinematic character controller
    pub character_controller: KinematicCharacterController,

    /// Character controller output
    pub controller_output: KinematicCharacterControllerOutput,

    /// Collision groups
    pub collision_groups: CollisionGroups,

    /// Solver groups
    pub solver_groups: SolverGroups,
}

#[cfg(feature = "rapier3d_030")]
impl Default for PhysicsCharacterBundle {
    fn default() -> Self {
        let capsule = CapsuleCollider::default();

        Self {
            player: Player,
            speed: Speed::default(),
            grounded: Grounded::default(),
            controller: CharacterController::new(),
            input: CharacterInput::default(),
            camera_target: CameraTarget::new(),
            capsule,
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
            rigid_body: RigidBody::KinematicPositionBased,
            collider: Collider::capsule_y(capsule.height / 2.0 - capsule.radius, capsule.radius),
            character_controller: KinematicCharacterController {
                // Allow climbing small steps/curbs
                max_slope_climb_angle: 45.0_f32.to_radians(),
                // Prevent getting stuck on small gaps
                min_slope_slide_angle: 30.0_f32.to_radians(),
                // Auto-step feature for stairs
                autostep: Some(CharacterAutostep {
                    max_height: CharacterLength::Absolute(0.3),
                    min_width: CharacterLength::Absolute(0.2),
                    include_dynamic_bodies: false,
                }),
                // Snap to ground for stable movement
                snap_to_ground: Some(CharacterLength::Absolute(0.2)),
                ..default()
            },
            controller_output: KinematicCharacterControllerOutput::default(),
            collision_groups: CollisionGroups::new(
                Group::GROUP_1, // Character group
                Group::ALL,     // Collides with everything
            ),
            solver_groups: SolverGroups::new(
                Group::GROUP_1, // Character group
                Group::ALL,     // Interacts with everything
            ),
        }
    }
}

/// Simple character bundle without physics (for basic movement)
#[derive(Bundle)]
pub struct SimpleCharacterBundle {
    /// Base character components
    pub player: Player,
    pub speed: Speed,
    pub grounded: Grounded,
    pub controller: CharacterController,
    pub input: CharacterInput,
    pub camera_target: CameraTarget,
    pub capsule: CapsuleCollider,

    /// Transform components
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,

    /// Mesh for visual representation
    pub mesh: Mesh3d,

    /// Material
    pub material: MeshMaterial3d<StandardMaterial>,
}

impl SimpleCharacterBundle {
    pub fn new(mesh: Handle<Mesh>, material: Handle<StandardMaterial>) -> Self {
        Self {
            player: Player,
            speed: Speed::default(),
            grounded: Grounded::default(),
            controller: CharacterController::new(),
            input: CharacterInput::default(),
            camera_target: CameraTarget::new(),
            capsule: CapsuleCollider::default(),
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
            mesh: Mesh3d(mesh),
            material: MeshMaterial3d(material),
        }
    }
}
