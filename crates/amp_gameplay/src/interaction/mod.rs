//! Vehicle interaction systems
//!
//! This module provides F key vehicle interaction functionality for seamless
//! character-to-vehicle and vehicle-to-character transitions.

pub mod components;
pub mod systems;

use bevy::app::{App, Plugin, Update};
use bevy::prelude::*;

use self::systems::*;

/// Plugin for vehicle interaction (F key enter/exit)
#[derive(Default)]
pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register component types
            .register_type::<components::VehicleInteraction>()
            .register_type::<components::InteractionPrompt>()
            .register_type::<components::PlayerState>()
            .register_type::<components::VehicleCameraRig>()
            .register_type::<components::CharacterCameraRig>()
            .register_type::<components::InVehicle>()
            // Resources
            .init_resource::<resources::InteractionState>()
            // Events
            .add_event::<events::VehicleEnterEvent>()
            .add_event::<events::VehicleExitEvent>()
            .add_event::<events::InteractionPromptEvent>()
            // Systems
            .add_systems(
                Update,
                (
                    proximity::detect_vehicle_proximity,
                    input::handle_interaction_input,
                    vehicle::handle_vehicle_enter,
                    vehicle::handle_vehicle_exit,
                    camera::switch_camera_rigs,
                    camera::update_vehicle_camera,
                    camera::update_character_camera,
                    ui::update_interaction_prompts,
                    ui::cleanup_interaction_prompts,
                )
                    .chain(),
            );
    }
}

/// Resources module
pub mod resources {
    use bevy::prelude::*;

    /// Interaction state resource
    #[derive(Resource, Default, Debug, Reflect)]
    #[reflect(Resource)]
    pub struct InteractionState {
        /// Player's current state
        pub player_state: crate::interaction::components::PlayerState,
        /// Currently targeted vehicle entity
        pub target_vehicle: Option<Entity>,
        /// Distance to target vehicle
        pub target_distance: f32,
        /// Whether interaction prompt is visible
        pub prompt_visible: bool,
    }
}

/// Events module
pub mod events {
    use bevy::prelude::*;

    /// Event fired when player enters a vehicle
    #[derive(Event, Debug, Clone)]
    pub struct VehicleEnterEvent {
        pub player_entity: Entity,
        pub vehicle_entity: Entity,
    }

    /// Event fired when player exits a vehicle
    #[derive(Event, Debug, Clone)]
    pub struct VehicleExitEvent {
        pub player_entity: Entity,
        pub vehicle_entity: Entity,
        pub exit_position: Vec3,
    }

    /// Event fired when interaction prompt should be shown/hidden
    #[derive(Event, Debug, Clone)]
    pub struct InteractionPromptEvent {
        pub visible: bool,
        pub prompt_text: String,
        pub target_entity: Entity,
    }
}
