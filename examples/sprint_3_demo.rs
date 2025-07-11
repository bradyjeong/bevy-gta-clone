//! Sprint 3 Demo: Core Gameplay & Physics Integration
//!
//! This demo showcases the completed Sprint 3 deliverables:
//! - Audio system integration with vehicle physics
//! - Event-driven architecture for performance
//! - Real-time performance monitoring
//! - Interactive vehicle controls with audio feedback
//!
//! Controls:
//! - WASD: Drive vehicle
//! - Space: Brake
//! - V: Spawn new vehicle
//! - F1: Toggle debug info
//! - ESC: Exit

use amp_gameplay::prelude::*;
use amp_gameplay::vehicle::components::Engine;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

fn main() {
    println!("🎮 Sprint 3 Demo: Core Gameplay & Physics Integration");
    println!("📊 Performance Target: 60+ FPS with audio integration");
    println!("🎵 Audio Events: Engine sounds based on RPM and throttle");
    println!("🚗 Controls: WASD to drive, Space to brake, V to spawn vehicle");

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(GameplayPlugins)
        .add_systems(Startup, setup_sprint3_demo)
        .add_systems(
            Update,
            (
                handle_demo_input,
                update_performance_display,
                spawn_vehicle_on_v_key,
            ),
        )
        .run();
}

#[derive(Component)]
struct DemoVehicle;

#[derive(Component)]
struct PerformanceText;

fn setup_sprint3_demo(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 15.0, 25.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Ground
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Plane3d::default().mesh().size(50.0, 50.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.5, 0.3),
            ..default()
        })),
        Transform::default(),
    ));

    // Spawn initial demo vehicle
    spawn_demo_vehicle(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(0.0, 2.0, 0.0),
    );

    // Lighting
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Performance display
    commands.spawn((
        Text::new("Sprint 3 Demo - Performance Monitor"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor::WHITE,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        PerformanceText,
    ));

    // Instructions
    commands.spawn((
        Text::new(
            "🎮 Sprint 3 Demo: Core Gameplay & Physics Integration\n\
        📊 Performance Target: 60+ FPS with audio integration\n\
        🎵 Audio Events: Engine sounds based on RPM and throttle\n\
        🚗 Controls: WASD to drive, Space to brake, V to spawn vehicle\n\
        🎯 Press F1 for debug info, ESC to exit",
        ),
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

fn spawn_demo_vehicle(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
) {
    commands.spawn((
        VehicleBundle {
            transform: Transform::from_translation(position),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
            vehicle: amp_gameplay::vehicle::components::Vehicle {
                mass: 1200.0,
                ..default()
            },
            engine: Engine {
                max_torque: 300.0,
                max_rpm: 6500.0,
                idle_rpm: 800.0,
                ..default()
            },
            audio: VehicleAudio {
                engine_sound_enabled: true,
                tire_screech_enabled: true,
                ..default()
            },
            input: VehicleInput::default(),
            ..default()
        },
        Mesh3d(meshes.add(Mesh::from(Cuboid::new(2.0, 1.0, 4.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.2, 0.2),
            ..default()
        })),
        DemoVehicle,
        Name::new("Demo Vehicle"),
    ));
}

fn handle_demo_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut vehicle_query: Query<&mut VehicleInput, With<DemoVehicle>>,
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
            input.throttle = -0.5;
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

fn update_performance_display(
    mut text_query: Query<&mut Text, With<PerformanceText>>,
    diagnostics: Res<DiagnosticsStore>,
    vehicle_query: Query<
        (&Engine, &bevy_rapier3d::dynamics::Velocity, &VehicleInput),
        With<DemoVehicle>,
    >,
    audio_events: Res<Events<VehicleEngineAudioEvent>>,
) {
    if let Ok(mut text) = text_query.single_mut() {
        let fps = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
            .unwrap_or(0.0);

        let frame_time = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
            .and_then(|frame_time| frame_time.smoothed())
            .unwrap_or(0.0);

        let audio_events_count = audio_events.len();

        let mut vehicle_info = String::new();
        if let Ok((engine, velocity, input)) = vehicle_query.single() {
            let speed_kmh = velocity.linvel.length() * 3.6;
            vehicle_info = format!(
                "🚗 Vehicle: {speed_kmh:.1} km/h | RPM: {:.0} | Throttle: {:.1}%",
                engine.rpm,
                input.throttle * 100.0
            );
        }

        let performance_status = if fps >= 60.0 {
            "✅ EXCELLENT"
        } else if fps >= 45.0 {
            "⚠️ GOOD"
        } else {
            "❌ NEEDS OPTIMIZATION"
        };

        **text = format!(
            "🎮 Sprint 3 Demo: Core Gameplay & Physics Integration\n\
            📊 Performance: {fps:.1} FPS ({frame_time:.2}ms) - {performance_status}\n\
            🎵 Audio Events: {audio_events_count} events queued\n\
            {vehicle_info}\n\
            🎯 Target: 60+ FPS with audio integration - {}",
            if fps >= 60.0 {
                "✅ TARGET MET"
            } else {
                "⚠️ OPTIMIZING"
            }
        );
    }
}

fn spawn_vehicle_on_v_key(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if keys.just_pressed(KeyCode::KeyV) {
        // Spawn a new vehicle at a random position
        let x = (rand::random() % 20) as f32 - 10.0;
        let z = (rand::random() % 20) as f32 - 10.0;
        spawn_demo_vehicle(
            &mut commands,
            &mut meshes,
            &mut materials,
            Vec3::new(x, 2.0, z),
        );
        println!("🚗 New vehicle spawned at ({x:.1}, 2.0, {z:.1})");
    }
}

/// Simple random number generator for demo
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
        hasher.finish() as u32
    }
}
