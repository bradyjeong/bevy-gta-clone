//! Vehicle physics integration example with audio
//!
//! This example demonstrates:
//! - Vehicle physics simulation with Rapier3D
//! - Audio integration with bevy_kira_audio
//! - Real-time performance monitoring
//! - Interactive vehicle controls

use amp_engine::prelude::*;
use amp_gameplay::prelude::*;
use amp_gameplay::vehicle::components::{Engine, Steering, Suspension};
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    input::common_conditions::input_pressed,
    prelude::*,
};

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        LogDiagnosticsPlugin::default(),
        FrameTimeDiagnosticsPlugin::default(),
    ))
    .add_plugins(AAAPlugins::default());

    #[cfg(feature = "rapier3d_030")]
    {
        app.add_plugins(bevy_rapier3d::render::RapierDebugRenderPlugin::default());
    }

    app.add_systems(Startup, setup_scene)
        .add_systems(
            Update,
            (
                handle_keyboard_input,
                update_camera,
                display_vehicle_info,
                spawn_vehicle_on_keypress.run_if(input_pressed(KeyCode::KeyV)),
            ),
        )
        .run();
}

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct PlayerVehicle;

#[derive(Component)]
struct VehicleInfoText;

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainCamera,
    ));

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Plane3d::default().mesh().size(100.0, 100.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.5, 0.3),
            ..default()
        })),
        Transform::default(),
    ));

    // Ground collider
    #[cfg(feature = "rapier3d_030")]
    commands.spawn((
        bevy_rapier3d::geometry::Collider::cuboid(50.0, 0.1, 50.0),
        Transform::from_xyz(0.0, -0.1, 0.0),
        GlobalTransform::default(),
    ));

    // Some obstacles
    for i in 0..10 {
        let x = (i as f32 - 5.0) * 8.0;
        let z = 10.0;

        let obstacle_bundle = (
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(2.0, 2.0, 2.0)))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.7, 0.6),
                ..default()
            })),
            Transform::from_xyz(x, 1.0, z),
        );

        #[cfg(feature = "rapier3d_030")]
        {
            commands.spawn((
                obstacle_bundle,
                bevy_rapier3d::geometry::Collider::cuboid(1.0, 1.0, 1.0),
            ));
        }
        #[cfg(not(feature = "rapier3d_030"))]
        {
            commands.spawn(obstacle_bundle);
        }
    }

    // Spawn initial vehicle
    spawn_vehicle(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(0.0, 2.0, 0.0),
        true,
    );

    // Lighting
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // UI for vehicle info
    commands.spawn((
        Text::new("Vehicle Info"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        VehicleInfoText,
    ));

    // Instructions
    commands.spawn((
        Text::new("Controls:\nWASD - Drive\nSpace - Brake\nV - Spawn Vehicle\nMouse - Look Around"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor::WHITE,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));
}

fn spawn_vehicle(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    is_player: bool,
) {
    let vehicle_bundle = VehicleBundle {
        transform: Transform::from_translation(position),
        global_transform: GlobalTransform::default(),
        visibility: Visibility::default(),
        inherited_visibility: InheritedVisibility::default(),
        view_visibility: ViewVisibility::default(),
        vehicle: amp_gameplay::vehicle::components::Vehicle {
            mass: 1200.0,
            drag_coefficient: 0.3,
            ..default()
        },
        engine: Engine {
            max_torque: 500.0,
            max_rpm: 7000.0,
            idle_rpm: 800.0,
            ..default()
        },
        suspension: Suspension {
            spring_stiffness: 35000.0,
            damper_damping: 3500.0,
            rest_length: 0.4,
            ..default()
        },
        steering: Steering {
            max_angle: 35.0_f32.to_radians(),
            steering_rate: 5.0,
            ..default()
        },
        audio: VehicleAudio {
            engine_sound_enabled: true,
            tire_screech_enabled: true,
            engine_volume: 0.8,
            ..default()
        },
        ..default()
    };

    // Add visual representation
    let base_bundle = (
        vehicle_bundle,
        Mesh3d(meshes.add(Mesh::from(Cuboid::new(2.0, 1.0, 4.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: if is_player {
                Color::srgb(0.8, 0.2, 0.2)
            } else {
                Color::srgb(0.2, 0.8, 0.2)
            },
            ..default()
        })),
        Name::new("Vehicle"),
    );

    let vehicle_entity = {
        #[cfg(feature = "rapier3d_030")]
        {
            commands
                .spawn((
                    base_bundle,
                    bevy_rapier3d::geometry::Collider::cuboid(1.0, 0.5, 2.0),
                    bevy_rapier3d::dynamics::RigidBody::Dynamic,
                    bevy_rapier3d::dynamics::Velocity::default(),
                ))
                .id()
        }
        #[cfg(not(feature = "rapier3d_030"))]
        {
            commands.spawn(base_bundle).id()
        }
    };

    if is_player {
        commands.entity(vehicle_entity).insert(PlayerVehicle);
    }
}

fn handle_keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut vehicle_query: Query<&mut VehicleInput, With<PlayerVehicle>>,
) {
    for mut input in vehicle_query.iter_mut() {
        // Reset inputs
        input.throttle = 0.0;
        input.brake = 0.0;
        input.steering = 0.0;

        // Handle throttle
        if keys.pressed(KeyCode::KeyW) {
            input.throttle = 1.0;
        }
        if keys.pressed(KeyCode::KeyS) {
            input.throttle = -0.5; // Reverse
        }

        // Handle braking
        if keys.pressed(KeyCode::Space) {
            input.brake = 1.0;
        }

        // Handle steering
        if keys.pressed(KeyCode::KeyA) {
            input.steering = -1.0;
        }
        if keys.pressed(KeyCode::KeyD) {
            input.steering = 1.0;
        }
    }
}

fn update_camera(
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<PlayerVehicle>)>,
    vehicle_query: Query<&Transform, (With<PlayerVehicle>, Without<MainCamera>)>,
    time: Res<Time>,
) {
    if let Ok(mut camera_transform) = camera_query.single_mut() {
        if let Ok(vehicle_transform) = vehicle_query.single() {
            // Follow the player vehicle
            let target_pos = vehicle_transform.translation + Vec3::new(0.0, 8.0, 15.0);
            let target_look = vehicle_transform.translation + Vec3::new(0.0, 1.0, 0.0);

            // Smooth camera movement
            let lerp_factor = time.delta_secs() * 2.0;
            camera_transform.translation =
                camera_transform.translation.lerp(target_pos, lerp_factor);
            camera_transform.look_at(target_look, Vec3::Y);
        }
    }
}

#[cfg(feature = "rapier3d_030")]
fn display_vehicle_info(
    mut text_query: Query<&mut Text, With<VehicleInfoText>>,
    vehicle_query: Query<
        (
            &Engine,
            &bevy_rapier3d::dynamics::Velocity,
            &VehicleInput,
            &amp_gameplay::vehicle::components::Vehicle,
        ),
        With<PlayerVehicle>,
    >,
) {
    if let Ok(mut text) = text_query.single_mut() {
        if let Ok((engine, velocity, input, vehicle)) = vehicle_query.single() {
            let speed_kmh = velocity.linvel.length() * 3.6;
            let speed_mph = speed_kmh * 0.621371;

            **text = format!(
                "Vehicle Info:\n\
                Speed: {:.1} km/h ({:.1} mph)\n\
                RPM: {:.0} / {:.0}\n\
                Throttle: {:.1}%\n\
                Brake: {:.1}%\n\
                Steering: {:.1}°\n\
                Mass: {:.0} kg",
                speed_kmh,
                speed_mph,
                engine.rpm,
                engine.max_rpm,
                input.throttle * 100.0,
                input.brake * 100.0,
                input.steering * 30.0, // Convert to degrees
                vehicle.mass
            );
        }
    }
}

#[cfg(not(feature = "rapier3d_030"))]
fn display_vehicle_info(
    mut text_query: Query<&mut Text, With<VehicleInfoText>>,
    vehicle_query: Query<
        (
            &Engine,
            &VehicleInput,
            &amp_gameplay::vehicle::components::Vehicle,
        ),
        With<PlayerVehicle>,
    >,
) {
    if let Ok(mut text) = text_query.single_mut() {
        if let Ok((engine, input, vehicle)) = vehicle_query.single() {
            **text = format!(
                "Vehicle Info:\n\
                Speed: N/A (no Rapier3D)\n\
                Engine RPM: {:.0}/{:.0}\n\
                Throttle: {:.1}%\n\
                Brake: {:.1}%\n\
                Steering: {:.1}°\n\
                Mass: {:.1} kg",
                engine.rpm,
                engine.max_rpm,
                input.throttle * 100.0,
                input.brake * 100.0,
                input.steering * 30.0, // Convert to degrees
                vehicle.mass
            );
        }
    }
}

fn spawn_vehicle_on_keypress(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn a new vehicle at a random position
    let x = (rand::random() % 20) as f32 - 10.0;
    let z = (rand::random() % 20) as f32 - 10.0;
    spawn_vehicle(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(x, 2.0, z),
        false,
    );
}

/// Add a simple random number generator
mod rand {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn random() -> u32 {
        let mut hasher = DefaultHasher::new();
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .hash(&mut hasher);
        let hash = hasher.finish();
        hash as u32
    }
}
