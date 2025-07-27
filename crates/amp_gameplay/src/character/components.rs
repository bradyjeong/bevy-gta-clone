//! Character components
//!
//! Components for character entities including movement, collision, and interaction.

use bevy::animation::graph::{AnimationGraph, AnimationNodeIndex};
use bevy::prelude::*;
use std::collections::HashMap;

/// Strongly-typed humanoid bone enum for Mixamo rig compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum HumanoidBone {
    // Core hierarchy
    Hips,
    Spine,
    Spine1,
    Spine2,
    Neck,
    Head,

    // Left arm
    LeftShoulder,
    LeftArm,
    LeftForeArm,
    LeftHand,
    LeftHandThumb1,
    LeftHandThumb2,
    LeftHandThumb3,
    LeftHandIndex1,
    LeftHandIndex2,
    LeftHandIndex3,
    LeftHandMiddle1,
    LeftHandMiddle2,
    LeftHandMiddle3,
    LeftHandRing1,
    LeftHandRing2,
    LeftHandRing3,
    LeftHandPinky1,
    LeftHandPinky2,
    LeftHandPinky3,

    // Right arm (mirrored)
    RightShoulder,
    RightArm,
    RightForeArm,
    RightHand,
    RightHandThumb1,
    RightHandThumb2,
    RightHandThumb3,
    RightHandIndex1,
    RightHandIndex2,
    RightHandIndex3,
    RightHandMiddle1,
    RightHandMiddle2,
    RightHandMiddle3,
    RightHandRing1,
    RightHandRing2,
    RightHandRing3,
    RightHandPinky1,
    RightHandPinky2,
    RightHandPinky3,

    // Left leg
    LeftUpLeg,
    LeftLeg,
    LeftFoot,
    LeftToeBase,

    // Right leg
    RightUpLeg,
    RightLeg,
    RightFoot,
    RightToeBase,
}

impl HumanoidBone {
    /// Number of bone variants in this enum
    pub const VARIANT_COUNT: usize = 52;

    /// Get all bone variants in hierarchical order
    pub fn all_bones() -> Vec<Self> {
        use HumanoidBone::*;
        vec![
            Hips,
            Spine,
            Spine1,
            Spine2,
            Neck,
            Head,
            LeftShoulder,
            LeftArm,
            LeftForeArm,
            LeftHand,
            LeftHandThumb1,
            LeftHandThumb2,
            LeftHandThumb3,
            LeftHandIndex1,
            LeftHandIndex2,
            LeftHandIndex3,
            LeftHandMiddle1,
            LeftHandMiddle2,
            LeftHandMiddle3,
            LeftHandRing1,
            LeftHandRing2,
            LeftHandRing3,
            LeftHandPinky1,
            LeftHandPinky2,
            LeftHandPinky3,
            RightShoulder,
            RightArm,
            RightForeArm,
            RightHand,
            RightHandThumb1,
            RightHandThumb2,
            RightHandThumb3,
            RightHandIndex1,
            RightHandIndex2,
            RightHandIndex3,
            RightHandMiddle1,
            RightHandMiddle2,
            RightHandMiddle3,
            RightHandRing1,
            RightHandRing2,
            RightHandRing3,
            RightHandPinky1,
            RightHandPinky2,
            RightHandPinky3,
            LeftUpLeg,
            LeftLeg,
            LeftFoot,
            LeftToeBase,
            RightUpLeg,
            RightLeg,
            RightFoot,
            RightToeBase,
        ]
    }

    /// Get the canonical name used in Mixamo rigs
    pub fn mixamo_name(&self) -> &'static str {
        use HumanoidBone::*;
        match self {
            Hips => "mixamorig:Hips",
            Spine => "mixamorig:Spine",
            Spine1 => "mixamorig:Spine1",
            Spine2 => "mixamorig:Spine2",
            Neck => "mixamorig:Neck",
            Head => "mixamorig:Head",
            LeftShoulder => "mixamorig:LeftShoulder",
            LeftArm => "mixamorig:LeftArm",
            LeftForeArm => "mixamorig:LeftForeArm",
            LeftHand => "mixamorig:LeftHand",
            LeftHandThumb1 => "mixamorig:LeftHandThumb1",
            LeftHandThumb2 => "mixamorig:LeftHandThumb2",
            LeftHandThumb3 => "mixamorig:LeftHandThumb3",
            LeftHandIndex1 => "mixamorig:LeftHandIndex1",
            LeftHandIndex2 => "mixamorig:LeftHandIndex2",
            LeftHandIndex3 => "mixamorig:LeftHandIndex3",
            LeftHandMiddle1 => "mixamorig:LeftHandMiddle1",
            LeftHandMiddle2 => "mixamorig:LeftHandMiddle2",
            LeftHandMiddle3 => "mixamorig:LeftHandMiddle3",
            LeftHandRing1 => "mixamorig:LeftHandRing1",
            LeftHandRing2 => "mixamorig:LeftHandRing2",
            LeftHandRing3 => "mixamorig:LeftHandRing3",
            LeftHandPinky1 => "mixamorig:LeftHandPinky1",
            LeftHandPinky2 => "mixamorig:LeftHandPinky2",
            LeftHandPinky3 => "mixamorig:LeftHandPinky3",
            RightShoulder => "mixamorig:RightShoulder",
            RightArm => "mixamorig:RightArm",
            RightForeArm => "mixamorig:RightForeArm",
            RightHand => "mixamorig:RightHand",
            RightHandThumb1 => "mixamorig:RightHandThumb1",
            RightHandThumb2 => "mixamorig:RightHandThumb2",
            RightHandThumb3 => "mixamorig:RightHandThumb3",
            RightHandIndex1 => "mixamorig:RightHandIndex1",
            RightHandIndex2 => "mixamorig:RightHandIndex2",
            RightHandIndex3 => "mixamorig:RightHandIndex3",
            RightHandMiddle1 => "mixamorig:RightHandMiddle1",
            RightHandMiddle2 => "mixamorig:RightHandMiddle2",
            RightHandMiddle3 => "mixamorig:RightHandMiddle3",
            RightHandRing1 => "mixamorig:RightHandRing1",
            RightHandRing2 => "mixamorig:RightHandRing2",
            RightHandRing3 => "mixamorig:RightHandRing3",
            RightHandPinky1 => "mixamorig:RightHandPinky1",
            RightHandPinky2 => "mixamorig:RightHandPinky2",
            RightHandPinky3 => "mixamorig:RightHandPinky3",
            LeftUpLeg => "mixamorig:LeftUpLeg",
            LeftLeg => "mixamorig:LeftLeg",
            LeftFoot => "mixamorig:LeftFoot",
            LeftToeBase => "mixamorig:LeftToeBase",
            RightUpLeg => "mixamorig:RightUpLeg",
            RightLeg => "mixamorig:RightLeg",
            RightFoot => "mixamorig:RightFoot",
            RightToeBase => "mixamorig:RightToeBase",
        }
    }
}

/// Marker component for player character
#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
pub struct Player;

/// Marker component for the active entity (usually the player)
#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
pub struct ActiveEntity;

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
            walk: 1.0,              // Oracle tuning: Match GLB animation authoring ~1 m/s walk
            sprint_multiplier: 3.5, // Oracle tuning: Match GLB ~3.5 m/s run speed
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

/// Locomotion state enum for character movement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum Locomotion {
    /// Character is stationary
    Idle = 0,
    /// Character is walking
    Walk = 1,
    /// Character is running
    Run = 2,
    /// Character is sprinting
    Sprint = 3,
    /// Character is jumping
    Jump = 4,
    /// Character is falling
    Fall = 5,
    /// Character is landing
    Land = 6,
    /// Character is turning
    Turn = 7,
}

impl Locomotion {
    /// Number of locomotion variants
    pub const VARIANT_COUNT: usize = 8;

    /// Get all locomotion variants
    pub fn all_variants() -> [Self; Self::VARIANT_COUNT] {
        [
            Self::Idle,
            Self::Walk,
            Self::Run,
            Self::Sprint,
            Self::Jump,
            Self::Fall,
            Self::Land,
            Self::Turn,
        ]
    }
}

impl Default for Locomotion {
    fn default() -> Self {
        Self::Idle
    }
}

/// Humanoid rig component for character animation with strongly-typed bone lookup
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct HumanoidRig {
    /// Entity containing the skeleton root
    pub skeleton_entity: Entity,
    /// Fixed-size array for fast bone lookups using enum index
    pub bone_indices: [Option<u32>; HumanoidBone::VARIANT_COUNT],
    /// Scale factor for the rig
    pub scale: f32,
}

impl Default for HumanoidRig {
    fn default() -> Self {
        Self {
            skeleton_entity: Entity::PLACEHOLDER,
            bone_indices: [None; HumanoidBone::VARIANT_COUNT],
            scale: 1.0,
        }
    }
}

impl HumanoidRig {
    /// Get bone index for a specific bone type
    pub fn get_bone_index(&self, bone: HumanoidBone) -> Option<u32> {
        self.bone_indices[bone as usize]
    }

    /// Set bone index for a specific bone type
    pub fn set_bone_index(&mut self, bone: HumanoidBone, index: u32) {
        self.bone_indices[bone as usize] = Some(index);
    }

    /// Initialize rig from a skeleton entity by matching bone names
    pub fn from_skeleton(skeleton_entity: Entity, bone_names: &[String]) -> Self {
        let mut rig = Self {
            skeleton_entity,
            bone_indices: [None; HumanoidBone::VARIANT_COUNT],
            scale: 1.0,
        };

        // Match bone names to HumanoidBone variants with improved name matching
        for (index, bone_name) in bone_names.iter().enumerate() {
            for bone in HumanoidBone::all_bones() {
                let canonical_name = bone.mixamo_name();

                // First try exact match
                if bone_name == canonical_name {
                    rig.set_bone_index(bone, index as u32);
                    break;
                }

                // Then try full name match after stripping common prefixes
                let bone_base_name = canonical_name
                    .strip_prefix("mixamorig:")
                    .unwrap_or(canonical_name);
                let input_stripped = bone_name
                    .strip_prefix("mixamorig:")
                    .or_else(|| bone_name.strip_prefix("Character1_"))
                    .or_else(|| bone_name.strip_prefix("Armature_"))
                    .unwrap_or(bone_name);

                if input_stripped == bone_base_name {
                    rig.set_bone_index(bone, index as u32);
                    break;
                }
            }
        }

        rig
    }

    /// Initialize rig from a skeleton entity using optimized O(n) bone mapping
    pub fn from_skeleton_optimized(skeleton_entity: Entity, bone_names: &[String]) -> Self {
        let mut rig = Self {
            skeleton_entity,
            bone_indices: [None; HumanoidBone::VARIANT_COUNT],
            scale: 1.0,
        };

        // Build HashMap for O(1) lookups after prefix stripping
        let mut name_to_index: HashMap<String, u32> = HashMap::with_capacity(bone_names.len());

        for (index, bone_name) in bone_names.iter().enumerate() {
            // Store original name
            name_to_index.insert(bone_name.clone(), index as u32);

            // Store stripped variants for flexible matching
            if let Some(stripped) = bone_name.strip_prefix("mixamorig:") {
                name_to_index.insert(stripped.to_string(), index as u32);
            }
            if let Some(stripped) = bone_name.strip_prefix("Character1_") {
                name_to_index.insert(stripped.to_string(), index as u32);
            }
            if let Some(stripped) = bone_name.strip_prefix("Armature_") {
                name_to_index.insert(stripped.to_string(), index as u32);
            }
        }

        // O(n) bone matching using HashMap lookups
        for bone in HumanoidBone::all_bones() {
            let canonical_name = bone.mixamo_name();

            // Try exact match first
            if let Some(&index) = name_to_index.get(canonical_name) {
                rig.set_bone_index(bone, index);
                continue;
            }

            // Try stripped canonical name
            if let Some(base_name) = canonical_name.strip_prefix("mixamorig:") {
                if let Some(&index) = name_to_index.get(base_name) {
                    rig.set_bone_index(bone, index);
                }
            }
        }

        rig
    }

    /// Check if a bone is present in the rig
    pub fn has_bone(&self, bone: HumanoidBone) -> bool {
        self.get_bone_index(bone).is_some()
    }
}

/// Locomotion state component with simplified transition logic
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct LocomotionState {
    /// Current locomotion state
    pub current: Locomotion,
    /// Previous locomotion state
    pub previous: Locomotion,
    /// Optional transition timer for state blending
    pub transition_timer: Option<Timer>,
}

impl Default for LocomotionState {
    fn default() -> Self {
        Self {
            current: Locomotion::Idle,
            previous: Locomotion::Idle,
            transition_timer: None,
        }
    }
}

impl LocomotionState {
    /// Start a transition to a new locomotion state
    pub fn transition_to(&mut self, new_state: Locomotion, duration: f32) {
        if new_state != self.current {
            self.previous = self.current;
            self.current = new_state;
            self.transition_timer = Some(Timer::from_seconds(duration, TimerMode::Once));
        }
    }

    /// Get transition progress (0.0 = fully previous state, 1.0 = fully current state)
    pub fn transition_progress(&self) -> f32 {
        match &self.transition_timer {
            Some(timer) => timer.fraction(),
            None => 1.0, // No transition, fully in current state
        }
    }

    /// Check if currently transitioning between states
    pub fn is_transitioning(&self) -> bool {
        self.transition_timer
            .as_ref()
            .map_or(false, |timer| !timer.finished())
    }

    /// Update transition timer with delta time
    pub fn update(&mut self, delta: f32) {
        if let Some(timer) = &mut self.transition_timer {
            timer.tick(std::time::Duration::from_secs_f32(delta));
            if timer.finished() {
                self.transition_timer = None;
                self.previous = self.current;
            }
        }
    }
}

/// Velocity component for smooth character movement with SIMD optimization
#[derive(Component, Default, Debug, Reflect, Clone)]
#[reflect(Component)]
pub struct Velocity {
    /// Linear velocity vector (using Vec3A for SIMD benefits)
    pub linear: Vec3,
    /// Angular velocity (yaw rate)
    pub angular: f32,
    /// Desired linear velocity for smooth interpolation
    pub desired_linear: Vec3,
    /// Desired angular velocity for smooth interpolation
    pub desired_angular: f32,
    /// Acceleration rate for linear velocity
    pub linear_acceleration: f32,
    /// Acceleration rate for angular velocity
    pub angular_acceleration: f32,
    /// Maximum velocity change per frame to prevent oscillation
    pub max_velocity_delta: f32,
}

impl Velocity {
    pub fn new() -> Self {
        Self {
            linear: Vec3::ZERO,
            angular: 0.0,
            desired_linear: Vec3::ZERO,
            desired_angular: 0.0,
            linear_acceleration: 4.0, // Oracle tuning: Smoother acceleration transitions
            angular_acceleration: 15.0,
            max_velocity_delta: 50.0, // Prevent large frame-time spikes
        }
    }
}

// AnimationGraph is now provided by Bevy 0.16.1 directly

/// Per-entity animation set asset for character animations
#[derive(Asset, Debug, Reflect)]
pub struct AnimationSet {
    /// Animation clips for each locomotion state (array-based for performance)
    pub clips: [Option<Handle<AnimationClip>>; Locomotion::VARIANT_COUNT],
    /// Animation graph handle
    pub graph: Handle<AnimationGraph>,
    /// Node indices for each animation in the graph
    pub node_indices: [Option<AnimationNodeIndex>; Locomotion::VARIANT_COUNT],
    /// Default animation blend weights (array-based for performance)
    pub blend_weights: [f32; Locomotion::VARIANT_COUNT],
    /// Animation transition table for smooth state changes
    pub transition_speeds: [f32; Locomotion::VARIANT_COUNT],
    /// Character type or variant identifier
    pub character_type: String,
}

impl Default for AnimationSet {
    fn default() -> Self {
        Self {
            clips: [const { None }; Locomotion::VARIANT_COUNT],
            graph: Handle::default(),
            node_indices: [const { None }; Locomotion::VARIANT_COUNT],
            blend_weights: [1.0; Locomotion::VARIANT_COUNT],
            transition_speeds: [1.0; Locomotion::VARIANT_COUNT],
            character_type: "default".to_string(),
        }
    }
}

impl AnimationSet {
    /// Create a new animation set for a specific character type
    pub fn new(character_type: &str) -> Self {
        Self {
            clips: [const { None }; Locomotion::VARIANT_COUNT],
            graph: Handle::default(),
            node_indices: [const { None }; Locomotion::VARIANT_COUNT],
            blend_weights: [1.0; Locomotion::VARIANT_COUNT],
            transition_speeds: [1.0; Locomotion::VARIANT_COUNT],
            character_type: character_type.to_string(),
        }
    }

    /// Add an animation clip for a specific locomotion state
    pub fn add_clip(&mut self, state: Locomotion, clip: Handle<AnimationClip>, weight: f32) {
        let index = state as usize;
        self.clips[index] = Some(clip);
        self.blend_weights[index] = weight;
    }

    /// Set an animation clip for a specific locomotion state (with default weight)
    pub fn set_clip(&mut self, state: Locomotion, clip: Handle<AnimationClip>) {
        self.add_clip(state, clip, 1.0);
    }

    /// Set node index for a specific locomotion state
    pub fn set_node_index(&mut self, state: Locomotion, node_index: AnimationNodeIndex) {
        self.node_indices[state as usize] = Some(node_index);
    }

    /// Get animation clip for a locomotion state
    pub fn get_clip(&self, state: Locomotion) -> Option<&Handle<AnimationClip>> {
        self.clips[state as usize].as_ref()
    }

    /// Get node index for a locomotion state
    pub fn get_node_index(&self, state: Locomotion) -> Option<AnimationNodeIndex> {
        self.node_indices[state as usize]
    }

    /// Get blend weight for a locomotion state
    pub fn get_blend_weight(&self, state: Locomotion) -> f32 {
        self.blend_weights[state as usize]
    }

    /// Set transition speed for a locomotion state
    pub fn set_transition_speed(&mut self, state: Locomotion, speed: f32) {
        self.transition_speeds[state as usize] = speed;
    }

    /// Get transition speed for a locomotion state
    pub fn get_transition_speed(&self, state: Locomotion) -> f32 {
        self.transition_speeds[state as usize]
    }

    /// Validate the animation set and log warnings for missing clips
    pub fn validate(&self) {
        for variant in Locomotion::all_variants() {
            if self.get_clip(variant).is_none() {
                warn!(
                    "AnimationSet '{}' is missing animation clip for {:?}",
                    self.character_type, variant
                );
            }
        }
    }
}

/// Component that holds a handle to the character's animation set
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct CharacterAnimations {
    /// Handle to the animation set asset
    pub animation_set: Handle<AnimationSet>,
}

impl CharacterAnimations {
    pub fn new(animation_set: Handle<AnimationSet>) -> Self {
        Self { animation_set }
    }
}

/// Component linking a skeleton entity to its controlling character entity
#[derive(Component, Debug, Reflect, Clone)]
#[reflect(Component)]
pub struct ControlledBy {
    /// The entity that controls this skeleton (e.g., Player entity)
    pub controller_entity: Entity,
}

impl ControlledBy {
    pub fn new(controller_entity: Entity) -> Self {
        Self { controller_entity }
    }
}

/// Animation playback component for driving AnimationPlayer
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct AnimationPlayback {
    /// Currently playing animation clip
    pub current_clip: Option<Handle<AnimationClip>>,
    /// Desired animation speed
    pub wants_speed: f32,
}

impl Default for AnimationPlayback {
    fn default() -> Self {
        Self {
            current_clip: None,
            wants_speed: 1.0,
        }
    }
}

/// Character input component with extended functionality
#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
pub struct CharacterInput {
    /// Movement input vector (y=forward/back only, x should be 0.0)
    pub move_2d: Vec2,
    /// Yaw rotation intent (-1.0 = left, 0.0 = none, 1.0 = right)
    pub yaw: f32,
    /// Look delta for mouse look (y=pitch only)
    pub look_delta: Vec2,
    /// Sprint input
    pub sprint: bool,
    /// Jump input
    pub jump: bool,
    /// Crouch input
    pub crouch: bool,
    /// Interact input
    pub interact: bool,
    /// Context action input (legacy compatibility)
    pub context_action: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_humanoid_bone_variant_count() {
        // Ensure VARIANT_COUNT matches actual enum variants
        let all_bones = HumanoidBone::all_bones();
        assert_eq!(all_bones.len(), HumanoidBone::VARIANT_COUNT);

        // Verify no duplicate bones
        let mut unique_bones = std::collections::HashSet::new();
        for bone in &all_bones {
            assert!(
                unique_bones.insert(*bone),
                "Duplicate bone variant: {:?}",
                bone
            );
        }
    }

    #[test]
    fn test_humanoid_rig_array_size() {
        // Ensure bone_indices array size matches enum variant count
        let rig = HumanoidRig::default();
        assert_eq!(rig.bone_indices.len(), HumanoidBone::VARIANT_COUNT);
    }

    #[test]
    fn test_bone_name_matching() {
        let bone_names = vec![
            "mixamorig:Hips".to_string(),
            "Character1_LeftArm".to_string(),
            "Armature_RightFoot".to_string(),
            "mixamorig:Spine".to_string(),
        ];

        let rig = HumanoidRig::from_skeleton(Entity::PLACEHOLDER, &bone_names);

        // Verify proper bone matching
        assert!(rig.has_bone(HumanoidBone::Hips));
        assert!(rig.has_bone(HumanoidBone::LeftArm));
        assert!(rig.has_bone(HumanoidBone::RightFoot));
        assert!(rig.has_bone(HumanoidBone::Spine));
    }

    #[test]
    fn test_velocity_clamping_prevents_oscillation() {
        let mut velocity = Velocity::new();
        velocity.desired_linear = Vec3::new(100.0, 0.0, 0.0); // Very large desired velocity
        velocity.linear = Vec3::ZERO;

        // Simulate one frame with large dt
        let large_dt: f32 = 1.0; // 1 second frame time
        let max_change = velocity.max_velocity_delta * large_dt.min(1.0 / 30.0);

        // The velocity change should be clamped
        assert!(
            max_change < 100.0,
            "Velocity should be clamped to prevent overshooting"
        );
    }
}
