//! Benchmarks for suspension physics systems.
//!
//! This module provides performance benchmarks for suspension raycasting
//! and physics calculations to ensure Oracle's performance requirements.

use amp_physics::{Suspension, SuspensionRay, WheelState};
use bevy::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn create_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app
}

fn setup_suspension_entities(app: &mut App, count: usize) {
    let world = app.world_mut();

    // Create suspension entities
    for i in 0..count {
        let x = (i % 100) as f32 * 0.1;
        let z = (i / 100) as f32 * 0.1;

        world.spawn((
            Transform::from_xyz(x, 1.0, z),
            SuspensionRay {
                cast_distance: 2.0,
                ray_origin: Vec3::ZERO,
                ray_direction: Vec3::NEG_Y,
                ..Default::default()
            },
            WheelState::default(),
            Suspension::default(),
        ));
    }
}

fn bench_raycast_1000(c: &mut Criterion) {
    let mut app = create_test_app();
    setup_suspension_entities(&mut app, 1000);

    // Warm up the physics world
    for _ in 0..10 {
        app.update();
    }

    c.bench_function("raycast_1000_per_frame", |b| {
        b.iter(|| {
            // Simulate the suspension raycast system with simplified ground detection
            let world = app.world_mut();
            let mut suspension_query = world.query::<(&SuspensionRay, &GlobalTransform)>();
            let mut raycast_count = 0;

            for (suspension_ray, global_transform) in suspension_query.iter(world) {
                let ray_start = global_transform.translation()
                    + global_transform.rotation() * suspension_ray.ray_origin;
                let ray_dir = global_transform.rotation() * suspension_ray.ray_direction;

                // Simplified ground contact detection (assumes flat ground at y=0)
                let ground_y = 0.0;
                let distance_to_ground = (ray_start.y - ground_y) / -ray_dir.y;

                if distance_to_ground > 0.0 && distance_to_ground <= suspension_ray.cast_distance {
                    raycast_count += 1;
                }
            }

            black_box(raycast_count);
        });
    });
}

fn bench_spring_damper_calculation(c: &mut Criterion) {
    let suspension = Suspension::default();
    let compression_distance = 0.1f32;
    let compression_velocity = 0.5f32;

    c.bench_function("spring_damper_calculation", |b| {
        b.iter(|| {
            let spring_force = black_box(suspension.spring_stiffness * compression_distance);
            let damper_force = black_box(suspension.damper_damping * compression_velocity);
            let total_force = black_box(spring_force + damper_force);

            black_box(total_force);
        });
    });
}

fn bench_suspension_travel_limits(c: &mut Criterion) {
    let suspension = Suspension::default();
    let compressions = vec![0.05f32, 0.1, 0.2, 0.3, -0.1, -0.2, -0.3];

    c.bench_function("suspension_travel_limits", |b| {
        b.iter(|| {
            for compression in &compressions {
                let clamped = black_box(
                    compression.clamp(-suspension.max_extension, suspension.max_compression),
                );
                black_box(clamped);
            }
        });
    });
}

fn bench_force_application_1000(c: &mut Criterion) {
    let mut app = create_test_app();
    setup_suspension_entities(&mut app, 1000);

    c.bench_function("force_application_1000", |b| {
        b.iter(|| {
            // Simulate force application for 1000 suspension points
            let forces: Vec<Vec3> = (0..1000)
                .map(|_| {
                    let spring_force = 35000.0 * 0.1;
                    let damper_force = 3500.0 * 0.5;
                    let total_force = spring_force + damper_force;
                    Vec3::Y * total_force
                })
                .collect();

            black_box(forces);
        });
    });
}

fn bench_ground_contact_detection(c: &mut Criterion) {
    let mut suspension_rays = Vec::new();

    // Create test data
    for i in 0..1000 {
        let ray = SuspensionRay {
            hit_distance: if i % 3 == 0 { Some(0.5) } else { None },
            ..Default::default()
        };
        suspension_rays.push(ray);
    }

    c.bench_function("ground_contact_detection_1000", |b| {
        b.iter(|| {
            let mut contact_count = 0;
            for ray in &suspension_rays {
                if ray.hit_distance.is_some() {
                    contact_count += 1;
                }
            }
            black_box(contact_count);
        });
    });
}

fn bench_compression_velocity_calculation(c: &mut Criterion) {
    let dt = 0.016667f32; // 60 FPS
    let test_data: Vec<(Option<f32>, Option<f32>)> = (0..1000)
        .map(|i| {
            let current = Some(0.5 + (i as f32 * 0.001));
            let previous = Some(0.5 + ((i - 1) as f32 * 0.001));
            (current, previous)
        })
        .collect();

    c.bench_function("compression_velocity_calculation_1000", |b| {
        b.iter(|| {
            let velocities: Vec<f32> = test_data
                .iter()
                .map(|(current, previous)| {
                    if let (Some(curr), Some(prev)) = (current, previous) {
                        (prev - curr) / dt
                    } else {
                        0.0
                    }
                })
                .collect();

            black_box(velocities);
        });
    });
}

criterion_group!(
    benches,
    bench_raycast_1000,
    bench_spring_damper_calculation,
    bench_suspension_travel_limits,
    bench_force_application_1000,
    bench_ground_contact_detection,
    bench_compression_velocity_calculation
);

criterion_main!(benches);
