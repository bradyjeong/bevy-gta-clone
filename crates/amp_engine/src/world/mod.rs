//! World management and ECS integration
//!
//! This crate provides a high-level interface for managing the game world,
//! including ECS world management, component registration, and scheduling.

#![deny(missing_docs)]

// Re-export commonly used ECS types
#[cfg(feature = "bevy16")]
pub use bevy::prelude::*;

/// Future world management implementation
pub struct WorldManager {
    /// The ECS world
    world: World,
}

impl WorldManager {
    /// Create a new world manager
    pub fn new() -> Self {
        Self {
            world: World::new(),
        }
    }

    /// Get a reference to the world
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Get a mutable reference to the world
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
}

impl Default for WorldManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_manager_creation() {
        let manager = WorldManager::new();
        assert_eq!(manager.world().entities().len(), 0);
    }

    #[test]
    fn test_world_manager_default() {
        let manager = WorldManager::default();
        assert_eq!(manager.world().entities().len(), 0);
    }
}
