//! NPC behavior system
//!
//! Provides AI components, systems, and behavior state machines for NPCs.
//! Includes distance-based tick rates, batch processing integration, and
//! configurable behavior patterns.

pub mod components;
pub mod config;
pub mod systems;

#[cfg(test)]
mod tests;

pub use components::*;
pub use config::*;
pub use systems::*;

use bevy::prelude::*;

/// Plugin for NPC behavior system
#[derive(Default)]
pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register config resource
            .init_resource::<NpcConfig>()
            .init_resource::<NpcMetrics>()
            // Register component types for reflection
            .register_type::<NPC>()
            .register_type::<NpcState>()
            .register_type::<NpcBrainHandle>()
            .register_type::<LastUpdateFrame>()
            .register_type::<NpcBehaviorState>()
            .register_type::<NpcMetrics>()
            // Add systems to appropriate sets
            .add_systems(
                Update,
                (npc_brain_system, npc_metrics_system).in_set(NpcSystemSet::Brain),
            );
    }
}

/// System set for NPC behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum NpcSystemSet {
    /// Brain processing and state updates
    Brain,
    /// Metrics collection and performance monitoring
    Metrics,
}
