//! Demonstration of GPU culling feature flag behavior
//!
//! This example shows how the gpu_culling feature flag controls
//! the culling pipeline selection (GPU vs CPU fallback).
//!
//! Run with:
//! ```
//! cargo run --example gpu_culling_feature_demo --features gpu_culling
//! cargo run --example gpu_culling_feature_demo  # CPU fallback
//! ```

use amp_engine::prelude::*;
use amp_render::prelude::*;
use bevy::prelude::*;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins(AAAPlugins::default())
        .add_plugins(BatchingPlugin);

    // Setup camera
    app.add_systems(Startup, setup_scene);
    app.add_systems(Update, (info_system, spawn_test_entities));

    // Feature flag status reporting
    #[cfg(feature = "gpu_culling")]
    info!("GPU culling feature ENABLED - using compute shaders");

    #[cfg(not(feature = "gpu_culling"))]
    info!("GPU culling feature DISABLED - using CPU fallback");

    app.run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 50.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            1.0,
            -std::f32::consts::FRAC_PI_4,
        )),
    ));

    // Create shared mesh and material for batching
    let mesh = meshes.add(Cuboid::default());
    let material = materials.add(Color::srgb(0.8, 0.7, 0.6));

    // Store in world for entity spawning
    commands.insert_resource(TestResources { mesh, material });
}

#[derive(Resource)]
struct TestResources {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

fn spawn_test_entities(
    mut commands: Commands,
    resources: Res<TestResources>,
    time: Res<Time>,
    entities: Query<Entity, With<BatchKey>>,
) {
    let count = entities.iter().count();

    // Spawn entities gradually to test culling performance
    if time.elapsed_secs() > 2.0 && count < 1000 {
        let batch_key = BatchKey::new(&resources.mesh, &resources.material);

        for i in 0..10 {
            let x = (count as f32 * 0.1 + i as f32) % 50.0 - 25.0;
            let z = (count as f32 * 0.1 + i as f32) / 50.0 * 20.0 - 10.0;

            commands.spawn((
                Mesh3d(resources.mesh.clone()),
                MeshMaterial3d(resources.material.clone()),
                Transform::from_xyz(x, 0.0, z),
                batch_key.clone(),
                Cullable::new(1.0),
            ));
        }
    }
}

fn info_system(time: Res<Time>, entities: Query<Entity, With<BatchKey>>) {
    if time.elapsed_secs() as u32 % 3 == 0 && time.delta_secs() < 0.02 {
        let count = entities.iter().count();

        #[cfg(feature = "gpu_culling")]
        info!("GPU Culling: {} entities being processed", count);

        #[cfg(not(feature = "gpu_culling"))]
        info!("CPU Culling: {} entities being processed", count);
    }
}
