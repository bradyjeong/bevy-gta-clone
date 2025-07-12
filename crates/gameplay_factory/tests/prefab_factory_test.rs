//! Comprehensive tests for the prefab factory MVP

use bevy::prelude::*;
use gameplay_factory::{
    BasicPrefab, PrefabChild, PrefabFactory, PrefabFactoryPlugin, PrefabFactoryResource, PrefabId,
    SpawnPrefabEvent, component_init, prefab,
};

#[derive(Component, Clone, Default)]
struct TestComponent {
    value: i32,
}

#[derive(Component, Clone, Default)]
struct TestMarker;

#[test]
fn test_prefab_macro_basic() {
    let prefab = prefab!(TestEntity => {
        test_component: TestComponent { value: 42 },
        test_marker: TestMarker,
    });

    assert_eq!(prefab.len(), 2);
    assert_eq!(prefab.metadata().name, "TestEntity");
}

#[test]
fn test_prefab_macro_with_bundle() {
    let prefab = prefab!(TestEntity => {
        bundle: (TestComponent { value: 10 }, TestMarker),
        test_component: TestComponent { value: 42 },
    });

    assert_eq!(prefab.len(), 1);
    assert_eq!(prefab.metadata().name, "TestEntity");
    assert!(prefab.bundle().is_some());
}

#[test]
fn test_prefab_with_children() {
    let child = PrefabChild {
        prefab: prefab!(Child => {
            test_component: TestComponent { value: 100 },
        }),
        transform: Transform::from_xyz(1.0, 2.0, 3.0),
        name: "TestChild".to_string(),
    };

    let prefab = prefab!(Parent => {
        children: [child],
        test_component: TestComponent { value: 50 },
    });

    assert_eq!(prefab.len(), 1);
    assert_eq!(prefab.children().len(), 1);
    assert_eq!(prefab.metadata().name, "Parent");
}

#[test]
fn test_prefab_factory_registration() {
    let mut factory = PrefabFactory::new();
    let id = PrefabId::new(123);

    let prefab = prefab!(TestPrefab => {
        test_component: TestComponent { value: 42 },
    });

    assert!(factory.register_prefab(id, "test_prefab", prefab).is_ok());
    assert!(factory.has_prefab("test_prefab"));
    assert_eq!(factory.prefab_count(), 1);
    assert_eq!(factory.get_prefab_names(), vec!["test_prefab".to_string()]);
}

#[test]
fn test_prefab_factory_spawn_by_name() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let mut factory = PrefabFactory::new();
    let id = PrefabId::new(123);

    let prefab = prefab!(TestPrefab => {
        test_component: TestComponent { value: 42 },
    });

    factory.register_prefab(id, "test_prefab", prefab).unwrap();

    let mut world = World::new();
    let mut commands = world.commands();

    let entity = factory
        .spawn_prefab_named(&mut commands, "test_prefab")
        .unwrap();
    assert_ne!(entity, Entity::PLACEHOLDER);
}

#[test]
fn test_prefab_factory_spawn_by_id() {
    let mut factory = PrefabFactory::new();
    let id = PrefabId::new(123);

    let prefab = prefab!(TestPrefab => {
        test_component: TestComponent { value: 42 },
    });

    factory.register_prefab(id, "test_prefab", prefab).unwrap();

    let mut world = World::new();
    let mut commands = world.commands();

    let entity = factory.spawn_prefab(&mut commands, id).unwrap();
    assert_ne!(entity, Entity::PLACEHOLDER);
}

#[test]
fn test_prefab_factory_batch_spawn() {
    let mut factory = PrefabFactory::new();
    let id1 = PrefabId::new(123);
    let id2 = PrefabId::new(456);

    let prefab1 = prefab!(TestPrefab1 => {
        test_component: TestComponent { value: 42 },
    });

    let prefab2 = prefab!(TestPrefab2 => {
        test_component: TestComponent { value: 84 },
    });

    factory
        .register_prefab(id1, "test_prefab1", prefab1)
        .unwrap();
    factory
        .register_prefab(id2, "test_prefab2", prefab2)
        .unwrap();

    let mut world = World::new();
    let mut commands = world.commands();

    let spawns = vec![
        (id1, Transform::from_xyz(1.0, 0.0, 0.0)),
        (id2, Transform::from_xyz(2.0, 0.0, 0.0)),
    ];

    let entities = factory.batch_spawn(&mut commands, &spawns).unwrap();
    assert_eq!(entities.len(), 2);
}

#[test]
fn test_prefab_factory_batch_spawn_named() {
    let mut factory = PrefabFactory::new();
    let id1 = PrefabId::new(123);
    let id2 = PrefabId::new(456);

    let prefab1 = prefab!(TestPrefab1 => {
        test_component: TestComponent { value: 42 },
    });

    let prefab2 = prefab!(TestPrefab2 => {
        test_component: TestComponent { value: 84 },
    });

    factory
        .register_prefab(id1, "test_prefab1", prefab1)
        .unwrap();
    factory
        .register_prefab(id2, "test_prefab2", prefab2)
        .unwrap();

    let mut world = World::new();
    let mut commands = world.commands();

    let spawns = vec![
        ("test_prefab1", Transform::from_xyz(1.0, 0.0, 0.0)),
        ("test_prefab2", Transform::from_xyz(2.0, 0.0, 0.0)),
    ];

    let entities = factory.batch_spawn_named(&mut commands, &spawns).unwrap();
    assert_eq!(entities.len(), 2);
}

#[test]
fn test_prefab_factory_helper_methods() {
    let vehicle_prefab =
        PrefabFactory::create_vehicle_prefab("TestCar", Vec3::new(1.0, 2.0, 3.0), Color::RED);

    assert_eq!(vehicle_prefab.metadata().name, "Vehicle");
    assert_eq!(vehicle_prefab.len(), 4);

    let building_prefab = PrefabFactory::create_building_prefab(
        "TestBuilding",
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(2.0, 3.0, 4.0),
    );

    assert_eq!(building_prefab.metadata().name, "Building");
    assert_eq!(building_prefab.len(), 3);

    let character_prefab =
        PrefabFactory::create_character_prefab("TestCharacter", Vec3::new(5.0, 0.0, 5.0));

    assert_eq!(character_prefab.metadata().name, "Character");
    assert_eq!(character_prefab.len(), 3);
}

#[test]
fn test_prefab_factory_resource() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(PrefabFactoryPlugin);

    assert!(
        app.world()
            .get_resource::<PrefabFactoryResource>()
            .is_some()
    );
}

#[test]
fn test_spawn_prefab_event() {
    let event_by_name = SpawnPrefabEvent::ByName {
        name: "test_prefab".to_string(),
        transform: Some(Transform::from_xyz(1.0, 2.0, 3.0)),
    };

    match event_by_name {
        SpawnPrefabEvent::ByName { name, transform } => {
            assert_eq!(name, "test_prefab");
            assert!(transform.is_some());
        }
        _ => panic!("Expected ByName event"),
    }

    let event_by_id = SpawnPrefabEvent::ById {
        id: PrefabId::new(123),
        transform: None,
    };

    match event_by_id {
        SpawnPrefabEvent::ById { id, transform } => {
            assert_eq!(id.raw(), 123);
            assert!(transform.is_none());
        }
        _ => panic!("Expected ById event"),
    }
}

#[test]
fn test_prefab_with_assets() {
    let prefab = BasicPrefab::new()
        .with_asset("models/car.glb")
        .with_asset("textures/car_diffuse.png");

    assert_eq!(prefab.metadata().asset_paths.len(), 2);
    assert!(
        prefab
            .metadata()
            .asset_paths
            .contains(&"models/car.glb".to_string())
    );
    assert!(
        prefab
            .metadata()
            .asset_paths
            .contains(&"textures/car_diffuse.png".to_string())
    );
}

#[test]
fn test_prefab_metadata() {
    let prefab = BasicPrefab::with_name("TestPrefab").with_asset("test.asset");

    let metadata = prefab.metadata();
    assert_eq!(metadata.name, "TestPrefab");
    assert_eq!(metadata.type_id, "()");
    assert_eq!(metadata.asset_paths.len(), 1);
    assert_eq!(metadata.component_count, 0);
}

#[test]
fn test_component_init_macro() {
    let init = component_init!(TestComponent { value: 42 });

    // Test that it's a valid ComponentInit - we can't access the private type directly
    // but we can verify it implements the trait
    use gameplay_factory::ComponentInit;
    let _: Box<dyn ComponentInit> = init;
}

#[test]
fn test_prefab_compile_time_checking() {
    // This test ensures the prefab! macro provides compile-time type checking
    let prefab = prefab!(TypedPrefab => {
        test_component: TestComponent { value: 42 },
        test_marker: TestMarker,
    });

    // The fact that this compiles confirms type checking works
    assert_eq!(prefab.len(), 2);
}

#[test]
fn test_prefab_factory_error_handling() {
    let factory = PrefabFactory::new();

    let mut world = World::new();
    let mut commands = world.commands();

    // Test spawning non-existent prefab by name
    let result = factory.spawn_prefab_named(&mut commands, "nonexistent");
    assert!(result.is_err());

    // Test spawning non-existent prefab by ID
    let result = factory.spawn_prefab(&mut commands, PrefabId::new(999));
    assert!(result.is_err());
}

#[test]
fn test_prefab_child_relationships() {
    let child = PrefabChild {
        prefab: prefab!(Child => {
            test_component: TestComponent { value: 100 },
        }),
        transform: Transform::from_xyz(1.0, 2.0, 3.0),
        name: "TestChild".to_string(),
    };

    let parent = prefab!(Parent => {
        children: [child],
        test_component: TestComponent { value: 50 },
    });

    // Test child access
    assert_eq!(parent.children().len(), 1);
    assert_eq!(parent.children()[0].name, "TestChild");
    assert_eq!(parent.children()[0].prefab.len(), 1);

    // Test transform
    let expected_transform = Transform::from_xyz(1.0, 2.0, 3.0);
    assert_eq!(
        parent.children()[0].transform.translation,
        expected_transform.translation
    );
}
