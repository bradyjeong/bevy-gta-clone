//! Prefab definitions and component initialization

use amp_core::Error;
use bevy::prelude::*;
use std::any::Any;

/// Trait for initializing components on spawned entities
pub trait ComponentInit: Send + Sync {
    /// Initialize the component on the given entity
    fn init(&self, cmd: &mut Commands, entity: Entity) -> Result<(), Error>;

    /// Get the component as Any for downcasting
    fn as_any(&self) -> &dyn Any;
}

/// A prefab definition containing component initializers
pub struct Prefab {
    /// Component initializers for this prefab
    components: Vec<Box<dyn ComponentInit>>,
}

impl Prefab {
    /// Create a new empty prefab
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    /// Add a component initializer to this prefab
    pub fn with_component(mut self, component: Box<dyn ComponentInit>) -> Self {
        self.components.push(component);
        self
    }

    /// Add a component initializer to this prefab (mutable)
    pub fn add_component(&mut self, component: Box<dyn ComponentInit>) {
        self.components.push(component);
    }

    /// Spawn an entity from this prefab
    ///
    /// Returns the spawned entity ID on success. If any component initialization
    /// fails, the entity is despawned to maintain transaction safety.
    pub fn spawn(&self, cmd: &mut Commands) -> Result<Entity, Error> {
        // Spawn the entity first
        let entity = cmd.spawn_empty().id();

        // Initialize all components for this entity
        // If any fail, despawn the entity to maintain transaction safety
        for component in &self.components {
            if let Err(e) = component.init(cmd, entity) {
                cmd.entity(entity).despawn();
                return Err(e);
            }
        }

        Ok(entity)
    }

    /// Get the number of components in this prefab
    pub fn len(&self) -> usize {
        self.components.len()
    }

    /// Check if this prefab has no components
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    /// Get an iterator over the components in this prefab
    pub fn components(&self) -> impl Iterator<Item = &Box<dyn ComponentInit>> {
        self.components.iter()
    }
}

impl Default for Prefab {
    fn default() -> Self {
        Self::new()
    }
}
