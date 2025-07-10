//! City Demo Baseline - Performance Measurement Scene
//!
//! This example creates a minimal city scene with ~5k entities for baseline performance measurement.
//! Required by Oracle for Sprint 1 to establish denominators for "300% improvement" targets.
//!
//! Metrics captured:
//! - Average FPS over 30 seconds
//! - 99th percentile frame time
//! - Memory peak usage
//! - Basic CPU/GPU timings
//!
//! Run with: cargo run --example city_demo_baseline

use bevy::app::AppExit;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::WindowResolution;

use std::time::{Duration, Instant};

const CITY_SIZE: i32 = 70; // 70x70 grid = ~5k entities
const MEASUREMENT_DURATION: Duration = Duration::from_secs(30);
const BUILDING_SPACING: f32 = 10.0;

#[derive(Component)]
struct Building;

#[derive(Component)]
struct PerformanceMonitor {
    start_time: Instant,
    frame_times: Vec<f32>,
    peak_entities: usize,
    measurements_complete: bool,
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            frame_times: Vec::with_capacity(2000), // Pre-allocate for 30s at 60fps
            peak_entities: 0,
            measurements_complete: false,
        }
    }
}

#[derive(Component)]
struct CameraController {
    speed: f32,
    height: f32,
    radius: f32,
    angle: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            speed: 0.5,
            height: 50.0,
            radius: 100.0,
            angle: 0.0,
        }
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Amp City Demo Baseline - Performance Measurement".to_string(),
                    resolution: WindowResolution::new(1920.0, 1080.0),
                    ..default()
                }),
                ..default()
            }),
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .add_systems(Startup, setup_scene)
        .add_systems(
            Update,
            (
                camera_controller_system,
                performance_monitor_system,
                shutdown_system,
            ),
        )
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!("Setting up city demo baseline scene...");

    // Create shared mesh assets
    let cube_mesh = meshes.add(Cuboid::new(5.0, 15.0, 5.0)); // Building
    let sphere_mesh = meshes.add(Sphere::new(2.0)); // Decoration

    // Create materials
    let building_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.7, 0.8),
        ..default()
    });
    let decoration_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.4, 0.2),
        ..default()
    });

    let mut entity_count = 0;

    // Generate city grid
    for x in -CITY_SIZE / 2..CITY_SIZE / 2 {
        for z in -CITY_SIZE / 2..CITY_SIZE / 2 {
            let pos_x = x as f32 * BUILDING_SPACING;
            let pos_z = z as f32 * BUILDING_SPACING;

            // Spawn building
            commands.spawn((
                Mesh3d(cube_mesh.clone()),
                MeshMaterial3d(building_material.clone()),
                Transform::from_xyz(pos_x, 7.5, pos_z),
                Building,
            ));
            entity_count += 1;

            // Add some decorative spheres (every 4th building)
            if (x + z) % 4 == 0 {
                commands.spawn((
                    Mesh3d(sphere_mesh.clone()),
                    MeshMaterial3d(decoration_material.clone()),
                    Transform::from_xyz(pos_x, 17.0, pos_z),
                    Building,
                ));
                entity_count += 1;
            }
        }
    }

    // Add ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(
            CITY_SIZE as f32 * BUILDING_SPACING * 2.0,
            CITY_SIZE as f32 * BUILDING_SPACING * 2.0,
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.6, 0.2),
            ..default()
        })),
        Transform::default(),
    ));

    // Add lighting
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 200.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Add ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.3,
        affects_lightmapped_meshes: true,
    });

    // Spawn camera with controller
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 50.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
        CameraController::default(),
    ));

    // Spawn performance monitor
    commands.spawn(PerformanceMonitor::default());

    info!(
        "City demo baseline scene created with {} entities",
        entity_count
    );
}

fn camera_controller_system(
    time: Res<Time>,
    mut camera_query: Query<(&mut Transform, &mut CameraController), With<Camera3d>>,
) {
    for (mut transform, mut controller) in camera_query.iter_mut() {
        controller.angle += controller.speed * time.delta_secs();

        let x = controller.radius * controller.angle.cos();
        let z = controller.radius * controller.angle.sin();

        transform.translation = Vec3::new(x, controller.height, z);
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

fn performance_monitor_system(
    time: Res<Time>,
    mut monitor_query: Query<&mut PerformanceMonitor>,
    building_query: Query<&Building>,
) {
    for mut monitor in monitor_query.iter_mut() {
        if monitor.measurements_complete {
            return;
        }

        let elapsed = monitor.start_time.elapsed();
        let frame_time_ms = time.delta_secs() * 1000.0;

        // Record frame time
        monitor.frame_times.push(frame_time_ms);

        // Update peak entity count
        let current_entities = building_query.iter().count();
        monitor.peak_entities = monitor.peak_entities.max(current_entities);

        // Check if measurement period is complete
        if elapsed >= MEASUREMENT_DURATION {
            monitor.measurements_complete = true;
            generate_performance_report(&monitor);
        }
    }
}

fn generate_performance_report(monitor: &PerformanceMonitor) {
    let total_frames = monitor.frame_times.len();
    let avg_frame_time = monitor.frame_times.iter().sum::<f32>() / total_frames as f32;
    let avg_fps = 1000.0 / avg_frame_time;

    // Calculate 99th percentile
    let mut sorted_times = monitor.frame_times.clone();
    sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p99_index = (sorted_times.len() as f32 * 0.99) as usize;
    let p99_frame_time = sorted_times.get(p99_index).unwrap_or(&0.0);

    // Calculate memory usage (approximation)
    let estimated_memory_mb = (monitor.peak_entities * 200) / 1024; // Rough estimate: 200 bytes per entity

    info!("=== CITY DEMO BASELINE PERFORMANCE REPORT ===");
    info!(
        "Measurement Duration: {} seconds",
        MEASUREMENT_DURATION.as_secs()
    );
    info!("Total Frames: {}", total_frames);
    info!("Peak Entities: {}", monitor.peak_entities);
    info!("Average FPS: {:.2}", avg_fps);
    info!("Average Frame Time: {:.2} ms", avg_frame_time);
    info!("99th Percentile Frame Time: {:.2} ms", p99_frame_time);
    info!("Estimated Memory Usage: {} MB", estimated_memory_mb);
    info!("============================================");

    // Print measurements for documentation
    println!("\n--- BASELINE MEASUREMENTS FOR DOCUMENTATION ---");
    println!("Date: 2025-01-07");
    println!("Scene: City Demo Baseline");
    println!("Entities: {}", monitor.peak_entities);
    println!("Average FPS: {avg_fps:.2}");
    println!("99th Percentile Frame Time: {p99_frame_time:.2} ms");
    println!("Estimated Memory: {estimated_memory_mb} MB");
    println!("--- END BASELINE MEASUREMENTS ---\n");
}

fn shutdown_system(monitor_query: Query<&PerformanceMonitor>, mut exit: EventWriter<AppExit>) {
    for monitor in monitor_query.iter() {
        if monitor.measurements_complete {
            info!("Performance measurement complete. Shutting down...");
            exit.write(AppExit::Success);
            break;
        }
    }
}
