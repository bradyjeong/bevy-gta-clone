//! PrefabFactory with spawn API and asset integration

use crate::{BasicPrefab, Error, Factory, PrefabId};
use bevy::prelude::*;
use std::collections::HashMap;

/// Enhanced factory for prefab-based entity creation with spawn API
#[derive(Default)]
pub struct PrefabFactory {
    /// Core factory for prefab registration
    factory: Factory,
    /// Named prefab registry for string-based lookups
    named_registry: HashMap<String, PrefabId>,
    // Asset handles for prefab resources (removed for MVP)
    // asset_handles: HashMap<String, Handle<BasicPrefab>>,
}

impl PrefabFactory {
    /// Create a new prefab factory
    pub fn new() -> Self {
        Self {
            factory: Factory::new(),
            named_registry: HashMap::new(),
            // asset_handles: HashMap::new(),
        }
    }

    /// Register a prefab with both ID and name
    pub fn register_prefab(
        &mut self,
        id: PrefabId,
        name: &str,
        prefab: BasicPrefab,
    ) -> Result<(), Error> {
        // Register with core factory
        self.factory.register(id, prefab)?;

        // Register name mapping
        self.named_registry.insert(name.to_string(), id);

        Ok(())
    }

    /// Spawn a prefab by ID
    pub fn spawn_prefab(&self, commands: &mut Commands, id: PrefabId) -> Result<Entity, Error> {
        self.factory.spawn(commands, id)
    }

    /// Spawn a prefab by name
    pub fn spawn_prefab_named(&self, commands: &mut Commands, name: &str) -> Result<Entity, Error> {
        let id = self.named_registry.get(name).ok_or_else(|| {
            Error::resource_load("prefab", format!("No prefab named '{name}' found"))
        })?;

        self.factory.spawn(commands, *id)
    }

    /// Check if a prefab exists by name
    pub fn has_prefab(&self, name: &str) -> bool {
        self.named_registry.contains_key(name)
    }

    /// Get all registered prefab names
    pub fn get_prefab_names(&self) -> Vec<String> {
        self.named_registry.keys().cloned().collect()
    }

    /// Get prefab count
    pub fn prefab_count(&self) -> usize {
        self.factory.len()
    }

    /// Load prefabs from directory
    #[cfg(feature = "ron")]
    pub fn load_directory(
        &mut self,
        settings: &config_core::FactorySettings,
    ) -> Result<usize, Error> {
        self.factory.load_directory(settings)
    }

    /// Create a vehicle prefab (example usage)
    pub fn create_vehicle_prefab(name: &str, position: Vec3, color: Color) -> BasicPrefab {
        let mut prefab = BasicPrefab::with_metadata("Vehicle".to_string(), "vehicle".to_string());

        // Use custom serialization wrappers for Bevy components
        prefab.add_component(
            "Transform".to_string(),
            format!(
                "(translation: {:?}, rotation: {:?}, scale: {:?})",
                position,
                Vec3::ZERO,
                Vec3::ONE
            ),
        );
        prefab.add_component(
            "Name".to_string(),
            ron::to_string(&Name::new(name.to_string())).unwrap(),
        );
        prefab.add_component("Visibility".to_string(), "Visible".to_string());
        prefab.add_component("Color".to_string(), ron::to_string(&color).unwrap());
        prefab
    }

    /// Create a building prefab (example usage)
    pub fn create_building_prefab(name: &str, position: Vec3, size: Vec3) -> BasicPrefab {
        let mut prefab = BasicPrefab::with_metadata("Building".to_string(), "building".to_string());

        // Use custom serialization wrappers for Bevy components
        prefab.add_component(
            "Transform".to_string(),
            format!(
                "(translation: {:?}, rotation: {:?}, scale: {:?})",
                position,
                Vec3::ZERO,
                size
            ),
        );
        prefab.add_component(
            "Name".to_string(),
            ron::to_string(&Name::new(name.to_string())).unwrap(),
        );
        prefab.add_component("Visibility".to_string(), "Visible".to_string());
        prefab
    }

    /// Create a character prefab (example usage)
    pub fn create_character_prefab(name: &str, position: Vec3) -> BasicPrefab {
        let mut prefab =
            BasicPrefab::with_metadata("Character".to_string(), "character".to_string());

        // Use custom serialization wrappers for Bevy components
        prefab.add_component(
            "Transform".to_string(),
            format!(
                "(translation: {:?}, rotation: {:?}, scale: {:?})",
                position,
                Vec3::ZERO,
                Vec3::ONE
            ),
        );
        prefab.add_component(
            "Name".to_string(),
            ron::to_string(&Name::new(name.to_string())).unwrap(),
        );
        prefab.add_component("Visibility".to_string(), "Visible".to_string());
        prefab
    }

    /// Batch spawn multiple prefabs
    pub fn batch_spawn(
        &self,
        commands: &mut Commands,
        spawns: &[(PrefabId, Transform)],
    ) -> Result<Vec<Entity>, Error> {
        #[cfg(feature = "tracy")]
        let _span = tracy_client::span!("prefab_batch_spawn");

        let mut entities = Vec::new();

        for (id, transform) in spawns {
            let entity = self.spawn_prefab(commands, *id)?;
            commands.entity(entity).insert(*transform);
            entities.push(entity);
        }

        Ok(entities)
    }

    // Commented out for MVP - complex blueprint optimization
    // /// High-performance batch spawn using blueprint cache and memory pools
    // ///
    // /// Oracle's 37× optimization: This method bypasses the Commands system and uses
    // /// direct world manipulation with pre-compiled blueprints for maximum performance
    // pub fn batch_spawn_optimized(
    //     &self,
    //     world: &mut World,
    //     spawns: &[(PrefabId, usize)], // (prefab_id, count)
    //     pooled_factory: &mut crate::PooledEntityFactory,
    //     type_registry: &AppTypeRegistry,
    // ) -> Result<Vec<Entity>, Error> {
    //     #[cfg(feature = "tracy")]
    //     let _span = tracy_client::span!("prefab_batch_spawn_optimized");

    //     // Collect component maps for all requested prefabs
    //     let mut component_maps = std::collections::HashMap::new();
    //     for (prefab_id, _count) in spawns {
    //         if let Some(prefab) = self.factory.registry.get(prefab_id) {
    //             // Convert BasicPrefab to ComponentMap for blueprint compilation
    //             // This is a simplified conversion - in practice, we'd need proper
    //             // conversion logic from BasicPrefab to ComponentMap
    //             let component_map = crate::ComponentMap {
    //                 components: std::collections::HashMap::new(), // TODO: Convert from BasicPrefab
    //                 metadata: crate::ComponentMapMetadata {
    //                     source_path: None,
    //                     validation_status: crate::ValidationStatus::Valid,
    //                     component_count: 0,
    //                 },
    //             };
    //             component_maps.insert(*prefab_id, component_map);
    //         } else {
    //             return Err(Error::resource_load(
    //                 "prefab",
    //                 format!("Prefab {prefab_id:?} not found in registry"),
    //             ));
    //         }
    //     }

    //     // Use pooled factory for optimized batch spawning
    //     pooled_factory.spawn_batch(spawns, &component_maps, world, type_registry)
    // }

    /// Batch spawn by name
    pub fn batch_spawn_named(
        &self,
        commands: &mut Commands,
        spawns: &[(&str, Transform)],
    ) -> Result<Vec<Entity>, Error> {
        let mut entities = Vec::new();

        for (name, transform) in spawns {
            let entity = self.spawn_prefab_named(commands, name)?;
            commands.entity(entity).insert(*transform);
            entities.push(entity);
        }

        Ok(entities)
    }

    /// Get hot-reload receiver if available
    #[cfg(feature = "hot-reload")]
    pub fn take_hot_reload_receiver(&mut self) -> Option<crate::HotReloadReceiver> {
        self.factory.take_hot_reload_receiver()
    }

    /// Stub method when hot-reload is disabled
    #[cfg(not(feature = "hot-reload"))]
    pub fn take_hot_reload_receiver(&mut self) -> Option<crate::HotReloadReceiver> {
        None
    }
}

/// Bevy resource for the prefab factory
#[derive(Resource)]
pub struct PrefabFactoryResource {
    /// The prefab factory
    pub factory: PrefabFactory,
}

impl Default for PrefabFactoryResource {
    fn default() -> Self {
        Self {
            factory: PrefabFactory::new(),
        }
    }
}

/// Plugin for prefab factory integration
pub struct PrefabFactoryPlugin;

impl Plugin for PrefabFactoryPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PrefabFactoryResource::default());
    }
}

/// System for spawning prefabs from events
pub fn spawn_prefab_events(
    mut commands: Commands,
    mut events: EventReader<SpawnPrefabEvent>,
    factory: Res<PrefabFactoryResource>,
) {
    for event in events.read() {
        match event {
            SpawnPrefabEvent::ByName { name, transform } => {
                if let Ok(entity) = factory.factory.spawn_prefab_named(&mut commands, name) {
                    if let Some(transform) = transform {
                        commands.entity(entity).insert(*transform);
                    }
                    info!("Spawned prefab '{}' as entity {:?}", name, entity);
                } else {
                    error!("Failed to spawn prefab '{}'", name);
                }
            }
            SpawnPrefabEvent::ById { id, transform } => {
                if let Ok(entity) = factory.factory.spawn_prefab(&mut commands, *id) {
                    if let Some(transform) = transform {
                        commands.entity(entity).insert(*transform);
                    }
                    info!("Spawned prefab {:?} as entity {:?}", id, entity);
                } else {
                    error!("Failed to spawn prefab {:?}", id);
                }
            }
        }
    }
}

/// Event for spawning prefabs
#[derive(Event)]
pub enum SpawnPrefabEvent {
    /// Spawn by name
    ByName {
        name: String,
        transform: Option<Transform>,
    },
    /// Spawn by ID
    ById {
        id: PrefabId,
        transform: Option<Transform>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::component_init; // Not needed for current tests

    #[test]
    fn test_prefab_factory_creation() {
        let factory = PrefabFactory::new();
        assert_eq!(factory.prefab_count(), 0);
        assert!(factory.get_prefab_names().is_empty());
    }

    #[test]
    fn test_prefab_registration() {
        let mut factory = PrefabFactory::new();
        let id = PrefabId::new(123);
        let prefab = BasicPrefab::new();

        assert!(factory.register_prefab(id, "test", prefab).is_ok());
        assert!(factory.has_prefab("test"));
        assert_eq!(factory.prefab_count(), 1);
        assert_eq!(factory.get_prefab_names(), vec!["test".to_string()]);
    }

    #[test]
    fn test_vehicle_prefab_creation() {
        let prefab = PrefabFactory::create_vehicle_prefab(
            "TestCar",
            Vec3::new(1.0, 2.0, 3.0),
            Color::srgb(1.0, 0.0, 0.0), // Red color
        );

        assert_eq!(prefab.component_count(), 4); // transform, name, visibility, color
        assert_eq!(prefab.metadata().name, "Vehicle");
    }

    #[test]
    fn test_building_prefab_creation() {
        let prefab = PrefabFactory::create_building_prefab(
            "TestBuilding",
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(2.0, 3.0, 4.0),
        );

        assert_eq!(prefab.component_count(), 3); // transform, name, visibility
        assert_eq!(prefab.metadata().name, "Building");
    }

    #[test]
    fn test_character_prefab_creation() {
        let prefab =
            PrefabFactory::create_character_prefab("TestCharacter", Vec3::new(5.0, 0.0, 5.0));

        assert_eq!(prefab.component_count(), 3); // transform, name, visibility
        assert_eq!(prefab.metadata().name, "Character");
    }

    #[test]
    fn test_spawn_prefab_event() {
        let event = SpawnPrefabEvent::ByName {
            name: "test_prefab".to_string(),
            transform: Some(Transform::from_xyz(1.0, 2.0, 3.0)),
        };

        match event {
            SpawnPrefabEvent::ByName { name, transform } => {
                assert_eq!(name, "test_prefab");
                assert!(transform.is_some());
            }
            _ => panic!("Expected ByName event"),
        }
    }
}
