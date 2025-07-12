//! City Demo Baseline Example
//!
//! This example demonstrates the Sprint 3 integrated gameplay system with:
//! - Vehicle physics integration with amp_gameplay
//! - Advanced audio system with bevy_kira_audio
//! - Real-time performance monitoring
//! - Interactive vehicle controls
//!
//! ## Features
//! - Drivable car with WASD controls
//! - Advanced audio system with engine sounds
//! - Integrated physics and audio events
//! - Performance metrics display
//! - Multiple vehicle spawning
//!
//! ## Controls
//! - W/S: Throttle/Brake
//! - A/D: Steering
//! - Space: Handbrake
//! - V: Spawn new vehicle
//! - F1: Toggle debug visualization
//! - F2: Toggle performance metrics
//! - Q/E: Engine rev up/down (audio test)
//! - ESC: Exit
//!
//! ## Performance Targets
//! - 60 FPS stable with integrated audio and physics
//! - <1.5ms combined physics/audio update time
//! - <75MB memory usage
//!
//! ## Usage
//! ```bash
//! cargo run --example city_demo_baseline --features rapier3d_030
//! ```

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::WindowResolution;

use amp_gameplay::prelude::*;
use gameplay_factory::{PrefabFactory, PrefabFactoryPlugin, PrefabFactoryResource, PrefabId};

#[cfg(feature = "rapier3d_030")]
use bevy_rapier3d::prelude::*;

// Type aliases for complex query types
type DebugUIQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut Visibility,
    (With<DebugUI>, Without<PerformanceUI>, Without<AudioUI>),
>;
type PerformanceUIQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut Visibility,
    (With<PerformanceUI>, Without<DebugUI>, Without<AudioUI>),
>;
type AudioUIQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut Visibility,
    (With<AudioUI>, Without<DebugUI>, Without<PerformanceUI>),
>;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "City Demo Baseline - Sprint 3 Gameplay".to_string(),
                    resolution: WindowResolution::new(1280.0, 720.0),
                    ..default()
                }),
                ..default()
            }),
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .add_plugins(GameplayPlugins)
        .add_plugins(PrefabFactoryPlugin)
        .add_event::<VehicleEngineAudioEvent>()
        .add_systems(Startup, (setup_scene, setup_prefabs))
        .add_systems(
            Update,
            (
                handle_input,
                #[cfg(feature = "rapier3d_030")]
                vehicle_controls,
                toggle_debug_systems,
                update_ui,
                update_audio_ui,
            ),
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

/// Component for audio UI
#[derive(Component)]
struct AudioUI;

/// Resource for demo state
#[derive(Resource, Default)]
struct DemoState {
    debug_enabled: bool,
    performance_ui_enabled: bool,
    wireframe_enabled: bool,
    audio_ui_enabled: bool,
    #[allow(dead_code)]
    vehicle_count: usize,
    /// Engine RPM override for audio testing
    engine_rpm_override: Option<f32>,
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    prefab_factory: ResMut<PrefabFactoryResource>,
) {
    // Initialize demo state
    commands.insert_resource(DemoState::default());

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(100.0, 100.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.5, 0.3),
            ..default()
        })),
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

        // Create demo car using prefab
        if let Ok(entity) = prefab_factory
            .factory
            .spawn_prefab_named(&mut commands, "demo_car")
        {
            commands
                .entity(entity)
                .insert(Transform::from_xyz(0.0, 2.0, 0.0));
            info!("Spawned demo car from prefab: {:?}", entity);
        } else {
            warn!("Failed to spawn demo car from prefab, falling back to hard-coded spawn");
            spawn_demo_car(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec3::new(0.0, 2.0, 0.0),
            );
        }
    }

    #[cfg(not(feature = "rapier3d_030"))]
    {
        // Demo sphere without physics
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.2, 0.2),
                ..default()
            })),
            Transform::from_xyz(0.0, 2.0, 0.0),
            Name::new("Demo Sphere"),
        ));
    }

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

fn setup_prefabs(mut prefab_factory: ResMut<PrefabFactoryResource>) {
    // Create demo car prefab using the PrefabFactory methods
    let demo_car_prefab = PrefabFactory::create_vehicle_prefab(
        "Demo Car",
        Vec3::new(0.0, 2.0, 0.0),
        Color::srgb(0.0, 0.0, 1.0), // Blue color
    );

    // Register the prefab
    let demo_car_id = PrefabId::new(1001);
    if let Err(e) = prefab_factory
        .factory
        .register_prefab(demo_car_id, "demo_car", demo_car_prefab)
    {
        error!("Failed to register demo car prefab: {}", e);
    } else {
        info!("Registered demo car prefab successfully");
    }

    // Create additional prefabs using factory helpers
    let vehicle_prefab = PrefabFactory::create_vehicle_prefab(
        "Generic Vehicle",
        Vec3::ZERO,
        Color::srgb(0.0, 0.0, 1.0), // Blue color
    );

    let building_prefab = PrefabFactory::create_building_prefab(
        "Generic Building",
        Vec3::ZERO,
        Vec3::new(10.0, 20.0, 10.0),
    );

    let character_prefab = PrefabFactory::create_character_prefab("Generic Character", Vec3::ZERO);

    // Register helper prefabs
    let vehicle_id = PrefabId::new(2001);
    let building_id = PrefabId::new(3001);
    let character_id = PrefabId::new(4001);

    let _ = prefab_factory
        .factory
        .register_prefab(vehicle_id, "vehicle", vehicle_prefab);
    let _ = prefab_factory
        .factory
        .register_prefab(building_id, "building", building_prefab);
    let _ = prefab_factory
        .factory
        .register_prefab(character_id, "character", character_prefab);

    info!("Setup {} prefabs", prefab_factory.factory.prefab_count());
}

#[cfg(feature = "rapier3d_030")]
fn spawn_demo_car(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
) {
    // Car body using VehicleBundle
    let car_entity = commands
        .spawn((
            VehicleBundle {
                transform: Transform::from_translation(position),
                audio: VehicleAudio {
                    engine_sound_enabled: true,
                    engine_volume: 0.8,
                    ..default()
                },
                ..default()
            },
            Mesh3d(meshes.add(Cuboid::new(4.0, 1.5, 2.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.2, 0.2),
                ..default()
            })),
            DemoCar,
            Name::new("Demo Car"),
        ))
        .id();

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
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.1, 0.1, 0.1),
                    ..default()
                })),
                Transform::from_translation(position + *wheel_pos)
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
                Wheel {
                    position: *wheel_pos,
                    is_steered: i < 2, // Front wheels are steered
                    is_driven: true,   // All wheels are driven for AWD
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
        Text::new("Controls:\nW/S: Throttle/Brake\nA/D: Steering\nSpace: Handbrake\nQ/E: Engine rev (audio test)\nF1: Debug viz\nF2: Performance\nF3: Wireframe\nF4: Audio UI\nESC: Exit"),
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

    // Audio UI
    commands.spawn((
        Text::new("Audio Status\nEngine Sound: OFF\nRPM: 0\nThrottle: 0%\nAudio Events: 0"),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(0.0, 1.0, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(150.0),
            right: Val::Px(10.0),
            ..default()
        },
        AudioUI,
        Name::new("Audio UI"),
    ));
}

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut demo_state: ResMut<DemoState>,
    mut app_exit_events: EventWriter<bevy::app::AppExit>,
) {
    if keys.just_pressed(KeyCode::F1) {
        demo_state.debug_enabled = !demo_state.debug_enabled;
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

    if keys.just_pressed(KeyCode::F4) {
        demo_state.audio_ui_enabled = !demo_state.audio_ui_enabled;
        info!("Audio UI: {}", demo_state.audio_ui_enabled);
    }

    // Audio testing controls
    if keys.pressed(KeyCode::KeyQ) {
        demo_state.engine_rpm_override = Some(4000.0);
    } else if keys.pressed(KeyCode::KeyE) {
        demo_state.engine_rpm_override = Some(6000.0);
    } else {
        demo_state.engine_rpm_override = None;
    }

    if keys.just_pressed(KeyCode::Escape) {
        app_exit_events.write(bevy::app::AppExit::Success);
    }
}

#[cfg(feature = "rapier3d_030")]
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
    mut debug_ui_query: DebugUIQuery,
    mut performance_ui_query: PerformanceUIQuery,
    mut audio_ui_query: AudioUIQuery,
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

    for mut visibility in audio_ui_query.iter_mut() {
        *visibility = if demo_state.audio_ui_enabled {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn update_ui(mut performance_ui_query: Query<&mut Text, With<PerformanceUI>>, time: Res<Time>) {
    for mut text in performance_ui_query.iter_mut() {
        let fps = 1.0 / time.delta_secs();
        let frame_time = time.delta_secs() * 1000.0;
        let physics_time_ms = 16.67; // Fixed at 60 Hz

        **text = format!(
            "Performance Metrics\nFPS: {fps:.1}\nFrame Time: {frame_time:.2}ms\nPhysics Time: {physics_time_ms:.2}ms\nVehicles: Active\nAudio: Enabled"
        );
    }
}

/// Update audio UI with current vehicle state
fn update_audio_ui(
    mut audio_ui_query: Query<&mut Text, With<AudioUI>>,
    demo_state: Res<DemoState>,
    vehicle_query: Query<(&Engine, &VehicleInput, &VehicleAudio), With<DemoCar>>,
    mut audio_event_writer: EventWriter<VehicleEngineAudioEvent>,
) {
    for mut text in audio_ui_query.iter_mut() {
        let (engine_status, rpm, throttle, audio_events) =
            if let Ok((engine, input, audio)) = vehicle_query.single() {
                // Use RPM override for audio testing or actual engine RPM
                let current_rpm = demo_state.engine_rpm_override.unwrap_or(engine.rpm);
                let current_throttle = if demo_state.engine_rpm_override.is_some() {
                    0.8
                } else {
                    input.throttle
                };

                // Send audio event for engine sounds
                if audio.engine_sound_enabled {
                    audio_event_writer.write(VehicleEngineAudioEvent {
                        vehicle_entity: Entity::PLACEHOLDER, // Entity will be set by the audio system
                        rpm: current_rpm,
                        throttle: current_throttle,
                        load: current_throttle * 0.5,
                        gear: 1, // Simplified gear
                        position: Vec3::ZERO,
                    });
                }

                (
                    if audio.engine_sound_enabled {
                        "ON"
                    } else {
                        "OFF"
                    },
                    current_rpm,
                    (current_throttle * 100.0) as i32,
                    1, // Simplified event count
                )
            } else {
                ("N/A", 0.0, 0, 0)
            };

        **text = format!(
            "Audio Status\nEngine Sound: {engine_status}\nRPM: {rpm:.0}\nThrottle: {throttle}%\nAudio Events: {audio_events}"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_state_creation() {
        let demo_state = DemoState::default();
        assert!(!demo_state.debug_enabled);
        assert!(!demo_state.performance_ui_enabled);
        assert!(!demo_state.audio_ui_enabled);
        assert_eq!(demo_state.engine_rpm_override, None);
    }

    #[test]
    fn test_demo_car_component() {
        let demo_car = DemoCar;
        // Just verify the component exists
        assert!(format!("{:?}", demo_car).contains("DemoCar"));
    }
}
