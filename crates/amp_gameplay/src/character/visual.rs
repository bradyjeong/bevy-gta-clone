//! Visual character components
//!
//! Components for creating human-like multi-part character representations using Bevy primitives.

use bevy::prelude::*;

/// Marker component for the main character entity (physics body)
#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
pub struct VisualCharacter;

/// Marker components for different body parts
#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
pub struct CharacterHead;

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
pub struct CharacterTorso;

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
pub struct CharacterLeftArm;

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
pub struct CharacterRightArm;

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
pub struct CharacterLeftLeg;

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
pub struct CharacterRightLeg;

/// Component that tracks all body part entities for a character
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct BodyParts {
    pub head: Entity,
    pub torso: Entity,
    pub left_arm: Entity,
    pub right_arm: Entity,
    pub left_leg: Entity,
    pub right_leg: Entity,
}

/// Animation state for body parts
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct BodyPartAnimation {
    /// Base rotation for this body part
    pub base_rotation: Quat,
    /// Current animation offset rotation
    pub animation_offset: Quat,
    /// Oscillation phase for cyclic animations
    pub phase: f32,
    /// Animation speed multiplier
    pub speed_multiplier: f32,
    /// Maximum rotation angle for swinging animations
    pub max_swing_angle: f32,
}

impl Default for BodyPartAnimation {
    fn default() -> Self {
        Self {
            base_rotation: Quat::IDENTITY,
            animation_offset: Quat::IDENTITY,
            phase: 0.0,
            speed_multiplier: 1.0,
            max_swing_angle: 30.0_f32.to_radians(),
        }
    }
}

impl BodyPartAnimation {
    pub fn new_with_swing(max_swing_degrees: f32) -> Self {
        Self {
            max_swing_angle: max_swing_degrees.to_radians(),
            ..default()
        }
    }

    pub fn new_with_phase(phase_offset: f32) -> Self {
        Self {
            phase: phase_offset,
            ..default()
        }
    }
}

/// Character visual configuration
#[derive(Debug, Clone, Reflect)]
pub struct CharacterVisualConfig {
    // Body part sizes
    pub head_radius: f32,
    pub torso_size: Vec3,
    pub arm_size: Vec3,
    pub leg_size: Vec3,

    // Body part positions (relative to character center)
    pub head_offset: Vec3,
    pub torso_offset: Vec3,
    pub left_arm_offset: Vec3,
    pub right_arm_offset: Vec3,
    pub left_leg_offset: Vec3,
    pub right_leg_offset: Vec3,

    // Colors
    pub head_color: Color,
    pub torso_color: Color,
    pub arm_color: Color,
    pub leg_color: Color,

    // Animation settings
    pub arm_swing_angle: f32, // degrees
    pub leg_swing_angle: f32, // degrees
    pub head_bob_amount: f32, // units
    pub animation_speed_base: f32,
}

impl Default for CharacterVisualConfig {
    fn default() -> Self {
        Self {
            // Proportions based on a 1.8m tall character
            head_radius: 0.12,
            torso_size: Vec3::new(0.4, 0.6, 0.2),
            arm_size: Vec3::new(0.08, 0.5, 0.08),
            leg_size: Vec3::new(0.1, 0.7, 0.1),

            // Positions (Y=0 is character feet)
            head_offset: Vec3::new(0.0, 1.6, 0.0),
            torso_offset: Vec3::new(0.0, 1.0, 0.0),
            left_arm_offset: Vec3::new(-0.25, 1.2, 0.0),
            right_arm_offset: Vec3::new(0.25, 1.2, 0.0),
            left_leg_offset: Vec3::new(-0.1, 0.35, 0.0),
            right_leg_offset: Vec3::new(0.1, 0.35, 0.0),

            // Colors for visual distinction
            head_color: Color::srgb(0.9, 0.8, 0.7), // Skin tone
            torso_color: Color::srgb(0.2, 0.4, 0.8), // Blue shirt
            arm_color: Color::srgb(0.9, 0.8, 0.7),  // Skin tone
            leg_color: Color::srgb(0.1, 0.1, 0.3),  // Dark pants

            // Animation parameters
            arm_swing_angle: 30.0,
            leg_swing_angle: 20.0,
            head_bob_amount: 0.05,
            animation_speed_base: 2.0,
        }
    }
}

/// Bundle for spawning a complete visual character
#[derive(Bundle)]
pub struct VisualCharacterBundle {
    /// Visual character marker
    pub visual_character: VisualCharacter,
    /// Body parts tracker
    pub body_parts: BodyParts,
    /// Transform components
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl VisualCharacterBundle {
    pub fn new(body_parts: BodyParts) -> Self {
        Self {
            visual_character: VisualCharacter,
            body_parts,
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
        }
    }
}

/// Helper functions for spawning visual characters
impl CharacterVisualConfig {
    /// Spawn a complete visual character with all body parts
    pub fn spawn_visual_character(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        parent_transform: Transform,
    ) -> BodyParts {
        // Create meshes
        let head_mesh = meshes.add(Sphere::new(self.head_radius));
        let torso_mesh = meshes.add(Cuboid::from_size(self.torso_size));
        let arm_mesh = meshes.add(Capsule3d::new(self.arm_size.x, self.arm_size.y));
        let leg_mesh = meshes.add(Capsule3d::new(self.leg_size.x, self.leg_size.y));

        // Create materials
        let head_material = materials.add(StandardMaterial {
            base_color: self.head_color,
            ..default()
        });
        let torso_material = materials.add(StandardMaterial {
            base_color: self.torso_color,
            ..default()
        });
        let arm_material = materials.add(StandardMaterial {
            base_color: self.arm_color,
            ..default()
        });
        let leg_material = materials.add(StandardMaterial {
            base_color: self.leg_color,
            ..default()
        });

        // Spawn head
        let head = commands
            .spawn((
                Mesh3d(head_mesh),
                MeshMaterial3d(head_material),
                Transform::from_translation(self.head_offset),
                CharacterHead,
                BodyPartAnimation::default(),
                Name::new("Character Head"),
            ))
            .id();

        // Spawn torso
        let torso = commands
            .spawn((
                Mesh3d(torso_mesh),
                MeshMaterial3d(torso_material),
                Transform::from_translation(self.torso_offset),
                CharacterTorso,
                BodyPartAnimation::default(),
                Name::new("Character Torso"),
            ))
            .id();

        // Spawn left arm
        let left_arm = commands
            .spawn((
                Mesh3d(arm_mesh.clone()),
                MeshMaterial3d(arm_material.clone()),
                Transform::from_translation(self.left_arm_offset),
                CharacterLeftArm,
                BodyPartAnimation::new_with_swing(self.arm_swing_angle),
                Name::new("Character Left Arm"),
            ))
            .id();

        // Spawn right arm (phase offset for natural swing)
        let right_arm = commands
            .spawn((
                Mesh3d(arm_mesh),
                MeshMaterial3d(arm_material),
                Transform::from_translation(self.right_arm_offset),
                CharacterRightArm,
                BodyPartAnimation::new_with_phase(std::f32::consts::PI), // 180 degrees out of phase
                Name::new("Character Right Arm"),
            ))
            .id();

        // Spawn left leg
        let left_leg = commands
            .spawn((
                Mesh3d(leg_mesh.clone()),
                MeshMaterial3d(leg_material.clone()),
                Transform::from_translation(self.left_leg_offset),
                CharacterLeftLeg,
                BodyPartAnimation::new_with_swing(self.leg_swing_angle),
                Name::new("Character Left Leg"),
            ))
            .id();

        // Spawn right leg (phase offset for natural gait)
        let right_leg = commands
            .spawn((
                Mesh3d(leg_mesh),
                MeshMaterial3d(leg_material),
                Transform::from_translation(self.right_leg_offset),
                CharacterRightLeg,
                BodyPartAnimation::new_with_phase(std::f32::consts::PI), // 180 degrees out of phase
                Name::new("Character Right Leg"),
            ))
            .id();

        BodyParts {
            head,
            torso,
            left_arm,
            right_arm,
            left_leg,
            right_leg,
        }
    }
}
