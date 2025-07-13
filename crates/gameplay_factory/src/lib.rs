//! Unified entity factory for prefab-based gameplay systems
//!
//! This crate provides a factory pattern for creating game entities from prefab definitions.
//! It supports loading prefabs from various sources and spawning them into the ECS world.

use bevy::prelude::*;
use dashmap::DashSet;
use once_cell::sync::Lazy;
use std::collections::HashMap;

pub use amp_core::Error;

// Re-export component registry functions
pub use component_registry::{
    ComponentDeserializer, call_component_deserializer, register_component,
    register_default_components, registered_components,
};

mod component_registry;

mod prefab;
pub use prefab::*;

#[cfg(feature = "legacy_ron_loader")]
mod ron_loader;
#[cfg(feature = "legacy_ron_loader")]
pub use ron_loader::*;

mod hot_reload;
pub use hot_reload::*;

mod dsl;
pub use dsl::*;

mod factory_dsl_integration;
pub use factory_dsl_integration::*;

mod assets;
pub use assets::{
    PrefabAsset, PrefabAssetLoader, PrefabAssetPlugin, convert_prefab_asset_to_runtime_prefab,
};

mod vehicle_factory;
pub use vehicle_factory::*;

mod prefab_factory;
pub use prefab_factory::*;

/// Unique identifier for prefab definitions
///
/// This is a hardened type that prevents silent narrowing and uses a global
/// collision detection system to ensure uniqueness across all Factory instances.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PrefabId(u64);

impl PrefabId {
    /// Create a new PrefabId from a u64 value
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the raw u64 value
    pub fn raw(&self) -> u64 {
        self.0
    }
}

impl From<u64> for PrefabId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<u32> for PrefabId {
    fn from(id: u32) -> Self {
        Self(id as u64)
    }
}

impl std::fmt::Display for PrefabId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PrefabId({})", self.0)
    }
}

/// Global collision detection registry
///
/// This singleton tracks all registered PrefabIds across all Factory instances
/// to prevent ID collisions even when using multiple factories.
static GLOBAL_PREFAB_IDS: Lazy<DashSet<PrefabId>> = Lazy::new(DashSet::new);

/// Check if a PrefabId has been registered globally
pub fn is_prefab_id_registered(id: PrefabId) -> bool {
    GLOBAL_PREFAB_IDS.contains(&id)
}

/// Get all registered PrefabIds
pub fn get_all_prefab_ids() -> Vec<PrefabId> {
    GLOBAL_PREFAB_IDS.iter().map(|entry| *entry).collect()
}

/// Clear all registered PrefabIds (primarily for testing)
pub fn clear_all_prefab_ids() {
    GLOBAL_PREFAB_IDS.clear();
}

/// Trait for loading prefab definitions from various sources
pub trait PrefabSource {
    /// Load a prefab from this source
    fn load(&self) -> Result<BasicPrefab, Error>;
}

/// Factory for creating entities from prefab definitions
///
/// # Example
///
/// ```rust
/// use gameplay_factory::{Factory, PrefabId, BasicPrefab};
///
/// // Create a new factory instance
/// let mut factory = Factory::new();
///
/// // Create a prefab ID and basic prefab
/// let id = PrefabId::new(42);
/// let prefab = BasicPrefab::new();
///
/// // Note: This doctest doesn't actually register to avoid global state issues
/// // In real usage: factory.register(id, prefab).unwrap();
/// assert_eq!(id.raw(), 42);
/// ```
pub struct Factory {
    registry: HashMap<PrefabId, BasicPrefab>,
    #[cfg(feature = "hot-reload")]
    hot_reload_sender: Option<HotReloadSender>,
    #[cfg(feature = "hot-reload")]
    watcher_handle: Option<WatcherHandle>,
}

impl Factory {
    /// Create a new empty factory
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
            #[cfg(feature = "hot-reload")]
            hot_reload_sender: None,
            #[cfg(feature = "hot-reload")]
            watcher_handle: None,
        }
    }

    /// Register a prefab with the factory
    pub fn register(&mut self, id: PrefabId, prefab: BasicPrefab) -> Result<(), Error> {
        // Check for global collision detection first
        if GLOBAL_PREFAB_IDS.contains(&id) {
            return Err(Error::validation(format!("Duplicate PrefabId {id:?}")));
        }

        // Check for local collision detection
        if self.registry.contains_key(&id) {
            log::warn!(
                "Prefab ID {id:?} already exists in local registry, replacing existing prefab"
            );
        }

        // Register globally and locally
        GLOBAL_PREFAB_IDS.insert(id);
        self.registry.insert(id, prefab);
        Ok(())
    }

    /// Load and register a prefab from a source
    pub fn load_from_source(
        &mut self,
        id: PrefabId,
        source: &dyn PrefabSource,
    ) -> Result<(), Error> {
        let prefab = source.load()?;
        self.register(id, prefab)?;
        Ok(())
    }

    /// Spawn an entity from a registered prefab
    pub fn spawn(&self, cmd: &mut Commands, id: PrefabId) -> Result<Entity, Error> {
        let prefab = self.registry.get(&id).ok_or_else(|| {
            Error::resource_load(format!("Prefab {id:?}"), "not found in registry")
        })?;

        prefab.spawn(cmd)
    }

    /// Check if a prefab is registered
    pub fn contains(&self, id: PrefabId) -> bool {
        self.registry.contains_key(&id)
    }

    /// Get the number of registered prefabs
    pub fn len(&self) -> usize {
        self.registry.len()
    }

    /// Check if the factory is empty
    pub fn is_empty(&self) -> bool {
        self.registry.is_empty()
    }

    /// Load prefabs from a directory based on factory settings
    #[cfg(feature = "ron")]
    pub fn load_directory(
        &mut self,
        settings: &config_core::FactorySettings,
    ) -> Result<usize, Error> {
        use std::path::Path;

        // Expand tilde in prefab_path
        let expanded_path = settings.expanded_prefab_path()?;

        // Handle hot_reload setting
        if settings.hot_reload {
            self.setup_file_watcher(&expanded_path)?;
        }

        // Use glob to find matching files
        let paths = glob::glob(&expanded_path).map_err(|e| {
            Error::resource_load(
                "glob pattern",
                format!("Invalid glob pattern '{expanded_path}': {e}"),
            )
        })?;

        let mut loaded_count = 0;
        let mut errors = Vec::new();

        for path_result in paths {
            match path_result {
                Ok(path) => {
                    // Generate a unique ID based on the filename
                    let prefab_id = self.generate_prefab_id_from_path(&path)?;

                    // Load the prefab file
                    match self.load_prefab_file(&path) {
                        Ok(prefab) => match self.register(prefab_id, prefab) {
                            Ok(()) => {
                                loaded_count += 1;
                                log::debug!(
                                    "Loaded prefab {:?} from {}",
                                    prefab_id,
                                    path.display()
                                );
                            }
                            Err(e) => {
                                errors.push(format!(
                                    "Failed to register prefab from {}: {}",
                                    path.display(),
                                    e
                                ));
                            }
                        },
                        Err(e) => {
                            errors.push(format!("Failed to load {}: {}", path.display(), e));
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!("Glob error: {e}"));
                }
            }
        }

        // If we have errors but also loaded some files, log warnings
        if !errors.is_empty() && loaded_count > 0 {
            for error in &errors {
                log::warn!("{error}");
            }
        }

        // If we have errors and loaded nothing, return the first error
        if !errors.is_empty() && loaded_count == 0 {
            return Err(Error::resource_load("prefab directory", &errors[0]));
        }

        // If no files were found, check if the directory exists
        if loaded_count == 0 {
            let parent_dir = Path::new(&expanded_path)
                .parent()
                .unwrap_or_else(|| Path::new("."));

            if !parent_dir.exists() {
                return Err(Error::resource_load(
                    "prefab directory",
                    format!("Directory {} does not exist", parent_dir.display()),
                ));
            }

            // Directory exists but no matching files
            log::info!("No .ron files found matching pattern: {expanded_path}");
        }

        Ok(loaded_count)
    }

    /// Generate a PrefabId from a file path
    #[cfg(feature = "ron")]
    pub fn generate_prefab_id_from_path(&self, path: &std::path::Path) -> Result<PrefabId, Error> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Use the full path for better collision resistance
        let full_path = path
            .to_str()
            .ok_or_else(|| Error::resource_load("filename", "Non-UTF8 path"))?;

        // Create a full 64-bit hash of the path (no truncation)
        let mut hasher = DefaultHasher::new();
        full_path.hash(&mut hasher);
        let hash = hasher.finish();

        // Check for collision in global registry
        let id = PrefabId(hash);
        if GLOBAL_PREFAB_IDS.contains(&id) {
            return Err(Error::validation(format!(
                "Hash collision detected for path {full_path}: ID {id:?} already exists globally"
            )));
        }

        Ok(id)
    }

    /// Load a prefab from a RON file
    #[cfg(feature = "ron")]
    fn load_prefab_file(&self, path: &std::path::Path) -> Result<BasicPrefab, Error> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            Error::resource_load(
                format!("prefab file {}", path.display()),
                format!("IO error: {e}"),
            )
        })?;

        // Use the DSL system to load the prefab
        let config = DslConfig::default();
        let type_registry = AppTypeRegistry::default();
        let component_map = parse_prefab_ron(&content, &type_registry, &config)?;
        create_prefab_from_component_map(&component_map, &type_registry)
    }

    /// Set up file watcher for hot-reload functionality
    #[cfg(feature = "ron")]
    fn setup_file_watcher(&mut self, _path: &str) -> Result<(), Error> {
        #[cfg(feature = "hot-reload")]
        {
            // Create channel for hot-reload events
            let (tx, _rx) = create_reload_channel();

            // Store the sender for later use
            self.hot_reload_sender = Some(tx.clone());

            // Start the watcher
            let pattern = _path.to_string();
            let watcher_handle = tokio::task::spawn(async move {
                if let Err(e) = watcher::run_watcher(&pattern, tx).await {
                    log::error!("Hot-reload watcher error: {e}");
                }
            });

            self.watcher_handle = Some(WatcherHandle::new(watcher_handle));

            log::info!("Hot-reload file watcher set up for path: {_path}");
            Ok(())
        }
        #[cfg(not(feature = "hot-reload"))]
        {
            log::warn!("Hot-reload requested but hot-reload feature is not enabled");
            Ok(())
        }
    }

    /// Get the hot-reload receiver if hot-reload is enabled
    #[cfg(feature = "hot-reload")]
    pub fn take_hot_reload_receiver(&mut self) -> Option<HotReloadReceiver> {
        if let Some(_tx) = &self.hot_reload_sender {
            let (new_tx, rx) = create_reload_channel();
            self.hot_reload_sender = Some(new_tx);
            Some(rx)
        } else {
            None
        }
    }

    /// Stub method when hot-reload is disabled
    #[cfg(not(feature = "hot-reload"))]
    pub fn take_hot_reload_receiver(&mut self) -> Option<HotReloadReceiver> {
        None
    }
}

impl Default for Factory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::collections::HashSet;

    #[test]
    fn test_prefab_id_collision_detection() {
        // Clear global registry before test
        clear_all_prefab_ids();

        let mut factory = Factory::new();

        // Test successful registration
        let id1 = PrefabId::new(12345);
        assert!(factory.register(id1, BasicPrefab::new()).is_ok());

        // Test collision detection
        let id2 = PrefabId::new(12345); // Same ID
        assert!(factory.register(id2, BasicPrefab::new()).is_err());

        // Test different factory instances share global registry
        let mut factory2 = Factory::new();
        let id3 = PrefabId::new(12345); // Same ID as id1
        assert!(factory2.register(id3, BasicPrefab::new()).is_err());

        // Test different ID works
        let id4 = PrefabId::new(54321);
        assert!(factory2.register(id4, BasicPrefab::new()).is_ok());

        // Clean up
        clear_all_prefab_ids();
    }

    #[test]
    fn test_prefab_id_from_u32() {
        // Test successful conversion
        let id = PrefabId::from(12345u32);
        assert_eq!(id.raw(), 12345u64);

        // Test maximum u32 value
        let id = PrefabId::from(u32::MAX);
        assert_eq!(id.raw(), u32::MAX as u64);
    }

    #[test]
    #[serial]
    fn test_global_registry_functions() {
        clear_all_prefab_ids();

        let id1 = PrefabId::new(111);
        let id2 = PrefabId::new(222);

        // Initially empty
        clear_all_prefab_ids(); // Ensure clean state
        assert!(!is_prefab_id_registered(id1));
        assert!(!is_prefab_id_registered(id2));
        assert!(get_all_prefab_ids().is_empty());

        // Register through factory
        let mut factory = Factory::new();
        factory.register(id1, BasicPrefab::new()).unwrap();

        // Check registration
        assert!(is_prefab_id_registered(id1));
        assert!(!is_prefab_id_registered(id2));
        assert_eq!(get_all_prefab_ids().len(), 1);

        // Register another
        factory.register(id2, BasicPrefab::new()).unwrap();
        assert!(is_prefab_id_registered(id2));
        assert_eq!(get_all_prefab_ids().len(), 2);

        // Clean up
        clear_all_prefab_ids();
        assert!(get_all_prefab_ids().is_empty());
    }

    #[test]
    #[serial]
    fn test_prefab_id_fuzzer() {
        clear_all_prefab_ids();

        let mut registered_ids = HashSet::new();
        let mut factory = Factory::new();

        // Generate a large number of random paths and ensure no duplicates slip through
        let test_paths = [
            "/path/to/prefab1.ron",
            "/path/to/prefab2.ron",
            "/different/path/prefab1.ron",
            "/assets/characters/player.ron",
            "/assets/vehicles/car.ron",
            "/assets/weapons/gun.ron",
            "/prefabs/buildings/house.ron",
            "/prefabs/props/table.ron",
            "/data/npcs/guard.ron",
            "/config/items/key.ron",
            // Add some that might have hash collisions
            "/a/very/long/path/that/might/cause/issues.ron",
            "/another/very/long/path/that/might/cause/issues.ron",
        ];

        for path in &test_paths {
            let _path_obj = std::path::Path::new(path);

            // Generate ID using the same logic as generate_prefab_id_from_path
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let mut hasher = DefaultHasher::new();
            path.hash(&mut hasher);
            let hash = hasher.finish();
            let id = PrefabId::new(hash);

            // Try to register
            match factory.register(id, BasicPrefab::new()) {
                Ok(()) => {
                    // Should not be a duplicate
                    assert!(
                        registered_ids.insert(id),
                        "Duplicate ID {id:?} for path {path}"
                    );
                    assert!(is_prefab_id_registered(id));
                }
                Err(_) => {
                    // Should be a duplicate
                    assert!(
                        registered_ids.contains(&id),
                        "ID {id:?} for path {path} was not previously registered"
                    );
                }
            }
        }

        // Verify all registered IDs are in global registry
        for id in &registered_ids {
            assert!(
                is_prefab_id_registered(*id),
                "ID {id:?} should be registered in global registry"
            );
        }

        // Verify counts match
        assert_eq!(get_all_prefab_ids().len(), registered_ids.len());

        clear_all_prefab_ids();
    }
}
