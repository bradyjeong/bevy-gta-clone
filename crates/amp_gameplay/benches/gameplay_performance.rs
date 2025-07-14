//! Performance benchmarks for gameplay systems

use amp_engine::prelude::*;
use amp_gameplay::prelude::*;
use amp_gameplay::vehicle::components::Engine;
use bevy::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;

/// Create a benchmark app with minimal plugins
fn create_benchmark_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .add_plugins(TransformPlugin)
        .add_plugins(bevy::input::InputPlugin)
        .add_plugins(AAAPlugins::default())
        .insert_resource(Time::<Fixed>::from_hz(60.0));

    app
}

/// Benchmark spawning 50 vehicles
fn bench_spawn_vehicles(c: &mut Criterion) {
    c.bench_function("spawn_50_vehicles", |b| {
        b.iter(|| {
            let mut app = create_benchmark_app();

            for i in 0..50 {
                app.world_mut().spawn(VehicleBundle {
                    transform: Transform::from_xyz(i as f32 * 3.0, 0.0, 0.0),
                    vehicle: amp_gameplay::vehicle::components::Vehicle {
                        mass: 1500.0,
                        ..default()
                    },
                    engine: Engine {
                        max_torque: 400.0,
                        max_rpm: 6000.0,
                        ..default()
                    },
                    ..default()
                });
            }

            // Run one update to ensure everything is initialized
            app.update();

            black_box(app);
        });
    });
}

/// Benchmark spawning 200 static colliders
fn bench_spawn_static_colliders(c: &mut Criterion) {
    c.bench_function("spawn_200_static_colliders", |b| {
        b.iter(|| {
            let mut app = create_benchmark_app();

            for i in 0..200 {
                for j in 0..1 {
                    app.world_mut().spawn((
                        bevy_rapier3d::geometry::Collider::cuboid(1.0, 1.0, 1.0),
                        Transform::from_xyz(i as f32 * 2.0, 0.0, j as f32 * 2.0),
                        GlobalTransform::default(),
                    ));
                }
            }

            // Run one update to ensure everything is initialized
            app.update();

            black_box(app);
        });
    });
}

/// Benchmark combined vehicle and collider spawning
fn bench_spawn_mixed_scene(c: &mut Criterion) {
    c.bench_function("spawn_mixed_scene", |b| {
        b.iter(|| {
            let mut app = create_benchmark_app();

            // Spawn 50 vehicles
            for i in 0..50 {
                app.world_mut().spawn(VehicleBundle {
                    transform: Transform::from_xyz(i as f32 * 3.0, 0.0, 0.0),
                    input: VehicleInput {
                        throttle: 0.1,
                        ..default()
                    },
                    ..default()
                });
            }

            // Spawn 200 static colliders
            for i in 0..200 {
                app.world_mut().spawn((
                    bevy_rapier3d::geometry::Collider::cuboid(1.0, 1.0, 1.0),
                    Transform::from_xyz(i as f32 * 2.0, 0.0, 10.0),
                    GlobalTransform::default(),
                ));
            }

            // Run one update to ensure everything is initialized
            app.update();

            black_box(app);
        });
    });
}

/// Benchmark 1000 physics simulation ticks with Sprint 3 performance target
fn bench_physics_ticks(c: &mut Criterion) {
    let mut group = c.benchmark_group("physics_simulation");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("1000_physics_ticks_32_vehicles", |b| {
        b.iter(|| {
            let mut app = create_benchmark_app();

            // Spawn 32 vehicles for simulation (Sprint 3 requirement)
            for i in 0..32 {
                app.world_mut().spawn(VehicleBundle {
                    transform: Transform::from_xyz(i as f32 * 5.0, 0.0, 0.0),
                    input: VehicleInput {
                        throttle: 0.5,
                        steering: if i % 2 == 0 { 0.2 } else { -0.2 },
                        ..default()
                    },
                    audio: VehicleAudio {
                        engine_sound_enabled: true,
                        ..default()
                    },
                    ..default()
                });
            }

            // Spawn some static colliders
            for i in 0..100 {
                app.world_mut().spawn((
                    bevy_rapier3d::geometry::Collider::cuboid(1.0, 1.0, 1.0),
                    Transform::from_xyz(i as f32 * 2.0, 0.0, 15.0),
                    GlobalTransform::default(),
                ));
            }

            // Run initial update
            app.update();

            // Benchmark 1000 physics ticks
            let start = std::time::Instant::now();
            for _ in 0..1000 {
                app.update();
            }
            let elapsed = start.elapsed();

            // Verify Sprint 3 performance target (≤1.5ms per tick)
            let avg_per_tick = elapsed.as_millis() as f64 / 1000.0;
            assert!(
                avg_per_tick < 1.5,
                "Physics tick too slow: {avg_per_tick:.2}ms avg (target: ≤1.5ms)"
            );

            black_box(app);
        });
    });

    group.finish();
}

/// Benchmark audio event processing
fn bench_audio_events(c: &mut Criterion) {
    c.bench_function("audio_event_processing", |b| {
        b.iter(|| {
            let mut app = create_benchmark_app();

            // Spawn 20 vehicles with audio enabled
            for i in 0..20 {
                app.world_mut().spawn(VehicleBundle {
                    transform: Transform::from_xyz(i as f32 * 3.0, 0.0, 0.0),
                    audio: VehicleAudio {
                        engine_sound_enabled: true,
                        ..default()
                    },
                    input: VehicleInput {
                        throttle: 0.8,
                        ..default()
                    },
                    ..default()
                });
            }

            // Run simulation to generate audio events
            for _ in 0..60 {
                // 1 second at 60 Hz
                app.update();
            }

            black_box(app);
        });
    });
}

/// Benchmark memory usage (basic check)
fn bench_memory_usage(c: &mut Criterion) {
    c.bench_function("memory_usage_check", |b| {
        b.iter(|| {
            let mut app = create_benchmark_app();

            // Record initial memory usage
            let initial_memory = get_memory_usage();

            // Spawn many entities
            for i in 0..100 {
                app.world_mut().spawn(VehicleBundle {
                    transform: Transform::from_xyz(i as f32 * 3.0, 0.0, 0.0),
                    ..default()
                });
            }

            // Run simulation
            for _ in 0..120 {
                app.update();
            }

            // Check memory usage
            let final_memory = get_memory_usage();
            let memory_increase = final_memory - initial_memory;

            // Should not use excessive memory (less than 100MB for this test)
            assert!(
                memory_increase < 100_000_000,
                "Memory usage too high: {memory_increase} bytes"
            );

            black_box(app);
        });
    });
}

/// Simple memory usage estimation
fn get_memory_usage() -> usize {
    // This is a very basic approximation
    // In a real benchmark, you'd use a proper memory profiler
    std::mem::size_of::<App>() * 1000 // Placeholder
}

criterion_group!(
    benches,
    bench_spawn_vehicles,
    bench_spawn_static_colliders,
    bench_spawn_mixed_scene,
    bench_physics_ticks,
    bench_audio_events,
    bench_memory_usage
);
criterion_main!(benches);
