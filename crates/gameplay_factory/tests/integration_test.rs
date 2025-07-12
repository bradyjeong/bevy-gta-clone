// Integration tests re-enabled for Bevy 0.16.1 migration

use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use gameplay_factory::*;
use serial_test::serial;

// Simple test components for testing
#[derive(Component, Debug, PartialEq)]
pub struct TestTransform {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Component, Debug, PartialEq)]
pub struct TestName(pub String);

#[derive(Component, Debug, PartialEq)]
pub struct TestVisibility(pub bool);

#[test]
fn test_component_registry_initialization() {
    // Initialize the component registry
    register_default_components();
    clear_all_prefab_ids();

    // This test just verifies that the registry can be initialized
    // without panicking
    // Test passes if we reach this point without panicking
}

#[test]
#[serial]
fn test_factory_creation() {
    // Create a factory
    let factory = Factory::new();

    // Test that it can be created without errors
    assert_eq!(factory.len(), 0);
    assert!(factory.is_empty());
}

#[test]
#[serial]
fn test_basic_entity_spawning() {
    let world = World::new();
    let mut factory = Factory::new();

    // Create a basic prefab
    let prefab_id = PrefabId::new(12345);

    // Register a simple prefab
    let result = factory.register(prefab_id, BasicPrefab::new());
    assert!(result.is_ok());

    // Try to spawn an entity
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &world);
    let result = factory.spawn(&mut commands, prefab_id);

    // This should work even with an empty prefab
    assert!(result.is_ok());
}

#[test]
#[serial]
fn test_component_registry_static_functions() {
    use std::thread;

    let handles: Vec<_> = (0..4)
        .map(|i| {
            thread::spawn(move || {
                // Each thread will register a component with a static name
                // to avoid lifetime issues
                let result = match i {
                    0 => register_component(
                        "TestComponent0",
                        Box::new(move |_value, _commands, _entity| Ok(())),
                    ),
                    1 => register_component(
                        "TestComponent1",
                        Box::new(move |_value, _commands, _entity| Ok(())),
                    ),
                    2 => register_component(
                        "TestComponent2",
                        Box::new(move |_value, _commands, _entity| Ok(())),
                    ),
                    _ => register_component(
                        "TestComponent3",
                        Box::new(move |_value, _commands, _entity| Ok(())),
                    ),
                };
                assert!(result.is_ok());
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // This test just verifies no panics occurred
    // Test passes if we reach this point without panicking
}

#[test]
#[serial]
fn test_error_handling() {
    let factory = Factory::new();
    let world = World::new();
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &world);

    // Try to spawn a non-existent prefab
    let result = factory.spawn(&mut commands, PrefabId::new(99999));
    assert!(result.is_err());

    // Check that the error is meaningful
    let error = result.unwrap_err();
    assert!(error.to_string().contains("not found"));
}

#[test]
#[serial]
fn test_clear_functionality() {
    // Add some test data
    register_default_components();

    // Clear and verify
    clear_all_prefab_ids();
    let tracked_ids = get_all_prefab_ids();
    assert!(tracked_ids.is_empty());
}

#[test]
#[serial]
fn test_prefab_contains_check() {
    let mut factory = Factory::new();
    let prefab_id = PrefabId::new(11111);

    // Initially should not contain the prefab
    assert!(!factory.contains(prefab_id));

    // After registration, should contain it
    factory.register(prefab_id, BasicPrefab::new()).unwrap();
    assert!(factory.contains(prefab_id));
}

#[test]
#[serial]
fn test_prefab_id_global_tracking() {
    clear_all_prefab_ids();

    let prefab_id = PrefabId::new(22222);
    assert!(!is_prefab_id_registered(prefab_id));

    // After registering in a factory, it should be globally tracked
    let mut factory = Factory::new();
    factory.register(prefab_id, BasicPrefab::new()).unwrap();
    assert!(is_prefab_id_registered(prefab_id));

    // Verify it's in the global list
    let all_ids = get_all_prefab_ids();
    assert!(all_ids.contains(&prefab_id));
}
