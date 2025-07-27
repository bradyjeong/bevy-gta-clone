//! Vegetation LOD Demo
//!
//! This example demonstrates the vegetation Level of Detail (LOD) system
//! with distance-based culling and performance monitoring.

use amp_render::{
    distance_cache::DistanceCachePlugin,
    vegetation::{VegetationBillboard, VegetationLOD, VegetationLODPlugin, VegetationMeshLOD},
    BatchingPlugin,
};
use bevy::prelude::*;
use config_core::{ConfigLoader, VegetationConfig};

fn main() {
    // Load vegetation configuration
    let loader = ConfigLoader::new();
    let config = loader
        .load_with_merge::<VegetationConfig>()
        .unwrap_or_else(|_| VegetationConfig::default());

    println!("Loaded vegetation config: {:?}", config);

    App::new()
        .add_plugins((DefaultPlugins, BatchingPlugin, DistanceCachePlugin))
        .add_plugins((VegetationLODPlugin,))
        .insert_resource(config)
        .add_systems(Startup, setup_vegetation_demo)
        .add_systems(Update, (move_camera, update_vegetation_info))
        .run();
}

fn setup_vegetation_demo(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Set up camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Create different meshes for LOD levels
    let full_mesh = meshes.add(Mesh::from(Cuboid::new(2.0, 4.0, 2.0)));
    let medium_mesh = meshes.add(Mesh::from(Cuboid::new(1.5, 3.0, 1.5)));
    let billboard_mesh = meshes.add(Mesh::from(Plane3d::default().mesh().size(2.0, 3.0)));

    // Create materials
    let green_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.8, 0.3),
        ..default()
    });

    // Spawn vegetation entities at different distances
    let positions = [
        (0.0, 0.0, 0.0),   // Very close - Full LOD
        (25.0, 0.0, 0.0),  // Close - Full LOD
        (75.0, 0.0, 0.0),  // Medium distance - Medium LOD
        (125.0, 0.0, 0.0), // Far - Medium LOD
        (200.0, 0.0, 0.0), // Very far - Billboard LOD
        (350.0, 0.0, 0.0), // Extremely far - Should be culled
    ];

    for (i, (x, y, z)) in positions.iter().enumerate() {
        commands.spawn((
            Name::new(format!("Vegetation_{}", i)),
            Mesh3d(full_mesh.clone()),
            MeshMaterial3d(green_material.clone()),
            Transform::from_xyz(*x, *y, *z),
            GlobalTransform::default(),
            Visibility::Visible,
            ViewVisibility::default(),
            InheritedVisibility::default(),
            VegetationLOD::new(),
            VegetationMeshLOD::new(
                full_mesh.clone(),
                medium_mesh.clone(),
                billboard_mesh.clone(),
            ),
            VegetationBillboard::new(Handle::default(), Vec2::new(2.0, 3.0)),
        ));
    }

    // Add lighting
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.7, 0.4, 0.0)),
    ));

    // Add ground plane
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Plane3d::default().mesh().size(1000.0, 1000.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.5, 0.3),
            ..default()
        })),
        Transform::from_xyz(0.0, -2.0, 0.0),
    ));

    // Add UI for information display
    commands.spawn((
        Text::new("Vegetation LOD Demo\n\nControls:\nWASD: Move Camera\nMouse: Look Around\n\nLOD Info will appear here..."),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            max_width: Val::Px(300.0),
            ..default()
        },
    ));
}

fn move_camera(
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    let speed = 50.0 * time.delta_secs();
    let mut direction = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::KeyW) {
        direction += camera_transform.forward().as_vec3();
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction += camera_transform.back().as_vec3();
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        direction += camera_transform.left().as_vec3();
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction += camera_transform.right().as_vec3();
    }

    if direction.length() > 0.0 {
        camera_transform.translation += direction.normalize() * speed;
    }
}

fn update_vegetation_info(
    vegetation_query: Query<(&Name, &VegetationLOD, &Transform), With<VegetationLOD>>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<VegetationLOD>)>,
    mut text_query: Query<&mut Text>,
    lod_stats: Option<Res<amp_render::vegetation::VegetationLODStats>>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    let Ok(mut text) = text_query.single_mut() else {
        return;
    };

    let camera_pos = camera_transform.translation;

    let mut info =
        String::from("Vegetation LOD Demo\n\nControls:\nWASD: Move Camera\nMouse: Look Around\n\n");

    if let Some(stats) = lod_stats {
        info.push_str(&format!(
            "LOD Statistics:\nFull: {}\nMedium: {}\nBillboard: {}\nCulled: {}\nTotal Visible: {}\n\n",
            stats.full_count, stats.medium_count, stats.billboard_count,
            stats.culled_count, stats.total_entities
        ));
    }

    info.push_str("Vegetation Entities:\n");

    for (name, veg_lod, transform) in vegetation_query.iter() {
        let distance = camera_pos.distance(transform.translation);
        let lod_level = match veg_lod.detail_level {
            amp_render::vegetation::VegetationDetailLevel::Full => "Full",
            amp_render::vegetation::VegetationDetailLevel::Medium => "Medium",
            amp_render::vegetation::VegetationDetailLevel::Billboard => "Billboard",
            amp_render::vegetation::VegetationDetailLevel::Culled => "Culled",
        };

        info.push_str(&format!(
            "{}: {:.1}m - {}\n",
            name.as_str(),
            distance,
            lod_level
        ));
    }

    **text = info;
}
