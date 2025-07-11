use bevy::prelude::*;
use criterion::{Criterion, criterion_group, criterion_main};
use gameplay_factory::*;
use std::collections::HashMap;

fn spawn_dsl_simple(c: &mut Criterion) {
    c.bench_function("factory_spawn_dsl_simple", |b| {
        b.iter(|| {
            let mut app = App::new();
            app.add_plugins(DefaultPlugins);

            let mut world = World::default();
            let mut commands = world.commands();
            let type_registry = app.world().resource::<AppTypeRegistry>().clone();

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

fn spawn_factory_prefab(c: &mut Criterion) {
    c.bench_function("factory_spawn_prefab", |b| {
        b.iter(|| {
            let mut world = World::default();
            let mut commands = world.commands();

            // Create a simple prefab
            let prefab = Prefab::new();
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

criterion_group!(benches, spawn_dsl_simple, spawn_factory_prefab);
criterion_main!(benches);
