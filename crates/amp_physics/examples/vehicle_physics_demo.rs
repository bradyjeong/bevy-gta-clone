//! Vehicle physics demonstration with debug visualization.
//!
//! This example shows how to set up a complete vehicle physics simulation
//! with suspension, drivetrain, and debug visualization.

use amp_physics::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (keyboard_input, update_camera, toggle_debug))
        .run();
}

/// System to set up the demo scene.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut physics_config: ResMut<PhysicsConfig>,
    mut debug_config: ResMut<DebugConfig>,
) {
    // Enable debug rendering
    physics_config.debug_rendering = true;
    debug_config.enabled = true;
    debug_config.show_suspension_rays = true;
    debug_config.show_force_vectors = true;
    debug_config.show_contact_points = true;
    debug_config.show_engine_state = true;

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::from_xyz(0.0, -0.1, 0.0),
    ));

    // Spawn demo vehicles
    spawn_demo_vehicle(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(0.0, 1.0, 0.0),
    );
    spawn_demo_vehicle(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(10.0, 1.0, 0.0),
    );
    spawn_demo_vehicle(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(-10.0, 1.0, 0.0),
    );

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 15.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Lighting
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // UI Instructions
    commands.spawn((
        Text::new("Vehicle Physics Demo\n\nControls:\n  WASD: Drive vehicles\n  Space: Brake\n  F1: Toggle debug visualization\n  F2: Toggle performance metrics\n\nPhysics: 60Hz fixed timestep\nDebug: Green=suspension rays, Red=forces, Blue=contact points"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

/// Spawn a demo vehicle with realistic physics setup.
fn spawn_demo_vehicle(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
) {
    // Vehicle body
    let vehicle_entity = commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(2.0, 0.5, 4.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.7, 0.2, 0.2))),
            Transform::from_translation(position),
            GlobalTransform::from(Transform::from_translation(position)),
            // Physics components
            Engine {
                max_rpm: 6000.0,
                idle_rpm: 800.0,
                max_torque: 350.0,
                torque_curve: vec![
                    (0.0, 0.0),
                    (1000.0, 200.0),
                    (2000.0, 300.0),
                    (3000.0, 350.0),
                    (4000.0, 340.0),
                    (5000.0, 300.0),
                    (6000.0, 250.0),
                ],
                engine_braking: 40.0,
                throttle: 0.0,
                rpm: 800.0,
                torque: 0.0,
                fuel_consumption: 15.0,
            },
            Transmission {
                gear_ratios: vec![3.5, 2.1, 1.4, 1.0, 0.8],
                final_drive_ratio: 4.1,
                current_gear: 1,
            },
            Steering {
                angle: 0.0,
                max_angle: 35.0f32.to_radians(),
                steering_rate: 5.0,
                return_force: 5.0,
                wheelbase: 2.7,
                track_width: 1.5,
            },
            Brakes {
                brake_input: 0.0,
                max_brake_torque: 8000.0,
                brake_bias: 0.6,
                abs_enabled: true,
                abs_threshold: 0.1,
            },
            VehicleInput {
                throttle: 0.0,
                brake: 0.0,
                steering: 0.0,
                handbrake: 0.0,
                smoothing: 0.15,
                deadzone: 0.05,
            },
            Suspension {
                spring_stiffness: 35000.0,
                damper_damping: 3500.0,
                rest_length: 0.4,
                max_compression: 0.15,
                max_extension: 0.15,
                anti_roll_bar_stiffness: 1000.0,
                travel: 0.3,
            },
        ))
        .id();

    // Wheel positions relative to vehicle center
    let wheel_positions = [
        Vec3::new(-1.0, -0.25, 1.5),  // Front left
        Vec3::new(1.0, -0.25, 1.5),   // Front right
        Vec3::new(-1.0, -0.25, -1.5), // Rear left
        Vec3::new(1.0, -0.25, -1.5),  // Rear right
    ];

    // Create suspension rays and wheels
    for wheel_position in wheel_positions.iter() {
        let global_wheel_pos = position + *wheel_position;

        // Wheel visual
        let wheel_entity = commands
            .spawn((
                Mesh3d(meshes.add(Cylinder::new(0.35, 0.2))),
                MeshMaterial3d(materials.add(Color::srgb(0.1, 0.1, 0.1))),
                Transform::from_translation(global_wheel_pos)
                    .with_rotation(Quat::from_rotation_x(90.0f32.to_radians())),
                GlobalTransform::from(Transform::from_translation(global_wheel_pos)),
                // Physics components
                SuspensionRay {
                    cast_distance: 1.0,
                    ray_origin: *wheel_position,
                    ray_direction: Vec3::NEG_Y,
                    hit_distance: None,
                    hit_normal: None,
                    hit_point: None,
                    previous_hit_distance: None,
                    compression_velocity: 0.0,
                },
                WheelState {
                    rotation_angle: 0.0,
                    radius: 0.35,
                    width: 0.2,
                    mass: 20.0,
                    in_contact: false,
                    contact_force: 0.0,
                    angular_velocity: 0.0,
                },
            ))
            .id();

        // Parent wheel to vehicle
        commands.entity(vehicle_entity).add_child(wheel_entity);
    }
}

/// System to handle keyboard input for vehicle control.
fn keyboard_input(keys: Res<ButtonInput<KeyCode>>, mut input_query: Query<&mut VehicleInput>) {
    for mut input in input_query.iter_mut() {
        // Reset inputs
        input.throttle = 0.0;
        input.brake = 0.0;
        input.steering = 0.0;
        input.handbrake = 0.0;

        // Throttle and brake
        if keys.pressed(KeyCode::KeyW) {
            input.throttle = 1.0;
        }
        if keys.pressed(KeyCode::KeyS) {
            input.throttle = -0.5; // Reverse
        }
        if keys.pressed(KeyCode::Space) {
            input.brake = 1.0;
        }

        // Steering
        if keys.pressed(KeyCode::KeyA) {
            input.steering = -1.0;
        }
        if keys.pressed(KeyCode::KeyD) {
            input.steering = 1.0;
        }

        // Handbrake
        if keys.pressed(KeyCode::KeyH) {
            input.handbrake = 1.0;
        }
    }
}

/// System to update camera position to follow vehicles.
fn update_camera(
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<Engine>)>,
    vehicle_query: Query<&Transform, (With<Engine>, Without<Camera3d>)>,
) {
    if let Ok(mut camera_transform) = camera_query.single_mut() {
        if let Ok(vehicle_transform) = vehicle_query.single() {
            let target_pos = vehicle_transform.translation + Vec3::new(0.0, 10.0, 10.0);
            camera_transform.translation = camera_transform.translation.lerp(target_pos, 0.02);
            camera_transform.look_at(vehicle_transform.translation, Vec3::Y);
        }
    }
}

/// System to toggle debug visualization.
fn toggle_debug(
    keys: Res<ButtonInput<KeyCode>>,
    mut debug_config: ResMut<DebugConfig>,
    mut physics_config: ResMut<PhysicsConfig>,
) {
    if keys.just_pressed(KeyCode::F1) {
        debug_config.enabled = !debug_config.enabled;
        physics_config.debug_rendering = debug_config.enabled;

        info!(
            "Debug visualization: {}",
            if debug_config.enabled { "ON" } else { "OFF" }
        );
    }

    if keys.just_pressed(KeyCode::F2) {
        physics_config.performance_monitoring = !physics_config.performance_monitoring;

        info!(
            "Performance monitoring: {}",
            if physics_config.performance_monitoring {
                "ON"
            } else {
                "OFF"
            }
        );
    }
}
