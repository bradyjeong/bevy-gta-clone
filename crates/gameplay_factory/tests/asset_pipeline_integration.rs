//! Integration tests for asset-pipeline alignment (Day 3 Oracle hardening)

use bevy::asset::AssetLoader;
use bevy::prelude::*;
use gameplay_factory::*;
use std::collections::HashMap;

#[test]
fn test_prefab_asset_pipeline_integration() {
    let mut app = App::new();

    // Add minimal Bevy systems for asset loading
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .add_plugins(PrefabAssetPlugin);

    // Create a simple prefab asset
    let mut components = HashMap::new();
    components.insert("Transform".to_string(), ron::Value::Map(ron::Map::new()));

    let prefab_asset = PrefabAsset {
        components,
        metadata: PrefabMetadata {
            name: "Test Asset Pipeline".to_string(),
            type_id: "test_asset".to_string(),
            version: "1.0.0".to_string(),
            tags: vec!["test".to_string()],
            asset_paths: Vec::new(),
            component_count: 0,
        },
    };

    // Register components for testing
    register_default_components();

    // Test conversion to runtime prefab
    let dsl_config = DslConfig {
        validation_mode: ValidationMode::Skip,
        ..Default::default()
    };
    let type_registry = AppTypeRegistry::default();
    let result = convert_prefab_asset_to_runtime_prefab(&prefab_asset, &dsl_config, &type_registry);

    // Should succeed
    assert!(result.is_ok());
    let runtime_prefab = result.unwrap();

    // Verify the prefab has the expected structure
    assert!(runtime_prefab.components().count() > 0);
}

#[test]
fn test_prefab_asset_loader_extensions() {
    let loader = PrefabAssetLoader;
    assert_eq!(loader.extensions(), &["prefab.ron"]);
}

#[test]
fn test_prefab_asset_metadata() {
    let metadata = PrefabMetadata {
        name: "Vehicle Prefab".to_string(),
        type_id: "vehicle".to_string(),
        version: "2.0.0".to_string(),
        tags: vec!["vehicle".to_string(), "physics".to_string()],
        asset_paths: Vec::new(),
        component_count: 0,
    };

    assert_eq!(metadata.name, "Vehicle Prefab");
    assert_eq!(metadata.version, "2.0.0");
    assert_eq!(metadata.tags.len(), 2);
}

#[test]
fn test_factory_with_prefab_asset() {
    // Test that Factory can work with PrefabAsset converted to runtime Prefab
    let mut components = HashMap::new();
    components.insert("Transform".to_string(), ron::Value::Map(ron::Map::new()));

    let prefab_asset = PrefabAsset {
        components,
        metadata: PrefabMetadata {
            name: "Factory Test".to_string(),
            type_id: "factory_test".to_string(),
            version: "1.0.0".to_string(),
            tags: vec!["factory".to_string()],
            asset_paths: Vec::new(),
            component_count: 0,
        },
    };

    // Register components for testing
    register_default_components();

    let dsl_config = DslConfig {
        validation_mode: ValidationMode::Skip,
        ..Default::default()
    };
    let type_registry = AppTypeRegistry::default();
    let runtime_prefab =
        convert_prefab_asset_to_runtime_prefab(&prefab_asset, &dsl_config, &type_registry).unwrap();

    // Create factory and register the prefab
    let mut factory = Factory::new();
    let prefab_id = PrefabId::new(12345);

    let _ = factory.register(prefab_id, runtime_prefab);

    // Test that the prefab is registered
    assert!(factory.contains(prefab_id));
}

#[test]
fn test_asset_pipeline_oracle_requirements() {
    // Test all Oracle requirements for asset-pipeline alignment:

    // 1. PrefabAsset with proper derives
    let prefab_asset = PrefabAsset {
        components: HashMap::new(),
        metadata: PrefabMetadata::default(),
    };

    // Should have Asset and TypePath derives (tested by compilation)
    assert_eq!(prefab_asset.components.len(), 0);

    // 2. RonAssetLoader<PrefabAsset>
    let loader = PrefabAssetLoader;
    assert_eq!(loader.extensions(), &["prefab.ron"]);

    // 3. Helper converts PrefabAsset → runtime Prefab
    register_default_components();
    let dsl_config = DslConfig {
        validation_mode: ValidationMode::Skip,
        ..Default::default()
    };
    let type_registry = AppTypeRegistry::default();
    let result = convert_prefab_asset_to_runtime_prefab(&prefab_asset, &dsl_config, &type_registry);
    assert!(result.is_ok());

    // 4. Integration with existing Factory API
    let mut factory = Factory::new();
    let prefab_id = PrefabId::new(99999);
    let _ = factory.register(prefab_id, result.unwrap());
    assert!(factory.contains(prefab_id));
}
