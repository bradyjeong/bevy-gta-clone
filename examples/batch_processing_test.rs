//! Test the RenderWorld batch processing system
//!
//! Demonstrates high-performance instanced rendering with
//! Extract→Prepare→Queue pipeline targeting ≤4ms.

use amp_render::prelude::*;
use bevy::prelude::*;

fn main() {
    println!("🚀 Testing RenderWorld Batch Processing System");

    App::new()
        .add_plugins((DefaultPlugins, BatchingPlugin))
        .add_systems(Startup, setup_test_scene)
        .add_systems(Update, update_instances)
        .run();
}

/// Component for test entities
#[derive(Component)]
struct TestInstance {
    pub speed: f32,
    pub radius: f32,
}

fn setup_test_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    println!("📦 Setting up test scene with 1000 instances");

    // Create shared mesh and material
    let mesh_handle = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let material_handle = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.5, 0.8),
        ..default()
    });

    // Create batch key for all instances
    let batch_key = BatchKey::new(&mesh_handle, &material_handle);

    // Spawn 1000 test instances
    for i in 0..1000 {
        let x = (i % 32) as f32 * 2.0 - 32.0;
        let z = (i / 32) as f32 * 2.0 - 32.0;
        let y = 0.0;

        let transform = Mat4::from_translation(Vec3::new(x, y, z));

        commands.spawn((
            TestInstance {
                speed: 1.0 + (i as f32 * 0.01),
                radius: 10.0,
            },
            ExtractedInstance::new(transform, batch_key.clone(), Vec3::ZERO),
        ));
    }

    // Spawn camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 30.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Spawn light
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

    println!("✅ Scene setup complete - 1000 instances ready for batching");
}

fn update_instances(
    time: Res<Time>,
    mut instances: Query<(&mut ExtractedInstance, &TestInstance)>,
    batch_manager: Option<Res<BatchManager>>,
    instance_meta: Option<Res<InstanceMeta>>,
) {
    let elapsed = time.elapsed_secs();

    // Update instance transforms
    for (mut instance, test_data) in instances.iter_mut() {
        let angle = elapsed * test_data.speed;
        let x = angle.cos() * test_data.radius;
        let z = angle.sin() * test_data.radius;

        instance.transform = Mat4::from_translation(Vec3::new(x, 0.0, z));
    }

    // Log performance metrics every second
    if elapsed as u32 > 0 && (elapsed as u32) != ((elapsed - time.delta_secs()) as u32) {
        if let Some(batch_manager) = batch_manager {
            println!(
                "📊 BatchManager: {} batches, {} instances",
                batch_manager.batch_count(),
                batch_manager.instance_count()
            );
        }

        if let Some(instance_meta) = instance_meta {
            println!(
                "⚡ RenderWorld: {:.2}ms prepare, {:.2}ms queue, {} batches total",
                instance_meta.prepare_time_ms,
                instance_meta.queue_time_ms,
                instance_meta.batch_count()
            );

            // Check if we meet the ≤4ms target
            let total_time = instance_meta.prepare_time_ms + instance_meta.queue_time_ms;
            if total_time > 4.0 {
                println!("⚠️  Performance target missed: {total_time:.2}ms > 4.0ms");
            } else {
                println!("✅ Performance target met: {total_time:.2}ms ≤ 4.0ms");
            }
        }
    }
}
