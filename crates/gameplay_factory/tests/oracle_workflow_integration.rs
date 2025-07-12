//! Oracle-specified comprehensive integration test for gameplay_factory
//!
//! This test verifies the complete workflow identified by Oracle assessment:
//! 1. Register default components
//! 2. Load a prefab RON file containing components (e.g., Transform)
//! 3. Spawn the entity using the factory
//! 4. Assert the Transform component is present in the ECS world
//!
//! This test is designed to catch the bugs that were just fixed:
//! - Factory::spawn() creating empty prefabs instead of using loaded ones
//! - Factory::load_from_source() ignoring loaded prefab data
//! - load_prefab_file() being non-functional

use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use gameplay_factory::*;
use serial_test::serial;
use std::fs;
use tempfile::TempDir;

/// Test the complete Oracle-specified workflow end-to-end
#[test]
#[serial]
fn test_oracle_complete_workflow() {
    // Step 1: Register default components
    register_default_components();
    clear_all_prefab_ids();

    // Step 2: Create a test RON file with realistic component data
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ron_file_path = temp_dir.path().join("test_entity.ron");

    let ron_content = r#"
(
    components: {
        "Transform": (
            translation: (x: 10.0, y: 20.0, z: 30.0),
            rotation: (x: 0.0, y: 0.0, z: 0.0, w: 1.0),
            scale: (x: 2.0, y: 2.0, z: 2.0),
        ),
        "Name": "TestEntity",
        "Visibility": "Visible",
    }
)
"#;

    fs::write(&ron_file_path, ron_content).expect("Failed to write RON file");

    // Step 3: Load the prefab from file using the factory
    let mut factory = Factory::new();
    let prefab_id = PrefabId::new(42);

    // Create a file-based prefab source for testing
    struct TestPrefabSource {
        file_path: std::path::PathBuf,
    }

    impl PrefabSource for TestPrefabSource {
        fn load(&self) -> Result<BasicPrefab, Error> {
            let content = std::fs::read_to_string(&self.file_path).map_err(|e| {
                Error::resource_load(
                    format!("prefab file {}", self.file_path.display()),
                    format!("IO error: {e}"),
                )
            })?;

            // Use the DSL system to load the prefab
            let config = DslConfig {
                validation_mode: ValidationMode::Skip,
                ..Default::default()
            };
            let type_registry = AppTypeRegistry::default();
            let component_map = parse_prefab_ron(&content, &type_registry, &config)?;
            create_prefab_from_component_map(&component_map, &type_registry)
        }
    }

    let source = TestPrefabSource {
        file_path: ron_file_path.clone(),
    };

    // Load the prefab from the source
    let load_result = factory.load_from_source(prefab_id, &source);
    assert!(
        load_result.is_ok(),
        "Failed to load prefab from source: {:?}",
        load_result.err()
    );

    // Verify the prefab was actually loaded
    assert!(
        factory.contains(prefab_id),
        "Factory should contain the loaded prefab"
    );
    assert!(
        !factory.is_empty(),
        "Factory should not be empty after loading"
    );
    assert_eq!(
        factory.len(),
        1,
        "Factory should contain exactly one prefab"
    );

    // Step 4: Spawn the entity using the factory
    let mut world = World::new();
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &world);

    let spawn_result = factory.spawn(&mut commands, prefab_id);
    assert!(
        spawn_result.is_ok(),
        "Failed to spawn entity: {:?}",
        spawn_result.err()
    );

    let entity = spawn_result.unwrap();

    // Apply the commands to the world
    queue.apply(&mut world);

    // Step 5: Assert the Transform component is present in the ECS world
    let has_transform = world.entity(entity).contains::<Transform>();
    assert!(
        has_transform,
        "Spawned entity should have Transform component"
    );

    // Step 6: Verify component values are correct
    let entity_ref = world.entity(entity);
    let transform = entity_ref
        .get::<Transform>()
        .expect("Transform component should be present");

    assert_eq!(
        transform.translation.x, 10.0,
        "Transform translation.x should be 10.0"
    );
    assert_eq!(
        transform.translation.y, 20.0,
        "Transform translation.y should be 20.0"
    );
    assert_eq!(
        transform.translation.z, 30.0,
        "Transform translation.z should be 30.0"
    );
    assert_eq!(transform.scale.x, 2.0, "Transform scale.x should be 2.0");
    assert_eq!(transform.scale.y, 2.0, "Transform scale.y should be 2.0");
    assert_eq!(transform.scale.z, 2.0, "Transform scale.z should be 2.0");

    // Step 7: Verify other components are also present
    let has_name = world.entity(entity).contains::<Name>();
    assert!(has_name, "Spawned entity should have Name component");

    let name = entity_ref
        .get::<Name>()
        .expect("Name component should be present");
    assert_eq!(
        name.as_str(),
        "TestEntity",
        "Name component should have correct value"
    );

    let has_visibility = world.entity(entity).contains::<Visibility>();
    assert!(
        has_visibility,
        "Spawned entity should have Visibility component"
    );

    let visibility = entity_ref
        .get::<Visibility>()
        .expect("Visibility component should be present");
    assert_eq!(
        *visibility,
        Visibility::Visible,
        "Visibility component should be Visible"
    );

    println!("Oracle workflow integration test passed successfully!");
}

/// Test that the factory handles multiple prefabs correctly
#[test]
#[serial]
fn test_multiple_prefabs_workflow() {
    register_default_components();
    clear_all_prefab_ids();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create multiple RON files with different component data
    let ron_files = vec![
        (
            "entity1.ron",
            r#"
(
    components: {
        "Transform": (
            translation: (x: 1.0, y: 2.0, z: 3.0),
            rotation: (x: 0.0, y: 0.0, z: 0.0, w: 1.0),
            scale: (x: 1.0, y: 1.0, z: 1.0),
        ),
        "Name": "Entity1",
    }
)
"#,
        ),
        (
            "entity2.ron",
            r#"
(
    components: {
        "Transform": (
            translation: (x: 100.0, y: 200.0, z: 300.0),
            rotation: (x: 0.0, y: 0.0, z: 0.0, w: 1.0),
            scale: (x: 3.0, y: 3.0, z: 3.0),
        ),
        "Name": "Entity2",
        "Visibility": "Hidden",
    }
)
"#,
        ),
    ];

    // Write RON files
    for (filename, content) in &ron_files {
        let file_path = temp_dir.path().join(filename);
        fs::write(&file_path, content).expect("Failed to write RON file");
    }

    // Load prefabs using the factory
    let mut factory = Factory::new();
    let mut loaded_prefabs = Vec::new();

    for (i, (filename, _)) in ron_files.iter().enumerate() {
        let file_path = temp_dir.path().join(filename);
        let prefab_id = PrefabId::new(100 + i as u64);

        struct TestPrefabSource {
            file_path: std::path::PathBuf,
        }

        impl PrefabSource for TestPrefabSource {
            fn load(&self) -> Result<BasicPrefab, Error> {
                let content = std::fs::read_to_string(&self.file_path).map_err(|e| {
                    Error::resource_load(
                        format!("prefab file {}", self.file_path.display()),
                        format!("IO error: {e}"),
                    )
                })?;

                let config = DslConfig {
                    validation_mode: ValidationMode::Skip,
                    ..Default::default()
                };
                let type_registry = AppTypeRegistry::default();
                let component_map = parse_prefab_ron(&content, &type_registry, &config)?;
                create_prefab_from_component_map(&component_map, &type_registry)
            }
        }

        let source = TestPrefabSource { file_path };
        factory
            .load_from_source(prefab_id, &source)
            .expect("Failed to load prefab");
        loaded_prefabs.push(prefab_id);
    }

    // Verify all prefabs were loaded
    assert_eq!(factory.len(), 2, "Factory should contain 2 prefabs");
    for &prefab_id in &loaded_prefabs {
        assert!(
            factory.contains(prefab_id),
            "Factory should contain prefab {prefab_id:?}"
        );
    }

    // Spawn entities and verify they have different component values
    let mut world = World::new();
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &world);

    let entity1 = factory
        .spawn(&mut commands, loaded_prefabs[0])
        .expect("Failed to spawn entity1");
    let entity2 = factory
        .spawn(&mut commands, loaded_prefabs[1])
        .expect("Failed to spawn entity2");

    queue.apply(&mut world);

    // Check entity1 components
    let entity1_ref = world.entity(entity1);
    let transform1 = entity1_ref
        .get::<Transform>()
        .expect("Entity1 should have Transform");
    assert_eq!(
        transform1.translation.x, 1.0,
        "Entity1 translation.x should be 1.0"
    );
    assert_eq!(
        transform1.translation.y, 2.0,
        "Entity1 translation.y should be 2.0"
    );
    assert_eq!(
        transform1.translation.z, 3.0,
        "Entity1 translation.z should be 3.0"
    );

    let name1 = entity1_ref.get::<Name>().expect("Entity1 should have Name");
    assert_eq!(
        name1.as_str(),
        "Entity1",
        "Entity1 name should be 'Entity1'"
    );

    // Check entity2 components
    let entity2_ref = world.entity(entity2);
    let transform2 = entity2_ref
        .get::<Transform>()
        .expect("Entity2 should have Transform");
    assert_eq!(
        transform2.translation.x, 100.0,
        "Entity2 translation.x should be 100.0"
    );
    assert_eq!(
        transform2.translation.y, 200.0,
        "Entity2 translation.y should be 200.0"
    );
    assert_eq!(
        transform2.translation.z, 300.0,
        "Entity2 translation.z should be 300.0"
    );

    let name2 = entity2_ref.get::<Name>().expect("Entity2 should have Name");
    assert_eq!(
        name2.as_str(),
        "Entity2",
        "Entity2 name should be 'Entity2'"
    );

    let visibility2 = entity2_ref
        .get::<Visibility>()
        .expect("Entity2 should have Visibility");
    assert_eq!(
        *visibility2,
        Visibility::Hidden,
        "Entity2 visibility should be Hidden"
    );

    println!("Multiple prefabs workflow integration test passed successfully!");
}

/// Test error handling in the complete workflow
#[test]
#[serial]
fn test_workflow_error_handling() {
    register_default_components();
    clear_all_prefab_ids();

    let mut factory = Factory::new();
    let world = World::new();
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &world);

    // Test 1: Spawn non-existent prefab
    let non_existent_id = PrefabId::new(999);
    let spawn_result = factory.spawn(&mut commands, non_existent_id);
    assert!(
        spawn_result.is_err(),
        "Spawning non-existent prefab should fail"
    );

    let error = spawn_result.unwrap_err();
    assert!(
        error.to_string().contains("not found"),
        "Error should mention 'not found'"
    );

    // Test 2: Load from non-existent file
    struct BadPrefabSource;

    impl PrefabSource for BadPrefabSource {
        fn load(&self) -> Result<BasicPrefab, Error> {
            Err(Error::resource_load("test file", "File not found"))
        }
    }

    let bad_source = BadPrefabSource;
    let load_result = factory.load_from_source(PrefabId::new(888), &bad_source);
    assert!(load_result.is_err(), "Loading from bad source should fail");

    // Test 3: Duplicate prefab ID registration
    let duplicate_id = PrefabId::new(777);
    let empty_prefab = BasicPrefab::new();

    let first_register = factory.register(duplicate_id, empty_prefab);
    assert!(first_register.is_ok(), "First registration should succeed");

    let second_register = factory.register(duplicate_id, BasicPrefab::new());
    assert!(
        second_register.is_err(),
        "Second registration with same ID should fail"
    );

    println!("Workflow error handling integration test passed successfully!");
}

/// Test that verifies the Oracle's critical bug fixes are working
#[test]
#[serial]
fn test_oracle_critical_bugs_fixed() {
    register_default_components();
    clear_all_prefab_ids();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ron_file_path = temp_dir.path().join("bug_test.ron");

    // Create a RON file with multiple components to test proper loading
    let ron_content = r#"
(
    components: {
        "Transform": (
            translation: (x: 50.0, y: 60.0, z: 70.0),
            rotation: (x: 0.0, y: 0.0, z: 0.0, w: 1.0),
            scale: (x: 1.5, y: 1.5, z: 1.5),
        ),
        "Name": "BugTestEntity",
        "Visibility": "Visible",
    }
)
"#;

    fs::write(&ron_file_path, ron_content).expect("Failed to write RON file");

    // Test the critical bug: Factory::spawn() should use loaded prefabs, not create empty ones
    let mut factory = Factory::new();
    let prefab_id = PrefabId::new(555);

    struct BugTestSource {
        file_path: std::path::PathBuf,
    }

    impl PrefabSource for BugTestSource {
        fn load(&self) -> Result<BasicPrefab, Error> {
            let content = std::fs::read_to_string(&self.file_path).map_err(|e| {
                Error::resource_load(
                    format!("prefab file {}", self.file_path.display()),
                    format!("IO error: {e}"),
                )
            })?;

            let config = DslConfig {
                validation_mode: ValidationMode::Skip,
                ..Default::default()
            };
            let type_registry = AppTypeRegistry::default();
            let component_map = parse_prefab_ron(&content, &type_registry, &config)?;
            create_prefab_from_component_map(&component_map, &type_registry)
        }
    }

    let source = BugTestSource {
        file_path: ron_file_path,
    };

    // Load the prefab - this should NOT ignore the loaded prefab data
    factory
        .load_from_source(prefab_id, &source)
        .expect("Failed to load prefab");

    // Verify the loaded prefab is not empty (this would fail with the old bug)
    assert!(
        factory.contains(prefab_id),
        "Loaded prefab should be in factory"
    );
    assert!(
        !factory.is_empty(),
        "Factory should not be empty after loading"
    );

    // Spawn the entity - this should NOT create an empty prefab
    let mut world = World::new();
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &world);

    let entity = factory
        .spawn(&mut commands, prefab_id)
        .expect("Failed to spawn entity");
    queue.apply(&mut world);

    // The critical test: verify the spawned entity has the components from the loaded prefab
    // This would fail with the old bug where spawn() created empty prefabs
    let entity_ref = world.entity(entity);

    // Check that Transform component exists and has the correct values from the RON file
    let transform = entity_ref
        .get::<Transform>()
        .expect("Transform component should be present");
    assert_eq!(
        transform.translation.x, 50.0,
        "Transform should have loaded values, not default"
    );
    assert_eq!(
        transform.translation.y, 60.0,
        "Transform should have loaded values, not default"
    );
    assert_eq!(
        transform.translation.z, 70.0,
        "Transform should have loaded values, not default"
    );
    assert_eq!(
        transform.scale.x, 1.5,
        "Transform should have loaded values, not default"
    );

    // Check that Name component exists and has the correct value from the RON file
    let name = entity_ref
        .get::<Name>()
        .expect("Name component should be present");
    assert_eq!(
        name.as_str(),
        "BugTestEntity",
        "Name should have loaded value, not default"
    );

    // Check that Visibility component exists and has the correct value from the RON file
    let visibility = entity_ref
        .get::<Visibility>()
        .expect("Visibility component should be present");
    assert_eq!(
        *visibility,
        Visibility::Visible,
        "Visibility should have loaded value, not default"
    );

    println!("Oracle critical bugs fixed integration test passed successfully!");
    println!("✅ Factory::spawn() now uses loaded prefabs instead of creating empty ones");
    println!("✅ Factory::load_from_source() now properly uses loaded prefab data");
    println!("✅ Component data is correctly preserved through the entire workflow");
}
