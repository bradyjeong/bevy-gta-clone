//! Unified prefab system for gameplay factory
//!
//! This module provides a simplified prefab system that works with Bevy's Asset and Reflect systems.
//! It avoids the complex generic issues by using concrete types and string-based component storage.

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

/// Concrete prefab definition that avoids generic issues
#[derive(Asset, Reflect, Clone)]
pub struct BasicPrefab {
    /// Component data stored as (component_name, ron_data) pairs
    #[reflect(ignore)]
    pub component_data: Vec<(String, String)>,
    /// Child prefabs that should be spawned as children
    pub children: Vec<PrefabChild>,
    /// Prefab metadata
    pub metadata: PrefabMetadata,
}

/// Child prefab configuration
#[derive(Debug, Clone, Reflect)]
pub struct PrefabChild {
    /// The child prefab data
    pub component_data: Vec<(String, String)>,
    /// Transform relative to parent
    pub transform: Transform,
    /// Child name
    pub name: String,
}

/// Metadata for prefabs
#[derive(Debug, Clone, Reflect)]
pub struct PrefabMetadata {
    /// Prefab name
    pub name: String,
    /// Prefab type identifier
    pub type_id: String,
    /// Asset paths referenced by this prefab
    pub asset_paths: Vec<String>,
    /// Component count
    pub component_count: usize,
}

impl Default for PrefabMetadata {
    fn default() -> Self {
        Self {
            name: "Unnamed".to_string(),
            type_id: "generic".to_string(),
            asset_paths: Vec::new(),
            component_count: 0,
        }
    }
}

impl Default for BasicPrefab {
    fn default() -> Self {
        Self::new()
    }
}

impl BasicPrefab {
    /// Create a new empty prefab
    pub fn new() -> Self {
        Self {
            component_data: Vec::new(),
            children: Vec::new(),
            metadata: PrefabMetadata::default(),
        }
    }

    /// Create a prefab with metadata
    pub fn with_metadata(name: String, type_id: String) -> Self {
        Self {
            component_data: Vec::new(),
            children: Vec::new(),
            metadata: PrefabMetadata {
                name,
                type_id,
                asset_paths: Vec::new(),
                component_count: 0,
            },
        }
    }

    /// Add a component to the prefab
    pub fn add_component(&mut self, component_name: String, ron_data: String) -> &mut Self {
        self.component_data.push((component_name, ron_data));
        self.metadata.component_count = self.component_data.len();
        self
    }

    /// Add a child prefab
    pub fn add_child(&mut self, child: PrefabChild) -> &mut Self {
        self.children.push(child);
        self
    }

    /// Spawn this prefab as an entity
    pub fn spawn(&self, cmd: &mut Commands) -> Result<Entity, Error> {
        // Create the main entity
        let entity = cmd.spawn(()).id();

        // Apply components
        for (component_name, ron_data) in &self.component_data {
            if let Err(e) = self.apply_component(cmd, entity, component_name, ron_data) {
                log::warn!("Failed to apply component {component_name}: {e}");
            }
        }

        // Spawn children
        for child in &self.children {
            if let Ok(child_entity) = self.spawn_child(cmd, child) {
                cmd.entity(child_entity).insert(ChildOf(entity));
            }
        }

        Ok(entity)
    }

    /// Apply a component to an entity
    fn apply_component(
        &self,
        cmd: &mut Commands,
        entity: Entity,
        component_name: &str,
        ron_data: &str,
    ) -> Result<(), Error> {
        // Parse the RON data first
        let ron_value: ron::Value = ron::from_str(ron_data)
            .map_err(|e| Error::serialization(format!("Failed to parse RON data: {e}")))?;

        // Use the component registry to deserialize and apply the component
        crate::call_component_deserializer(component_name, &ron_value, cmd, entity)
    }

    /// Spawn a child prefab
    fn spawn_child(&self, cmd: &mut Commands, child: &PrefabChild) -> Result<Entity, Error> {
        // Create child entity
        let child_entity = cmd.spawn(child.transform).id();

        // Apply child components
        for (component_name, ron_data) in &child.component_data {
            if let Err(e) = self.apply_component(cmd, child_entity, component_name, ron_data) {
                log::warn!("Failed to apply child component {component_name}: {e}");
            }
        }

        // Add name component if specified
        if !child.name.is_empty() {
            cmd.entity(child_entity)
                .insert(Name::new(child.name.clone()));
        }

        Ok(child_entity)
    }

    /// Get component count
    pub fn component_count(&self) -> usize {
        self.component_data.len()
    }

    /// Get child count
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /// Get metadata
    pub fn metadata(&self) -> &PrefabMetadata {
        &self.metadata
    }

    /// Set metadata
    pub fn set_metadata(&mut self, metadata: PrefabMetadata) -> &mut Self {
        self.metadata = metadata;
        self
    }
}

/// Macro to simplify component initialization
#[macro_export]
macro_rules! component_init {
    ($component:expr) => {{
        (
            std::any::type_name::<$component>()
                .split("::")
                .last()
                .unwrap_or("Unknown")
                .to_string(),
            ron::to_string(&$component).map_err(|e| {
                Error::serialization(format!("Failed to serialize component: {}", e))
            })?,
        )
    }};
}

/// Macro to create a prefab with components
#[macro_export]
macro_rules! prefab {
    ($name:expr, $type_id:expr, { $($component_name:expr => $component:expr),* $(,)? }) => {{
        let mut prefab = BasicPrefab::with_metadata($name.to_string(), $type_id.to_string());
        $(
            let serialized = ron::to_string(&$component)
                .map_err(|e| Error::serialization(format!("Failed to serialize component: {}", e)))?;
            prefab.add_component($component_name.to_string(), serialized);
        )*
        prefab
    }};
}

/// Helper function to serialize components with proper error handling
pub fn serialize_component<T: serde::Serialize>(component: &T) -> Result<String, Error> {
    ron::to_string(component)
        .map_err(|e| Error::serialization(format!("Failed to serialize component: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    // use bevy::prelude::*; // Not needed for current tests

    #[test]
    fn test_basic_prefab_creation() {
        let mut prefab = BasicPrefab::new();
        assert_eq!(prefab.component_count(), 0);
        assert_eq!(prefab.child_count(), 0);

        // Add a component
        prefab.add_component("Transform".to_string(), "()".to_string());
        assert_eq!(prefab.component_count(), 1);
    }

    #[test]
    fn test_prefab_with_metadata() {
        let prefab = BasicPrefab::with_metadata("TestPrefab".to_string(), "test".to_string());
        assert_eq!(prefab.metadata().name, "TestPrefab");
        assert_eq!(prefab.metadata().type_id, "test");
    }

    #[test]
    fn test_prefab_child() {
        let child = PrefabChild {
            component_data: vec![("Transform".to_string(), "()".to_string())],
            transform: Transform::default(),
            name: "TestChild".to_string(),
        };

        let mut prefab = BasicPrefab::new();
        prefab.add_child(child);
        assert_eq!(prefab.child_count(), 1);
    }

    #[test]
    fn test_serialize_component() {
        // Test with a type that implements Serialize
        let name = Name::new("TestEntity");
        let result = serialize_component(&name);
        assert!(result.is_ok());
    }
}
