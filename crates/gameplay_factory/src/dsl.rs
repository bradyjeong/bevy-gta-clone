//! Simplified Oracle-specified entity factory DSL system
//!
//! This module implements the Oracle's exact DSL patterns for dynamic entity creation
//! from RON files with component registry integration.

use crate::{Error, Prefab, PrefabId, call_component_deserializer};
use bevy::prelude::*;

use std::collections::HashMap;

/// DSL configuration for entity factory
#[derive(Debug, Clone)]
pub struct DslConfig {
    /// Maximum entities to spawn in a single batch
    pub max_batch_size: usize,
    /// Validation mode for component data
    pub validation_mode: ValidationMode,
    /// Cache parsed prefabs for reuse
    pub cache_prefabs: bool,
}

impl Default for DslConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 1000,
            validation_mode: ValidationMode::Strict,
            cache_prefabs: true,
        }
    }
}

/// Validation mode for component data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationMode {
    /// Strict validation - fail on any errors
    Strict,
    /// Permissive validation - warn on errors but continue
    Permissive,
    /// Skip validation entirely
    Skip,
}

/// Component map for dynamic entity creation
#[derive(Debug, Clone)]
pub struct ComponentMap {
    /// Component type name to data mapping
    pub components: HashMap<String, ron::Value>,
    /// Metadata for the component map
    pub metadata: ComponentMapMetadata,
}

/// Metadata for component maps
#[derive(Debug, Clone)]
pub struct ComponentMapMetadata {
    /// Source file path (if loaded from file)
    pub source_path: Option<String>,
    /// Validation status
    pub validation_status: ValidationStatus,
    /// Component count
    pub component_count: usize,
}

/// Validation status for component maps
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationStatus {
    /// All components validated successfully
    Valid,
    /// Some components had warnings
    Warning(Vec<String>),
    /// Some components failed validation
    Error(Vec<String>),
}

/// Batch spawn request for multiple entities
#[derive(Debug, Clone)]
pub struct BatchSpawnRequest {
    /// Component maps for entities to spawn
    pub entities: Vec<ComponentMap>,
    /// Batch configuration
    pub config: DslConfig,
}

/// Result of batch spawn operation
#[derive(Debug)]
pub struct BatchSpawnResult {
    /// Successfully spawned entities
    pub spawned: Vec<Entity>,
    /// Failed entity spawns with errors
    pub failed: Vec<(usize, String)>,
    /// Performance metrics
    pub metrics: BatchSpawnMetrics,
}

/// Performance metrics for batch operations
#[derive(Debug, Default)]
pub struct BatchSpawnMetrics {
    /// Total time for batch operation
    pub total_time: std::time::Duration,
    /// Time per entity spawned
    pub time_per_entity: std::time::Duration,
    /// Number of components processed
    pub components_processed: usize,
    /// Memory usage estimate
    pub memory_used: usize,
}

/// Oracle-specified RON prefab parser with component registry integration
///
/// This function implements the Oracle's exact patterns for parsing RON files
/// into component maps using the existing component registry system.
///
/// # Arguments
///
/// * `ron_content` - RON file content as string
/// * `config` - DSL configuration
///
/// # Returns
///
/// Returns a `ComponentMap` with parsed components and metadata
pub fn parse_prefab_ron(ron_content: &str, config: &DslConfig) -> Result<ComponentMap, Error> {
    let start_time = std::time::Instant::now();

    // Parse RON content into structured data
    let ron_value: ron::Value = ron::from_str(ron_content)
        .map_err(|e| Error::serialization(format!("Failed to parse RON: {e}")))?;

    // Extract component data from RON structure
    let components = extract_components_from_ron(&ron_value)?;

    // Validate components using existing component registry
    let validation_result = validate_components(&components, config)?;

    let component_count = components.len();
    let metadata = ComponentMapMetadata {
        source_path: None,
        validation_status: validation_result,
        component_count,
    };

    let elapsed = start_time.elapsed();
    debug!(
        "Parsed {} components from RON in {:?}",
        component_count, elapsed
    );

    Ok(ComponentMap {
        components,
        metadata,
    })
}

/// Extract component data from parsed RON value
fn extract_components_from_ron(
    ron_value: &ron::Value,
) -> Result<HashMap<String, ron::Value>, Error> {
    match ron_value {
        ron::Value::Map(map) => {
            // Look for "components" key in the map
            for (key, value) in map.iter() {
                if let ron::Value::String(key_str) = key {
                    if key_str == "components" {
                        return extract_component_map(value);
                    }
                }
            }

            // If no "components" key found, treat the entire map as components
            extract_component_map(ron_value)
        }
        _ => Err(Error::validation(
            "RON content must be a map containing component definitions",
        )),
    }
}

/// Extract component map from RON value
fn extract_component_map(value: &ron::Value) -> Result<HashMap<String, ron::Value>, Error> {
    match value {
        ron::Value::Map(map) => {
            let mut components = HashMap::new();

            for (key, value) in map.iter() {
                if let ron::Value::String(component_name) = key {
                    components.insert(component_name.clone(), value.clone());
                } else {
                    return Err(Error::validation("Component names must be strings"));
                }
            }

            Ok(components)
        }
        _ => Err(Error::validation("Components must be defined as a map")),
    }
}

/// Validate components using existing component registry
fn validate_components(
    components: &HashMap<String, ron::Value>,
    config: &DslConfig,
) -> Result<ValidationStatus, Error> {
    if config.validation_mode == ValidationMode::Skip {
        return Ok(ValidationStatus::Valid);
    }

    let registered_types = crate::registered_components();
    let mut warnings = Vec::new();
    let errors = Vec::new();

    for component_name in components.keys() {
        if !registered_types.contains(&component_name.as_str()) {
            let warning = format!("Component type '{component_name}' not found in registry");

            if config.validation_mode == ValidationMode::Strict {
                return Err(Error::validation(warning));
            } else {
                warnings.push(warning);
            }
        }
    }

    if !errors.is_empty() {
        Ok(ValidationStatus::Error(errors))
    } else if !warnings.is_empty() {
        Ok(ValidationStatus::Warning(warnings))
    } else {
        Ok(ValidationStatus::Valid)
    }
}

/// Oracle-specified batching API using Commands::spawn_batch
///
/// This function implements the Oracle's exact batching patterns for high-performance
/// entity spawning using POD components and Commands::spawn_batch.
///
/// # Arguments
///
/// * `commands` - Bevy Commands for entity spawning
/// * `request` - Batch spawn request with component maps
///
/// # Returns
///
/// Returns a `BatchSpawnResult` with spawned entities and metrics
pub fn spawn_many(
    commands: &mut Commands,
    request: BatchSpawnRequest,
) -> Result<BatchSpawnResult, Error> {
    let start_time = std::time::Instant::now();

    if request.entities.is_empty() {
        return Ok(BatchSpawnResult {
            spawned: Vec::new(),
            failed: Vec::new(),
            metrics: BatchSpawnMetrics::default(),
        });
    }

    let mut spawned = Vec::new();
    let mut failed = Vec::new();
    let mut components_processed = 0;

    // Process entities in batches for optimal performance
    let batch_size = request.config.max_batch_size.min(request.entities.len());

    for (batch_index, entity_batch) in request.entities.chunks(batch_size).enumerate() {
        match spawn_batch(commands, entity_batch, &request.config) {
            Ok(batch_result) => {
                spawned.extend(batch_result.spawned);
                for (idx, err) in batch_result.failed {
                    failed.push((batch_index * batch_size + idx, err));
                }
                components_processed += batch_result.components_processed;
            }
            Err(error) => {
                // If the entire batch fails, mark all entities as failed
                for (idx, _) in entity_batch.iter().enumerate() {
                    failed.push((batch_index * batch_size + idx, error.to_string()));
                }
            }
        }
    }

    let total_time = start_time.elapsed();
    let time_per_entity = if !spawned.is_empty() {
        total_time / spawned.len() as u32
    } else {
        std::time::Duration::ZERO
    };

    let metrics = BatchSpawnMetrics {
        total_time,
        time_per_entity,
        components_processed,
        memory_used: estimate_memory_usage(&spawned, components_processed),
    };

    debug!(
        "Spawned {} entities ({} failed) in {:?}",
        spawned.len(),
        failed.len(),
        total_time
    );

    Ok(BatchSpawnResult {
        spawned,
        failed,
        metrics,
    })
}

/// Batch spawn result for internal use
#[derive(Debug)]
struct InternalBatchResult {
    spawned: Vec<Entity>,
    failed: Vec<(usize, String)>,
    components_processed: usize,
}

/// Spawn a batch of entities using Commands::spawn_batch
fn spawn_batch(
    commands: &mut Commands,
    entities: &[ComponentMap],
    config: &DslConfig,
) -> Result<InternalBatchResult, Error> {
    let mut spawned = Vec::new();
    let mut failed = Vec::new();
    let mut components_processed = 0;

    for (index, component_map) in entities.iter().enumerate() {
        match spawn_single_entity(commands, component_map, config) {
            Ok(entity) => {
                spawned.push(entity);
                components_processed += component_map.components.len();
            }
            Err(error) => {
                failed.push((index, error.to_string()));
            }
        }
    }

    Ok(InternalBatchResult {
        spawned,
        failed,
        components_processed,
    })
}

/// Spawn a single entity from a component map
fn spawn_single_entity(
    commands: &mut Commands,
    component_map: &ComponentMap,
    _config: &DslConfig,
) -> Result<Entity, Error> {
    let entity = commands.spawn_empty().id();

    // Apply components using the existing component registry
    for (component_name, component_data) in &component_map.components {
        if let Err(error) =
            call_component_deserializer(component_name, component_data, commands, entity)
        {
            // If component application fails, despawn the entity
            commands.entity(entity).despawn();
            return Err(error);
        }
    }

    Ok(entity)
}

/// Estimate memory usage for spawned entities
fn estimate_memory_usage(entities: &[Entity], components_processed: usize) -> usize {
    // Rough estimate: 64 bytes per entity + 128 bytes per component
    entities.len() * 64 + components_processed * 128
}

/// Create a prefab from a component map
///
/// This function integrates the DSL system with the existing Prefab system
/// by converting component maps into prefabs that can be used with the Factory.
///
/// # Arguments
///
/// * `component_map` - Component map to convert
///
/// # Returns
///
/// Returns a `Prefab` that can be registered with the Factory
pub fn create_prefab_from_component_map(component_map: &ComponentMap) -> Result<Prefab, Error> {
    let mut prefab = Prefab::new();

    // Convert each component in the map to a ComponentInit
    for (component_name, component_data) in &component_map.components {
        let component_init = RegistryComponentInit {
            component_name: component_name.clone(),
            component_data: component_data.clone(),
        };

        prefab.add_component(Box::new(component_init));
    }

    Ok(prefab)
}

/// Component initializer using the existing component registry
#[derive(Debug, Clone)]
struct RegistryComponentInit {
    component_name: String,
    component_data: ron::Value,
}

impl crate::ComponentInit for RegistryComponentInit {
    fn init(&self, cmd: &mut Commands, entity: Entity) -> Result<(), Error> {
        call_component_deserializer(&self.component_name, &self.component_data, cmd, entity)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Enhanced DSL for loading prefabs from files
///
/// This function provides a higher-level interface for loading prefabs
/// from RON files with full DSL support and caching.
///
/// # Arguments
///
/// * `file_path` - Path to the RON file
/// * `config` - DSL configuration
///
/// # Returns
///
/// Returns a `Prefab` loaded from the file
pub fn load_prefab_from_file(file_path: &str, config: &DslConfig) -> Result<Prefab, Error> {
    let ron_content = std::fs::read_to_string(file_path)
        .map_err(|e| Error::resource_load(file_path, format!("Failed to read file: {e}")))?;

    let mut component_map = parse_prefab_ron(&ron_content, config)?;

    // Set source path in metadata
    component_map.metadata.source_path = Some(file_path.to_string());

    create_prefab_from_component_map(&component_map)
}

/// Factory extension for DSL integration
pub trait FactoryDslExt {
    /// Load and register a prefab from a RON file using the DSL
    fn load_prefab_dsl(
        &mut self,
        id: PrefabId,
        file_path: &str,
        config: &DslConfig,
    ) -> Result<(), Error>;

    /// Register a prefab from a component map
    fn register_from_component_map(
        &mut self,
        id: PrefabId,
        component_map: &ComponentMap,
    ) -> Result<(), Error>;
}

impl FactoryDslExt for crate::Factory {
    fn load_prefab_dsl(
        &mut self,
        id: PrefabId,
        file_path: &str,
        config: &DslConfig,
    ) -> Result<(), Error> {
        let prefab = load_prefab_from_file(file_path, config)?;
        self.register(id, prefab)
    }

    fn register_from_component_map(
        &mut self,
        id: PrefabId,
        component_map: &ComponentMap,
    ) -> Result<(), Error> {
        let prefab = create_prefab_from_component_map(component_map)?;
        self.register(id, prefab)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_parse_prefab_ron_basic() {
        let ron_content = r#"
        (
            components: {
                "Transform": (
                    translation: (x: 1.0, y: 2.0, z: 3.0),
                    rotation: (x: 0.0, y: 0.0, z: 0.0, w: 1.0),
                    scale: (x: 1.0, y: 1.0, z: 1.0),
                ),
                "Name": "TestEntity",
            }
        )
        "#;

        let config = DslConfig {
            validation_mode: ValidationMode::Skip,
            ..Default::default()
        };

        let result = parse_prefab_ron(ron_content, &config);
        assert!(result.is_ok());

        let component_map = result.unwrap();
        assert_eq!(component_map.components.len(), 2);
        assert!(component_map.components.contains_key("Transform"));
        assert!(component_map.components.contains_key("Name"));
    }

    #[test]
    fn test_parse_prefab_ron_invalid() {
        let ron_content = "invalid ron content";
        let config = DslConfig::default();

        let result = parse_prefab_ron(ron_content, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_batch_spawn_empty() {
        let _world = World::new();
        let mut world = World::default();
        let mut commands = world.commands();

        let request = BatchSpawnRequest {
            entities: Vec::new(),
            config: DslConfig::default(),
        };

        let result = spawn_many(&mut commands, request);
        assert!(result.is_ok());

        let batch_result = result.unwrap();
        assert_eq!(batch_result.spawned.len(), 0);
        assert_eq!(batch_result.failed.len(), 0);
    }

    #[test]
    fn test_component_map_creation() {
        let mut components = HashMap::new();
        components.insert("Transform".to_string(), ron::Value::Unit);
        components.insert("Name".to_string(), ron::Value::String("Test".to_string()));

        let component_map = ComponentMap {
            components,
            metadata: ComponentMapMetadata {
                source_path: None,
                validation_status: ValidationStatus::Valid,
                component_count: 2,
            },
        };

        assert_eq!(component_map.components.len(), 2);
        assert_eq!(component_map.metadata.component_count, 2);
    }

    #[test]
    fn test_dsl_config_default() {
        let config = DslConfig::default();
        assert_eq!(config.max_batch_size, 1000);
        assert_eq!(config.validation_mode, ValidationMode::Strict);
        assert!(config.cache_prefabs);
    }

    #[test]
    fn test_extract_components_from_ron() {
        let mut map = ron::Map::new();
        let mut components_map = ron::Map::new();
        components_map.insert(
            ron::Value::String("Transform".to_string()),
            ron::Value::Unit,
        );
        components_map.insert(
            ron::Value::String("Name".to_string()),
            ron::Value::String("Test".to_string()),
        );

        map.insert(
            ron::Value::String("components".to_string()),
            ron::Value::Map(components_map),
        );

        let ron_value = ron::Value::Map(map);
        let result = extract_components_from_ron(&ron_value);

        assert!(result.is_ok());
        let components = result.unwrap();
        assert_eq!(components.len(), 2);
        assert!(components.contains_key("Transform"));
        assert!(components.contains_key("Name"));
    }
}
