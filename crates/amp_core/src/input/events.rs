//! Input event system for high-level input notifications
//!
//! Provides events that can be sent between systems to communicate
//! input state changes without tight coupling.

use crate::input::{ActionState, InputAction, InputContext};
use serde::{Deserialize, Serialize};

/// High-level input event that represents an action state change
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputEvent {
    /// The action that triggered this event
    pub action: InputAction,
    /// The context in which this event occurred
    pub context: InputContext,
    /// The new state of the action
    pub state: ActionState,
    /// Timestamp when the event occurred (game time in seconds)
    pub timestamp: f32,
    /// Optional source identifier (for debugging/analytics)
    pub source: Option<String>,
}

impl InputEvent {
    /// Create a new input event
    pub fn new(
        action: InputAction,
        context: InputContext,
        state: ActionState,
        timestamp: f32,
    ) -> Self {
        Self {
            action,
            context,
            state,
            timestamp,
            source: None,
        }
    }

    /// Create a new input event with a source identifier
    pub fn with_source(
        action: InputAction,
        context: InputContext,
        state: ActionState,
        timestamp: f32,
        source: impl Into<String>,
    ) -> Self {
        Self {
            action,
            context,
            state,
            timestamp,
            source: Some(source.into()),
        }
    }

    /// Check if this event represents a press
    pub fn is_pressed(&self) -> bool {
        self.state.just_pressed
    }

    /// Check if this event represents a release
    pub fn is_released(&self) -> bool {
        self.state.just_released
    }

    /// Check if this event represents an active/held state
    pub fn is_active(&self) -> bool {
        self.state.pressed
    }

    /// Get the strength/value of the input (0.0 to 1.0)
    pub fn strength(&self) -> f32 {
        self.state.value
    }

    /// Get how long the action has been held
    pub fn held_duration(&self) -> f32 {
        self.state.held_duration
    }
}

/// Context change event - fired when input context changes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextChangeEvent {
    /// The previous context
    pub from: InputContext,
    /// The new context
    pub to: InputContext,
    /// Timestamp when the change occurred
    pub timestamp: f32,
    /// Reason for the context change
    pub reason: ContextChangeReason,
}

/// Reasons why an input context might change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContextChangeReason {
    /// Player entered a vehicle
    EnteredVehicle,
    /// Player exited a vehicle
    ExitedVehicle,
    /// Menu was opened
    MenuOpened,
    /// Menu was closed
    MenuClosed,
    /// Manual context switch (debug/testing)
    Manual,
    /// Game state change
    GameStateChange,
    /// Emergency reset
    Emergency,
}

impl ContextChangeReason {
    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::EnteredVehicle => "Entered Vehicle",
            Self::ExitedVehicle => "Exited Vehicle",
            Self::MenuOpened => "Menu Opened",
            Self::MenuClosed => "Menu Closed",
            Self::Manual => "Manual Change",
            Self::GameStateChange => "Game State Change",
            Self::Emergency => "Emergency Reset",
        }
    }
}

/// Input configuration change event - fired when bindings are modified
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigChangeEvent {
    /// The action whose binding changed
    pub action: InputAction,
    /// The context in which the binding changed
    pub context: InputContext,
    /// The old key/input (if any)
    pub old_binding: Option<String>,
    /// The new key/input
    pub new_binding: String,
    /// Timestamp when the change occurred
    pub timestamp: f32,
    /// Whether this change requires a restart or hot-reload
    pub hot_reloadable: bool,
}

/// Input system status event - for monitoring and debugging
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputSystemEvent {
    /// Type of system event
    pub event_type: InputSystemEventType,
    /// Timestamp when the event occurred
    pub timestamp: f32,
    /// Optional message with details
    pub message: Option<String>,
    /// Performance metrics (if available)
    pub metrics: Option<InputMetrics>,
}

/// Types of input system events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputSystemEventType {
    /// System initialized successfully
    Initialized,
    /// Configuration reloaded
    ConfigReloaded,
    /// Performance warning (slow processing)
    PerformanceWarning,
    /// Error occurred during processing
    Error,
    /// Emergency reset triggered
    EmergencyReset,
    /// Context stack corrupted/reset
    ContextReset,
}

/// Performance metrics for input system monitoring
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct InputMetrics {
    /// Time spent processing input this frame (microseconds)
    pub process_time_us: u64,
    /// Number of active actions this frame
    pub active_actions: u32,
    /// Number of events generated this frame
    pub events_generated: u32,
    /// Maximum processing time this session (microseconds)
    pub max_process_time_us: u64,
    /// Average processing time over last 60 frames (microseconds)
    pub avg_process_time_us: u64,
}

impl InputMetrics {
    /// Check if performance is within acceptable limits
    pub fn is_performance_good(&self) -> bool {
        // Performance thresholds
        const MAX_PROCESS_TIME_US: u64 = 1000; // 1ms
        const MAX_AVG_PROCESS_TIME_US: u64 = 500; // 0.5ms average

        self.process_time_us <= MAX_PROCESS_TIME_US
            && self.avg_process_time_us <= MAX_AVG_PROCESS_TIME_US
    }

    /// Get performance status as a string
    pub fn performance_status(&self) -> &'static str {
        if self.process_time_us > 2000 {
            "Critical"
        } else if self.process_time_us > 1000 {
            "Warning"
        } else {
            "Good"
        }
    }
}

/// Batch of input events for efficient processing
#[derive(Debug, Clone, Default)]
pub struct InputEventBatch {
    /// Input action events
    pub input_events: Vec<InputEvent>,
    /// Context change events
    pub context_events: Vec<ContextChangeEvent>,
    /// Configuration change events
    pub config_events: Vec<ConfigChangeEvent>,
    /// System events
    pub system_events: Vec<InputSystemEvent>,
}

impl InputEventBatch {
    /// Create a new empty batch
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an input event to the batch
    pub fn add_input_event(&mut self, event: InputEvent) {
        self.input_events.push(event);
    }

    /// Add a context change event to the batch
    pub fn add_context_event(&mut self, event: ContextChangeEvent) {
        self.context_events.push(event);
    }

    /// Add a configuration change event to the batch
    pub fn add_config_event(&mut self, event: ConfigChangeEvent) {
        self.config_events.push(event);
    }

    /// Add a system event to the batch
    pub fn add_system_event(&mut self, event: InputSystemEvent) {
        self.system_events.push(event);
    }

    /// Check if the batch is empty
    pub fn is_empty(&self) -> bool {
        self.input_events.is_empty()
            && self.context_events.is_empty()
            && self.config_events.is_empty()
            && self.system_events.is_empty()
    }

    /// Get total number of events in the batch
    pub fn total_events(&self) -> usize {
        self.input_events.len()
            + self.context_events.len()
            + self.config_events.len()
            + self.system_events.len()
    }

    /// Clear all events from the batch
    pub fn clear(&mut self) {
        self.input_events.clear();
        self.context_events.clear();
        self.config_events.clear();
        self.system_events.clear();
    }

    /// Filter events by timestamp range
    pub fn filter_by_time_range(&self, start: f32, end: f32) -> InputEventBatch {
        let mut filtered = InputEventBatch::new();

        filtered.input_events = self
            .input_events
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .cloned()
            .collect();

        filtered.context_events = self
            .context_events
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .cloned()
            .collect();

        filtered.config_events = self
            .config_events
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .cloned()
            .collect();

        filtered.system_events = self
            .system_events
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .cloned()
            .collect();

        filtered
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{InputAction, InputContext};

    #[test]
    fn test_input_event_creation() {
        let state = ActionState::just_pressed(1.0);
        let event = InputEvent::new(InputAction::MoveForward, InputContext::Walking, state, 1.0);

        assert_eq!(event.action, InputAction::MoveForward);
        assert_eq!(event.context, InputContext::Walking);
        assert!(event.is_pressed());
        assert!(!event.is_released());
        assert!(event.is_active());
        assert_eq!(event.strength(), 1.0);
    }

    #[test]
    fn test_context_change_event() {
        let event = ContextChangeEvent {
            from: InputContext::Walking,
            to: InputContext::Driving,
            timestamp: 1.0,
            reason: ContextChangeReason::EnteredVehicle,
        };

        assert_eq!(event.reason.description(), "Entered Vehicle");
    }

    #[test]
    fn test_input_metrics() {
        let mut metrics = InputMetrics::default();
        assert!(metrics.is_performance_good());
        assert_eq!(metrics.performance_status(), "Good");

        metrics.process_time_us = 1500;
        assert!(!metrics.is_performance_good());
        assert_eq!(metrics.performance_status(), "Warning");

        metrics.process_time_us = 2500;
        assert_eq!(metrics.performance_status(), "Critical");
    }

    #[test]
    fn test_event_batch() {
        let mut batch = InputEventBatch::new();
        assert!(batch.is_empty());

        let input_event = InputEvent::new(
            InputAction::MoveForward,
            InputContext::Walking,
            ActionState::just_pressed(1.0),
            1.0,
        );

        batch.add_input_event(input_event);
        assert!(!batch.is_empty());
        assert_eq!(batch.total_events(), 1);

        batch.clear();
        assert!(batch.is_empty());
    }

    #[test]
    fn test_event_filtering() {
        let mut batch = InputEventBatch::new();

        // Add events at different timestamps
        batch.add_input_event(InputEvent::new(
            InputAction::MoveForward,
            InputContext::Walking,
            ActionState::just_pressed(1.0),
            1.0,
        ));

        batch.add_input_event(InputEvent::new(
            InputAction::Jump,
            InputContext::Walking,
            ActionState::just_pressed(1.0),
            2.5,
        ));

        batch.add_input_event(InputEvent::new(
            InputAction::Sprint,
            InputContext::Walking,
            ActionState::just_pressed(1.0),
            4.0,
        ));

        // Filter events between 2.0 and 3.0
        let filtered = batch.filter_by_time_range(2.0, 3.0);
        assert_eq!(filtered.input_events.len(), 1);
        assert_eq!(filtered.input_events[0].action, InputAction::Jump);
    }
}
