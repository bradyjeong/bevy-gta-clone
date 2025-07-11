use bevy::prelude::*;
use criterion::{Criterion, criterion_group, criterion_main};
use gameplay_factory::*;

fn spawn_1k(c: &mut Criterion) {
    c.bench_function("factory_spawn_1k", |b| {
        b.iter(|| {
            let mut world = World::default();
            let mut commands = world.commands();

            // Create a simple prefab using RON components
            let mut prefab = Prefab::new();
            let transform_component = RonComponent {
                component_type: "Transform".to_string(),
                data: ron::Value::Map(
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
            };
            prefab.add_component(Box::new(transform_component));

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

fn spawn_batch_1k(c: &mut Criterion) {
    c.bench_function("factory_spawn_batch_1k", |b| {
        b.iter(|| {
            let mut world = World::default();
            let mut commands = world.commands();

            // Create a simple prefab using RON components
            let mut prefab = Prefab::new();
            let transform_component = RonComponent {
                component_type: "Transform".to_string(),
                data: ron::Value::Map(
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
            };
            prefab.add_component(Box::new(transform_component));

            let mut factory = Factory::new();
            let prefab_id = PrefabId::new(1);
            factory.register(prefab_id, prefab).unwrap();

            // Spawn 1k entities in batches
            let batch_size = 100;
            for _ in 0..(1000 / batch_size) {
                for _ in 0..batch_size {
                    factory.spawn(&mut commands, prefab_id).unwrap();
                }
            }

            // Commands are applied automatically
        })
    });
}

fn spawn_dsl_simple(c: &mut Criterion) {
    c.bench_function("factory_spawn_dsl_simple", |b| {
        b.iter(|| {
            let mut world = World::default();
            let mut commands = world.commands();

            // Create a simple component map
            let component_map = ComponentMap {
                components: std::collections::HashMap::new(),
                metadata: ComponentMapMetadata {
                    source_path: None,
                    validation_status: ValidationStatus::Valid,
                    component_count: 0,
                },
            };

            let entities: Vec<ComponentMap> = (0..1000).map(|_| component_map.clone()).collect();
            let request = BatchSpawnRequest {
                entities,
                config: DslConfig::default(),
            };

            // Spawn using DSL
            spawn_many(&mut commands, request).unwrap();

            // Commands are applied automatically
        })
    });
}

criterion_group!(benches, spawn_1k, spawn_batch_1k, spawn_dsl_simple);
criterion_main!(benches);
