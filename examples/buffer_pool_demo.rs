//! Buffer Pool Memory Leak Prevention Demo
//!
//! This example demonstrates how the TransientBufferPool prevents the critical memory leak
//! that was identified by Oracle as a production-blocking issue.
//!
//! Run with: `cargo run --example buffer_pool_demo`
//! 
//! The demo will show buffer pool statistics and memory usage over time to verify
//! that memory stays stable even with varying buffer sizes.

use bevy::prelude::*;
use bevy::render::RenderApp;
use amp_render::prelude::*;
use amp_render::{BatchKey, ALPHA_FLAG};

#[derive(Component)]
struct DemoRenderable {
    batch_key: BatchKey,
    size_multiplier: u32,
}

#[derive(Resource)]
struct DemoTimer(Timer);

/// Extract dummy instances with varying buffer sizes
fn extract_demo_instances(
    query: Query<(&GlobalTransform, &DemoRenderable)>,
    mut extracted: ResMut<ExtractedInstances>,
) {
    extracted.instances.clear();
    
    for (transform, renderable) in query.iter() {
        // Create variable instance counts to stress-test buffer allocation
        for i in 0..renderable.size_multiplier {
            extracted.instances.push(amp_render::ExtractedInstance {
                transform: transform.compute_matrix(),
                batch_key: renderable.batch_key.clone(),
                distance: i as f32,
                visible: true,
            });
        }
    }
}

/// Vary batch sizes to create different buffer allocation patterns
fn vary_batch_sizes(
    mut query: Query<&mut DemoRenderable>,
    time: Res<Time>,
) {
    let cycle = (time.elapsed_secs() * 0.3).sin();
    
    for mut renderable in query.iter_mut() {
        // Vary size multiplier from 1 to 200 to create different buffer sizes
        renderable.size_multiplier = ((cycle + 1.0) * 100.0) as u32 + 1;
    }
}

/// Display buffer pool statistics
fn display_buffer_stats(
    stats: Option<Res<BufferPoolStats>>,
    mut timer: ResMut<DemoTimer>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());
    
    if timer.0.just_finished() {
        if let Some(stats) = stats {
            info!(
                "🔧 Buffer Pool Stats: {:.2}MB allocated, {:.2}MB pooled, {} buffers, {:.1}% reuse",
                stats.total_allocated_bytes as f64 / (1024.0 * 1024.0),
                stats.pooled_bytes as f64 / (1024.0 * 1024.0),
                stats.pooled_buffers,
                stats.reuse_ratio * 100.0
            );
            
            // Warning if memory is growing (potential leak)
            if stats.total_allocated_bytes > 50 * 1024 * 1024 {
                warn!("⚠️  High memory usage detected: {:.2}MB", 
                      stats.total_allocated_bytes as f64 / (1024.0 * 1024.0));
            }
        }
    }
}

/// Setup demo entities
fn setup_demo(mut commands: Commands) {
    info!("🚀 Starting Buffer Pool Memory Leak Prevention Demo");
    info!("📊 Watch the buffer pool statistics to verify memory stays stable");
    info!("🔧 Varying buffer sizes from 1-200 instances to stress-test allocation");
    
    // Create camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(0.0, 10.0, 20.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));
    
    // Create test entities with different batch keys
    let batch_key_1 = BatchKey { mesh_id: 1, material_id: 1, flags: 0 };
    let batch_key_2 = BatchKey { mesh_id: 2, material_id: 2, flags: 0 };
    let batch_key_3 = BatchKey { mesh_id: 3, material_id: 3, flags: ALPHA_FLAG };
    
    commands.spawn((
        DemoRenderable { batch_key: batch_key_1, size_multiplier: 50 },
        GlobalTransform::from_translation(Vec3::new(-5.0, 0.0, 0.0)),
    ));
    
    commands.spawn((
        DemoRenderable { batch_key: batch_key_2, size_multiplier: 100 },
        GlobalTransform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
    ));
    
    commands.spawn((
        DemoRenderable { batch_key: batch_key_3, size_multiplier: 75 },
        GlobalTransform::from_translation(Vec3::new(5.0, 0.0, 0.0)),
    ));
    
    info!("✅ Demo entities spawned with varying batch configurations");
}

fn main() {
    let mut app = App::new();
    
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Buffer Pool Memory Leak Prevention Demo".to_string(),
                resolution: (1024.0, 768.0).into(),
                ..default()
            }),
            ..default()
        }),
        RenderWorldPlugin,
    ));
    
    // Add demo systems
    app.add_systems(Startup, setup_demo);
    app.add_systems(Update, (vary_batch_sizes, display_buffer_stats));
    
    // Initialize timer for periodic stats display
    app.insert_resource(DemoTimer(Timer::from_seconds(2.0, TimerMode::Repeating)));
    
    // Override render world extraction to use our demo data
    if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
        render_app.add_systems(bevy::render::ExtractSchedule, extract_demo_instances);
    }
    
    info!("🎮 Use Ctrl+C to exit the demo");
    
    app.run();
}
