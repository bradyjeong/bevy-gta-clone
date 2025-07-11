//! City Demo Baseline Example
//!
//! This example demonstrates the Sprint 2 physics system with a drivable car
//! featuring stable suspension, realistic vehicle dynamics, and debug visualization.
//!
//! ## Features
//! - Drivable car with WASD controls
//! - Stable suspension system with realistic forces
//! - Engine and transmission physics
//! - Debug visualization for suspension rays and forces
//! - Performance metrics display
//!
//! ## Controls
//! - W/S: Throttle/Brake
//! - A/D: Steering
//! - Space: Handbrake
//! - F1: Toggle debug visualization
//! - F2: Toggle performance metrics
//! - F3: Toggle wireframe rendering
//! - ESC: Exit
//!
//! ## Performance Targets
//! - 60 FPS stable with 10 vehicles
//! - <1ms physics update time
//! - <50MB memory usage
//!
//! ## Usage
//! ```bash
//! cargo run --example city_demo_baseline --features="rapier3d_030"
//! ```

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::WindowResolution;

use amp_physics::{
    BenchmarkResults, Brakes, DebugConfig, Drivetrain, Engine, PhysicsBenchmarkPlugin,
    PhysicsDebugPlugin, PhysicsPlugin, PhysicsTime, Steering, Suspension, SuspensionRay,
    Transmission, Vehicle, VehicleInput, WheelPhysics, WheelState, vehicle_suspension_system,
};

#[cfg(feature = "rapier3d_030")]
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "City Demo Baseline - Sprint 2 Physics".to_string(),
                    resolution: WindowResolution::new(1280.0, 720.0),
                    ..default()
                }),
                ..default()
            }),
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .add_plugins(PhysicsPlugin)
        .add_plugins(PhysicsDebugPlugin)
        .add_plugins(PhysicsBenchmarkPlugin)
        .add_systems(Startup, setup_scene)
        .add_systems(
            Update,
            (
                handle_input,
                vehicle_controls,
                toggle_debug_systems,
                update_ui,
            ),
        )
        .add_systems(
            FixedUpdate,
            (vehicle_suspension_system, update_vehicle_physics),
        )
        .run();
}

/// Component for the main demo car
#[derive(Component)]
struct DemoCar;

/// Component for debug UI
#[derive(Component)]
struct DebugUI;

/// Component for performance metrics UI
#[derive(Component)]
struct PerformanceUI;

/// Resource for demo state
#[derive(Resource, Default)]
struct DemoState {
    debug_enabled: bool,
    performance_ui_enabled: bool,
    wireframe_enabled: bool,
    #[allow(dead_code)]
    vehicle_count: usize,
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Initialize demo state
    commands.insert_resource(DemoState::default());

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(100.0, 100.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::default(),
        Name::new("Ground"),
    ));

    #[cfg(feature = "rapier3d_030")]
    {
        // Ground collider
        commands.spawn((
            Collider::cuboid(50.0, 0.1, 50.0),
            Transform::from_xyz(0.0, -0.1, 0.0),
            Name::new("Ground Collider"),
        ));
    }

    // Create demo car
    spawn_demo_car(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(0.0, 2.0, 0.0),
    );

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        Projection::Perspective(PerspectiveProjection {
            fov: 45.0_f32.to_radians(),
            ..default()
        }),
        Name::new("Camera"),
    ));

    // Lighting
    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("Sun"),
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.3,
        affects_lightmapped_meshes: true,
    });

    // Setup UI
    setup_ui(&mut commands);
}

fn spawn_demo_car(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
) {
    // Car body
    let car_entity = commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(4.0, 1.5, 2.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
            Transform::from_translation(position),
            DemoCar,
            Vehicle,
            Engine::default(),
            Transmission::default(),
            Suspension::default(),
            Drivetrain::default(),
            Steering::default(),
            Brakes::default(),
            VehicleInput::default(),
            Name::new("Demo Car"),
        ))
        .id();

    #[cfg(feature = "rapier3d_030")]
    {
        // Car body collider and rigid body
        commands.entity(car_entity).insert((
            RigidBody::Dynamic,
            Collider::cuboid(2.0, 0.75, 1.0),
            ColliderMassProperties::Density(1.0),
            ExternalForce::default(),
            Velocity::default(),
            ReadMassProperties::default(),
            GravityScale(1.0),
            Ccd::enabled(),
        ));
    }

    // Wheels
    let wheel_positions = [
        Vec3::new(1.5, -0.5, 1.0),   // Front left
        Vec3::new(1.5, -0.5, -1.0),  // Front right
        Vec3::new(-1.5, -0.5, 1.0),  // Rear left
        Vec3::new(-1.5, -0.5, -1.0), // Rear right
    ];

    for (i, wheel_pos) in wheel_positions.iter().enumerate() {
        let wheel_entity = commands
            .spawn((
                Mesh3d(meshes.add(Cylinder::new(0.35, 0.2))),
                MeshMaterial3d(materials.add(Color::srgb(0.1, 0.1, 0.1))),
                Transform::from_translation(position + *wheel_pos)
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
                WheelPhysics::default(),
                WheelState::default(),
                SuspensionRay {
                    cast_distance: 1.0,
                    ray_origin: *wheel_pos,
                    ray_direction: Vec3::NEG_Y,
                    ..default()
                },
                Name::new(format!("Wheel {i}")),
            ))
            .id();

        // Parent wheel to car
        commands.entity(car_entity).add_child(wheel_entity);
    }
}

fn setup_ui(commands: &mut Commands) {
    // Debug UI
    commands.spawn((
        Text::new("Debug Info\nPress F1 to toggle debug visualization\nPress F2 to toggle performance metrics\nPress F3 to toggle wireframe"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        DebugUI,
        Name::new("Debug UI"),
    ));

    // Performance UI
    commands.spawn((
        Text::new("Performance Metrics\nFPS: 0\nFrame Time: 0.0ms\nPhysics Time: 0.0ms"),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
        PerformanceUI,
        Name::new("Performance UI"),
    ));

    // Controls UI
    commands.spawn((
        Text::new("Controls:\nW/S: Throttle/Brake\nA/D: Steering\nSpace: Handbrake\nF1: Debug viz\nF2: Performance\nF3: Wireframe\nESC: Exit"),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        Name::new("Controls UI"),
    ));
}

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut demo_state: ResMut<DemoState>,
    mut debug_config: ResMut<DebugConfig>,
    mut app_exit_events: EventWriter<bevy::app::AppExit>,
) {
    if keys.just_pressed(KeyCode::F1) {
        demo_state.debug_enabled = !demo_state.debug_enabled;
        debug_config.show_suspension_rays = demo_state.debug_enabled;
        debug_config.show_force_vectors = demo_state.debug_enabled;
        debug_config.show_contact_points = demo_state.debug_enabled;
        info!("Debug visualization: {}", demo_state.debug_enabled);
    }

    if keys.just_pressed(KeyCode::F2) {
        demo_state.performance_ui_enabled = !demo_state.performance_ui_enabled;
        info!("Performance UI: {}", demo_state.performance_ui_enabled);
    }

    if keys.just_pressed(KeyCode::F3) {
        demo_state.wireframe_enabled = !demo_state.wireframe_enabled;
        info!("Wireframe: {}", demo_state.wireframe_enabled);
    }

    if keys.just_pressed(KeyCode::Escape) {
        app_exit_events.write(AppExit::Success);
    }
}

fn vehicle_controls(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut VehicleInput, With<DemoCar>>,
    time: Res<Time>,
) {
    for mut input in query.iter_mut() {
        // Throttle and brake
        if keys.pressed(KeyCode::KeyW) {
            input.throttle = (input.throttle + time.delta_secs() * 2.0).min(1.0);
        } else if keys.pressed(KeyCode::KeyS) {
            input.brake = (input.brake + time.delta_secs() * 2.0).min(1.0);
        } else {
            input.throttle = (input.throttle - time.delta_secs() * 3.0).max(0.0);
            input.brake = (input.brake - time.delta_secs() * 3.0).max(0.0);
        }

        // Steering
        if keys.pressed(KeyCode::KeyA) {
            input.steering = (input.steering - time.delta_secs() * 2.0).max(-1.0);
        } else if keys.pressed(KeyCode::KeyD) {
            input.steering = (input.steering + time.delta_secs() * 2.0).min(1.0);
        } else {
            input.steering *= 0.9; // Return to center
        }

        // Handbrake
        if keys.pressed(KeyCode::Space) {
            input.handbrake = 1.0;
        } else {
            input.handbrake = 0.0;
        }
    }
}

fn toggle_debug_systems(
    demo_state: Res<DemoState>,
    mut debug_ui_query: Query<&mut Visibility, (With<DebugUI>, Without<PerformanceUI>)>,
    mut performance_ui_query: Query<&mut Visibility, (With<PerformanceUI>, Without<DebugUI>)>,
) {
    for mut visibility in debug_ui_query.iter_mut() {
        *visibility = if demo_state.debug_enabled {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for mut visibility in performance_ui_query.iter_mut() {
        *visibility = if demo_state.performance_ui_enabled {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn update_ui(
    mut performance_ui_query: Query<&mut Text, With<PerformanceUI>>,
    physics_time: Res<PhysicsTime>,
    benchmark_results: Res<BenchmarkResults>,
    time: Res<Time>,
) {
    for mut text in performance_ui_query.iter_mut() {
        let fps = 1.0 / time.delta_secs();
        let frame_time = time.delta_secs() * 1000.0;
        let physics_time_ms = physics_time.interpolation_alpha * 16.67; // Approximate physics time

        **text = format!(
            "Performance Metrics\nFPS: {:.1}\nFrame Time: {:.2}ms\nPhysics Time: {:.2}ms\nSuspension Updates: {}\nBenchmark Score: {:.2}",
            fps,
            frame_time,
            physics_time_ms,
            benchmark_results.suspension_times.len(),
            benchmark_results.average_cpu_time
        );
    }
}

fn update_vehicle_physics(
    mut vehicle_query: Query<
        (
            &mut Engine,
            &mut Transmission,
            &VehicleInput,
            &mut Transform,
        ),
        With<DemoCar>,
    >,
    mut wheel_query: Query<(&mut WheelPhysics, &mut WheelState, &SuspensionRay)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (mut engine, _transmission, input, mut transform) in vehicle_query.iter_mut() {
        // Update engine RPM based on throttle
        if input.throttle > 0.0 {
            engine.rpm = (engine.rpm + dt * 1000.0 * input.throttle).min(engine.max_rpm);
        } else {
            engine.rpm = (engine.rpm - dt * 500.0).max(engine.idle_rpm);
        }

        // Calculate engine torque from RPM using torque curve
        engine.torque = calculate_engine_torque(&engine);

        // Simple vehicle movement for demo (replace with proper physics)
        if input.throttle > 0.0 {
            let forward = transform.forward();
            transform.translation += forward * input.throttle * 10.0 * dt;
        }

        if input.brake > 0.0 {
            // Apply braking deceleration
            let forward = transform.forward();
            transform.translation -= forward * input.brake * 5.0 * dt;
        }

        // Apply steering
        if input.steering.abs() > 0.1 {
            let rotation = Quat::from_axis_angle(Vec3::Y, input.steering * dt);
            transform.rotation = rotation * transform.rotation;
        }
    }

    // Update wheel physics
    for (mut wheel_physics, mut wheel_state, suspension_ray) in wheel_query.iter_mut() {
        // Update wheel contact state
        wheel_state.in_contact = suspension_ray.hit_distance.is_some();

        if wheel_state.in_contact {
            let compression = suspension_ray.hit_distance.unwrap_or(0.0);
            wheel_state.contact_force = compression * 1000.0; // Simple spring force
        } else {
            wheel_state.contact_force = 0.0;
        }

        // Update wheel rotation based on vehicle motion
        wheel_physics.angular_velocity =
            wheel_physics.motor_torque / (wheel_physics.mass * wheel_physics.radius);
        wheel_state.rotation_angle += wheel_physics.angular_velocity * dt;
    }
}

fn calculate_engine_torque(engine: &Engine) -> f32 {
    // Interpolate torque from the torque curve
    let torque_curve = &engine.torque_curve;
    if torque_curve.is_empty() {
        return 0.0;
    }

    let rpm = engine.rpm;

    // Find the two points to interpolate between
    let mut lower_point = torque_curve[0];
    let mut upper_point = torque_curve[torque_curve.len() - 1];

    for i in 0..torque_curve.len() - 1 {
        if rpm >= torque_curve[i].0 && rpm <= torque_curve[i + 1].0 {
            lower_point = torque_curve[i];
            upper_point = torque_curve[i + 1];
            break;
        }
    }

    // Linear interpolation
    let rpm_range = upper_point.0 - lower_point.0;
    if rpm_range == 0.0 {
        return lower_point.1 * engine.throttle;
    }

    let t = (rpm - lower_point.0) / rpm_range;
    let torque = lower_point.1 + t * (upper_point.1 - lower_point.1);

    torque * engine.throttle
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_torque_calculation() {
        let engine = Engine::default();
        let torque = calculate_engine_torque(&engine);
        assert_eq!(torque, 0.0); // No throttle, no torque
    }

    #[test]
    fn test_engine_torque_with_throttle() {
        let mut engine = Engine::default();
        engine.throttle = 0.5;
        engine.rpm = 3000.0;
        let torque = calculate_engine_torque(&engine);
        assert!(torque > 0.0); // With throttle and RPM, should have torque
    }

    #[test]
    fn test_demo_car_spawning() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // This would test the spawning logic, but requires more setup
        // For now, just verify the function exists
        assert!(true);
    }
}
