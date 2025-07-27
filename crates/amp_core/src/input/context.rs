//! Input context definitions for context-sensitive input handling
//!
//! Different game states (walking, driving, flying, menu) have different
//! available actions and input interpretations.

use serde::{Deserialize, Serialize};

/// Input contexts that determine which actions are available and how they're interpreted
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputContext {
    /// Player is walking/on foot
    Walking,
    /// Player is driving a car/vehicle
    Driving,
    /// Player is flying (helicopter/aircraft)
    Flying,
    /// Player is in a menu or UI
    Menu,
}

impl InputContext {
    /// Get a human-readable name for the context
    pub fn name(&self) -> &'static str {
        match self {
            Self::Walking => "Walking",
            Self::Driving => "Driving",
            Self::Flying => "Flying",
            Self::Menu => "Menu",
        }
    }

    /// Get the priority of this context (higher = more specific)
    pub fn priority(&self) -> u8 {
        match self {
            Self::Menu => 100, // Highest priority - menu overrides everything
            Self::Flying => 50,
            Self::Driving => 40,
            Self::Walking => 10, // Lowest priority - default context
        }
    }

    /// Check if this context allows simultaneous contexts
    pub fn allows_concurrent(&self, other: &InputContext) -> bool {
        match (self, other) {
            // Menu context is exclusive
            (InputContext::Menu, _) | (_, InputContext::Menu) => false,
            // Walking can be concurrent with camera controls
            (InputContext::Walking, _) | (_, InputContext::Walking) => true,
            // Vehicle contexts are exclusive with each other
            (InputContext::Driving, InputContext::Flying)
            | (InputContext::Flying, InputContext::Driving) => false,
            // Same context is always allowed
            (a, b) if a == b => true,
            _ => false,
        }
    }

    /// Get actions that should be disabled in this context
    pub fn disabled_actions(&self) -> &'static [crate::input::InputAction] {
        use crate::input::InputAction;

        match self {
            Self::Walking => &[
                InputAction::Accelerate,
                InputAction::Brake,
                InputAction::SteerLeft,
                InputAction::SteerRight,
                InputAction::Handbrake,
                InputAction::Turbo,
                InputAction::PitchUp,
                InputAction::PitchDown,
                InputAction::RollLeft,
                InputAction::RollRight,
                InputAction::YawLeft,
                InputAction::YawRight,
                InputAction::VerticalUp,
                InputAction::VerticalDown,
                InputAction::Afterburner,
            ],
            Self::Driving => &[
                InputAction::MoveForward,
                InputAction::MoveBackward,
                InputAction::TurnLeft,
                InputAction::TurnRight,
                InputAction::Sprint,
                InputAction::Jump,
                InputAction::Crouch,
                InputAction::PitchUp,
                InputAction::PitchDown,
                InputAction::RollLeft,
                InputAction::RollRight,
                InputAction::YawLeft,
                InputAction::YawRight,
                InputAction::VerticalUp,
                InputAction::VerticalDown,
                InputAction::Afterburner,
            ],
            Self::Flying => &[
                InputAction::MoveForward,
                InputAction::MoveBackward,
                InputAction::TurnLeft,
                InputAction::TurnRight,
                InputAction::Sprint,
                InputAction::Jump,
                InputAction::Crouch,
                InputAction::Accelerate,
                InputAction::Brake,
                InputAction::SteerLeft,
                InputAction::SteerRight,
                InputAction::Handbrake,
                InputAction::Turbo,
            ],
            Self::Menu => &[
                // Most gameplay actions disabled in menu
                InputAction::MoveForward,
                InputAction::MoveBackward,
                InputAction::TurnLeft,
                InputAction::TurnRight,
                InputAction::Sprint,
                InputAction::Jump,
                InputAction::Crouch,
                InputAction::Accelerate,
                InputAction::Brake,
                InputAction::SteerLeft,
                InputAction::SteerRight,
                InputAction::Handbrake,
                InputAction::Turbo,
                InputAction::PitchUp,
                InputAction::PitchDown,
                InputAction::RollLeft,
                InputAction::RollRight,
                InputAction::YawLeft,
                InputAction::YawRight,
                InputAction::VerticalUp,
                InputAction::VerticalDown,
                InputAction::Afterburner,
            ],
        }
    }

    /// Get the default actions for this context
    pub fn default_actions(&self) -> &'static [crate::input::InputAction] {
        use crate::input::InputAction;

        match self {
            Self::Walking => &[
                InputAction::MoveForward,
                InputAction::MoveBackward,
                InputAction::TurnLeft,
                InputAction::TurnRight,
                InputAction::Sprint,
                InputAction::Jump,
                InputAction::Interact,
                InputAction::EnterVehicle,
            ],
            Self::Driving => &[
                InputAction::Accelerate,
                InputAction::Brake,
                InputAction::SteerLeft,
                InputAction::SteerRight,
                InputAction::Handbrake,
                InputAction::Turbo,
                InputAction::ExitVehicle,
            ],
            Self::Flying => &[
                InputAction::PitchUp,
                InputAction::PitchDown,
                InputAction::RollLeft,
                InputAction::RollRight,
                InputAction::YawLeft,
                InputAction::YawRight,
                InputAction::VerticalUp,
                InputAction::VerticalDown,
                InputAction::Afterburner,
                InputAction::ExitVehicle,
            ],
            Self::Menu => &[
                InputAction::ContextPrimary,
                InputAction::ContextSecondary,
                InputAction::ContextTertiary,
            ],
        }
    }
}

/// Context stack for managing multiple simultaneous contexts
#[derive(Debug, Clone)]
pub struct ContextStack {
    contexts: Vec<InputContext>,
}

impl Default for ContextStack {
    fn default() -> Self {
        Self {
            contexts: vec![InputContext::Walking], // Default to walking
        }
    }
}

impl ContextStack {
    /// Create a new context stack with the given primary context
    pub fn new(primary_context: InputContext) -> Self {
        Self {
            contexts: vec![primary_context],
        }
    }

    /// Push a new context onto the stack
    pub fn push_context(&mut self, context: InputContext) -> Result<(), String> {
        // Check if the new context is compatible with existing ones
        for existing in &self.contexts {
            if !context.allows_concurrent(existing) {
                return Err(format!(
                    "Context {context:?} is not compatible with existing context {existing:?}"
                ));
            }
        }

        // Insert in priority order (highest priority first)
        let insert_pos = self
            .contexts
            .iter()
            .position(|c| c.priority() < context.priority())
            .unwrap_or(self.contexts.len());

        self.contexts.insert(insert_pos, context);
        Ok(())
    }

    /// Pop a specific context from the stack
    pub fn pop_context(&mut self, context: InputContext) -> bool {
        if let Some(pos) = self.contexts.iter().position(|&c| c == context) {
            self.contexts.remove(pos);

            // Ensure we always have at least one context
            if self.contexts.is_empty() {
                self.contexts.push(InputContext::Walking);
            }
            true
        } else {
            false
        }
    }

    /// Get the highest priority (active) context
    pub fn active_context(&self) -> InputContext {
        self.contexts
            .first()
            .copied()
            .unwrap_or(InputContext::Walking)
    }

    /// Get all active contexts in priority order
    pub fn contexts(&self) -> &[InputContext] {
        &self.contexts
    }

    /// Check if a specific context is active
    pub fn has_context(&self, context: InputContext) -> bool {
        self.contexts.contains(&context)
    }

    /// Check if an action is available in the current context stack
    pub fn is_action_available(&self, action: crate::input::InputAction) -> bool {
        // Check if any active context allows this action
        for context in &self.contexts {
            if action.is_available_in_context(context) {
                // Make sure it's not disabled by a higher priority context
                for higher_priority in &self.contexts {
                    if higher_priority.priority() > context.priority()
                        && higher_priority.disabled_actions().contains(&action)
                    {
                        return false;
                    }
                }
                return true;
            }
        }
        false
    }

    /// Clear all contexts and set to the given context
    pub fn set_context(&mut self, context: InputContext) {
        self.contexts.clear();
        self.contexts.push(context);
    }

    /// Reset to default walking context
    pub fn reset(&mut self) {
        self.contexts.clear();
        self.contexts.push(InputContext::Walking);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::InputAction;

    #[test]
    fn test_context_priorities() {
        assert!(InputContext::Menu.priority() > InputContext::Flying.priority());
        assert!(InputContext::Flying.priority() > InputContext::Driving.priority());
        assert!(InputContext::Driving.priority() > InputContext::Walking.priority());
    }

    #[test]
    fn test_context_concurrency() {
        assert!(!InputContext::Menu.allows_concurrent(&InputContext::Walking));
        assert!(!InputContext::Driving.allows_concurrent(&InputContext::Flying));
        assert!(InputContext::Walking.allows_concurrent(&InputContext::Walking));
    }

    #[test]
    fn test_context_stack() {
        let mut stack = ContextStack::default();
        assert_eq!(stack.active_context(), InputContext::Walking);

        // Push driving context
        stack.push_context(InputContext::Driving).unwrap();
        assert_eq!(stack.active_context(), InputContext::Driving);

        // Cannot push conflicting context
        assert!(stack.push_context(InputContext::Flying).is_err());

        // Pop driving context
        assert!(stack.pop_context(InputContext::Driving));
        assert_eq!(stack.active_context(), InputContext::Walking);
    }

    #[test]
    fn test_action_availability() {
        let mut stack = ContextStack::new(InputContext::Walking);
        assert!(stack.is_action_available(InputAction::MoveForward));
        assert!(!stack.is_action_available(InputAction::Accelerate));

        stack.set_context(InputContext::Driving);
        assert!(!stack.is_action_available(InputAction::MoveForward));
        assert!(stack.is_action_available(InputAction::Accelerate));
    }
}
