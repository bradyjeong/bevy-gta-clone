//! Simple LOD System Demo
//!
//! Basic demonstration of the distance-based LOD system.

use amp_render::prelude::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BatchingPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, move_camera)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create different quality meshes for LOD levels
    let sphere_high = meshes.add(Sphere::new(1.0).mesh().uv(32, 18));
    let sphere_medium = meshes.add(Sphere::new(1.0).mesh().uv(16, 9));
    let sphere_low = meshes.add(Sphere::new(1.0).mesh().uv(8, 4));

    // Create material
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.5, 0.5),
        ..default()
    });

    // Spawn camera that will move back and forth
    commands.spawn((Camera3d::default(), Transform::from_xyz(0.0, 0.0, 10.0)));

    // Create LOD object
    let lod_levels = vec![
        LodLevel::new(15.0, sphere_high.clone()), // High detail for close
        LodLevel::new(35.0, sphere_medium.clone()), // Medium detail
        LodLevel::new(60.0, sphere_low.clone()),  // Low detail for far
    ];

    let lod_group = LodGroup::new(lod_levels)
        .with_hysteresis(5.0)
        .with_cross_fade_duration(0.5);

    // Spawn LOD object at origin
    commands.spawn((
        Mesh3d(sphere_high),
        MeshMaterial3d(material),
        Transform::from_xyz(0.0, 0.0, 0.0),
        lod_group,
    ));

    // Add lighting
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.3, 0.5, 0.0)),
    ));
}

fn move_camera(time: Res<Time>, mut cameras: Query<&mut Transform, With<Camera3d>>) {
    let Ok(mut camera_transform) = cameras.get_single_mut() else {
        return;
    };

    // Move camera back and forth to trigger LOD changes
    let t = time.elapsed_secs() * 0.5;
    let distance = 5.0 + 40.0 * (t.sin() + 1.0) / 2.0; // Oscillate between 5 and 45

    camera_transform.translation.z = distance;
    camera_transform.look_at(Vec3::ZERO, Vec3::Y);

    // Print current distance for debugging
    if (time.elapsed_secs() * 2.0) as i32 % 60 == 0 {
        info!("Camera distance: {:.1}", distance);
    }
}
