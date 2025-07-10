//! Scene prefab asset definition for Bevy asset pipeline

use bevy::asset::{Asset, Handle};
use bevy::ecs::entity::Entity;
use bevy::ecs::system::Commands;
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};

/// Scene prefab asset for Bevy asset pipeline
#[derive(Asset, TypePath, Debug, Clone)]
pub struct AmpScenePrefab {
    /// Component definitions for this scene prefab
    pub components: Vec<AmpSceneComponent>,
}

impl AmpScenePrefab {
    /// Create a new empty scene prefab
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    /// Add a component to this scene prefab
    pub fn add_component(&mut self, component: AmpSceneComponent) {
        self.components.push(component);
    }

    /// Get the number of components in this scene prefab
    pub fn len(&self) -> usize {
        self.components.len()
    }

    /// Check if this scene prefab is empty
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    /// Instantiate this scene prefab as an entity
    pub fn instantiate(&self, commands: &mut Commands) -> Entity {
        let entity = commands.spawn_empty().id();

        for component in &self.components {
            if let Err(e) = component.init(commands, entity) {
                bevy::log::error!(
                    "Failed to initialize component '{}': {}",
                    component.component_type,
                    e
                );
            }
        }

        entity
    }
}

impl Default for AmpScenePrefab {
    fn default() -> Self {
        Self::new()
    }
}

/// Component definition for scene prefabs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmpSceneComponent {
    /// Component type name
    pub component_type: String,
    /// Component data as RON value
    pub data: ron::Value,
}

impl AmpSceneComponent {
    /// Create a new scene component
    pub fn new(component_type: String, data: ron::Value) -> Self {
        Self {
            component_type,
            data,
        }
    }

    /// Initialize this component on an entity
    pub fn init(
        &self,
        _commands: &mut Commands,
        entity: Entity,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // This will be implemented to use the component registry
        // For now, just a placeholder that logs the component type
        bevy::log::info!(
            "Initializing component '{}' on entity {:?}",
            self.component_type,
            entity
        );
        Ok(())
    }
}

/// Handle type for scene prefab assets
pub type AmpScenePrefabHandle = Handle<AmpScenePrefab>;
