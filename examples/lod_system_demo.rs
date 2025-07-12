//! LOD System Demo
//!
//! Demonstrates the distance-based LOD system with multiple objects,
//! hysteresis, smooth transitions, and BatchManager integration.

use amp_render::prelude::*;
use bevy::{input::mouse::MouseMotion, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BatchingPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                camera_controller,
                display_lod_info,
                update_object_transforms,
            ),
        )
        .run();
}

/// Component for objects that move in circles for LOD testing
#[derive(Component)]
struct CircularMotion {
    radius: f32,
    speed: f32,
    center: Vec3,
}

/// Component for LOD info display
#[derive(Component)]
struct LodInfoDisplay;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create meshes for different LOD levels
    let sphere_high = meshes.add(Sphere::new(1.0).mesh().uv(32, 18));
    let sphere_medium = meshes.add(Sphere::new(1.0).mesh().uv(16, 9));
    let sphere_low = meshes.add(Sphere::new(1.0).mesh().uv(8, 4));
    let sphere_very_low = meshes.add(Sphere::new(1.0).mesh().uv(4, 2));

    // Create materials
    let red_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.2, 0.2),
        ..default()
    });

    let green_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 1.0, 0.2),
        ..default()
    });

    let blue_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.2, 1.0),
        ..default()
    });

    // Spawn camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Spawn stationary LOD objects at various distances
    let positions = [
        Vec3::new(0.0, 0.0, 0.0),  // Very close
        Vec3::new(15.0, 0.0, 0.0), // Close
        Vec3::new(35.0, 0.0, 0.0), // Medium
        Vec3::new(60.0, 0.0, 0.0), // Far
        Vec3::new(90.0, 0.0, 0.0), // Very far
    ];

    for (i, &position) in positions.iter().enumerate() {
        let material = match i % 3 {
            0 => red_material.clone(),
            1 => green_material.clone(),
            _ => blue_material.clone(),
        };

        // Create LOD group with 4 levels
        let lod_levels = vec![
            LodLevel::new(20.0, sphere_high.clone()), // High detail for close
            LodLevel::new(40.0, sphere_medium.clone()), // Medium detail
            LodLevel::new(70.0, sphere_low.clone()),  // Low detail
            LodLevel::new(150.0, sphere_very_low.clone()), // Very low detail for far
        ];

        let lod_group = LodGroup::new(lod_levels)
            .with_hysteresis(8.0)
            .with_cross_fade_duration(0.5);

        commands.spawn((
            Mesh3d(sphere_high.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(position),
            lod_group,
        ));
    }

    // Spawn moving objects for dynamic LOD testing
    for i in 0..10 {
        let angle = i as f32 * std::f32::consts::TAU / 10.0;
        let radius = 30.0 + (i as f32 * 5.0);

        let lod_levels = vec![
            LodLevel::new(25.0, sphere_high.clone()),
            LodLevel::new(50.0, sphere_medium.clone()),
            LodLevel::new(80.0, sphere_low.clone()),
            LodLevel::new(120.0, sphere_very_low.clone()),
        ];

        let lod_group = LodGroup::new(lod_levels)
            .with_hysteresis(6.0)
            .with_cross_fade_duration(0.3);

        let start_pos = Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius);

        commands.spawn((
            Mesh3d(sphere_high.clone()),
            MeshMaterial3d(blue_material.clone()),
            Transform::from_translation(start_pos),
            lod_group,
            CircularMotion {
                radius,
                speed: 0.5 + (i as f32 * 0.1),
                center: Vec3::ZERO,
            },
        ));
    }

    // Spawn lighting
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.3, 0.5, 0.0)),
    ));

    // Spawn UI for LOD information
    commands.spawn((
        Text::new("LOD System Demo\nMove with WASD, Mouse to look\nPress C to toggle LOD info"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        TextColor(Color::WHITE),
        LodInfoDisplay,
    ));
}

fn camera_controller(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut cameras: Query<&mut Transform, With<Camera3d>>,
) {
    let Ok(mut camera_transform) = cameras.single_mut() else {
        return;
    };
    let dt = time.delta_secs();

    // Mouse look
    let mut mouse_delta = Vec2::ZERO;
    for motion in mouse_motion.read() {
        mouse_delta += motion.delta;
    }

    if mouse_delta != Vec2::ZERO {
        let sensitivity = 0.002;
        let yaw = -mouse_delta.x * sensitivity;
        let pitch = -mouse_delta.y * sensitivity;

        camera_transform.rotate_y(yaw);
        camera_transform.rotate_local_x(pitch);
    }

    // Movement
    let mut movement = Vec3::ZERO;
    if keyboard.pressed(KeyCode::KeyW) {
        movement -= camera_transform.forward().as_vec3();
    }
    if keyboard.pressed(KeyCode::KeyS) {
        movement += camera_transform.forward().as_vec3();
    }
    if keyboard.pressed(KeyCode::KeyA) {
        movement -= camera_transform.right().as_vec3();
    }
    if keyboard.pressed(KeyCode::KeyD) {
        movement += camera_transform.right().as_vec3();
    }
    if keyboard.pressed(KeyCode::Space) {
        movement += Vec3::Y;
    }
    if keyboard.pressed(KeyCode::ShiftLeft) {
        movement -= Vec3::Y;
    }

    let speed = if keyboard.pressed(KeyCode::ControlLeft) {
        50.0
    } else {
        10.0
    };
    camera_transform.translation += movement.normalize_or_zero() * speed * dt;
}

fn update_object_transforms(
    time: Res<Time>,
    mut objects: Query<(&mut Transform, &CircularMotion), Without<Camera3d>>,
) {
    for (mut transform, motion) in objects.iter_mut() {
        let elapsed = time.elapsed_secs() * motion.speed;
        let angle = elapsed;

        transform.translation = motion.center
            + Vec3::new(
                angle.cos() * motion.radius,
                0.0,
                angle.sin() * motion.radius,
            );
    }
}

fn display_lod_info(
    keyboard: Res<ButtonInput<KeyCode>>,
    cameras: Query<&Transform, With<Camera3d>>,
    lod_objects: Query<(&Transform, &LodGroup), Without<Camera3d>>,
    mut ui_query: Query<&mut Text, With<LodInfoDisplay>>,
    batch_manager: Res<BatchManager>,
) {
    static mut SHOW_INFO: bool = false;

    if keyboard.just_pressed(KeyCode::KeyC) {
        unsafe {
            SHOW_INFO = !SHOW_INFO;
        }
    }

    let Ok(mut text) = ui_query.single_mut() else {
        return;
    };

    unsafe {
        if !SHOW_INFO {
            text.0 = "LOD System Demo\nMove with WASD, Mouse to look\nPress C to toggle LOD info"
                .to_string();
            return;
        }
    }

    let Ok(camera_transform) = cameras.single() else {
        return;
    };
    let camera_pos = camera_transform.translation;
    let mut lod_counts = [0; 4];
    let mut cross_fading = 0;

    for (transform, lod_group) in lod_objects.iter() {
        let _distance = camera_pos.distance(transform.translation);
        let lod_index = lod_group.current_lod.min(3);
        lod_counts[lod_index] += 1;

        if lod_group.is_cross_fading() {
            cross_fading += 1;
        }
    }

    text.0 = format!(
        "LOD System Demo - Press C to hide\n\
        Camera Position: ({:.1}, {:.1}, {:.1})\n\
        Batch Count: {}\n\
        Instance Count: {}\n\
        LOD Distribution:\n\
          LOD 0 (High): {} objects\n\
          LOD 1 (Med):  {} objects\n\
          LOD 2 (Low):  {} objects\n\
          LOD 3 (Min):  {} objects\n\
        Cross-fading: {} objects",
        camera_pos.x,
        camera_pos.y,
        camera_pos.z,
        batch_manager.batch_count(),
        batch_manager.instance_count(),
        lod_counts[0],
        lod_counts[1],
        lod_counts[2],
        lod_counts[3],
        cross_fading
    );
}
