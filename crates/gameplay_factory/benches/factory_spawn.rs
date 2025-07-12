use bevy::prelude::*;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use gameplay_factory::*;
use std::collections::HashMap;

fn spawn_dsl_simple(c: &mut Criterion) {
    c.bench_function("factory_spawn_dsl_simple", |b| {
        b.iter(|| {
            let mut world = World::default();
            let mut commands = world.commands();
            let type_registry = AppTypeRegistry::default();

            // Create a simple component map for Transform
            let mut components = HashMap::new();
            components.insert(
                "Transform".to_string(),
                ron::Value::Map(
                    vec![(
                        ron::Value::String("translation".to_string()),
                        ron::Value::Map(
                            vec![
                                (
                                    ron::Value::String("x".to_string()),
                                    ron::Value::Number(ron::Number::new(0.0)),
                                ),
                                (
                                    ron::Value::String("y".to_string()),
                                    ron::Value::Number(ron::Number::new(0.0)),
                                ),
                                (
                                    ron::Value::String("z".to_string()),
                                    ron::Value::Number(ron::Number::new(0.0)),
                                ),
                            ]
                            .into_iter()
                            .collect(),
                        ),
                    )]
                    .into_iter()
                    .collect(),
                ),
            );

            let component_map = ComponentMap {
                components,
                metadata: ComponentMapMetadata {
                    source_path: None,
                    validation_status: ValidationStatus::Valid,
                    component_count: 1,
                },
            };

            let entities: Vec<ComponentMap> = (0..1000).map(|_| component_map.clone()).collect();
            let request = BatchSpawnRequest {
                entities,
                config: DslConfig::default(),
            };

            // Spawn using DSL
            spawn_many(&mut commands, request, &type_registry).unwrap();

            // Commands are applied automatically
        })
    });
}

fn spawn_100k_mixed_prefabs(c: &mut Criterion) {
    let mut group = c.benchmark_group("spawn_100k");

    // Benchmark with different entity counts for scaling analysis
    for &entity_count in &[1_000, 10_000, 100_000] {
        group.bench_with_input(
            BenchmarkId::new("mixed_prefabs", entity_count),
            &entity_count,
            |b, &entity_count| {
                b.iter(|| {
                    let mut world = World::default();
                    let mut commands = world.commands();
                    let type_registry = AppTypeRegistry::default();

                    // Create mixed prefab types: Vehicle, NPC, Building, Prop
                    let mut entities = Vec::with_capacity(entity_count);

                    for i in 0..entity_count {
                        let prefab_type = i % 4;
                        let component_map = match prefab_type {
                            0 => create_vehicle_prefab(i),
                            1 => create_npc_prefab(i),
                            2 => create_building_prefab(i),
                            _ => create_prop_prefab(i),
                        };
                        entities.push(component_map);
                    }

                    let request = BatchSpawnRequest {
                        entities,
                        config: DslConfig::default(),
                    };

                    // Spawn using DSL
                    spawn_many(&mut commands, request, &type_registry).unwrap();
                })
            },
        );
    }

    group.finish();
}

fn create_vehicle_prefab(index: usize) -> ComponentMap {
    let mut components = HashMap::new();

    // Transform component
    components.insert(
        "Transform".to_string(),
        ron::Value::Map(
            vec![(
                ron::Value::String("translation".to_string()),
                ron::Value::Map(
                    vec![
                        (
                            ron::Value::String("x".to_string()),
                            ron::Value::Number(ron::Number::new(index as f64 * 5.0)),
                        ),
                        (
                            ron::Value::String("y".to_string()),
                            ron::Value::Number(ron::Number::new(0.0)),
                        ),
                        (
                            ron::Value::String("z".to_string()),
                            ron::Value::Number(ron::Number::new(0.0)),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                ),
            )]
            .into_iter()
            .collect(),
        ),
    );

    // Vehicle-specific components (simplified)
    components.insert(
        "Name".to_string(),
        ron::Value::String(format!("Vehicle_{}", index)),
    );

    ComponentMap {
        components,
        metadata: ComponentMapMetadata {
            source_path: Some(format!("prefabs/vehicles/vehicle_{}.ron", index)),
            validation_status: ValidationStatus::Valid,
            component_count: 2,
        },
    }
}

fn create_npc_prefab(index: usize) -> ComponentMap {
    let mut components = HashMap::new();

    // Transform component
    components.insert(
        "Transform".to_string(),
        ron::Value::Map(
            vec![(
                ron::Value::String("translation".to_string()),
                ron::Value::Map(
                    vec![
                        (
                            ron::Value::String("x".to_string()),
                            ron::Value::Number(ron::Number::new(index as f64 * 2.0)),
                        ),
                        (
                            ron::Value::String("y".to_string()),
                            ron::Value::Number(ron::Number::new(0.0)),
                        ),
                        (
                            ron::Value::String("z".to_string()),
                            ron::Value::Number(ron::Number::new(10.0)),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                ),
            )]
            .into_iter()
            .collect(),
        ),
    );

    // NPC-specific components
    components.insert(
        "Name".to_string(),
        ron::Value::String(format!("NPC_{}", index)),
    );

    ComponentMap {
        components,
        metadata: ComponentMapMetadata {
            source_path: Some(format!("prefabs/npcs/npc_{}.ron", index)),
            validation_status: ValidationStatus::Valid,
            component_count: 2,
        },
    }
}

fn create_building_prefab(index: usize) -> ComponentMap {
    let mut components = HashMap::new();

    // Transform component
    components.insert(
        "Transform".to_string(),
        ron::Value::Map(
            vec![(
                ron::Value::String("translation".to_string()),
                ron::Value::Map(
                    vec![
                        (
                            ron::Value::String("x".to_string()),
                            ron::Value::Number(ron::Number::new(index as f64 * 50.0)),
                        ),
                        (
                            ron::Value::String("y".to_string()),
                            ron::Value::Number(ron::Number::new(0.0)),
                        ),
                        (
                            ron::Value::String("z".to_string()),
                            ron::Value::Number(ron::Number::new(20.0)),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                ),
            )]
            .into_iter()
            .collect(),
        ),
    );

    // Building-specific components
    components.insert(
        "Name".to_string(),
        ron::Value::String(format!("Building_{}", index)),
    );

    ComponentMap {
        components,
        metadata: ComponentMapMetadata {
            source_path: Some(format!("prefabs/buildings/building_{}.ron", index)),
            validation_status: ValidationStatus::Valid,
            component_count: 2,
        },
    }
}

fn create_prop_prefab(index: usize) -> ComponentMap {
    let mut components = HashMap::new();

    // Transform component
    components.insert(
        "Transform".to_string(),
        ron::Value::Map(
            vec![(
                ron::Value::String("translation".to_string()),
                ron::Value::Map(
                    vec![
                        (
                            ron::Value::String("x".to_string()),
                            ron::Value::Number(ron::Number::new(index as f64 * 1.0)),
                        ),
                        (
                            ron::Value::String("y".to_string()),
                            ron::Value::Number(ron::Number::new(0.0)),
                        ),
                        (
                            ron::Value::String("z".to_string()),
                            ron::Value::Number(ron::Number::new(5.0)),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                ),
            )]
            .into_iter()
            .collect(),
        ),
    );

    // Prop-specific components
    components.insert(
        "Name".to_string(),
        ron::Value::String(format!("Prop_{}", index)),
    );

    ComponentMap {
        components,
        metadata: ComponentMapMetadata {
            source_path: Some(format!("prefabs/props/prop_{}.ron", index)),
            validation_status: ValidationStatus::Valid,
            component_count: 2,
        },
    }
}

fn spawn_factory_prefab(c: &mut Criterion) {
    c.bench_function("factory_spawn_prefab", |b| {
        b.iter(|| {
            // Clear global registry for each iteration
            gameplay_factory::clear_all_prefab_ids();

            let mut world = World::default();
            let mut commands = world.commands();

            // Create a simple prefab
            let prefab = BasicPrefab::new();
            let mut factory = Factory::new();
            let prefab_id = PrefabId::new(1);
            factory.register(prefab_id, prefab).unwrap();

            // Spawn 1k entities
            for _ in 0..1000 {
                factory.spawn(&mut commands, prefab_id).unwrap();
            }

            // Commands are applied automatically
        })
    });
}

criterion_group!(
    benches,
    spawn_dsl_simple,
    spawn_100k_mixed_prefabs,
    spawn_factory_prefab
);
criterion_main!(benches);
