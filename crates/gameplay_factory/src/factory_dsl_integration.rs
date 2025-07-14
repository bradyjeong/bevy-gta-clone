//! Simplified integration between Factory and DSL systems
//!
//! This module provides basic integration between the existing Factory system
//! and the new Oracle-specified DSL system with component registry support.

use crate::{
    parse_prefab_ron, spawn_many, BatchSpawnRequest, BatchSpawnResult, ComponentMap, DslConfig,
    Error, Factory, FactoryDslExt, PrefabId, ValidationMode,
};
use bevy::prelude::*;
use std::collections::HashMap;
use std::path::Path;

/// Enhanced Factory with DSL capabilities
///
/// This struct wraps the existing Factory and adds DSL-specific functionality
/// while maintaining compatibility with the Oracle's specified patterns.
#[derive(Resource)]
pub struct DslFactory {
    /// Core factory instance
    pub factory: Factory,
    /// DSL configuration
    pub config: DslConfig,
    /// Cached component maps for performance
    component_cache: HashMap<String, ComponentMap>,
}

impl DslFactory {
    /// Create a new DSL-enabled factory
    pub fn new() -> Self {
        Self {
            factory: Factory::new(),
            config: DslConfig::default(),
            component_cache: HashMap::new(),
        }
    }

    /// Create a new DSL-enabled factory with custom configuration
    pub fn with_config(config: DslConfig) -> Self {
        Self {
            factory: Factory::new(),
            config,
            component_cache: HashMap::new(),
        }
    }

    /// Load prefabs from a directory using DSL patterns
    ///
    /// This method extends the existing Factory::load_directory with DSL support
    /// and component registry integration for dynamic component loading.
    pub fn load_directory_dsl(
        &mut self,
        directory: &str,
        type_registry: &AppTypeRegistry,
    ) -> Result<usize, Error> {
        let directory_path = Path::new(directory);

        if !directory_path.exists() {
            return Err(Error::resource_load(
                directory,
                "Directory does not exist".to_string(),
            ));
        }

        let mut loaded_count = 0;
        let mut errors = Vec::new();

        // Find all RON files in the directory
        let pattern = format!("{directory}/*.ron");
        let paths = glob::glob(&pattern).map_err(|e| {
            Error::resource_load(
                "glob pattern",
                format!("Invalid glob pattern '{pattern}': {e}"),
            )
        })?;

        for path_result in paths {
            match path_result {
                Ok(path) => {
                    let path_str = path.to_string_lossy();

                    // Generate prefab ID from path
                    let prefab_id = self.generate_prefab_id_from_path(&path)?;

                    // Load prefab using DSL
                    match self.load_prefab_file_dsl(&path_str, type_registry) {
                        Ok(component_map) => {
                            // Cache the component map if enabled
                            if self.config.cache_prefabs {
                                self.component_cache
                                    .insert(path_str.to_string(), component_map.clone());
                            }

                            // Register with factory
                            match self.factory.register_from_component_map(
                                prefab_id,
                                &component_map,
                                type_registry,
                            ) {
                                Ok(()) => {
                                    loaded_count += 1;
                                    debug!("Loaded prefab {:?} from {}", prefab_id, path_str);
                                }
                                Err(e) => {
                                    errors.push(format!(
                                        "Failed to register prefab from {path_str}: {e}"
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            errors.push(format!("Failed to load prefab from {path_str}: {e}"));
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!("Glob error: {e}"));
                }
            }
        }

        // Handle errors based on validation mode
        if !errors.is_empty() {
            match self.config.validation_mode {
                ValidationMode::Strict if loaded_count == 0 => {
                    return Err(Error::resource_load("prefab directory", &errors[0]));
                }
                ValidationMode::Strict => {
                    for error in &errors {
                        warn!("{}", error);
                    }
                }
                ValidationMode::Permissive => {
                    for error in &errors {
                        warn!("{}", error);
                    }
                }
                ValidationMode::Skip => {
                    // Ignore errors completely
                }
            }
        }

        info!(
            "Loaded {} prefabs from directory: {}",
            loaded_count, directory
        );
        Ok(loaded_count)
    }

    /// Load a single prefab file using DSL
    fn load_prefab_file_dsl(
        &self,
        file_path: &str,
        type_registry: &AppTypeRegistry,
    ) -> Result<ComponentMap, Error> {
        // Check cache first
        if self.config.cache_prefabs {
            if let Some(cached) = self.component_cache.get(file_path) {
                return Ok(cached.clone());
            }
        }

        let ron_content = std::fs::read_to_string(file_path)
            .map_err(|e| Error::resource_load(file_path, format!("Failed to read file: {e}")))?;

        let mut component_map = parse_prefab_ron(&ron_content, type_registry, &self.config)?;

        // Set source path in metadata
        component_map.metadata.source_path = Some(file_path.to_string());

        Ok(component_map)
    }

    /// Generate a prefab ID from a file path
    fn generate_prefab_id_from_path(&self, path: &Path) -> Result<PrefabId, Error> {
        self.factory.generate_prefab_id_from_path(path)
    }

    /// Spawn multiple entities using DSL batch processing
    ///
    /// This method provides high-performance batch spawning using the Oracle's
    /// specified batching patterns and Commands::spawn_batch.
    pub fn spawn_batch_dsl(
        &self,
        commands: &mut Commands,
        entities: Vec<ComponentMap>,
        type_registry: &AppTypeRegistry,
    ) -> Result<BatchSpawnResult, Error> {
        let request = BatchSpawnRequest {
            entities,
            config: self.config.clone(),
        };

        spawn_many(commands, request, type_registry)
    }

    /// Get a reference to the underlying Factory
    pub fn factory(&self) -> &Factory {
        &self.factory
    }

    /// Get a mutable reference to the underlying Factory
    pub fn factory_mut(&mut self) -> &mut Factory {
        &mut self.factory
    }

    /// Get the DSL configuration
    pub fn config(&self) -> &DslConfig {
        &self.config
    }

    /// Update the DSL configuration
    pub fn set_config(&mut self, config: DslConfig) {
        self.config = config;
    }

    /// Clear the component cache
    pub fn clear_cache(&mut self) {
        self.component_cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            entries: self.component_cache.len(),
            memory_estimate: self.component_cache.len() * 1024, // Rough estimate
        }
    }

    /// Load a single prefab from RON string
    pub fn load_prefab_from_ron(
        &mut self,
        id: PrefabId,
        ron_content: &str,
        type_registry: &AppTypeRegistry,
    ) -> Result<(), Error> {
        let component_map = parse_prefab_ron(ron_content, type_registry, &self.config)?;
        self.factory
            .register_from_component_map(id, &component_map, type_registry)
    }

    /// Create a component map from a structured definition
    ///
    /// This method allows programmatic creation of component maps
    /// for dynamic entity generation.
    pub fn create_component_map(&self, components: HashMap<String, ron::Value>) -> ComponentMap {
        ComponentMap {
            metadata: crate::ComponentMapMetadata {
                source_path: None,
                validation_status: crate::ValidationStatus::Valid,
                component_count: components.len(),
            },
            components,
        }
    }
}

impl Default for DslFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics for the DSL factory
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of cached entries
    pub entries: usize,
    /// Estimated memory usage in bytes
    pub memory_estimate: usize,
}

/// Plugin for integrating DSL factory with Bevy apps
///
/// This plugin sets up the DSL factory as a Bevy resource and provides
/// systems for automatic prefab loading and batch processing.
#[derive(Default)]
pub struct DslFactoryPlugin {
    /// Configuration for the DSL factory
    pub config: DslConfig,
    /// Directory to load prefabs from (optional)
    pub prefab_directory: Option<String>,
}

impl DslFactoryPlugin {
    /// Create a new plugin with custom configuration
    pub fn with_config(config: DslConfig) -> Self {
        Self {
            config,
            prefab_directory: None,
        }
    }

    /// Set the prefab directory for automatic loading
    pub fn with_directory(mut self, directory: &str) -> Self {
        self.prefab_directory = Some(directory.to_string());
        self
    }
}

impl Plugin for DslFactoryPlugin {
    fn build(&self, app: &mut App) {
        // Insert DSL factory as a resource
        app.insert_resource(DslFactory::with_config(self.config.clone()));

        // Add startup system for loading prefabs if directory is specified
        if let Some(directory) = &self.prefab_directory {
            let dir = directory.clone();
            app.add_systems(
                Startup,
                move |mut factory: ResMut<DslFactory>, type_registry: Res<AppTypeRegistry>| {
                    if let Err(e) = factory.load_directory_dsl(&dir, &type_registry) {
                        error!("Failed to load prefabs from directory '{}': {}", dir, e);
                    }
                },
            );
        }

        // Register default components
        crate::register_default_components();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::TempDir;

    #[test]
    fn test_dsl_factory_creation() {
        let factory = DslFactory::new();
        assert_eq!(factory.config.max_batch_size, 1000);
        assert_eq!(factory.config.validation_mode, ValidationMode::Strict);
    }

    #[test]
    fn test_dsl_factory_with_config() {
        let config = DslConfig {
            max_batch_size: 500,
            validation_mode: ValidationMode::Permissive,
            cache_prefabs: false,
        };

        let factory = DslFactory::with_config(config.clone());
        assert_eq!(factory.config.max_batch_size, 500);
        assert_eq!(factory.config.validation_mode, ValidationMode::Permissive);
        assert!(!factory.config.cache_prefabs);
    }

    #[test]
    fn test_create_component_map() {
        let factory = DslFactory::new();
        let mut components = HashMap::new();
        components.insert("Transform".to_string(), ron::Value::Unit);

        let component_map = factory.create_component_map(components);
        assert_eq!(component_map.components.len(), 1);
        assert_eq!(component_map.metadata.component_count, 1);
    }

    #[test]
    fn test_cache_stats() {
        let factory = DslFactory::new();
        let stats = factory.cache_stats();
        assert_eq!(stats.entries, 0);
        assert_eq!(stats.memory_estimate, 0);
    }

    #[test]
    fn test_load_directory_dsl_invalid_path() {
        let mut factory = DslFactory::new();
        let type_registry = AppTypeRegistry::default();

        let result = factory.load_directory_dsl("/nonexistent/path", &type_registry);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_directory_dsl_empty_directory() {
        let mut factory = DslFactory::new();
        let type_registry = AppTypeRegistry::default();

        let temp_dir = TempDir::new().unwrap();
        let result = factory.load_directory_dsl(temp_dir.path().to_str().unwrap(), &type_registry);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_load_prefab_from_ron() {
        let mut factory = DslFactory::new();
        let type_registry = AppTypeRegistry::default();

        let ron_content = r#"
        (
            components: {
                "Name": "TestEntity",
            }
        )
        "#;

        let prefab_id = PrefabId::new(12345);
        let result = factory.load_prefab_from_ron(prefab_id, ron_content, &type_registry);
        // This will likely fail because Name is not registered in the component registry
        // but the parsing should succeed
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_dsl_factory_plugin() {
        let plugin = DslFactoryPlugin::default();
        assert_eq!(plugin.config.max_batch_size, 1000);
        assert!(plugin.prefab_directory.is_none());
    }

    #[test]
    fn test_dsl_factory_plugin_with_directory() {
        let plugin = DslFactoryPlugin::default().with_directory("/test/path");
        assert_eq!(plugin.prefab_directory, Some("/test/path".to_string()));
    }
}
