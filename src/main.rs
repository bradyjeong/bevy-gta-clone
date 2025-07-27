//! GTA4-Style Game - Anti-Twitching Main Implementation
//!
//! This main implementation incorporates comprehensive anti-twitching systems:
//! - Physics-visual transform interpolation
//! - Schedule consistency (FixedUpdate for physics)
//! - GPU culling stall elimination
//! - Exponential camera damping
//! - Consolidated performance monitoring (no log spam)

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

// Import all our anti-twitching systems
use amp_core::system_ordering::SystemOrderingPlugin;
use amp_gameplay::character::bundles::PhysicsPlayerBundle;
use amp_gameplay::character::components::{AnimationSet, CharacterInput, Locomotion, Player};
use amp_gameplay::character::systems::asset_loading::LoadCharacterAsset;
use amp_gameplay::prelude::*;
use amp_physics::{InterpolatedTransform as PhysicsInterpolatedTransform, PhysicsTime};
use amp_render::{culling::CullingConfig, prelude::*, road::AsyncRoadMeshPlugin};

// Remove unused perf_ui module for now
// mod perf_ui;
// use perf_ui::PerfUiPlugin;

// Anti-twitching camera controller
mod camera;
use camera::SmoothCameraPlugin;

// Performance monitoring - consolidated system
mod perf;
use perf::PerfMonitoringPlugin;

fn main() {
    App::new()
        // Core Bevy plugins with optimized settings
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "rust gta".into(),
                present_mode: bevy::window::PresentMode::Fifo, // Stable VSync
                ..default()
            }),
            ..default()
        }))
        // Oracle's Production System Ordering - MUST be first
        .add_plugins(SystemOrderingPlugin)
        // Core gameplay with character systems (includes PhysicsPlugin)
        .add_plugins(GameplayPlugins)
        // Anti-twitching rendering systems
        .add_plugins(BatchingPlugin)
        // Road mesh generation system
        .add_plugins(AsyncRoadMeshPlugin::default())
        // Batch processing for performance
        .add_plugins(amp_engine::BatchProcessingPlugin)
        // World streaming and performance
        .add_plugins(amp_engine::WorldStreamingPlugin)
        .add_plugins(PerfMonitoringPlugin)
        // Smooth camera system
        .add_plugins(SmoothCameraPlugin)
        // UI and debugging
        // .add_plugins(PerfUiPlugin) // Disabled - UI overlay clutter
        // Optimized culling configuration
        .insert_resource(CullingConfig {
            max_distance: 800.0,
            enable_frustum_culling: true,
            enable_distance_culling: true,
        })
        // World streaming setup
        .insert_resource(amp_engine::WorldStreamer::default())
        // Clear sky color for outdoor feel
        .insert_resource(ClearColor(Color::srgb(0.53, 0.8, 0.95)))
        // Startup systems
        .add_systems(
            Startup,
            (
                setup_scene,
                setup_performance_monitoring,
                setup_controls_ui, // Clean controls UI only
            ),
        )
        // Update systems with proper scheduling
        .add_systems(
            Update,
            (
                // display_performance_info, // Disabled - UI overlay clutter
                spawn_city_ground.after(amp_gameplay::city::systems::city_setup),
                // debug_player_components, // Disabled - debug info clutter
                fix_character_orientation,
            ),
        )
        // Note: Character movement systems are handled by GameplayPlugins
        // The physics interpolation happens automatically in the amp_physics crate
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut animation_sets: ResMut<Assets<AnimationSet>>,
    mut animation_graphs: ResMut<Assets<bevy::animation::graph::AnimationGraph>>,
    asset_server: Res<AssetServer>,
) {
    // Create animation graph and set following Oracle's pattern
    let mut graph = bevy::animation::graph::AnimationGraph::new();
    let mut animation_set = AnimationSet::new("main_game");

    // Define locomotion states to load
    let locomotion_clips = [
        (
            Locomotion::Idle,
            "characters/mixamo/animations/idlebreathing.glb#Animation0",
        ),
        (
            Locomotion::Walk,
            "characters/mixamo/animations/walking.glb#Animation0",
        ),
        (
            Locomotion::Run,
            "characters/mixamo/animations/running.glb#Animation0",
        ),
    ];

    // Load animation clips and create graph nodes following Oracle's pattern
    for (locomotion, clip_path) in locomotion_clips {
        let clip_handle: Handle<AnimationClip> = asset_server.load(clip_path);
        // Initialize only idle animation with weight 1.0, all others at 0.0
        let weight = if locomotion == Locomotion::Idle {
            1.0
        } else {
            0.0
        };

        // Add clip to graph and store node index (Oracle's pattern)
        let node_index = graph.add_clip(clip_handle.clone(), weight, graph.root);
        animation_set.set_clip(locomotion, clip_handle);
        animation_set.set_node_index(locomotion, node_index);
    }

    // Store the animation graph and add handle to animation set
    let graph_handle = animation_graphs.add(graph);
    animation_set.graph = graph_handle;

    let animation_set_handle = animation_sets.add(animation_set);

    // Spawn player with physics and interpolated transform system
    let mut player_bundle = PhysicsPlayerBundle::new(animation_set_handle);
    // Set character to face away from camera initially (rotate 180 degrees)
    let initial_transform = Transform::from_xyz(0.0, 1.0, 0.0)
        .with_rotation(Quat::from_rotation_y(std::f32::consts::PI));
    player_bundle.character.transform = initial_transform;
    player_bundle.character.interpolated_transform =
        PhysicsInterpolatedTransform::new(initial_transform);

    // Use the asset loading pipeline to load the Mixamo character
    // This will trigger the complete asset loading pipeline including skeleton component migration
    let player_entity = commands
        .spawn((
            player_bundle,
            LoadCharacterAsset::new("characters/mixamo/models/character.glb", "player")
                .with_scale(1.0),
            amp_gameplay::character::components::ActiveEntity,
            Name::new("Player"),
        ))
        .id();

    info!("✅ Player spawned with asset loading pipeline - T-pose fix will run automatically");

    // Spawn camera with smooth controller
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        SmoothCamera::default(),
        Name::new("Smooth Camera"),
    ));

    // Optimized lighting setup
    commands.spawn((
        DirectionalLight {
            illuminance: 50000.0,
            shadows_enabled: true,
            color: Color::srgb(1.0, 0.98, 0.9),
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.3, 0.0)),
        Name::new("Sun"),
    ));

    // Ambient lighting
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.4, 0.6, 1.0),
        brightness: 2000.0,
        affects_lightmapped_meshes: true,
    });

    // Temporary ground plane
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Plane3d::default().mesh().size(200.0, 200.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.85, 0.7, 0.5),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.5, 0.0),
        Name::new("Temporary Ground"),
        TemporaryGround,
    ));

    info!("🎮 Scene setup complete with anti-twitching systems");
}

fn setup_performance_monitoring(mut commands: Commands) {
    // Insert performance tracking resources
    commands.insert_resource(FrameTimingHistory::default());
    commands.insert_resource(PhysicsPerformanceMetrics::default());
}

fn setup_controls_ui(mut commands: Commands) {
    // Clean controls UI - only show essential controls
    commands.spawn((
        Text::new("🎮 Controls\nWASD: Move\nA/D: Turn\nShift: Sprint\nSpace: Jump\nF: Enter Vehicle\nESC: Toggle Cursor"),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        ControlsUI,
    ));
}

fn display_performance_info(
    time: Res<Time>,
    physics_time: Res<PhysicsTime>,
    mut perf_query: Query<&mut Text, With<PerformanceDisplay>>,
    mut physics_query: Query<&mut Text, (With<PhysicsTimingDisplay>, Without<PerformanceDisplay>)>,
    frame_history: Res<FrameTimingHistory>,
) {
    let fps = 1.0 / time.delta_secs();
    let physics_alpha = physics_time.interpolation_alpha;

    // Update performance display
    for mut text in perf_query.iter_mut() {
        **text = format!(
            "FPS: {:.0}\nFrame: {:.2}ms\nPhysics α: {:.3}\nSmooth: {}",
            fps,
            time.delta_secs() * 1000.0,
            physics_alpha,
            if frame_history.is_smooth() {
                "✅"
            } else {
                "❌"
            }
        );
    }

    // Update physics timing display
    for mut text in physics_query.iter_mut() {
        **text = format!(
            "Physics: {:.0}Hz | Interpolation: {:.1}% | Fixed Step: {:.2}ms",
            1.0 / physics_time.fixed_timestep,
            physics_alpha * 100.0,
            physics_time.fixed_timestep * 1000.0
        );
    }
}

// Debug system to check player components
fn debug_player_components(
    player_query: Query<(Entity, &CharacterInput), With<Player>>,
    mut last_check: Local<f32>,
    time: Res<Time>,
) {
    *last_check += time.delta_secs();
    if *last_check > 2.0 {
        // Check every 2 seconds
        *last_check = 0.0;
        let count = player_query.iter().count();
        info!("Found {} player entities", count);

        for (entity, input) in player_query.iter() {
            info!(
                "Player entity {:?}: input movement={:?}",
                entity, input.move_2d
            );
        }
    }
}

// Note: Interpolation systems are handled by amp_physics crate

fn spawn_city_ground(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    city_config: Option<Res<CityConfig>>,
    temp_ground_query: Query<Entity, With<TemporaryGround>>,
    ground_query: Query<Entity, With<CityGround>>,
) {
    if let Some(config) = city_config {
        if ground_query.is_empty() {
            // Remove temporary ground
            for entity in temp_ground_query.iter() {
                commands.entity(entity).despawn();
            }

            let size_x = config.grid_size.x as f32 * config.tile_size;
            let size_z = config.grid_size.y as f32 * config.tile_size;

            // Spawn city-wide ground with interpolation
            commands.spawn((
                Mesh3d(meshes.add(Mesh::from(Cuboid::new(size_x, 0.2, size_z)))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.85, 0.7, 0.5),
                    perceptual_roughness: 0.9,
                    ..default()
                })),
                Transform::from_xyz(0.0, -0.1, 0.0),
                RigidBody::Fixed,
                Collider::cuboid(size_x / 2.0, 0.1, size_z / 2.0),
                PhysicsInterpolatedTransform::default(),
                Name::new("City Ground (Anti-Twitching)"),
                CityGround,
            ));

            info!(
                "🏙️  Spawned city ground: {}x{}m with interpolation",
                size_x, size_z
            );
        }
    }
}

// Component definitions for the game (physics interpolation handled by amp_physics)

#[derive(Component)]
pub struct SmoothCamera {
    pub damping_rate: f32,
    pub follow_distance: f32,
    pub follow_height: f32,
}

impl Default for SmoothCamera {
    fn default() -> Self {
        Self {
            damping_rate: 8.0,
            follow_distance: 6.0,
            follow_height: 2.5,
        }
    }
}

#[derive(Resource, Default)]
pub struct FrameTimingHistory {
    frame_times: Vec<f32>,
    max_samples: usize,
}

impl FrameTimingHistory {
    fn new() -> Self {
        Self {
            frame_times: Vec::new(),
            max_samples: 60, // Track last 60 frames
        }
    }

    fn add_frame_time(&mut self, delta: f32) {
        self.frame_times.push(delta);
        if self.frame_times.len() > self.max_samples {
            self.frame_times.remove(0);
        }
    }

    fn is_smooth(&self) -> bool {
        if self.frame_times.len() < 10 {
            return false;
        }

        let avg = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
        let variance = self
            .frame_times
            .iter()
            .map(|&x| (x - avg).powi(2))
            .sum::<f32>()
            / self.frame_times.len() as f32;

        variance < 0.0001 // Low variance = smooth
    }
}

#[derive(Resource, Default)]
pub struct PhysicsPerformanceMetrics {
    pub physics_frame_count: u64,
    pub total_physics_time: f32,
}

// UI Component markers
#[derive(Component)]
struct PerformanceDisplay;

#[derive(Component)]
struct ControlsUI;

#[derive(Component)]
struct PhysicsTimingDisplay;

#[derive(Component)]
struct TemporaryGround;

#[derive(Component)]
struct CityGround;

/// System to fix character orientation - apply rotation to the loaded scene model
fn fix_character_orientation(
    player_query: Query<Entity, With<Player>>,
    children_query: Query<&Children>,
    mut transform_query: Query<&mut Transform>,
) {
    for player_entity in player_query.iter() {
        // Find the first child with a Transform (the loaded model)
        if let Ok(children) = children_query.get(player_entity) {
            for child_entity in children {
                if let Ok(mut transform) = transform_query.get_mut(*child_entity) {
                    // Check if this transform hasn't been rotated yet
                    if transform.rotation == Quat::IDENTITY {
                        // Apply 180-degree rotation to face away from camera
                        transform.rotation = Quat::from_rotation_y(std::f32::consts::PI);
                        info!("✅ Fixed character orientation to face away from camera");
                        break;
                    }
                }
            }
        }
    }
}
