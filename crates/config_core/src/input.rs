//! Input configuration for hot-reloadable key bindings
//!
//! Provides configuration structures and hot-reload capabilities for
//! the advanced input system.

#[cfg(feature = "unstable_advanced_input")]
use amp_core::input::{InputAction, InputContext};
#[cfg(feature = "unstable_advanced_input")]
use ron::ser::{to_string_pretty, PrettyConfig};
#[cfg(feature = "unstable_advanced_input")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "unstable_advanced_input")]
use std::collections::HashMap;

/// Configuration for input bindings that can be hot-reloaded
#[cfg(feature = "unstable_advanced_input")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputBindingConfig {
    /// Version of the configuration format
    pub version: String,
    /// Context-specific key bindings
    pub bindings: HashMap<InputContext, ContextBindings>,
    /// Global input settings
    pub settings: InputSettings,
    /// Performance tuning options
    pub performance: PerformanceSettings,
}

#[cfg(feature = "unstable_advanced_input")]
impl Default for InputBindingConfig {
    fn default() -> Self {
        let mut bindings = HashMap::new();

        // Setup default bindings for each context
        bindings.insert(InputContext::Walking, ContextBindings::default_walking());
        bindings.insert(InputContext::Driving, ContextBindings::default_driving());
        bindings.insert(InputContext::Flying, ContextBindings::default_flying());
        bindings.insert(InputContext::Menu, ContextBindings::default_menu());

        Self {
            version: "1.0".to_string(),
            bindings,
            settings: InputSettings::default(),
            performance: PerformanceSettings::default(),
        }
    }
}

/// Bindings for a specific input context
#[cfg(feature = "unstable_advanced_input")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBindings {
    /// Primary key bindings (action -> key)
    pub primary: HashMap<InputAction, String>,
    /// Alternative key bindings (action -> key)
    pub alternative: HashMap<InputAction, String>,
    /// Gamepad bindings (action -> button/axis)
    pub gamepad: HashMap<InputAction, String>,
    /// Mouse bindings (action -> button/axis)
    pub mouse: HashMap<InputAction, String>,
}

#[cfg(feature = "unstable_advanced_input")]
impl ContextBindings {
    /// Create default walking bindings
    pub fn default_walking() -> Self {
        let mut primary = HashMap::new();
        let mut alternative = HashMap::new();

        // WASD movement
        primary.insert(InputAction::MoveForward, "KeyW".to_string());
        primary.insert(InputAction::MoveBackward, "KeyS".to_string());
        primary.insert(InputAction::TurnLeft, "KeyA".to_string());
        primary.insert(InputAction::TurnRight, "KeyD".to_string());

        // Arrow key alternatives
        alternative.insert(InputAction::MoveForward, "ArrowUp".to_string());
        alternative.insert(InputAction::MoveBackward, "ArrowDown".to_string());
        alternative.insert(InputAction::TurnLeft, "ArrowLeft".to_string());
        alternative.insert(InputAction::TurnRight, "ArrowRight".to_string());

        // Character actions
        primary.insert(InputAction::Sprint, "ShiftLeft".to_string());
        primary.insert(InputAction::Jump, "Space".to_string());
        primary.insert(InputAction::Crouch, "ControlLeft".to_string());
        primary.insert(InputAction::Interact, "KeyF".to_string());
        primary.insert(InputAction::EnterVehicle, "KeyE".to_string());

        // Debug actions
        primary.insert(InputAction::ToggleDebugInfo, "F1".to_string());
        primary.insert(InputAction::TogglePhysicsDebug, "F3".to_string());
        primary.insert(InputAction::EmergencyReset, "F2".to_string());
        primary.insert(InputAction::SaveGame, "F5".to_string());
        primary.insert(InputAction::LoadGame, "F9".to_string());

        Self {
            primary,
            alternative,
            gamepad: HashMap::new(),
            mouse: HashMap::new(),
        }
    }

    /// Create default driving bindings
    pub fn default_driving() -> Self {
        let mut primary = HashMap::new();
        let mut alternative = HashMap::new();

        // WASD vehicle controls
        primary.insert(InputAction::Accelerate, "KeyW".to_string());
        primary.insert(InputAction::Brake, "KeyS".to_string());
        primary.insert(InputAction::SteerLeft, "KeyA".to_string());
        primary.insert(InputAction::SteerRight, "KeyD".to_string());

        // Arrow key alternatives
        alternative.insert(InputAction::Accelerate, "ArrowUp".to_string());
        alternative.insert(InputAction::Brake, "ArrowDown".to_string());
        alternative.insert(InputAction::SteerLeft, "ArrowLeft".to_string());
        alternative.insert(InputAction::SteerRight, "ArrowRight".to_string());

        // Vehicle-specific actions
        primary.insert(InputAction::Handbrake, "Space".to_string());
        primary.insert(InputAction::Turbo, "ShiftLeft".to_string());
        primary.insert(InputAction::ExitVehicle, "KeyF".to_string());

        // Debug actions (same as walking)
        primary.insert(InputAction::ToggleDebugInfo, "F1".to_string());
        primary.insert(InputAction::TogglePhysicsDebug, "F3".to_string());
        primary.insert(InputAction::EmergencyReset, "F2".to_string());

        Self {
            primary,
            alternative,
            gamepad: HashMap::new(),
            mouse: HashMap::new(),
        }
    }

    /// Create default flying bindings
    pub fn default_flying() -> Self {
        let mut primary = HashMap::new();
        let mut alternative = HashMap::new();

        // Flight controls
        primary.insert(InputAction::PitchUp, "KeyW".to_string());
        primary.insert(InputAction::PitchDown, "KeyS".to_string());
        primary.insert(InputAction::RollLeft, "KeyA".to_string());
        primary.insert(InputAction::RollRight, "KeyD".to_string());
        primary.insert(InputAction::YawLeft, "KeyQ".to_string());
        primary.insert(InputAction::YawRight, "KeyE".to_string());

        // Arrow key alternatives for basic movement
        alternative.insert(InputAction::PitchUp, "ArrowUp".to_string());
        alternative.insert(InputAction::PitchDown, "ArrowDown".to_string());
        alternative.insert(InputAction::RollLeft, "ArrowLeft".to_string());
        alternative.insert(InputAction::RollRight, "ArrowRight".to_string());

        // Vertical movement
        primary.insert(InputAction::VerticalUp, "ShiftLeft".to_string());
        primary.insert(InputAction::VerticalDown, "ControlLeft".to_string());
        primary.insert(InputAction::Afterburner, "Space".to_string());
        primary.insert(InputAction::ExitVehicle, "KeyF".to_string());

        // Debug actions
        primary.insert(InputAction::ToggleDebugInfo, "F1".to_string());
        primary.insert(InputAction::TogglePhysicsDebug, "F3".to_string());
        primary.insert(InputAction::EmergencyReset, "F2".to_string());

        Self {
            primary,
            alternative,
            gamepad: HashMap::new(),
            mouse: HashMap::new(),
        }
    }

    /// Create default menu bindings
    pub fn default_menu() -> Self {
        let mut primary = HashMap::new();

        // Context actions for menu navigation
        primary.insert(InputAction::ContextPrimary, "Enter".to_string());
        primary.insert(InputAction::ContextSecondary, "Escape".to_string());
        primary.insert(InputAction::ContextTertiary, "Tab".to_string());

        // Arrow key navigation
        primary.insert(InputAction::MoveForward, "ArrowUp".to_string());
        primary.insert(InputAction::MoveBackward, "ArrowDown".to_string());
        primary.insert(InputAction::TurnLeft, "ArrowLeft".to_string());
        primary.insert(InputAction::TurnRight, "ArrowRight".to_string());

        Self {
            primary,
            alternative: HashMap::new(),
            gamepad: HashMap::new(),
            mouse: HashMap::new(),
        }
    }

    /// Get the primary binding for an action
    pub fn get_primary_binding(&self, action: InputAction) -> Option<&String> {
        self.primary.get(&action)
    }

    /// Get the alternative binding for an action
    pub fn get_alternative_binding(&self, action: InputAction) -> Option<&String> {
        self.alternative.get(&action)
    }

    /// Get all bindings for an action (primary, alternative, gamepad, mouse)
    pub fn get_all_bindings(&self, action: InputAction) -> Vec<&String> {
        let mut bindings = Vec::new();

        if let Some(primary) = self.primary.get(&action) {
            bindings.push(primary);
        }
        if let Some(alt) = self.alternative.get(&action) {
            bindings.push(alt);
        }
        if let Some(gamepad) = self.gamepad.get(&action) {
            bindings.push(gamepad);
        }
        if let Some(mouse) = self.mouse.get(&action) {
            bindings.push(mouse);
        }

        bindings
    }

    /// Set primary binding for an action
    pub fn set_primary_binding(&mut self, action: InputAction, binding: String) {
        self.primary.insert(action, binding);
    }

    /// Remove all bindings for an action
    pub fn remove_action(&mut self, action: InputAction) {
        self.primary.remove(&action);
        self.alternative.remove(&action);
        self.gamepad.remove(&action);
        self.mouse.remove(&action);
    }
}

/// Global input settings
#[cfg(feature = "unstable_advanced_input")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSettings {
    /// Enable input smoothing for analog inputs
    pub enable_smoothing: bool,
    /// Smoothing factor (0.0 to 1.0)
    pub smoothing_factor: f32,
    /// Dead zone for analog inputs (0.0 to 1.0)
    pub analog_deadzone: f32,
    /// Mouse sensitivity
    pub mouse_sensitivity: f32,
    /// Invert mouse Y axis
    pub invert_mouse_y: bool,
    /// Enable gamepad vibration
    pub gamepad_vibration: bool,
    /// Double-click time threshold (ms)
    pub double_click_time_ms: u32,
    /// Key repeat delay (ms)
    pub key_repeat_delay_ms: u32,
    /// Key repeat rate (ms)
    pub key_repeat_rate_ms: u32,
}

#[cfg(feature = "unstable_advanced_input")]
impl Default for InputSettings {
    fn default() -> Self {
        Self {
            enable_smoothing: true,
            smoothing_factor: 0.1,
            analog_deadzone: 0.1,
            mouse_sensitivity: 1.0,
            invert_mouse_y: false,
            gamepad_vibration: true,
            double_click_time_ms: 500,
            key_repeat_delay_ms: 500,
            key_repeat_rate_ms: 50,
        }
    }
}

/// Performance tuning settings
#[cfg(feature = "unstable_advanced_input")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    /// Enable performance monitoring
    pub enable_monitoring: bool,
    /// Maximum processing time per frame (microseconds)
    pub max_process_time_us: u64,
    /// Maximum events to process per frame
    pub max_events_per_frame: u32,
    /// Enable input prediction
    pub enable_prediction: bool,
    /// Prediction lookahead time (seconds)
    pub prediction_time_s: f32,
    /// Enable input batching
    pub enable_batching: bool,
    /// Batch size for event processing
    pub batch_size: u32,
}

#[cfg(feature = "unstable_advanced_input")]
impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            enable_monitoring: true,
            max_process_time_us: 1000, // 1ms
            max_events_per_frame: 100,
            enable_prediction: false,
            prediction_time_s: 0.033, // One frame at 30fps
            enable_batching: true,
            batch_size: 32,
        }
    }
}

#[cfg(feature = "unstable_advanced_input")]
impl InputBindingConfig {
    /// Get bindings for a specific context
    pub fn get_context_bindings(&self, context: InputContext) -> Option<&ContextBindings> {
        self.bindings.get(&context)
    }

    /// Get mutable bindings for a specific context
    pub fn get_context_bindings_mut(
        &mut self,
        context: InputContext,
    ) -> Option<&mut ContextBindings> {
        self.bindings.get_mut(&context)
    }

    /// Set bindings for a context
    pub fn set_context_bindings(&mut self, context: InputContext, bindings: ContextBindings) {
        self.bindings.insert(context, bindings);
    }

    /// Find which context(s) have a binding for the given key
    pub fn find_contexts_for_key(&self, key: &str) -> Vec<(InputContext, InputAction)> {
        let mut results = Vec::new();

        for (context, bindings) in &self.bindings {
            for (action, binding) in &bindings.primary {
                if binding == key {
                    results.push((*context, *action));
                }
            }
            for (action, binding) in &bindings.alternative {
                if binding == key {
                    results.push((*context, *action));
                }
            }
        }

        results
    }

    /// Validate the configuration for conflicts and issues
    pub fn validate(&self) -> Result<(), String> {
        // Check for conflicts within each context
        for (context, bindings) in &self.bindings {
            let mut used_keys = HashMap::new();

            // Check primary bindings
            for (action, key) in &bindings.primary {
                if let Some(existing_action) = used_keys.get(key) {
                    return Err(format!(
                        "Key conflict in {:?}: '{}' is bound to both {:?} and {:?}",
                        context, key, existing_action, action
                    ));
                }
                used_keys.insert(key.clone(), *action);
            }

            // Check alternative bindings (against primary and each other)
            for (action, key) in &bindings.alternative {
                if let Some(existing_action) = used_keys.get(key) {
                    return Err(format!(
                        "Key conflict in {:?}: '{}' is bound to both {:?} and {:?}",
                        context, key, existing_action, action
                    ));
                }
                used_keys.insert(key.clone(), *action);
            }
        }

        // Validate settings
        if self.settings.smoothing_factor < 0.0 || self.settings.smoothing_factor > 1.0 {
            return Err("Smoothing factor must be between 0.0 and 1.0".to_string());
        }

        if self.settings.analog_deadzone < 0.0 || self.settings.analog_deadzone > 1.0 {
            return Err("Analog deadzone must be between 0.0 and 1.0".to_string());
        }

        if self.settings.mouse_sensitivity <= 0.0 {
            return Err("Mouse sensitivity must be positive".to_string());
        }

        Ok(())
    }

    /// Export to RON format for hot-reloading
    pub fn to_ron(&self) -> Result<String, String> {
        ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(|e| format!("Failed to serialize config to RON: {}", e))
    }

    /// Import from RON format
    pub fn from_ron(ron_str: &str) -> Result<Self, String> {
        ron::from_str(ron_str).map_err(|e| format!("Failed to deserialize config from RON: {}", e))
    }

    /// Load from file
    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        Self::from_ron(&content)
    }

    /// Save to file
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), String> {
        let ron_content = self.to_ron()?;
        std::fs::write(path, ron_content).map_err(|e| format!("Failed to write config file: {}", e))
    }

    /// Reset to default configuration
    pub fn reset_to_defaults(&mut self) {
        *self = Self::default();
    }

    /// Create a performance-optimized configuration
    pub fn performance_optimized() -> Self {
        let mut config = Self::default();
        config.settings.enable_smoothing = false;
        config.settings.smoothing_factor = 0.0;
        config.settings.analog_deadzone = 0.05;
        config.performance.enable_prediction = false;
        config.performance.max_process_time_us = 500; // 0.5ms
        config.performance.enable_batching = true;
        config.performance.batch_size = 64;
        config
    }

    /// Create a precision-optimized configuration
    pub fn precision_optimized() -> Self {
        let mut config = Self::default();
        config.settings.enable_smoothing = true;
        config.settings.smoothing_factor = 0.05;
        config.settings.analog_deadzone = 0.02;
        config.performance.enable_prediction = true;
        config.performance.prediction_time_s = 0.016; // 60fps
        config.performance.enable_batching = false;
        config
    }
}

#[cfg(all(test, feature = "unstable_advanced_input"))]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_creation() {
        let config = InputBindingConfig::default();
        assert_eq!(config.version, "1.0");
        assert!(config.bindings.contains_key(&InputContext::Walking));
        assert!(config.bindings.contains_key(&InputContext::Driving));
        assert!(config.bindings.contains_key(&InputContext::Flying));
        assert!(config.bindings.contains_key(&InputContext::Menu));
    }

    #[test]
    fn test_walking_bindings() {
        let bindings = ContextBindings::default_walking();
        assert_eq!(
            bindings.get_primary_binding(InputAction::MoveForward),
            Some(&"KeyW".to_string())
        );
        assert_eq!(
            bindings.get_alternative_binding(InputAction::MoveForward),
            Some(&"ArrowUp".to_string())
        );
        assert_eq!(
            bindings.get_primary_binding(InputAction::Sprint),
            Some(&"ShiftLeft".to_string())
        );
    }

    #[test]
    fn test_driving_bindings() {
        let bindings = ContextBindings::default_driving();
        assert_eq!(
            bindings.get_primary_binding(InputAction::Accelerate),
            Some(&"KeyW".to_string())
        );
        assert_eq!(
            bindings.get_primary_binding(InputAction::Handbrake),
            Some(&"Space".to_string())
        );
    }

    #[test]
    fn test_flying_bindings() {
        let bindings = ContextBindings::default_flying();
        assert_eq!(
            bindings.get_primary_binding(InputAction::PitchUp),
            Some(&"KeyW".to_string())
        );
        assert_eq!(
            bindings.get_primary_binding(InputAction::YawLeft),
            Some(&"KeyQ".to_string())
        );
        assert_eq!(
            bindings.get_primary_binding(InputAction::Afterburner),
            Some(&"Space".to_string())
        );
    }

    #[test]
    fn test_config_validation() {
        let config = InputBindingConfig::default();
        assert!(config.validate().is_ok());

        let mut invalid_config = config.clone();
        invalid_config.settings.smoothing_factor = -0.1;
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_binding_modification() {
        let mut bindings = ContextBindings::default_walking();
        bindings.set_primary_binding(InputAction::MoveForward, "KeyR".to_string());
        assert_eq!(
            bindings.get_primary_binding(InputAction::MoveForward),
            Some(&"KeyR".to_string())
        );

        bindings.remove_action(InputAction::MoveForward);
        assert_eq!(bindings.get_primary_binding(InputAction::MoveForward), None);
    }

    #[test]
    fn test_ron_serialization() {
        let config = InputBindingConfig::default();
        let ron_str = config.to_ron().unwrap();
        let deserialized = InputBindingConfig::from_ron(&ron_str).unwrap();

        // Compare a few key fields to ensure round-trip works
        assert_eq!(config.version, deserialized.version);
        assert_eq!(config.bindings.len(), deserialized.bindings.len());
    }

    #[test]
    fn test_performance_config() {
        let perf_config = InputBindingConfig::performance_optimized();
        assert!(!perf_config.settings.enable_smoothing);
        assert_eq!(perf_config.settings.smoothing_factor, 0.0);
        assert_eq!(perf_config.performance.max_process_time_us, 500);
    }

    #[test]
    fn test_precision_config() {
        let precision_config = InputBindingConfig::precision_optimized();
        assert!(precision_config.settings.enable_smoothing);
        assert_eq!(precision_config.settings.analog_deadzone, 0.02);
        assert!(precision_config.performance.enable_prediction);
    }

    #[test]
    fn test_get_all_bindings() {
        let mut bindings = ContextBindings::default_walking();
        bindings
            .gamepad
            .insert(InputAction::MoveForward, "LeftStickUp".to_string());

        let all_bindings = bindings.get_all_bindings(InputAction::MoveForward);
        assert_eq!(all_bindings.len(), 3); // primary, alternative, gamepad
        assert!(all_bindings.contains(&&"KeyW".to_string()));
        assert!(all_bindings.contains(&&"ArrowUp".to_string()));
        assert!(all_bindings.contains(&&"LeftStickUp".to_string()));
    }
}

// Export is handled by the struct definition above
