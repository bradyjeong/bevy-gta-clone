//! Character bundles
//!
//! Bundle types for easily spawning character entities with all required components.

use bevy::animation::AnimationPlayer;
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use bevy::render::mesh::Mesh3d;

#[cfg(feature = "rapier3d_030")]
use bevy_rapier3d::prelude::*;

use super::components::*;
use super::systems::asset_loading::LoadCharacterAsset;
use super::visual::{CharacterVisualConfig, VisualCharacter};
use amp_physics::InterpolatedTransform;

// Resolve velocity ambiguity by being explicit
use crate::character::components::Velocity as CharacterVelocity;

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
    /// Character animations
    pub animations: CharacterAnimations,
    /// Humanoid rig for bone mapping
    pub humanoid_rig: HumanoidRig,
    /// Locomotion state
    pub locomotion: LocomotionState,
    /// Velocity for animation system integration
    pub velocity: CharacterVelocity,
    /// Interpolated transform for smooth physics-visual rendering
    pub interpolated_transform: InterpolatedTransform,
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

impl CharacterBundle {
    pub fn new(animation_set: Handle<AnimationSet>) -> Self {
        Self {
            player: Player,
            speed: Speed::default(),
            grounded: Grounded::default(),
            controller: CharacterController::new(),
            input: CharacterInput::default(),
            camera_target: CameraTarget::new(),
            capsule: CapsuleCollider::default(),
            animations: CharacterAnimations::new(animation_set),
            humanoid_rig: HumanoidRig::default(),
            locomotion: LocomotionState::default(),
            velocity: CharacterVelocity::default(),
            interpolated_transform: InterpolatedTransform::new(Transform::from_xyz(0.0, 1.0, 0.0)),
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

    /// Character animations
    pub animations: CharacterAnimations,
    /// Humanoid rig for bone mapping
    pub humanoid_rig: HumanoidRig,
    /// Locomotion state
    pub locomotion: LocomotionState,
    /// Velocity for animation system integration
    pub velocity: CharacterVelocity,

    /// Interpolated transform for smooth physics-visual rendering
    pub interpolated_transform: InterpolatedTransform,
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
impl PhysicsCharacterBundle {
    pub fn new(animation_set: Handle<AnimationSet>) -> Self {
        let capsule = CapsuleCollider::default();
        let collider = Collider::capsule_y(capsule.height / 2.0 - capsule.radius, capsule.radius);

        Self {
            player: Player,
            speed: Speed::default(),
            grounded: Grounded::default(),
            controller: CharacterController::new(),
            input: CharacterInput::default(),
            camera_target: CameraTarget::new(),
            capsule,
            animations: CharacterAnimations::new(animation_set),
            humanoid_rig: HumanoidRig::default(),
            locomotion: LocomotionState::default(),
            velocity: CharacterVelocity::default(),
            interpolated_transform: InterpolatedTransform::new(Transform::from_xyz(0.0, 1.0, 0.0)),
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
            rigid_body: RigidBody::KinematicPositionBased,
            collider,
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
    pub velocity: CharacterVelocity,

    /// Interpolated transform for smooth physics-visual rendering
    pub interpolated_transform: InterpolatedTransform,
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
            velocity: CharacterVelocity::default(),
            interpolated_transform: InterpolatedTransform::new(Transform::from_xyz(0.0, 1.0, 0.0)),
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

/// Bundle for spawning a complete player character with asset loading
#[derive(Bundle)]
pub struct PlayerBundle {
    /// Character components
    pub character: CharacterBundle,
    /// Animation player for character animations
    pub animation_player: AnimationPlayer,
}

impl PlayerBundle {
    pub fn new(animation_set: Handle<AnimationSet>) -> Self {
        Self {
            character: CharacterBundle::new(animation_set),
            animation_player: AnimationPlayer::default(),
        }
    }

    /// Create a player bundle with a mock skeleton entity
    pub fn new_with_skeleton(
        commands: &mut Commands,
        animation_set: Handle<AnimationSet>,
    ) -> (Self, Entity) {
        // Create a skeleton entity with AnimationPlayer
        let skeleton_entity = commands
            .spawn((
                AnimationPlayer::default(),
                Transform::default(),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                Name::new("Character Skeleton"),
            ))
            .id();

        // Create the character bundle with skeleton reference
        let mut bundle = Self::new(animation_set);
        bundle.character.humanoid_rig.skeleton_entity = skeleton_entity;

        (bundle, skeleton_entity)
    }
}

/// Bundle for spawning a player with physics
#[cfg(feature = "rapier3d_030")]
#[derive(Bundle)]
pub struct PhysicsPlayerBundle {
    /// Physics character components
    pub character: PhysicsCharacterBundle,
    /// Animation player for character animations
    pub animation_player: AnimationPlayer,
}

#[cfg(feature = "rapier3d_030")]
impl PhysicsPlayerBundle {
    pub fn new(animation_set: Handle<AnimationSet>) -> Self {
        Self {
            character: PhysicsCharacterBundle::new(animation_set),
            animation_player: AnimationPlayer::default(),
        }
    }
}

/// Helper functions for spawning characters
impl CharacterBundle {
    /// Spawn a player character with Mixamo asset loading
    pub fn spawn_player(
        commands: &mut Commands,
        gltf_path: impl Into<String>,
        animation_set: Handle<AnimationSet>,
    ) -> Entity {
        commands
            .spawn((
                PlayerBundle::new(animation_set),
                LoadCharacterAsset::new(gltf_path, "player"),
            ))
            .id()
    }

    /// Spawn a player character with custom scale
    pub fn spawn_player_with_scale(
        commands: &mut Commands,
        gltf_path: impl Into<String>,
        animation_set: Handle<AnimationSet>,
        scale: f32,
    ) -> Entity {
        commands
            .spawn((
                PlayerBundle::new(animation_set),
                LoadCharacterAsset::new(gltf_path, "player").with_scale(scale),
            ))
            .id()
    }
}

#[cfg(feature = "rapier3d_030")]
impl PhysicsCharacterBundle {
    /// Spawn a physics-enabled player character with Mixamo asset loading
    pub fn spawn_physics_player(
        commands: &mut Commands,
        gltf_path: impl Into<String>,
        animation_set: Handle<AnimationSet>,
    ) -> Entity {
        commands
            .spawn((
                PhysicsPlayerBundle::new(animation_set),
                LoadCharacterAsset::new(gltf_path, "player"),
            ))
            .id()
    }

    /// Spawn a physics-enabled player character with custom scale
    pub fn spawn_physics_player_with_scale(
        commands: &mut Commands,
        gltf_path: impl Into<String>,
        animation_set: Handle<AnimationSet>,
        scale: f32,
    ) -> Entity {
        commands
            .spawn((
                PhysicsPlayerBundle::new(animation_set),
                LoadCharacterAsset::new(gltf_path, "player").with_scale(scale),
            ))
            .id()
    }
}

/// Bundle for a visual character with physics (combines physics with visual representation)
#[cfg(feature = "rapier3d_030")]
#[derive(Bundle)]
pub struct VisualPhysicsCharacterBundle {
    /// Physics character components
    pub character: PhysicsCharacterBundle,
    /// Animation player for character animations
    pub animation_player: AnimationPlayer,
    /// Visual character marker and body parts
    pub visual_character: VisualCharacter,
}

#[cfg(feature = "rapier3d_030")]
impl VisualPhysicsCharacterBundle {
    pub fn new(animation_set: Handle<AnimationSet>) -> Self {
        Self {
            character: PhysicsCharacterBundle::new(animation_set),
            animation_player: AnimationPlayer::default(),
            visual_character: VisualCharacter,
        }
    }

    /// Spawn a visual physics character with all body parts
    pub fn spawn_visual_physics_character(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        animation_set: Handle<AnimationSet>,
        config: Option<CharacterVisualConfig>,
    ) -> Entity {
        let visual_config = config.unwrap_or_default();

        // Create the main character entity first
        let character_entity = commands.spawn(Self::new(animation_set)).id();

        // Spawn visual body parts
        let body_parts =
            visual_config.spawn_visual_character(commands, meshes, materials, Transform::default());

        // Set all body parts as children of the main character
        commands
            .entity(character_entity)
            .add_child(body_parts.head)
            .add_child(body_parts.torso)
            .add_child(body_parts.left_arm)
            .add_child(body_parts.right_arm)
            .add_child(body_parts.left_leg)
            .add_child(body_parts.right_leg);

        // Add body parts component to main character
        commands.entity(character_entity).insert(body_parts);

        character_entity
    }
}
