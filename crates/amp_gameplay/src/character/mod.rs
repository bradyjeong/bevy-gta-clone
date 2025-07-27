//! Character gameplay systems
//!
//! This module provides character movement, camera control, and interaction systems
//! for third-person character gameplay.

pub mod assets;
pub mod bundles;
pub mod components;
pub mod systems;
pub mod visual;

use bevy::app::{App, FixedUpdate, Plugin, Update};
use bevy::prelude::*;

use self::assets::CharacterAssetPlugin;
use self::systems::*;

/// Character plugin providing player movement and camera systems
#[derive(Default)]
pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add character asset plugin
            .add_plugins(CharacterAssetPlugin)
            // Register component types
            .register_type::<components::Player>()
            .register_type::<components::Grounded>()
            .register_type::<components::Speed>()
            .register_type::<components::CapsuleCollider>()
            .register_type::<components::CameraTarget>()
            .register_type::<components::CharacterController>()
            .register_type::<components::CharacterInput>()
            .register_type::<components::Locomotion>()
            .register_type::<components::HumanoidBone>()
            .register_type::<components::HumanoidRig>()
            .register_type::<components::LocomotionState>()
            .register_type::<components::Velocity>()
            .register_type::<components::CharacterAnimations>()
            .register_type::<components::AnimationPlayback>()
            .register_type::<components::ControlledBy>()
            // Visual character components
            .register_type::<visual::VisualCharacter>()
            .register_type::<visual::CharacterHead>()
            .register_type::<visual::CharacterTorso>()
            .register_type::<visual::CharacterLeftArm>()
            .register_type::<visual::CharacterRightArm>()
            .register_type::<visual::CharacterLeftLeg>()
            .register_type::<visual::CharacterRightLeg>()
            .register_type::<visual::BodyParts>()
            .register_type::<visual::BodyPartAnimation>()
            // Register assets
            .init_asset::<components::AnimationSet>()
            .register_type::<components::AnimationSet>()
            .init_resource::<systems::AssetLoadingState>()
            // .add_event::<systems::animation_playback::CharacterReady>()  // Remove missing event
            // Resources
            .init_resource::<resources::CharacterInputState>()
            .init_resource::<systems::animation::LocomotionThresholds>()
            // Startup systems
            .add_systems(Startup, input::setup_character_input)
            // Systems - player movement in FixedUpdate for physics consistency
            .add_systems(
                FixedUpdate,
                (
                    input::handle_character_input,
                    movement::player_movement,
                    movement::apply_gravity,
                    movement::handle_jumping,
                    movement::update_grounded_state,
                    movement::update_velocity_steering, // After movement to ensure final velocity is set
                    movement::sync_velocity_to_skeleton, // Oracle's Option A: Sync velocity to skeleton after updates
                )
                    .chain(),
            )
            // Animation systems in Update for real-time state transitions
            // Oracle's recommended system ordering to avoid race conditions
            .add_systems(
                Update,
                (
                    systems::setup_animation_graph_handles,
                    systems::animation_graph::initialise_animation_players_with_graph
                        .after(systems::setup_animation_graph_handles),
                    animation::debug_animation_components,
                    animation::update_locomotion_state,
                    animation::apply_animation_transitions
                        .after(animation::update_locomotion_state),
                    animation::drive_animation_player
                        .after(animation::apply_animation_transitions)
                        .after(bevy::animation::advance_animations)
                        .after(systems::animation_graph::initialise_animation_players_with_graph),
                    animation::preserve_character_position
                        .after(animation::drive_animation_player)
                        .after(bevy::animation::advance_animations),
                    // animation_playback::update_character_animations,
                    // animation_playback::handle_animation_transitions,
                    apply_animation_playback_stub, // Keep stub for now
                ),
            )
            // ─── BASIC VISUAL ANIMATION ────────────────────────────────
            .add_systems(
                Update,
                (
                    systems::visual_animation::update_body_part_animations,
                    systems::visual_animation::update_character_visual_state,
                )
                    .chain(),
            )
            // Animation playback systems - simplified
            // .add_systems(
            //     Update,
            //     (
            //         animation_playback::start_idle_animation_on_ready,
            //         animation_playback::initialize_animation_playback,
            //         animation_playback::update_animation_playback,
            //         animation_playback::apply_animation_player_updates,
            //         animation_playback::handle_animation_events,
            //         animation_playback::handle_initial_animation_startup,
            //         animation_playback::ensure_animation_player_components,
            //     )
            //         .chain(),
            // )
            // Camera systems disabled - using SmoothCameraPlugin instead to avoid conflicts
            // .add_systems(Update, (camera::camera_follow, camera::camera_orbit))
            // Asset loading systems
            .add_systems(
                Update,
                (
                    systems::handle_character_loading_requests,
                    systems::process_loaded_characters,
                    systems::apply_model_scale_corrections,
                ),
            );
    }
}

/// Resources module
pub mod resources {
    use bevy::prelude::*;

    /// Character input state resource
    #[derive(Resource, Default, Debug, Reflect)]
    #[reflect(Resource)]
    pub struct CharacterInputState {
        /// Movement input (-1.0 to 1.0 for forward/back)
        pub movement: Vec2,
        /// Rotation input (-1.0 to 1.0 for left/right rotation)
        pub rotation: f32,
        /// Jump input
        pub jump: bool,
        /// Sprint input
        pub sprint: bool,
        /// Context action input (for vehicle interaction)
        pub context_action: bool,
    }
}
