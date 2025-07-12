//! Basic tests for the prefab factory that don't rely on problematic macros

use bevy::prelude::*;
use gameplay_factory::{
    BasicPrefab, PrefabFactory, PrefabFactoryPlugin, PrefabId, SpawnPrefabEvent,
};

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
fn test_prefab_factory_basic() {
    let mut factory = PrefabFactory::new();
    let id = PrefabId::new(123);

    let prefab = BasicPrefab::with_metadata("TestPrefab".to_string(), "test".to_string());

    assert!(factory.register_prefab(id, "test_prefab", prefab).is_ok());
    assert!(factory.has_prefab("test_prefab"));
    assert_eq!(factory.prefab_count(), 1);
    assert_eq!(factory.get_prefab_names(), vec!["test_prefab".to_string()]);
}

#[test]
fn test_prefab_factory_bevy_app_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(PrefabFactoryPlugin);

    // Use Bevy 0.16.1 syntax for colors
    app.insert_resource(ClearColor(Color::srgb(1.0, 0.0, 0.0))); // Red color

    // Basic test - ensure the plugin was added
    app.update();
}

#[test]
fn test_spawning_events() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(PrefabFactoryPlugin);

    let id = PrefabId::new(42);
    app.world_mut().send_event(SpawnPrefabEvent::ById {
        id,
        transform: Some(Transform::default()),
    });

    app.update();
}

#[test]
fn test_spawn_config_bevy_app() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(PrefabFactoryPlugin);

    // Test spawning with event system
    let event = SpawnPrefabEvent::ByName {
        name: "test_prefab".to_string(),
        transform: Some(Transform::from_xyz(10.0, 20.0, 30.0)),
    };

    app.world_mut().send_event(event);
    app.update();
}
