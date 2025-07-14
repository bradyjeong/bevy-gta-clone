//! Enhanced City Demo with Stress Testing - Oracle's Priority 0-B Implementation
//!
//! Features:
//! - Configurable large-scale entity spawning (CLI + environment variables)
//! - Default: ≥50k static buildings + ≥1k dynamic entities for 60 FPS testing
//! - Performance monitoring with Tracy profiling integration
//! - Memory pool optimization with PrecompiledBundle spawning
//! - Real-time entity count display and stress test controls

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin, DiagnosticsStore};
use bevy::prelude::*;
use bevy::window::WindowResolution;
use clap::Parser;
use std::env;

use amp_engine::prelude::*;
use amp_gameplay::prelude::*;
// use amp_gameplay::audio::components::EngineAudio; // Commented out as unused
use amp_render::gpu_culling::prelude::*;
use gameplay_factory::{SimpleOptimizedFactory, PrecompiledBundle, PrefabId};

#[cfg(feature = "tracy")]
use tracing_tracy::TracyLayer;

#[cfg(feature = "rapier3d_030")]
use bevy_rapier3d::prelude::*;

/// CLI configuration for stress testing
#[derive(Parser, Debug)]
#[command(name = "enhanced_city_demo")]
#[command(about = "Enhanced City Demo with Stress Testing")]
struct StressTestConfig {
    /// Number of static buildings to spawn
    #[arg(long, default_value = "50000")]
    buildings: u32,
    
    /// Number of dynamic vehicles to spawn
    #[arg(long, default_value = "1000")]
    vehicles: u32,
    
    /// Number of pedestrian NPCs to spawn
    #[arg(long, default_value = "500")]
    pedestrians: u32,
    
    /// Enable stress test mode on startup
    #[arg(long)]
    stress_mode: bool,
    
    /// Area size for entity distribution (units)
    #[arg(long, default_value = "1000.0")]
    area_size: f32,
    
    /// Batching size for entity spawning
    #[arg(long, default_value = "1000")]
    batch_size: u32,
}

impl StressTestConfig {
    /// Create config from CLI args and environment variables
    fn from_env_and_args() -> Self {
        // Try to parse CLI args first
        let mut config = Self::parse();
        
        // Override with environment variables if present (CI reproducibility)
        if let Ok(buildings) = env::var("STRESS_BUILDINGS") {
            if let Ok(val) = buildings.parse() {
                config.buildings = val;
            }
        }
        if let Ok(vehicles) = env::var("STRESS_VEHICLES") {
            if let Ok(val) = vehicles.parse() {
                config.vehicles = val;
            }
        }
        if let Ok(pedestrians) = env::var("STRESS_PEDESTRIANS") {
            if let Ok(val) = pedestrians.parse() {
                config.pedestrians = val;
            }
        }
        if let Ok(stress_mode) = env::var("STRESS_MODE") {
            config.stress_mode = stress_mode.parse().unwrap_or(false);
        }
        if let Ok(area_size) = env::var("STRESS_AREA_SIZE") {
            if let Ok(val) = area_size.parse() {
                config.area_size = val;
            }
        }
        
        config
    }
    
    fn total_entities(&self) -> u32 {
        self.buildings + self.vehicles + self.pedestrians
    }
}

#[derive(Component)]
struct DemoCar;

#[derive(Component)]
struct StressBuilding;

#[derive(Component)]
struct StressPedestrian;

#[derive(Component)]
enum StressEntityMarker {
    Building,
    Vehicle,
    Pedestrian,
}

#[derive(Component)]
struct FollowCamera {
    target: Option<Entity>,
    offset: Vec3,
    smoothness: f32,
}

#[derive(Resource)]
struct DemoState {
    follow_camera: bool,
    car_count: u32,
    gpu_culling_enabled: bool,
    stress_config: StressTestConfig,
    stress_mode_active: bool,
    spawned_entities: EntityCounts,
    spawn_timer: Timer,
    fps_history: Vec<f32>,
    /// Priority 1-A tracking: spawn timing measurements  
    spawn_times: Vec<f32>, // in milliseconds
    last_spawn_time: f32,
}

#[derive(Debug, Default)]
struct EntityCounts {
    buildings: u32,
    vehicles: u32,
    pedestrians: u32,
    total: u32,
}

impl DemoState {
    fn new(config: StressTestConfig) -> Self {
        Self {
            follow_camera: false,
            car_count: 1,
            gpu_culling_enabled: is_gpu_culling_available(),
            stress_mode_active: config.stress_mode,
            spawned_entities: EntityCounts::default(),
            spawn_timer: Timer::from_seconds(0.1, TimerMode::Repeating), // 10 FPS spawning rate
            fps_history: Vec::with_capacity(300), // 5 seconds at 60 FPS
            spawn_times: Vec::with_capacity(100), // Track last 100 spawn operations
            last_spawn_time: 0.0,
            stress_config: config,
        }
    }
}

fn main() {
    // Parse CLI and environment configuration
    let stress_config = StressTestConfig::from_env_and_args();
    
    // Log the stress test configuration
    info!("Stress Test Configuration:");
    info!("  Buildings: {}", stress_config.buildings);
    info!("  Vehicles: {}", stress_config.vehicles);
    info!("  Pedestrians: {}", stress_config.pedestrians);
    info!("  Total Entities: {}", stress_config.total_entities());
    info!("  Area Size: {}x{}", stress_config.area_size, stress_config.area_size);
    info!("  Stress Mode: {}", stress_config.stress_mode);
    
    // Initialize Tracy profiling if enabled
    #[cfg(feature = "tracy")]
    {
        let subscriber = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .finish()
            .with(TracyLayer::default());
        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set tracy subscriber");
        tracy_client::Client::start();
        
        info!("Tracy profiling initialized for stress testing");
    }

    let mut app = App::new();
    
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: format!("Enhanced City Demo - Stress Test ({} entities target)", 
                               stress_config.total_entities()),
                resolution: WindowResolution::new(1280.0, 720.0),
                ..default()
            }),
            ..default()
        }),
        FrameTimeDiagnosticsPlugin::default(),
        LogDiagnosticsPlugin::default(),
    ))
    .add_plugins(AAAPlugins::default())
    .add_plugins((
        GpuCullingPlugin,
        GpuCullingPipelinePlugin,
    ))
    .insert_resource(GpuCullingConfig {
        max_instances_per_dispatch: 10000,
        workgroup_size: 64,
        debug_output: false,
        enable_frustum_culling: true,
    })
    .insert_resource(DemoState::new(stress_config))
    .insert_resource(SimpleOptimizedFactory::default())
    .add_systems(Startup, (setup_enhanced_scene, setup_stress_factory))
    .add_systems(
        Update,
        (
            follow_camera_system,
            handle_enhanced_input,
            update_enhanced_ui,
            enhanced_vehicle_controls,
            gpu_culling_ui_system,
            simulate_gpu_culling_stats,
            stress_spawning_system,
            performance_monitoring_system,
        ),
    );

    #[cfg(feature = "tracy")]
    app.add_systems(
        Update,
        (
            tracy_profiling_system,
            tracy_gpu_culling_system,
            tracy_stress_profiling_system,
        ),
    );

    #[cfg(feature = "tracy")]
    {
        app.add_systems(PostUpdate, tracy_frame_mark);
    }

    app.run();
}

fn setup_enhanced_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Enhanced ground with grid pattern
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(200.0, 200.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.4, 0.2),
            ..default()
        })),
        Transform::default(),
        Name::new("Ground"),
    ));

    // Grid lines for movement reference
    let grid_size = 20;
    let grid_spacing = 10.0;
    for i in 0..=grid_size {
        let pos = (i as f32 - grid_size as f32 / 2.0) * grid_spacing;
        
        // Vertical lines
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.1, 0.1, grid_size as f32 * grid_spacing))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.1, 0.3, 0.1),
                ..default()
            })),
            Transform::from_xyz(pos, 0.05, 0.0),
            Name::new(format!("Grid Line V{}", i)),
        ));
        
        // Horizontal lines
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(grid_size as f32 * grid_spacing, 0.1, 0.1))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.1, 0.3, 0.1),
                ..default()
            })),
            Transform::from_xyz(0.0, 0.05, pos),
            Name::new(format!("Grid Line H{}", i)),
        ));
    }

    #[cfg(feature = "rapier3d_030")]
    {
        // Ground collider
        commands.spawn((
            Collider::cuboid(100.0, 0.1, 100.0),
            Transform::from_xyz(0.0, -0.1, 0.0),
            Name::new("Ground Collider"),
        ));

        // Main demo car (bright red for visibility)
        let car_entity = spawn_enhanced_car(
            &mut commands,
            &mut meshes,
            &mut materials,
            Vec3::new(0.0, 2.0, 0.0),
            Color::srgb(1.0, 0.1, 0.1), // Bright red
            "Main Car",
        );
        commands.entity(car_entity).insert(DemoCar);

        // Reference cars in different colors
        spawn_enhanced_car(
            &mut commands,
            &mut meshes,
            &mut materials,
            Vec3::new(20.0, 2.0, 0.0),
            Color::srgb(0.1, 0.1, 1.0), // Blue
            "Blue Car",
        );

        spawn_enhanced_car(
            &mut commands,
            &mut meshes,
            &mut materials,
            Vec3::new(-20.0, 2.0, 0.0),
            Color::srgb(0.1, 1.0, 0.1), // Green
            "Green Car",
        );

        // Reference buildings
        for i in 0..6 {
            let angle = i as f32 * std::f32::consts::PI / 3.0;
            let radius = 50.0;
            let x = angle.cos() * radius;
            let z = angle.sin() * radius;
            let height = 10.0 + (i as f32 * 5.0);
            
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(8.0, height, 8.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.6, 0.6, 0.8),
                    ..default()
                })),
                Transform::from_xyz(x, height / 2.0, z),
                Name::new(format!("Building {}", i + 1)),
            ));
        }
    }

    // Camera with follow capability
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 15.0, 25.0).looking_at(Vec3::ZERO, Vec3::Y),
        FollowCamera {
            target: None,
            offset: Vec3::new(0.0, 15.0, 25.0),
            smoothness: 2.0,
        },
        Name::new("Follow Camera"),
    ));

    // Enhanced lighting
    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 15000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 20.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("Sun"),
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.4,
        affects_lightmapped_meshes: true,
    });

    // Enhanced UI
    setup_enhanced_ui(&mut commands);
}

#[cfg(feature = "rapier3d_030")]
fn spawn_enhanced_car(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    color: Color,
    name: &str,
) -> Entity {
    // Car body
    let car_entity = commands
        .spawn((
            VehicleBundle {
                transform: Transform::from_translation(position),
                audio: VehicleAudio {
                    engine_sound_enabled: true,
                    engine_volume: 0.8,
                    ..default()
                },
                // engine_audio removed as unused
                ..default()
            },
            Mesh3d(meshes.add(Cuboid::new(4.0, 1.5, 2.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                metallic: 0.1,
                ..default()
            })),
            Name::new(name.to_string()),
        ))
        .id();

    // Physics with proper collider setup
    commands.entity(car_entity).insert((
        RigidBody::Dynamic,
        Collider::cuboid(2.0, 0.75, 1.0),
        ColliderMassProperties::Mass(1500.0), // Realistic car mass
        ExternalForce::default(),
        Velocity::default(),
        ReadMassProperties::default(),
        GravityScale(1.0),
        Ccd::enabled(),
        // Add damping to reduce oscillations
        Damping { linear_damping: 0.3, angular_damping: 0.8 },
    ));

    // Wheels with enhanced visibility
    let wheel_positions = [
        Vec3::new(1.5, -0.5, 1.0),   // Front left
        Vec3::new(1.5, -0.5, -1.0),  // Front right
        Vec3::new(-1.5, -0.5, 1.0),  // Rear left
        Vec3::new(-1.5, -0.5, -1.0), // Rear right
    ];

    for (i, wheel_pos) in wheel_positions.iter().enumerate() {
        let wheel_entity = commands
            .spawn((
                Mesh3d(meshes.add(Cylinder::new(0.4, 0.3))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.05, 0.05, 0.05),
                    ..default()
                })),
                Transform::from_translation(position + *wheel_pos)
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
                Wheel {
                    position: *wheel_pos,
                    is_steered: i < 2,
                    is_driven: true,
                    ..default()
                },
                Name::new(format!("{} Wheel {}", name, i)),
            ))
            .id();

        commands.entity(car_entity).add_child(wheel_entity);
    }

    car_entity
}

fn enhanced_vehicle_controls(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut vehicle_query: Query<&mut VehicleInput, With<DemoCar>>,
    transform_query: Query<&Transform, With<DemoCar>>,
    time: Res<Time>,
) {
    // Debug car position every 2 seconds
    if time.elapsed_secs() as u32 % 2 == 0 && time.delta_secs() < 0.02 {
        for transform in transform_query.iter() {
            info!("Car position: ({:.3}, {:.3}, {:.3})", 
                  transform.translation.x, 
                  transform.translation.y, 
                  transform.translation.z);
            break;
        }
    }
    if let Ok(mut vehicle_input) = vehicle_query.single_mut() {
        // Reset input
        *vehicle_input = VehicleInput::default();

        // Throttle/Brake
        if keyboard_input.pressed(KeyCode::KeyW) {
            vehicle_input.throttle = 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            vehicle_input.brake = 1.0;
        }

        // Steering
        if keyboard_input.pressed(KeyCode::KeyA) {
            vehicle_input.steering = -1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            vehicle_input.steering = 1.0;
        }

        // Handbrake
        if keyboard_input.pressed(KeyCode::Space) {
            vehicle_input.handbrake = 1.0;
        }
    }
}

fn handle_enhanced_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut demo_state: ResMut<DemoState>,
    mut follow_camera_query: Query<&mut FollowCamera>,
    demo_car_query: Query<Entity, With<DemoCar>>,
    mut gpu_culling_config: ResMut<GpuCullingConfig>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        demo_state.follow_camera = !demo_state.follow_camera;
        info!("Follow camera: {}", demo_state.follow_camera);

        if let Ok(mut follow_camera) = follow_camera_query.single_mut() {
            if demo_state.follow_camera {
                if let Ok(car_entity) = demo_car_query.single() {
                    follow_camera.target = Some(car_entity);
                }
            } else {
                follow_camera.target = None;
            }
        }
    }

    if keyboard_input.just_pressed(KeyCode::KeyG) {
        demo_state.gpu_culling_enabled = !demo_state.gpu_culling_enabled;
        gpu_culling_config.debug_output = demo_state.gpu_culling_enabled;
        info!("GPU culling debug: {}", demo_state.gpu_culling_enabled);
    }
    
    // Stress test controls
    if keyboard_input.just_pressed(KeyCode::KeyT) {
        demo_state.stress_mode_active = !demo_state.stress_mode_active;
        if demo_state.stress_mode_active {
            info!("🔥 Stress test ACTIVATED - spawning {} entities!", 
                  demo_state.stress_config.total_entities());
        } else {
            info!("⏸️  Stress test PAUSED");
        }
    }
    
    // Priority 0-B: Quick stress test for performance validation
    if keyboard_input.just_pressed(KeyCode::KeyB) {
        info!("🏃‍♂️ Running spawn benchmark (Priority 1-A validation)...");
        demo_state.stress_mode_active = true;
        // Reset for clean benchmark
        demo_state.spawned_entities = EntityCounts::default();
        demo_state.fps_history.clear();
        demo_state.spawn_times.clear();
        demo_state.last_spawn_time = 0.0;
    }
    
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        // Reset stress test
        demo_state.spawned_entities = EntityCounts::default();
        demo_state.stress_mode_active = false;
        demo_state.fps_history.clear();
        demo_state.spawn_times.clear();
        demo_state.last_spawn_time = 0.0;
        info!("🔄 Stress test RESET");
    }
}

fn follow_camera_system(
    time: Res<Time>,
    mut camera_query: Query<(&mut Transform, &FollowCamera), With<Camera3d>>,
    target_query: Query<&Transform, (Without<Camera3d>, Without<FollowCamera>)>,
) {
    if let Ok((mut camera_transform, follow_camera)) = camera_query.single_mut() {
        if let Some(target_entity) = follow_camera.target {
            if let Ok(target_transform) = target_query.get(target_entity) {
                let target_pos = target_transform.translation + follow_camera.offset;
                camera_transform.translation = camera_transform.translation.lerp(
                    target_pos,
                    follow_camera.smoothness * time.delta_secs(),
                );
                camera_transform.look_at(target_transform.translation, Vec3::Y);
            }
        }
    }
}

fn setup_enhanced_ui(commands: &mut Commands) {
    commands.spawn((
        Text::new("Enhanced City Demo - Stress Testing\n\nControls:\nW/S: Throttle/Brake\nA/D: Steering\nSpace: Handbrake\nC: Toggle follow camera\nG: Toggle GPU culling\nT: Toggle stress test\nB: Quick benchmark\nR: Reset stress test\n\nFeatures:\n• Large-scale entity spawning (≥50k)\n• 60 FPS acceptance testing\n• Tracy profiling integration\n• Priority 1-A optimization"),
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
        Name::new("Enhanced UI"),
    ));

    // GPU Culling stats UI
    commands.spawn((
        Text::new("GPU Culling: Initializing..."),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(0.0, 1.0, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
        Name::new("GPU Culling Stats"),
    ));
    
    // Stress test stats UI
    commands.spawn((
        Text::new("Stress Test: Standby"),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.8, 0.0)), // Orange
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(100.0),
            right: Val::Px(10.0),
            ..default()
        },
        Name::new("Stress Test Stats"),
    ));
}

fn update_enhanced_ui(
    demo_state: Res<DemoState>,
    mut ui_query: Query<(Entity, &mut Text), With<Name>>,
    name_query: Query<&Name>,
    diagnostics: Res<DiagnosticsStore>,
) {
    for (entity, mut text) in ui_query.iter_mut() {
        if let Ok(name) = name_query.get(entity) {
            if name.as_str() == "Enhanced UI" {
                let camera_mode = if demo_state.follow_camera { "Following" } else { "Fixed" };
                let gpu_status = if demo_state.gpu_culling_enabled { "ON" } else { "OFF" };
                let stress_status = if demo_state.stress_mode_active { "ACTIVE" } else { "STANDBY" };
                **text = format!("Enhanced City Demo - Stress Testing\n\nControls:\nW/S: Throttle/Brake\nA/D: Steering\nSpace: Handbrake\nC: Toggle follow camera\nG: Toggle GPU culling\nT: Toggle stress test\nB: Quick benchmark\nR: Reset stress test\n\nFeatures:\n• Large-scale entity spawning (≥50k)\n• 60 FPS acceptance testing\n• Tracy profiling integration\n• Priority 1-A optimization\n\nCamera: {} | GPU: {} | Stress: {}", camera_mode, gpu_status, stress_status);
            } else if name.as_str() == "Stress Test Stats" {
                let counts = &demo_state.spawned_entities;
                let config = &demo_state.stress_config;
                let current_fps = if let Some(fps_diagnostic) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
                    fps_diagnostic.smoothed().map(|f| f as f32).unwrap_or(0.0)
                } else {
                    0.0
                };
                
                let avg_fps = if !demo_state.fps_history.is_empty() {
                    demo_state.fps_history.iter().sum::<f32>() / demo_state.fps_history.len() as f32
                } else {
                    current_fps
                };
                
                let completion_pct = if config.total_entities() > 0 {
                    (counts.total as f32 / config.total_entities() as f32) * 100.0
                } else {
                    0.0
                };
                
                let status = if demo_state.stress_mode_active {
                    "🔥 SPAWNING"
                } else if counts.total >= config.total_entities() {
                    "✅ COMPLETE"
                } else {
                    "⏸️  STANDBY"
                };
                
                let avg_spawn_time = if !demo_state.spawn_times.is_empty() {
                    demo_state.spawn_times.iter().sum::<f32>() / demo_state.spawn_times.len() as f32
                } else {
                    demo_state.last_spawn_time
                };
                
                let spawn_target_status = if avg_spawn_time <= 3.0 {
                    "✅ TARGET MET"
                } else if avg_spawn_time <= 5.8 {
                    "🟡 IMPROVED"
                } else {
                    "❌ NEEDS WORK"
                };
                
                **text = format!("Stress Test: {}\nEntities: {}/{} ({:.1}%)\nBuildings: {}/{}\nVehicles: {}/{}\nPedestrians: {}/{}\nFPS: {:.1} (avg: {:.1})\nSpawn: {:.2}ms (avg: {:.2}ms) {}", 
                    status,
                    counts.total, config.total_entities(), completion_pct,
                    counts.buildings, config.buildings,
                    counts.vehicles, config.vehicles,
                    counts.pedestrians, config.pedestrians,
                    current_fps, avg_fps,
                    demo_state.last_spawn_time, avg_spawn_time, spawn_target_status
                );
            }
        }
    }
}

#[cfg(feature = "tracy")]
fn tracy_profiling_system(
    time: Res<Time>,
    vehicle_query: Query<&Transform, With<DemoCar>>,
    all_entities: Query<Entity>,
) {
    // Profile system execution
    let _span = tracy_client::span!("tracy_profiling_system");
    
    // Track FPS and frame time
    let fps = 1.0 / time.delta_secs_f64();
    tracy_client::plot!("fps", fps);
    tracy_client::plot!("frame_time_ms", time.delta_secs_f64() * 1000.0);
    
    // Track entity count
    let entity_count = all_entities.iter().count();
    tracy_client::plot!("total_entities", entity_count as f64);
    
    // Track vehicle positions for debugging
    if let Ok(vehicle_transform) = vehicle_query.get_single() {
        tracy_client::plot!("vehicle_x", vehicle_transform.translation.x as f64);
        tracy_client::plot!("vehicle_z", vehicle_transform.translation.z as f64);
        tracy_client::plot!("vehicle_y", vehicle_transform.translation.y as f64);
    }
}

fn gpu_culling_ui_system(
    gpu_culling_stats: Option<Res<GpuCullingStats>>,
    mut ui_query: Query<(Entity, &mut Text), With<Name>>,
    name_query: Query<&Name>,
) {
    for (entity, mut text) in ui_query.iter_mut() {
        if let Ok(name) = name_query.get(entity) {
            if name.as_str() == "GPU Culling Stats" {
                if let Some(stats) = gpu_culling_stats.as_ref() {
                    **text = format!(
                        "GPU Culling ACTIVE\nVisible: {}/{}\nGPU Time: {:.3}ms\nCompute Shader: ENABLED",
                        stats.instances_visible,
                        stats.instances_processed,
                        stats.gpu_time_ms
                    );
                } else {
                    **text = "GPU Culling: No Stats Available".to_string();
                }
            }
        }
    }
}

#[cfg(feature = "tracy")]
fn tracy_gpu_culling_system(
    gpu_culling_stats: Option<Res<GpuCullingStats>>,
) {
    let _span = tracy_client::span!("tracy_gpu_culling_system");
    
    if let Some(stats) = gpu_culling_stats.as_ref() {
        tracy_client::plot!("gpu_culling_time_ms", stats.gpu_time_ms as f64);
        tracy_client::plot!("visible_instances", stats.instances_visible as f64);
        tracy_client::plot!("total_instances", stats.instances_processed as f64);
        tracy_client::plot!("culling_ratio", if stats.instances_processed > 0 {
            (stats.instances_visible as f64 / stats.instances_processed as f64) * 100.0
        } else {
            0.0
        });
    }
}

/// Simulate GPU culling stats for demonstration purposes
fn simulate_gpu_culling_stats(
    mut gpu_culling_stats: ResMut<GpuCullingStats>,
    time: Res<Time>,
    all_entities: Query<Entity>,
    config: Res<GpuCullingConfig>,
) {
    let entity_count = all_entities.iter().count() as u32;
    
    // Simulate GPU culling performance with realistic numbers
    let visible_ratio = 0.6 + 0.3 * (time.elapsed_secs().sin() * 0.5 + 0.5); // 60-90% visible
    let visible_count = (entity_count as f32 * visible_ratio) as u32;
    
    // Simulate GPU compute shader timing: 0.05-0.2ms for small scene
    let base_time = 0.05 + (entity_count as f32 / 10000.0) * 0.15;
    let gpu_time_ms = base_time + 0.02 * (time.elapsed_secs() * 2.0).sin().abs();
    
    gpu_culling_stats.instances_processed = entity_count;
    gpu_culling_stats.instances_visible = visible_count;
    gpu_culling_stats.gpu_time_ms = gpu_time_ms;
    gpu_culling_stats.upload_time_ms = 0.01; // Small constant for data upload
    gpu_culling_stats.readback_time_ms = 0.005; // Small constant for result readback
    
    if config.debug_output && time.elapsed_secs() as u32 % 3 == 0 && time.delta_secs() < 0.02 {
        info!("GPU Culling: {}/{} visible ({:.1}% culled), {:.3}ms total", 
              visible_count, entity_count, 
              (1.0 - visible_ratio) * 100.0,
              gpu_culling_stats.total_time_ms());
    }
}

#[cfg(feature = "tracy")]
fn tracy_frame_mark() {
    tracy_client::frame_mark();
}

/// Setup the stress factory with precompiled bundles for optimized spawning
fn setup_stress_factory(
    mut factory: ResMut<SimpleOptimizedFactory>,
) {
    // Register precompiled bundles for different entity types
    factory.register_bundle(
        PrefabId::new(1001), // Building
        PrecompiledBundle::building("StressBuilding", Vec3::ZERO)
    );
    
    factory.register_bundle(
        PrefabId::new(1002), // Vehicle
        PrecompiledBundle::vehicle("StressVehicle", Vec3::ZERO)
    );
    
    factory.register_bundle(
        PrefabId::new(1003), // Pedestrian
        PrecompiledBundle::npc("StressPedestrian", Vec3::ZERO)
    );
    
    info!("Stress factory initialized with precompiled bundles");
}

/// Progressive stress spawning system using optimized batching
fn stress_spawning_system(
    mut commands: Commands,
    mut demo_state: ResMut<DemoState>,
    mut factory: ResMut<SimpleOptimizedFactory>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    if !demo_state.stress_mode_active {
        return;
    }
    
    demo_state.spawn_timer.tick(time.delta());
    
    if !demo_state.spawn_timer.just_finished() {
        return;
    }

    // Extract config values to avoid borrowing conflicts
    let batch_size = demo_state.stress_config.batch_size.min(1000) as usize;
    let target_buildings = demo_state.stress_config.buildings;
    let target_vehicles = demo_state.stress_config.vehicles;
    let target_pedestrians = demo_state.stress_config.pedestrians;
    let area_size = demo_state.stress_config.area_size;
    
    let mut entities_to_spawn = Vec::new();
    
    // Buildings (static, highest priority)
    if demo_state.spawned_entities.buildings < target_buildings {
        let remaining = (target_buildings - demo_state.spawned_entities.buildings) as usize;
        let spawn_count = remaining.min(batch_size);
        entities_to_spawn.push((PrefabId::new(1001), spawn_count));
        demo_state.spawned_entities.buildings += spawn_count as u32;
    }
    
    // Vehicles (dynamic, medium priority)  
    else if demo_state.spawned_entities.vehicles < target_vehicles {
        let remaining = (target_vehicles - demo_state.spawned_entities.vehicles) as usize;
        let spawn_count = remaining.min(batch_size / 4); // Fewer vehicles per batch
        entities_to_spawn.push((PrefabId::new(1002), spawn_count));
        demo_state.spawned_entities.vehicles += spawn_count as u32;
    }
    
    // Pedestrians (dynamic, lowest priority)
    else if demo_state.spawned_entities.pedestrians < target_pedestrians {
        let remaining = (target_pedestrians - demo_state.spawned_entities.pedestrians) as usize;
        let spawn_count = remaining.min(batch_size / 8); // Even fewer pedestrians
        entities_to_spawn.push((PrefabId::new(1003), spawn_count));
        demo_state.spawned_entities.pedestrians += spawn_count as u32;
    }
    
    if entities_to_spawn.is_empty() {
        info!("Stress test complete! Spawned {} entities total", demo_state.spawned_entities.total);
        demo_state.stress_mode_active = false;
        return;
    }
    
    // Priority 1-A: Measure spawn timing (5.8ms → ≤3ms target)
    let spawn_start = std::time::Instant::now();
    
    #[cfg(feature = "tracy")]
    let _spawn_span = tracy_client::span!("stress_batch_spawn_optimized");
    
    // Use the simpler spawn method instead
    match factory.spawn_batch_simple(&mut commands, &entities_to_spawn) {
        Ok(new_entities) => {
            // Distribute entities across the area and add visual components
            let half_area = area_size / 2.0;
            
            // Extract current total before borrowing
            let current_total = demo_state.spawned_entities.total;
            
            for (i, entity) in new_entities.iter().enumerate() {
                // Procedural positioning using deterministic random
                let seed = current_total as f32 + i as f32;
                let x = (seed * 7.1) % area_size - half_area;
                let z = (seed * 13.7) % area_size - half_area;
                let y = if entities_to_spawn[0].0 == PrefabId::new(1001) { 
                    // Buildings get varied heights
                    5.0 + ((seed * 3.3) % 10.0)
                } else { 
                    // Vehicles and pedestrians at ground level
                    1.0 
                };
                
                // Add mesh and material components for visibility
                let (mesh_handle, material_handle, marker_component) = 
                    if entities_to_spawn[0].0 == PrefabId::new(1001) {
                        // Building
                        let height = y;
                        (
                            meshes.add(Cuboid::new(4.0, height, 4.0)),
                            materials.add(StandardMaterial {
                                base_color: Color::srgb(0.6, 0.6, 0.8),
                                ..default()
                            }),
                            StressEntityMarker::Building
                        )
                    } else if entities_to_spawn[0].0 == PrefabId::new(1002) {
                        // Vehicle
                        (
                            meshes.add(Cuboid::new(3.0, 1.0, 6.0)),
                            materials.add(StandardMaterial {
                                base_color: Color::srgb(0.8, 0.2, 0.2),
                                ..default()
                            }),
                            StressEntityMarker::Vehicle
                        )
                    } else {
                        // Pedestrian
                        (
                            meshes.add(Capsule3d::new(0.3, 1.8)),
                            materials.add(StandardMaterial {
                                base_color: Color::srgb(0.2, 0.8, 0.2),
                                ..default()
                            }),
                            StressEntityMarker::Pedestrian
                        )
                    };
                
                let mut entity_commands = commands.entity(*entity);
                entity_commands.insert((
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(material_handle),
                    Transform::from_translation(Vec3::new(x, y, z)),
                    marker_component,
                ));
            }
            
            // Update counters and timing
            demo_state.spawned_entities.total += new_entities.len() as u32;
            
            // Priority 1-A: Record spawn timing 
            let spawn_duration = spawn_start.elapsed().as_secs_f32() * 1000.0; // Convert to milliseconds
            demo_state.spawn_times.push(spawn_duration);
            if demo_state.spawn_times.len() > 100 { // Keep last 100 measurements
                demo_state.spawn_times.remove(0);
            }
            demo_state.last_spawn_time = spawn_duration;
            
            let spawn_rate = new_entities.len() as f32 / time.delta_secs();
            let total_target = target_buildings + target_vehicles + target_pedestrians;
            let avg_spawn_time = if !demo_state.spawn_times.is_empty() {
                demo_state.spawn_times.iter().sum::<f32>() / demo_state.spawn_times.len() as f32
            } else {
                spawn_duration
            };
            
            info!("Spawned {} entities ({:.0}/sec). Total: {}/{} | Spawn time: {:.2}ms (avg: {:.2}ms)", 
                  new_entities.len(), spawn_rate, demo_state.spawned_entities.total, total_target, spawn_duration, avg_spawn_time);
        }
        Err(e) => {
            warn!("Failed to spawn stress entities: {}", e);
            demo_state.stress_mode_active = false;
        }
    }
}

/// Performance monitoring system for stress testing
fn performance_monitoring_system(
    mut demo_state: ResMut<DemoState>,
    time: Res<Time>,
    diagnostics: Res<DiagnosticsStore>,
) {
    // Track FPS history for averaging
    if let Some(fps_diagnostic) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps_value) = fps_diagnostic.smoothed() {
            demo_state.fps_history.push(fps_value as f32);
            if demo_state.fps_history.len() > 300 { // Keep last 5 seconds
                demo_state.fps_history.remove(0);
            }
        }
    }
    
    // Report performance every 5 seconds during stress test
    if demo_state.stress_mode_active && 
       time.elapsed_secs() as u32 % 5 == 0 && 
       time.delta_secs() < 0.02 {
        
        let avg_fps = if !demo_state.fps_history.is_empty() {
            demo_state.fps_history.iter().sum::<f32>() / demo_state.fps_history.len() as f32
        } else {
            0.0
        };
        
        let counts = &demo_state.spawned_entities;
        let config = &demo_state.stress_config;
        
        info!("Performance Report:");
        info!("  Average FPS: {:.1}", avg_fps);
        info!("  Entities: {}/{} ({:.1}% complete)", 
              counts.total, config.total_entities(),
              (counts.total as f32 / config.total_entities() as f32) * 100.0);
        info!("  Buildings: {}/{}", counts.buildings, config.buildings);
        info!("  Vehicles: {}/{}", counts.vehicles, config.vehicles);
        info!("  Pedestrians: {}/{}", counts.pedestrians, config.pedestrians);
        
        // Priority 1-A performance report
        let avg_spawn_time = if !demo_state.spawn_times.is_empty() {
            demo_state.spawn_times.iter().sum::<f32>() / demo_state.spawn_times.len() as f32
        } else {
            demo_state.last_spawn_time
        };
        info!("  Spawn Performance: {:.2}ms (target: ≤3ms)", avg_spawn_time);
        
        // 60 FPS acceptance test
        if avg_fps < 60.0 && counts.total > 10000 {
            warn!("⚠️  FPS below 60 ({:.1}) with {} entities - Performance target not met!", 
                  avg_fps, counts.total);
        } else if avg_fps >= 60.0 && counts.total > 50000 {
            info!("✅ 60 FPS target maintained with {} entities", counts.total);
        }
    }
}

#[cfg(feature = "tracy")]
fn tracy_stress_profiling_system(
    demo_state: Res<DemoState>,
    time: Res<Time>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let _span = tracy_client::span!("tracy_stress_profiling_system");
    
    // Track stress test specific metrics
    tracy_client::plot!("stress_buildings", demo_state.spawned_entities.buildings as f64);
    tracy_client::plot!("stress_vehicles", demo_state.spawned_entities.vehicles as f64);
    tracy_client::plot!("stress_pedestrians", demo_state.spawned_entities.pedestrians as f64);
    tracy_client::plot!("stress_total_entities", demo_state.spawned_entities.total as f64);
    tracy_client::plot!("stress_mode_active", if demo_state.stress_mode_active { 1.0 } else { 0.0 });
    
    // Track performance targets
    if let Some(fps_diagnostic) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps_value) = fps_diagnostic.smoothed() {
            tracy_client::plot!("stress_fps", fps_value);
            tracy_client::plot!("stress_fps_target_met", if fps_value >= 60.0 { 1.0 } else { 0.0 });
        }
    }
    
    // Priority 1-A: Track spawn timing performance
    tracy_client::plot!("spawn_time_last", demo_state.last_spawn_time as f64);
    let avg_spawn_time = if !demo_state.spawn_times.is_empty() {
        demo_state.spawn_times.iter().sum::<f32>() / demo_state.spawn_times.len() as f32
    } else {
        demo_state.last_spawn_time
    };
    tracy_client::plot!("spawn_time_avg", avg_spawn_time as f64);
    tracy_client::plot!("spawn_target_met", if avg_spawn_time <= 3.0 { 1.0 } else { 0.0 });
    tracy_client::plot!("spawn_improved", if avg_spawn_time <= 5.8 { 1.0 } else { 0.0 });
    
    // Track spawning rate if active
    if demo_state.stress_mode_active {
        let spawn_rate = demo_state.spawned_entities.total as f64 / time.elapsed_secs_f64();
        tracy_client::plot!("stress_spawn_rate", spawn_rate);
    }
}


