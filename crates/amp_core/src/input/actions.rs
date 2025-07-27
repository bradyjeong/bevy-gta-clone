//! Input action definitions and abstractions
//!
//! Provides high-level action enums that represent gameplay intentions
//! rather than raw key codes.

use serde::{Deserialize, Serialize};

/// High-level input actions that represent player intentions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputAction {
    // Character Movement
    MoveForward,
    MoveBackward,
    TurnLeft,
    TurnRight,
    Sprint,
    Jump,
    Crouch,

    // Vehicle Controls
    Accelerate,
    Brake,
    SteerLeft,
    SteerRight,
    Handbrake,
    Turbo,

    // Aircraft Controls (Helicopter/F16)
    PitchUp,
    PitchDown,
    RollLeft,
    RollRight,
    YawLeft,
    YawRight,
    VerticalUp,
    VerticalDown,
    Afterburner,

    // Interaction
    Interact,
    EnterVehicle,
    ExitVehicle,

    // UI/Meta
    ToggleDebugInfo,
    TogglePhysicsDebug,
    EmergencyReset,
    SaveGame,
    LoadGame,

    // Camera
    CameraToggle,
    CameraZoomIn,
    CameraZoomOut,

    // Context-specific actions
    ContextPrimary,
    ContextSecondary,
    ContextTertiary,
}

impl InputAction {
    /// Get a human-readable description of the action
    pub fn description(&self) -> &'static str {
        match self {
            Self::MoveForward => "Move Forward",
            Self::MoveBackward => "Move Backward",
            Self::TurnLeft => "Turn Left",
            Self::TurnRight => "Turn Right",
            Self::Sprint => "Sprint/Run",
            Self::Jump => "Jump",
            Self::Crouch => "Crouch",

            Self::Accelerate => "Accelerate",
            Self::Brake => "Brake/Reverse",
            Self::SteerLeft => "Steer Left",
            Self::SteerRight => "Steer Right",
            Self::Handbrake => "Handbrake",
            Self::Turbo => "Turbo Boost",

            Self::PitchUp => "Pitch Up",
            Self::PitchDown => "Pitch Down",
            Self::RollLeft => "Roll Left",
            Self::RollRight => "Roll Right",
            Self::YawLeft => "Yaw Left",
            Self::YawRight => "Yaw Right",
            Self::VerticalUp => "Ascend/Climb",
            Self::VerticalDown => "Descend/Dive",
            Self::Afterburner => "Afterburner",

            Self::Interact => "Interact",
            Self::EnterVehicle => "Enter Vehicle",
            Self::ExitVehicle => "Exit Vehicle",

            Self::ToggleDebugInfo => "Toggle Debug Info",
            Self::TogglePhysicsDebug => "Toggle Physics Debug",
            Self::EmergencyReset => "Emergency Reset",
            Self::SaveGame => "Save Game",
            Self::LoadGame => "Load Game",

            Self::CameraToggle => "Toggle Camera Mode",
            Self::CameraZoomIn => "Zoom In",
            Self::CameraZoomOut => "Zoom Out",

            Self::ContextPrimary => "Primary Context Action",
            Self::ContextSecondary => "Secondary Context Action",
            Self::ContextTertiary => "Tertiary Context Action",
        }
    }

    /// Get the category this action belongs to
    pub fn category(&self) -> ActionCategory {
        match self {
            Self::MoveForward
            | Self::MoveBackward
            | Self::TurnLeft
            | Self::TurnRight
            | Self::Sprint
            | Self::Jump
            | Self::Crouch => ActionCategory::Character,

            Self::Accelerate
            | Self::Brake
            | Self::SteerLeft
            | Self::SteerRight
            | Self::Handbrake
            | Self::Turbo => ActionCategory::Vehicle,

            Self::PitchUp
            | Self::PitchDown
            | Self::RollLeft
            | Self::RollRight
            | Self::YawLeft
            | Self::YawRight
            | Self::VerticalUp
            | Self::VerticalDown
            | Self::Afterburner => ActionCategory::Aircraft,

            Self::Interact | Self::EnterVehicle | Self::ExitVehicle => ActionCategory::Interaction,

            Self::ToggleDebugInfo
            | Self::TogglePhysicsDebug
            | Self::EmergencyReset
            | Self::SaveGame
            | Self::LoadGame => ActionCategory::Debug,

            Self::CameraToggle | Self::CameraZoomIn | Self::CameraZoomOut => ActionCategory::Camera,

            Self::ContextPrimary | Self::ContextSecondary | Self::ContextTertiary => {
                ActionCategory::Context
            }
        }
    }

    /// Check if this action is available in the given context
    pub fn is_available_in_context(&self, context: &crate::input::InputContext) -> bool {
        use crate::input::InputContext;

        match context {
            InputContext::Walking => matches!(
                self.category(),
                ActionCategory::Character
                    | ActionCategory::Interaction
                    | ActionCategory::Debug
                    | ActionCategory::Camera
                    | ActionCategory::Context
            ),
            InputContext::Driving => matches!(
                self.category(),
                ActionCategory::Vehicle
                    | ActionCategory::Interaction
                    | ActionCategory::Debug
                    | ActionCategory::Camera
                    | ActionCategory::Context
            ),
            InputContext::Flying => matches!(
                self.category(),
                ActionCategory::Aircraft
                    | ActionCategory::Interaction
                    | ActionCategory::Debug
                    | ActionCategory::Camera
                    | ActionCategory::Context
            ),
            InputContext::Menu => matches!(
                self.category(),
                ActionCategory::Debug | ActionCategory::Context
            ),
        }
    }
}

/// Categories of input actions for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionCategory {
    Character,
    Vehicle,
    Aircraft,
    Interaction,
    Debug,
    Camera,
    Context,
}

impl ActionCategory {
    /// Get a human-readable name for the category
    pub fn name(&self) -> &'static str {
        match self {
            Self::Character => "Character",
            Self::Vehicle => "Vehicle",
            Self::Aircraft => "Aircraft",
            Self::Interaction => "Interaction",
            Self::Debug => "Debug",
            Self::Camera => "Camera",
            Self::Context => "Context",
        }
    }
}

/// Input action state - tracks current status
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ActionState {
    /// Whether the action is currently active/pressed
    pub pressed: bool,
    /// Whether the action was just pressed this frame
    pub just_pressed: bool,
    /// Whether the action was just released this frame  
    pub just_released: bool,
    /// How long the action has been held (in seconds)
    pub held_duration: f32,
    /// Analog value for actions that support it (0.0 to 1.0)
    pub value: f32,
}

impl Default for ActionState {
    fn default() -> Self {
        Self {
            pressed: false,
            just_pressed: false,
            just_released: false,
            held_duration: 0.0,
            value: 0.0,
        }
    }
}

impl ActionState {
    /// Create a new action state with pressed status
    pub fn pressed(value: f32) -> Self {
        Self {
            pressed: true,
            just_pressed: false,
            just_released: false,
            held_duration: 0.0,
            value: value.clamp(0.0, 1.0),
        }
    }

    /// Create a new action state for just pressed
    pub fn just_pressed(value: f32) -> Self {
        Self {
            pressed: true,
            just_pressed: true,
            just_released: false,
            held_duration: 0.0,
            value: value.clamp(0.0, 1.0),
        }
    }

    /// Create a new action state for just released
    pub fn just_released() -> Self {
        Self {
            pressed: false,
            just_pressed: false,
            just_released: true,
            held_duration: 0.0,
            value: 0.0,
        }
    }

    /// Update the state for a new frame
    pub fn update(&mut self, pressed: bool, value: f32, delta_time: f32) {
        let was_pressed = self.pressed;

        self.just_pressed = pressed && !was_pressed;
        self.just_released = !pressed && was_pressed;
        self.pressed = pressed;
        self.value = if pressed { value.clamp(0.0, 1.0) } else { 0.0 };

        if pressed {
            self.held_duration += delta_time;
        } else {
            self.held_duration = 0.0;
        }
    }

    /// Check if this is a binary (on/off) action
    pub fn is_binary(&self) -> bool {
        self.value == 0.0 || self.value == 1.0
    }

    /// Get the action strength (0.0 to 1.0)
    pub fn strength(&self) -> f32 {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::InputContext;

    #[test]
    fn test_action_descriptions() {
        assert_eq!(InputAction::MoveForward.description(), "Move Forward");
        assert_eq!(InputAction::Accelerate.description(), "Accelerate");
        assert_eq!(InputAction::PitchUp.description(), "Pitch Up");
    }

    #[test]
    fn test_action_categories() {
        assert_eq!(
            InputAction::MoveForward.category(),
            ActionCategory::Character
        );
        assert_eq!(InputAction::Accelerate.category(), ActionCategory::Vehicle);
        assert_eq!(InputAction::PitchUp.category(), ActionCategory::Aircraft);
    }

    #[test]
    fn test_context_availability() {
        assert!(InputAction::MoveForward.is_available_in_context(&InputContext::Walking));
        assert!(!InputAction::MoveForward.is_available_in_context(&InputContext::Driving));
        assert!(InputAction::Accelerate.is_available_in_context(&InputContext::Driving));
        assert!(!InputAction::Accelerate.is_available_in_context(&InputContext::Walking));
    }

    #[test]
    fn test_action_state_updates() {
        let mut state = ActionState::default();

        // First press
        state.update(true, 1.0, 0.016);
        assert!(state.pressed);
        assert!(state.just_pressed);
        assert!(!state.just_released);
        assert_eq!(state.value, 1.0);

        // Hold
        state.update(true, 1.0, 0.016);
        assert!(state.pressed);
        assert!(!state.just_pressed);
        assert!(!state.just_released);
        assert!(state.held_duration > 0.0);

        // Release
        state.update(false, 0.0, 0.016);
        assert!(!state.pressed);
        assert!(!state.just_pressed);
        assert!(state.just_released);
        assert_eq!(state.value, 0.0);
        assert_eq!(state.held_duration, 0.0);
    }
}
