//! Interaction system components
//!
//! Components for vehicle enter/exit interactions and UI prompts.

use bevy::prelude::*;

/// Marker component for vehicles that can be interacted with
#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
pub struct VehicleInteraction {
    /// Interaction radius in meters
    pub radius: f32,
    /// Whether this vehicle is currently occupied
    pub occupied: bool,
    /// Entity of the occupying player (if any)
    pub occupant: Option<Entity>,
}

impl VehicleInteraction {
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            occupied: false,
            occupant: None,
        }
    }
}

/// Component for interaction prompts
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct InteractionPrompt {
    /// Prompt text to display
    pub prompt_text: String,
    /// Whether the prompt is currently visible
    pub visible: bool,
    /// Target entity this prompt is for
    pub target_entity: Entity,
}

impl InteractionPrompt {
    pub fn new(prompt_text: impl Into<String>, target_entity: Entity) -> Self {
        Self {
            prompt_text: prompt_text.into(),
            visible: false,
            target_entity,
        }
    }
}

/// Player state component
#[derive(Component, Default, Debug, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component)]
pub enum PlayerState {
    #[default]
    Walking,
    Driving,
}

/// Camera rig component for vehicle cameras
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct VehicleCameraRig {
    /// Target entity to follow
    pub target_entity: Entity,
    /// Camera distance from vehicle
    pub follow_distance: f32,
    /// Camera height offset
    pub follow_height: f32,
    /// Camera movement damping
    pub follow_damping: f32,
    /// Look ahead distance
    pub look_ahead_distance: f32,
    /// Camera mode
    pub camera_mode: VehicleCameraMode,
}

/// Vehicle camera modes
#[derive(Default, Debug, Reflect, Clone, Copy, PartialEq)]
pub enum VehicleCameraMode {
    #[default]
    ThirdPerson,
    FirstPerson,
}

impl VehicleCameraRig {
    pub fn new(target_entity: Entity) -> Self {
        Self {
            target_entity,
            follow_distance: 8.0,
            follow_height: 4.0,
            follow_damping: 2.0,
            look_ahead_distance: 5.0,
            camera_mode: VehicleCameraMode::ThirdPerson,
        }
    }
}

/// Camera rig component for character cameras
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct CharacterCameraRig {
    /// Target entity to follow
    pub target_entity: Entity,
    /// Camera distance from character
    pub follow_distance: f32,
    /// Camera height offset
    pub follow_height: f32,
    /// Camera movement damping
    pub follow_damping: f32,
    /// Look sensitivity
    pub look_sensitivity: f32,
    /// Camera mode
    pub camera_mode: CharacterCameraMode,
}

/// Character camera modes
#[derive(Default, Debug, Reflect, Clone, Copy, PartialEq)]
pub enum CharacterCameraMode {
    #[default]
    ThirdPerson,
    FirstPerson,
}

impl CharacterCameraRig {
    pub fn new(target_entity: Entity) -> Self {
        Self {
            target_entity,
            follow_distance: 5.0,
            follow_height: 2.0,
            follow_damping: 4.0,
            look_sensitivity: 1.0,
            camera_mode: CharacterCameraMode::ThirdPerson,
        }
    }
}

/// Marker component for entities currently in a vehicle
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct InVehicle {
    /// Entity of the vehicle being occupied
    pub vehicle_entity: Entity,
    /// Seat index in the vehicle
    pub seat_index: u8,
}

impl InVehicle {
    pub fn new(vehicle_entity: Entity, seat_index: u8) -> Self {
        Self {
            vehicle_entity,
            seat_index,
        }
    }
}
