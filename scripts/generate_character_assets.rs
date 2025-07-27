//! Generate sample character assets for testing
//! This tool creates a simple rigged humanoid character using Bevy primitives

use bevy::prelude::*;
use bevy::gltf::GltfSaver;
use bevy::scene::SceneBundle;
use std::f32::consts::PI;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, generate_character_assets)
        .run();
}

fn generate_character_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut scenes: ResMut<Assets<Scene>>,
    asset_server: Res<AssetServer>,
) {
    // Create a simple humanoid character
    let character_entity = create_simple_humanoid(&mut commands, &mut meshes, &mut materials);
    
    // Save as GLTF
    let scene = Scene::new(commands.entity(character_entity).clone());
    let scene_handle = scenes.add(scene);
    
    info!("Generated simple humanoid character. Save manually as GLTF.");
}

fn create_simple_humanoid(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    let material = materials.add(StandardMaterial {
        base_color: Color::rgba(0.8, 0.7, 0.6, 1.0), // Skin tone
        ..default()
    });

    // Create root bone (Hips)
    let root = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere::new(0.15).mesh().ico(4).unwrap()),
            material: material.clone(),
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            ..default()
        },
        Name::new("Hips"),
    )).id();

    // Spine chain
    let spine = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Capsule3d::new(0.1, 0.3).mesh()),
            material: material.clone(),
            transform: Transform::from_xyz(0.0, 0.15, 0.0),
            ..default()
        },
        Name::new("Spine"),
    )).id();

    let chest = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Capsule3d::new(0.15, 0.4).mesh()),
            material: material.clone(),
            transform: Transform::from_xyz(0.0, 0.35, 0.0),
            ..default()
        },
        Name::new("Chest"),
    )).id();

    // Head
    let neck = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere::new(0.08).mesh().ico(4).unwrap()),
            material: material.clone(),
            transform: Transform::from_xyz(0.0, 0.15, 0.0),
            ..default()
        },
        Name::new("Neck"),
    )).id();

    let head = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere::new(0.12).mesh().ico(4).unwrap()),
            material: material.clone(),
            transform: Transform::from_xyz(0.0, 0.2, 0.0),
            ..default()
        },
        Name::new("Head"),
    )).id();

    // Left Arm
    let left_shoulder = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere::new(0.08).mesh().ico(4).unwrap()),
            material: material.clone(),
            transform: Transform::from_xyz(-0.25, 0.0, 0.0),
            ..default()
        },
        Name::new("LeftShoulder"),
    )).id();

    let left_upper_arm = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Capsule3d::new(0.06, 0.25).mesh()),
            material: material.clone(),
            transform: Transform::from_xyz(-0.15, 0.0, 0.0),
            ..default()
        },
        Name::new("LeftUpperArm"),
    )).id();

    let left_forearm = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Capsule3d::new(0.05, 0.2).mesh()),
            material: material.clone(),
            transform: Transform::from_xyz(-0.12, 0.0, 0.0),
            ..default()
        },
        Name::new("LeftForeArm"),
    )).id();

    let left_hand = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere::new(0.06).mesh().ico(4).unwrap()),
            material: material.clone(),
            transform: Transform::from_xyz(-0.1, 0.0, 0.0),
            ..default()
        },
        Name::new("LeftHand"),
    )).id();

    // Right Arm (mirror of left)
    let right_shoulder = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere::new(0.08).mesh().ico(4).unwrap()),
            material: material.clone(),
            transform: Transform::from_xyz(0.25, 0.0, 0.0),
            ..default()
        },
        Name::new("RightShoulder"),
    )).id();

    let right_upper_arm = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Capsule3d::new(0.06, 0.25).mesh()),
            material: material.clone(),
            transform: Transform::from_xyz(0.15, 0.0, 0.0),
            ..default()
        },
        Name::new("RightUpperArm"),
    )).id();

    let right_forearm = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Capsule3d::new(0.05, 0.2).mesh()),
            material: material.clone(),
            transform: Transform::from_xyz(0.12, 0.0, 0.0),
            ..default()
        },
        Name::new("RightForeArm"),
    )).id();

    let right_hand = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere::new(0.06).mesh().ico(4).unwrap()),
            material: material.clone(),
            transform: Transform::from_xyz(0.1, 0.0, 0.0),
            ..default()
        },
        Name::new("RightHand"),
    )).id();

    // Left Leg
    let left_upper_leg = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Capsule3d::new(0.08, 0.4).mesh()),
            material: material.clone(),
            transform: Transform::from_xyz(-0.1, -0.2, 0.0),
            ..default()
        },
        Name::new("LeftUpperLeg"),
    )).id();

    let left_lower_leg = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Capsule3d::new(0.06, 0.35).mesh()),
            material: material.clone(),
            transform: Transform::from_xyz(0.0, -0.2, 0.0),
            ..default()
        },
        Name::new("LeftLowerLeg"),
    )).id();

    let left_foot = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Box::new(0.15, 0.08, 0.05)),
            material: material.clone(),
            transform: Transform::from_xyz(0.0, -0.2, 0.1),
            ..default()
        },
        Name::new("LeftFoot"),
    )).id();

    // Right Leg (mirror of left)
    let right_upper_leg = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Capsule3d::new(0.08, 0.4).mesh()),
            material: material.clone(),
            transform: Transform::from_xyz(0.1, -0.2, 0.0),
            ..default()
        },
        Name::new("RightUpperLeg"),
    )).id();

    let right_lower_leg = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Capsule3d::new(0.06, 0.35).mesh()),
            material: material.clone(),
            transform: Transform::from_xyz(0.0, -0.2, 0.0),
            ..default()
        },
        Name::new("RightLowerLeg"),
    )).id();

    let right_foot = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Box::new(0.15, 0.08, 0.05)),
            material: material.clone(),
            transform: Transform::from_xyz(0.0, -0.2, 0.1),
            ..default()
        },
        Name::new("RightFoot"),
    )).id();

    // Build hierarchy
    commands.entity(root).push_children(&[spine]);
    commands.entity(spine).push_children(&[chest]);
    commands.entity(chest).push_children(&[neck, left_shoulder, right_shoulder]);
    commands.entity(neck).push_children(&[head]);
    
    commands.entity(left_shoulder).push_children(&[left_upper_arm]);
    commands.entity(left_upper_arm).push_children(&[left_forearm]);
    commands.entity(left_forearm).push_children(&[left_hand]);
    
    commands.entity(right_shoulder).push_children(&[right_upper_arm]);
    commands.entity(right_upper_arm).push_children(&[right_forearm]);
    commands.entity(right_forearm).push_children(&[right_hand]);
    
    commands.entity(root).push_children(&[left_upper_leg, right_upper_leg]);
    commands.entity(left_upper_leg).push_children(&[left_lower_leg]);
    commands.entity(left_lower_leg).push_children(&[left_foot]);
    commands.entity(right_upper_leg).push_children(&[right_lower_leg]);
    commands.entity(right_lower_leg).push_children(&[right_foot]);

    root
}
