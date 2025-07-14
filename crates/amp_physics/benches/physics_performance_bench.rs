//! Performance benchmarks for physics systems.

use amp_physics::*;
use bevy::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

/// Benchmark physics system performance with 10 vehicles.
fn benchmark_10_vehicles(c: &mut Criterion) {
    c.bench_function("physics_10_vehicles", |b| {
        b.iter(|| {
            let mut app = create_benchmark_app();

            // Spawn 10 vehicles
            for i in 0..10 {
                spawn_benchmark_vehicle(&mut app, Vec3::new(i as f32 * 5.0, 1.0, 0.0));
            }

            // Run 60 frames
            for _ in 0..60 {
                app.update();
                black_box(());
            }
        })
    });
}

/// Benchmark suspension system performance.
fn benchmark_suspension_system(c: &mut Criterion) {
    c.bench_function("suspension_system", |b| {
        b.iter(|| {
            let mut app = create_benchmark_app();

            // Spawn 20 vehicles for stress testing
            for i in 0..20 {
                spawn_benchmark_vehicle(&mut app, Vec3::new(i as f32 * 3.0, 1.0, 0.0));
            }

            // Run 100 frames
            for _ in 0..100 {
                app.update();
                black_box(());
            }
        })
    });
}

/// Benchmark physics time system performance.
fn benchmark_physics_time(c: &mut Criterion) {
    c.bench_function("physics_time_system", |b| {
        b.iter(|| {
            let mut physics_time = PhysicsTime::default();

            // Simulate 1000 frame updates
            for _ in 0..1000 {
                physics_time.update(black_box(0.016)); // 60 FPS

                while physics_time.should_step() {
                    physics_time.consume_step();
                }
            }
        })
    });
}

/// Benchmark debug rendering performance impact.
fn benchmark_debug_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("debug_rendering");

    // Benchmark without debug rendering
    group.bench_function("without_debug", |b| {
        b.iter(|| {
            let mut app = create_benchmark_app();

            // Disable debug rendering
            let mut debug_config = app.world_mut().resource_mut::<DebugConfig>();
            debug_config.enabled = false;

            // Spawn 5 vehicles
            for i in 0..5 {
                spawn_benchmark_vehicle(&mut app, Vec3::new(i as f32 * 4.0, 1.0, 0.0));
            }

            // Run 50 frames
            for _ in 0..50 {
                app.update();
                black_box(());
            }
        })
    });

    // Benchmark with debug rendering
    group.bench_function("with_debug", |b| {
        b.iter(|| {
            let mut app = create_benchmark_app();

            // Enable debug rendering
            let mut debug_config = app.world_mut().resource_mut::<DebugConfig>();
            debug_config.enabled = true;
            debug_config.show_suspension_rays = true;
            debug_config.show_force_vectors = true;
            debug_config.show_contact_points = true;

            let mut physics_config = app.world_mut().resource_mut::<PhysicsConfig>();
            physics_config.debug_rendering = true;

            // Spawn 5 vehicles
            for i in 0..5 {
                spawn_benchmark_vehicle(&mut app, Vec3::new(i as f32 * 4.0, 1.0, 0.0));
            }

            // Run 50 frames
            for _ in 0..50 {
                app.update();
                black_box(());
            }
        })
    });

    group.finish();
}

/// Benchmark memory allocation patterns.
fn benchmark_memory_allocation(c: &mut Criterion) {
    c.bench_function("memory_allocation", |b| {
        b.iter(|| {
            let mut app = create_benchmark_app();

            // Spawn and despawn vehicles to test allocation patterns
            for round in 0..5 {
                let mut entities = Vec::new();

                // Spawn 10 vehicles
                for i in 0..10 {
                    let entity = spawn_benchmark_vehicle(
                        &mut app,
                        Vec3::new(i as f32 * 3.0, 1.0, round as f32 * 5.0),
                    );
                    entities.push(entity);
                }

                // Run physics for a few frames
                for _ in 0..10 {
                    app.update();
                    black_box(());
                }

                // Despawn vehicles
                for entity in entities {
                    if let Ok(entity_commands) = app.world_mut().get_entity_mut(entity) {
                        entity_commands.despawn();
                    }
                }
            }
        })
    });
}

/// Create a benchmark app with minimal plugins for performance testing.
fn create_benchmark_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<bevy::input::ButtonInput<bevy::input::keyboard::KeyCode>>()
        .add_plugins(PhysicsPlugin::default());

    // Disable debug rendering by default
    let mut debug_config = app.world_mut().resource_mut::<DebugConfig>();
    debug_config.enabled = false;

    app
}

/// Spawn a benchmark vehicle with minimal setup.
fn spawn_benchmark_vehicle(app: &mut App, position: Vec3) -> Entity {
    let vehicle_transform = Transform::from_translation(position);

    // Spawn vehicle entity
    let vehicle_entity = app
        .world_mut()
        .spawn((
            vehicle_transform,
            GlobalTransform::from(vehicle_transform),
            Engine::default(),
            Transmission::default(),
            Steering::default(),
            Brakes::default(),
            VehicleInput::default(),
            Suspension::default(),
        ))
        .id();

    // Spawn wheel entities with suspension
    let wheel_positions = [
        Vec3::new(-1.0, 0.0, 1.5),  // Front left
        Vec3::new(1.0, 0.0, 1.5),   // Front right
        Vec3::new(-1.0, 0.0, -1.5), // Rear left
        Vec3::new(1.0, 0.0, -1.5),  // Rear right
    ];

    for wheel_position in wheel_positions {
        app.world_mut().spawn((
            SuspensionRay {
                cast_distance: 1.0,
                ray_origin: wheel_position,
                ray_direction: Vec3::NEG_Y,
                ..default()
            },
            WheelState::default(),
            Transform::from_translation(position + wheel_position),
            GlobalTransform::from(Transform::from_translation(position + wheel_position)),
        ));
    }

    vehicle_entity
}

criterion_group!(
    benches,
    benchmark_10_vehicles,
    benchmark_suspension_system,
    benchmark_physics_time,
    benchmark_debug_rendering,
    benchmark_memory_allocation
);
criterion_main!(benches);
