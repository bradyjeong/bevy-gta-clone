//! Core input abstractions and traits
//!
//! Defines the fundamental interfaces for input processing and management.

use crate::input::{ActionState, InputAction, InputContext, InputEvent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Trait for input devices that can provide input data
pub trait InputDevice: Send + Sync {
    /// Update the device state for the current frame
    fn update(&mut self, delta_time: f32);

    /// Get the current state of a specific action
    fn get_action_state(&self, action: InputAction) -> ActionState;

    /// Check if the device is connected/available
    fn is_connected(&self) -> bool;

    /// Get device identifier for debugging
    fn device_id(&self) -> String;

    /// Get device type (keyboard, gamepad, etc.)
    fn device_type(&self) -> InputDeviceType;

    /// Get all currently active actions
    fn get_active_actions(&self) -> Vec<InputAction>;
}

/// Types of input devices
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputDeviceType {
    Keyboard,
    Mouse,
    Gamepad,
    TouchScreen,
    VirtualDevice, // For AI/automated input
}

impl InputDeviceType {
    /// Get a human-readable name for the device type
    pub fn name(&self) -> &'static str {
        match self {
            Self::Keyboard => "Keyboard",
            Self::Mouse => "Mouse",
            Self::Gamepad => "Gamepad",
            Self::TouchScreen => "Touch Screen",
            Self::VirtualDevice => "Virtual Device",
        }
    }

    /// Check if this device type supports analog input
    pub fn supports_analog(&self) -> bool {
        matches!(self, Self::Gamepad | Self::TouchScreen)
    }

    /// Get the typical input range for this device
    pub fn input_range(&self) -> (f32, f32) {
        match self {
            Self::Keyboard | Self::Mouse => (0.0, 1.0), // Binary input
            Self::Gamepad | Self::TouchScreen => (-1.0, 1.0), // Analog input
            Self::VirtualDevice => (-1.0, 1.0),         // Configurable
        }
    }
}

/// Input mapping that translates raw device input to actions
pub trait InputMapping: Send + Sync {
    /// Map a raw input to an action in the given context
    fn map_input(&self, device_input: &RawInput, context: InputContext) -> Option<InputAction>;

    /// Get all possible actions for this mapping
    fn get_actions(&self) -> Vec<InputAction>;

    /// Check if a mapping exists for the given input and context
    fn has_mapping(&self, device_input: &RawInput, context: InputContext) -> bool;

    /// Update/modify a mapping
    fn update_mapping(
        &mut self,
        action: InputAction,
        context: InputContext,
        device_input: RawInput,
    ) -> Result<(), String>;

    /// Remove a mapping
    fn remove_mapping(&mut self, action: InputAction, context: InputContext) -> bool;

    /// Get the raw input mapped to an action in a context
    fn get_mapping(&self, action: InputAction, context: InputContext) -> Option<RawInput>;
}

/// Raw input data from a device
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawInput {
    /// Type of device this input came from
    pub device_type: InputDeviceType,
    /// Device-specific input identifier
    pub input_id: String,
    /// Current value (0.0-1.0 for binary, -1.0-1.0 for analog)
    pub value: f32,
    /// Whether this input was just pressed this frame
    pub just_pressed: bool,
    /// Whether this input was just released this frame
    pub just_released: bool,
    /// How long this input has been held (seconds)
    pub held_duration: f32,
}

impl RawInput {
    /// Create a new raw input
    pub fn new(device_type: InputDeviceType, input_id: String, value: f32) -> Self {
        Self {
            device_type,
            input_id,
            value,
            just_pressed: false,
            just_released: false,
            held_duration: 0.0,
        }
    }

    /// Create a keyboard key input
    pub fn keyboard_key(key: &str, pressed: bool) -> Self {
        Self {
            device_type: InputDeviceType::Keyboard,
            input_id: key.to_string(),
            value: if pressed { 1.0 } else { 0.0 },
            just_pressed: false,
            just_released: false,
            held_duration: 0.0,
        }
    }

    /// Create a gamepad axis input
    pub fn gamepad_axis(axis: &str, value: f32) -> Self {
        Self {
            device_type: InputDeviceType::Gamepad,
            input_id: axis.to_string(),
            value: value.clamp(-1.0, 1.0),
            just_pressed: false,
            just_released: false,
            held_duration: 0.0,
        }
    }

    /// Check if this is a binary input (on/off)
    pub fn is_binary(&self) -> bool {
        self.value == 0.0 || self.value == 1.0
    }

    /// Check if this input is currently active
    pub fn is_active(&self) -> bool {
        self.value.abs() > 0.01 // Small threshold for analog noise
    }
}

/// Input processor that converts raw input to high-level events
pub trait InputProcessor: Send + Sync {
    /// Process raw input and generate events
    fn process_input(
        &mut self,
        raw_inputs: &[RawInput],
        context: InputContext,
        delta_time: f32,
    ) -> Vec<InputEvent>;

    /// Get the current state of all actions
    fn get_action_states(&self) -> HashMap<InputAction, ActionState>;

    /// Reset the processor state
    fn reset(&mut self);

    /// Get processor performance metrics
    fn get_metrics(&self) -> ProcessorMetrics;
}

/// Performance metrics for input processing
#[derive(Debug, Clone, Default)]
pub struct ProcessorMetrics {
    /// Time spent processing input this frame (microseconds)
    pub process_time_us: u64,
    /// Number of raw inputs processed this frame
    pub inputs_processed: u32,
    /// Number of events generated this frame
    pub events_generated: u32,
    /// Number of actions updated this frame
    pub actions_updated: u32,
    /// Maximum processing time seen (microseconds)
    pub max_process_time_us: u64,
}

impl ProcessorMetrics {
    /// Update metrics with new processing data
    pub fn update(&mut self, process_time_us: u64, inputs: u32, events: u32, actions: u32) {
        self.process_time_us = process_time_us;
        self.inputs_processed = inputs;
        self.events_generated = events;
        self.actions_updated = actions;
        self.max_process_time_us = self.max_process_time_us.max(process_time_us);
    }

    /// Check if performance is acceptable
    pub fn is_performance_ok(&self) -> bool {
        const MAX_PROCESS_TIME_US: u64 = 1000; // 1ms
        self.process_time_us <= MAX_PROCESS_TIME_US
    }
}

/// Configuration for input processing behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    /// Whether to enable input smoothing for analog inputs
    pub enable_smoothing: bool,
    /// Smoothing factor (0.0 = no smoothing, 1.0 = maximum smoothing)
    pub smoothing_factor: f32,
    /// Dead zone for analog inputs (0.0 to 1.0)
    pub analog_deadzone: f32,
    /// Whether to enable input prediction
    pub enable_prediction: bool,
    /// Maximum time to predict input ahead (seconds)
    pub prediction_time: f32,
    /// Whether to generate events for inactive actions
    pub generate_inactive_events: bool,
    /// Performance monitoring settings
    pub performance_monitoring: bool,
    /// Debug logging enabled
    pub debug_logging: bool,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            enable_smoothing: true,
            smoothing_factor: 0.1,
            analog_deadzone: 0.1,
            enable_prediction: false,
            prediction_time: 0.033, // One frame at 30fps
            generate_inactive_events: false,
            performance_monitoring: true,
            debug_logging: false,
        }
    }
}

impl InputConfig {
    /// Create a config optimized for performance
    pub fn performance_optimized() -> Self {
        Self {
            enable_smoothing: false,
            smoothing_factor: 0.0,
            analog_deadzone: 0.05,
            enable_prediction: false,
            prediction_time: 0.0,
            generate_inactive_events: false,
            performance_monitoring: true,
            debug_logging: false,
        }
    }

    /// Create a config optimized for precision
    pub fn precision_optimized() -> Self {
        Self {
            enable_smoothing: true,
            smoothing_factor: 0.05,
            analog_deadzone: 0.02,
            enable_prediction: true,
            prediction_time: 0.016, // One frame at 60fps
            generate_inactive_events: true,
            performance_monitoring: true,
            debug_logging: false,
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.smoothing_factor < 0.0 || self.smoothing_factor > 1.0 {
            return Err("Smoothing factor must be between 0.0 and 1.0".to_string());
        }

        if self.analog_deadzone < 0.0 || self.analog_deadzone > 1.0 {
            return Err("Analog deadzone must be between 0.0 and 1.0".to_string());
        }

        if self.prediction_time < 0.0 {
            return Err("Prediction time cannot be negative".to_string());
        }

        Ok(())
    }
}

/// Utility functions for input processing
pub mod utils {
    use super::*;

    /// Apply smoothing to an analog input value
    pub fn smooth_input(current: f32, target: f32, smoothing: f32, delta_time: f32) -> f32 {
        if smoothing <= 0.0 {
            return target;
        }

        let smooth_factor = 1.0 - (smoothing * delta_time * 60.0).min(1.0);
        current * smooth_factor + target * (1.0 - smooth_factor)
    }

    /// Apply deadzone to an analog input
    pub fn apply_deadzone(value: f32, deadzone: f32) -> f32 {
        let abs_value = value.abs();
        if abs_value < deadzone {
            0.0
        } else {
            let sign = value.signum();
            let normalized = (abs_value - deadzone) / (1.0 - deadzone);
            sign * normalized.clamp(0.0, 1.0)
        }
    }

    /// Convert raw input to action state
    pub fn raw_to_action_state(
        raw: &RawInput,
        previous: &ActionState,
        delta_time: f32,
    ) -> ActionState {
        let mut state = ActionState::default();

        let is_pressed = raw.is_active();
        let was_pressed = previous.pressed;

        state.pressed = is_pressed;
        state.just_pressed = is_pressed && !was_pressed;
        state.just_released = !is_pressed && was_pressed;
        state.value = raw.value.abs();

        if is_pressed {
            state.held_duration = previous.held_duration + delta_time;
        } else {
            state.held_duration = 0.0;
        }

        state
    }

    /// Combine multiple action states (for multiple bindings to same action)
    pub fn combine_action_states(states: &[ActionState]) -> ActionState {
        if states.is_empty() {
            return ActionState::default();
        }

        ActionState {
            pressed: states.iter().any(|s| s.pressed),
            just_pressed: states.iter().any(|s| s.just_pressed),
            just_released: states.iter().all(|s| !s.pressed)
                && states.iter().any(|s| s.just_released),
            value: states.iter().map(|s| s.value).fold(0.0, f32::max),
            held_duration: states.iter().map(|s| s.held_duration).fold(0.0, f32::max),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::utils::*;
    use super::*;

    #[test]
    fn test_device_type_properties() {
        assert!(InputDeviceType::Gamepad.supports_analog());
        assert!(!InputDeviceType::Keyboard.supports_analog());
        assert_eq!(InputDeviceType::Keyboard.input_range(), (0.0, 1.0));
        assert_eq!(InputDeviceType::Gamepad.input_range(), (-1.0, 1.0));
    }

    #[test]
    fn test_raw_input_creation() {
        let keyboard_input = RawInput::keyboard_key("Space", true);
        assert_eq!(keyboard_input.device_type, InputDeviceType::Keyboard);
        assert_eq!(keyboard_input.input_id, "Space");
        assert_eq!(keyboard_input.value, 1.0);
        assert!(keyboard_input.is_binary());
        assert!(keyboard_input.is_active());

        let gamepad_input = RawInput::gamepad_axis("LeftStickX", 0.5);
        assert_eq!(gamepad_input.device_type, InputDeviceType::Gamepad);
        assert!(!gamepad_input.is_binary());
        assert!(gamepad_input.is_active());
    }

    #[test]
    fn test_input_config_validation() {
        let mut config = InputConfig::default();
        assert!(config.validate().is_ok());

        config.smoothing_factor = -0.1;
        assert!(config.validate().is_err());

        config.smoothing_factor = 0.5;
        config.analog_deadzone = 1.5;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_input_smoothing() {
        let current = 0.2;
        let target = 0.8;
        let smoothing = 0.5;
        let delta_time = 0.016; // 60fps

        let smoothed = smooth_input(current, target, smoothing, delta_time);
        assert!(smoothed > current && smoothed < target);
    }

    #[test]
    fn test_deadzone_application() {
        assert_eq!(apply_deadzone(0.05, 0.1), 0.0); // Below deadzone
        assert!(apply_deadzone(0.15, 0.1) > 0.0); // Above deadzone
        assert!(apply_deadzone(-0.15, 0.1) < 0.0); // Negative above deadzone
    }

    #[test]
    fn test_action_state_combination() {
        let state1 = ActionState {
            pressed: true,
            just_pressed: false,
            just_released: false,
            held_duration: 1.0,
            value: 0.5,
        };

        let state2 = ActionState {
            pressed: false,
            just_pressed: false,
            just_released: true,
            held_duration: 0.0,
            value: 0.0,
        };

        let combined = combine_action_states(&[state1, state2]);
        assert!(combined.pressed); // At least one is pressed
        assert!(combined.just_released); // One was just released
        assert_eq!(combined.value, 0.5); // Maximum value
        assert_eq!(combined.held_duration, 1.0); // Maximum held duration
    }
}
