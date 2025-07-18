//! Character gameplay systems
//!
//! This module provides character movement, camera control, and interaction systems
//! for third-person character gameplay.

pub mod bundles;
pub mod components;
pub mod systems;

use bevy::app::{App, FixedUpdate, Plugin, Update};
use bevy::prelude::*;

use self::systems::*;

/// Character plugin providing player movement and camera systems
#[derive(Default)]
pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register component types
            .register_type::<components::Player>()
            .register_type::<components::Grounded>()
            .register_type::<components::Speed>()
            .register_type::<components::CapsuleCollider>()
            .register_type::<components::CameraTarget>()
            .register_type::<components::CharacterController>()
            .register_type::<components::CharacterInput>()
            // Resources
            .init_resource::<resources::CharacterInputState>()
            // Systems - player movement in FixedUpdate for physics consistency
            .add_systems(
                FixedUpdate,
                (
                    input::handle_character_input,
                    movement::player_movement,
                    movement::apply_gravity,
                    movement::handle_jumping,
                    movement::update_grounded_state,
                )
                    .chain(),
            )
            // Camera systems in Update for smooth visual updates
            .add_systems(Update, (camera::camera_follow, camera::camera_orbit));
    }
}

/// Resources module
pub mod resources {
    use bevy::prelude::*;

    /// Character input state resource
    #[derive(Resource, Default, Debug, Reflect)]
    #[reflect(Resource)]
    pub struct CharacterInputState {
        /// Movement input (-1.0 to 1.0 for X and Z axes)
        pub movement: Vec2,
        /// Jump input
        pub jump: bool,
        /// Sprint input
        pub sprint: bool,
        /// Context action input (for vehicle interaction)
        pub context_action: bool,
        /// Mouse delta for camera
        pub mouse_delta: Vec2,
    }
}
