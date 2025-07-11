//! Example demonstrating the Oracle-specified DSL system
//!
//! This example shows how to use the new DSL system with bevy_reflect
//! for dynamic entity creation from RON files.

use bevy::prelude::*;
use gameplay_factory::{
    BatchSpawnRequest, DslConfig, DslFactory, DslFactoryPlugin, FactoryDslExt, ValidationMode,
    parse_prefab_ron, spawn_many,
};
use std::collections::HashMap;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DslFactoryPlugin::with_config(DslConfig {
            max_batch_size: 100,
            validation_mode: ValidationMode::Permissive,
            cache_prefabs: true,
        }))
        .add_systems(Startup, setup_dsl_example)
        .add_systems(Update, demonstrate_batch_spawning)
        .run();
}

fn setup_dsl_example(mut factory: ResMut<DslFactory>, type_registry: Res<AppTypeRegistry>) {
    info!("Setting up DSL example...");

    // Example 1: Parse RON content directly
    let ron_content = r#"
    (
        components: {
            "Transform": (
                translation: (x: 1.0, y: 2.0, z: 3.0),
                rotation: (x: 0.0, y: 0.0, z: 0.0, w: 1.0),
                scale: (x: 1.0, y: 1.0, z: 1.0),
            ),
            "Name": "DslEntity",
            "Visibility": "Visible",
        }
    )
    "#;

    match parse_prefab_ron(ron_content, &type_registry, factory.config()) {
        Ok(component_map) => {
            info!("Successfully parsed RON content:");
            info!("  Components: {}", component_map.components.len());
            info!(
                "  Validation: {:?}",
                component_map.metadata.validation_status
            );

            // Register the prefab with the factory
            let prefab_id = gameplay_factory::PrefabId::new(12345);
            if let Err(e) = factory.factory_mut().register_from_component_map(
                prefab_id,
                &component_map,
                &type_registry,
            ) {
                error!("Failed to register prefab: {}", e);
            } else {
                info!("Registered prefab with ID: {}", prefab_id);
            }
        }
        Err(e) => error!("Failed to parse RON content: {}", e),
    }

    // Example 2: Create component map programmatically
    let mut components = HashMap::new();
    components.insert(
        "Name".to_string(),
        ron::Value::String("ProgrammaticEntity".to_string()),
    );

    let component_map = factory.create_component_map(components);
    info!(
        "Created programmatic component map with {} components",
        component_map.components.len()
    );

    // Example 3: Load from RON string
    let simple_ron = r#"
    (
        components: {
            "Name": "SimpleEntity",
        }
    )
    "#;

    let prefab_id = gameplay_factory::PrefabId::new(67890);
    match factory.load_prefab_from_ron(prefab_id, simple_ron, &type_registry) {
        Ok(()) => info!("Loaded prefab from RON string with ID: {}", prefab_id),
        Err(e) => error!("Failed to load prefab from RON string: {}", e),
    }

    // Display cache statistics
    let cache_stats = factory.cache_stats();
    info!(
        "Cache statistics: {} entries, ~{} bytes",
        cache_stats.entries, cache_stats.memory_estimate
    );
}

fn demonstrate_batch_spawning(
    mut commands: Commands,
    factory: Res<DslFactory>,
    type_registry: Res<AppTypeRegistry>,
    mut spawn_timer: Local<Timer>,
    time: Res<Time>,
) {
    // Only demonstrate batch spawning every 5 seconds
    spawn_timer.set_duration(std::time::Duration::from_secs(5));
    spawn_timer.tick(time.delta());

    if !spawn_timer.just_finished() {
        return;
    }

    info!("Demonstrating batch spawning...");

    // Create multiple component maps for batch spawning
    let mut entities = Vec::new();

    for i in 0..5 {
        let mut components = HashMap::new();
        components.insert(
            "Name".to_string(),
            ron::Value::String(format!("BatchEntity{i}")),
        );

        // Add Transform component
        let transform_ron = ron::Value::Map({
            let mut map = ron::Map::new();

            // Translation
            let mut translation_map = ron::Map::new();
            translation_map.insert(
                ron::Value::String("x".to_string()),
                ron::Value::Number(ron::Number::new(i as f64)),
            );
            translation_map.insert(
                ron::Value::String("y".to_string()),
                ron::Value::Number(ron::Number::new(0.0)),
            );
            translation_map.insert(
                ron::Value::String("z".to_string()),
                ron::Value::Number(ron::Number::new(0.0)),
            );

            map.insert(
                ron::Value::String("translation".to_string()),
                ron::Value::Map(translation_map),
            );

            // Rotation (identity quaternion)
            let mut rotation_map = ron::Map::new();
            rotation_map.insert(
                ron::Value::String("x".to_string()),
                ron::Value::Number(ron::Number::new(0.0)),
            );
            rotation_map.insert(
                ron::Value::String("y".to_string()),
                ron::Value::Number(ron::Number::new(0.0)),
            );
            rotation_map.insert(
                ron::Value::String("z".to_string()),
                ron::Value::Number(ron::Number::new(0.0)),
            );
            rotation_map.insert(
                ron::Value::String("w".to_string()),
                ron::Value::Number(ron::Number::new(1.0)),
            );

            map.insert(
                ron::Value::String("rotation".to_string()),
                ron::Value::Map(rotation_map),
            );

            // Scale
            let mut scale_map = ron::Map::new();
            scale_map.insert(
                ron::Value::String("x".to_string()),
                ron::Value::Number(ron::Number::new(1.0)),
            );
            scale_map.insert(
                ron::Value::String("y".to_string()),
                ron::Value::Number(ron::Number::new(1.0)),
            );
            scale_map.insert(
                ron::Value::String("z".to_string()),
                ron::Value::Number(ron::Number::new(1.0)),
            );

            map.insert(
                ron::Value::String("scale".to_string()),
                ron::Value::Map(scale_map),
            );

            map
        });

        components.insert("Transform".to_string(), transform_ron);

        let component_map = factory.create_component_map(components);
        entities.push(component_map);
    }

    // Perform batch spawning
    match factory.spawn_batch_dsl(&mut commands, entities, &type_registry) {
        Ok(result) => {
            info!("Batch spawn results:");
            info!("  Spawned: {} entities", result.spawned.len());
            info!("  Failed: {} entities", result.failed.len());
            info!("  Total time: {:?}", result.metrics.total_time);
            info!("  Time per entity: {:?}", result.metrics.time_per_entity);
            info!(
                "  Components processed: {}",
                result.metrics.components_processed
            );
            info!("  Memory used: ~{} bytes", result.metrics.memory_used);

            // Log any failures
            for (index, error) in result.failed {
                error!("Entity {} failed to spawn: {}", index, error);
            }
        }
        Err(e) => error!("Batch spawn failed: {}", e),
    }
}

/// Example system showing how to use the DSL with validation modes
#[allow(dead_code)]
fn demonstrate_validation_modes(type_registry: Res<AppTypeRegistry>) {
    info!("Demonstrating validation modes...");

    // Valid RON content
    let valid_ron = r#"
    (
        components: {
            "Name": "ValidEntity",
        }
    )
    "#;

    // Invalid RON content (missing quotes)
    let invalid_ron = r#"
    (
        components: {
            Name: "InvalidEntity",
        }
    )
    "#;

    // Test with strict validation
    let strict_config = DslConfig {
        validation_mode: ValidationMode::Strict,
        ..Default::default()
    };

    match parse_prefab_ron(valid_ron, &type_registry, &strict_config) {
        Ok(_) => info!("Strict validation passed for valid RON"),
        Err(e) => error!("Strict validation failed for valid RON: {}", e),
    }

    match parse_prefab_ron(invalid_ron, &type_registry, &strict_config) {
        Ok(_) => info!("Strict validation unexpectedly passed for invalid RON"),
        Err(e) => info!("Strict validation correctly failed for invalid RON: {}", e),
    }

    // Test with permissive validation
    let permissive_config = DslConfig {
        validation_mode: ValidationMode::Permissive,
        ..Default::default()
    };

    match parse_prefab_ron(valid_ron, &type_registry, &permissive_config) {
        Ok(_) => info!("Permissive validation passed for valid RON"),
        Err(e) => error!("Permissive validation failed for valid RON: {}", e),
    }

    // Test with no validation
    let skip_config = DslConfig {
        validation_mode: ValidationMode::Skip,
        ..Default::default()
    };

    match parse_prefab_ron(valid_ron, &type_registry, &skip_config) {
        Ok(_) => info!("Skip validation passed for valid RON"),
        Err(e) => error!("Skip validation failed for valid RON: {}", e),
    }
}

/// Example system showing manual DSL usage without the factory
#[allow(dead_code)]
fn demonstrate_manual_dsl(mut commands: Commands, type_registry: Res<AppTypeRegistry>) {
    info!("Demonstrating manual DSL usage...");

    let ron_content = r#"
    (
        components: {
            "Name": "ManualEntity",
        }
    )
    "#;

    let config = DslConfig::default();

    match parse_prefab_ron(ron_content, &type_registry, &config) {
        Ok(component_map) => {
            // Create a batch spawn request
            let request = BatchSpawnRequest {
                entities: vec![component_map],
                config: config.clone(),
            };

            // Spawn the entities
            match spawn_many(&mut commands, request, &type_registry) {
                Ok(result) => {
                    info!("Manual DSL spawned {} entities", result.spawned.len());
                }
                Err(e) => error!("Manual DSL spawn failed: {}", e),
            }
        }
        Err(e) => error!("Manual DSL parse failed: {}", e),
    }
}
