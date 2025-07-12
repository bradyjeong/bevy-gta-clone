//! GPU Culling Demonstration
//!
//! Demonstrates Oracle's GPU compute-shader instance culling system
//! with 10K instances targeting <0.2ms GPU time vs 0.9ms CPU fallback.

use amp_render::prelude::*;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use std::f32::consts::PI;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(RenderPlugin {
                render_creation: bevy::render::settings::RenderCreation::Automatic(
                    bevy::render::settings::WgpuSettings {
                        features: bevy::render::settings::WgpuFeatures::PUSH_CONSTANTS,
                        ..default()
                    },
                ),
                ..default()
            }),
            BatchingPlugin,
        ))
        .add_systems(Startup, setup_scene)
        .add_systems(
            Update,
            (
                spawn_instances_system,
                rotate_camera_system,
                performance_display_system,
            ),
        )
        .run();
}

#[derive(Component)]
struct TestInstance;

#[derive(Component)]
struct RotatingCamera;

#[derive(Resource)]
struct InstanceSpawner {
    timer: Timer,
    count: usize,
    max_instances: usize,
}

impl Default for InstanceSpawner {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            count: 0,
            max_instances: 10000, // Oracle's target: 10K instances
        }
    }
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!("Setting up GPU culling demo with 10K instance target");

    // Create shared mesh and material
    let cube_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let material = materials.add(Color::srgb(0.8, 0.4, 0.2));

    // Spawn camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 50.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
        RotatingCamera,
    ));

    // Spawn directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 3000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -PI / 4.)),
    ));

    // Initialize instance spawner
    commands.insert_resource(InstanceSpawner::default());

    info!("Scene setup complete - ready for GPU culling demonstration");
}

fn spawn_instances_system(
    mut commands: Commands,
    mut spawner: ResMut<InstanceSpawner>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
    instances: Query<Entity, With<TestInstance>>,
) {
    spawner.timer.tick(time.delta());

    if spawner.timer.finished() && spawner.count < spawner.max_instances {
        // Create shared resources if not created
        let cube_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
        let material = materials.add(Color::srgb(
            0.2 + (spawner.count as f32 * 0.001) % 0.8,
            0.4,
            0.6,
        ));

        // Spawn instances in a grid pattern
        let grid_size = 100.0;
        let spacing = 2.0;
        let instances_per_side = (grid_size / spacing) as i32;

        for batch in 0..10 {
            let x = ((spawner.count % instances_per_side as usize) as f32
                - instances_per_side as f32 / 2.0)
                * spacing;
            let z = ((spawner.count / instances_per_side as usize) as f32
                - instances_per_side as f32 / 2.0)
                * spacing;
            let y = (batch as f32 * 5.0) + (spawner.count as f32 * 0.01).sin() * 2.0;

            let transform = Transform::from_xyz(x, y, z);
            let batch_key = BatchKey::new(&cube_mesh, &material);

            commands.spawn((
                Mesh3d(cube_mesh.clone()),
                MeshMaterial3d(material.clone()),
                transform,
                ExtractedInstance::new(transform.compute_matrix(), batch_key, Vec3::ZERO),
                Cullable::new(1.0), // 1.0 unit bounding radius
                TestInstance,
            ));

            spawner.count += 1;
            if spawner.count >= spawner.max_instances {
                break;
            }
        }

        if spawner.count % 1000 == 0 {
            info!(
                "Spawned {} instances (target: {})",
                spawner.count, spawner.max_instances
            );
        }
    }

    // Log final count
    if spawner.count >= spawner.max_instances {
        let current_count = instances.iter().count();
        if current_count == spawner.max_instances {
            info!(
                "✅ Reached target: {} instances ready for GPU culling test",
                current_count
            );
        }
    }
}

fn rotate_camera_system(mut cameras: Query<&mut Transform, With<RotatingCamera>>, time: Res<Time>) {
    for mut transform in cameras.iter_mut() {
        let radius = 150.0;
        let height = 75.0;
        let angle = time.elapsed_secs() * 0.2;

        let x = angle.cos() * radius;
        let z = angle.sin() * radius;

        transform.translation = Vec3::new(x, height, z);
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

fn performance_display_system(
    performance: Option<Res<CullingPerformance>>,
    instances: Query<&ExtractedInstance>,
    mut last_log: Local<f32>,
    time: Res<Time>,
) {
    // Log performance every 2 seconds
    if time.elapsed_secs() - *last_log > 2.0 {
        let instance_count = instances.iter().count();
        let visible_count = instances.iter().filter(|i| i.visible).count();

        if let Some(perf) = performance {
            info!(
                "Performance Report: {} total, {} visible | {:.3}ms avg ({:?}) | Target: {}",
                instance_count,
                visible_count,
                perf.average_timing(),
                perf.culling_method,
                if perf.meets_performance_target() {
                    "✅ MET"
                } else {
                    "❌ MISSED"
                }
            );
        } else {
            info!(
                "Culling Stats: {} total instances, {} visible",
                instance_count, visible_count
            );
        }

        *last_log = time.elapsed_secs();
    }
}
