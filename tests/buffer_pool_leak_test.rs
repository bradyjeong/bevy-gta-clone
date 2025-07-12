//! FrameTracer leak test to verify TransientBufferPool prevents memory accumulation
//!
//! This test validates that GPU buffer memory stays flat after N frames,
//! preventing the critical production-blocking memory leaks.

use amp_render::prelude::*;
use amp_render::{ALPHA_FLAG, BatchKey, ExtractedInstance};
use bevy::prelude::*;
use bevy::render::{ExtractSchedule, RenderApp};
use std::sync::{Arc, Mutex};

/// Test component for creating variable-sized batches
#[derive(Component)]
struct TestRenderable {
    batch_key: BatchKey,
    size_multiplier: u32, // Creates different buffer sizes
}

/// System to extract test entities (varying batch sizes)
fn extract_test_instances(
    query: Query<(&GlobalTransform, &TestRenderable)>,
    mut extracted: ResMut<ExtractedInstances>,
) {
    extracted.instances.clear();

    for (transform, renderable) in query.iter() {
        // Create variable instance counts based on size_multiplier
        for i in 0..renderable.size_multiplier {
            extracted.instances.push(ExtractedInstance {
                transform: transform.compute_matrix(),
                batch_key: renderable.batch_key.clone(),
                distance: i as f32,
                visible: true,
            });
        }
    }
}

/// Memory tracking for leak detection
#[derive(Resource, Default)]
struct MemoryTracker {
    frame_count: u32,
    memory_samples: Arc<Mutex<Vec<u64>>>,
    peak_memory: u64,
}

impl MemoryTracker {
    fn record_frame(&mut self, allocated_bytes: u64) {
        self.frame_count += 1;
        self.peak_memory = self.peak_memory.max(allocated_bytes);

        if let Ok(mut samples) = self.memory_samples.lock() {
            samples.push(allocated_bytes);
        }
    }

    fn analyze_leak(&self) -> LeakAnalysis {
        if let Ok(samples) = self.memory_samples.lock() {
            if samples.len() < 10 {
                return LeakAnalysis::InsufficientData;
            }

            // Check if memory has plateaued (last 20% of samples should be stable)
            let plateau_start = samples.len() * 4 / 5;
            let plateau_samples = &samples[plateau_start..];

            let plateau_min = plateau_samples.iter().min().unwrap();
            let plateau_max = plateau_samples.iter().max().unwrap();

            // Memory is considered stable if variation is < 10%
            let variation_ratio = (*plateau_max - *plateau_min) as f32 / *plateau_min as f32;

            if variation_ratio < 0.1 {
                LeakAnalysis::Stable {
                    plateau_memory_mb: *plateau_min as f64 / (1024.0 * 1024.0),
                    peak_memory_mb: self.peak_memory as f64 / (1024.0 * 1024.0),
                }
            } else {
                LeakAnalysis::MemoryLeak {
                    growth_rate_mb: (*plateau_max - samples[0]) as f64 / (1024.0 * 1024.0),
                    peak_memory_mb: self.peak_memory as f64 / (1024.0 * 1024.0),
                }
            }
        } else {
            LeakAnalysis::InsufficientData
        }
    }
}

#[derive(Debug)]
enum LeakAnalysis {
    Stable {
        plateau_memory_mb: f64,
        peak_memory_mb: f64,
    },
    MemoryLeak {
        growth_rate_mb: f64,
        peak_memory_mb: f64,
    },
    InsufficientData,
}

/// System to track memory usage and detect leaks
fn track_memory_usage(
    instance_meta: Option<Res<InstanceMeta>>,
    mut tracker: ResMut<MemoryTracker>,
) {
    if let Some(meta) = instance_meta {
        let stats = meta.buffer_pool.get_stats();
        tracker.record_frame(stats.total_allocated_bytes);

        // Log progress every 100 frames
        if tracker.frame_count % 100 == 0 {
            info!(
                "Frame {}: {:.2}MB allocated, {:.2}MB pooled, {:.1}% reuse",
                tracker.frame_count,
                stats.total_allocated_bytes as f64 / (1024.0 * 1024.0),
                stats.pooled_bytes as f64 / (1024.0 * 1024.0),
                stats.reuse_ratio * 100.0
            );
        }
    }
}

/// System to vary batch sizes to stress-test buffer allocation
fn vary_batch_sizes(mut query: Query<&mut TestRenderable>, time: Res<Time>) {
    let cycle = (time.elapsed_secs() * 0.5).sin();

    for mut renderable in query.iter_mut() {
        // Vary size multiplier from 1 to 100 to create different buffer sizes
        renderable.size_multiplier = ((cycle + 1.0) * 50.0) as u32 + 1;
    }
}

#[test]
fn test_transient_buffer_pool_prevents_memory_leaks() {
    let mut app = App::new();

    // Add required Bevy systems
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        bevy::render::RenderPlugin::default(),
        RenderWorldPlugin,
    ));

    // Add test-specific systems
    app.add_systems(Update, (vary_batch_sizes, track_memory_usage));

    // Override render world extraction
    if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
        render_app.add_systems(ExtractSchedule, extract_test_instances);
    }

    app.init_resource::<MemoryTracker>();

    // Spawn test entities with varying batch configurations
    let batch_key_1 = BatchKey {
        mesh_id: 1,
        material_id: 1,
        flags: 0,
    };
    let batch_key_2 = BatchKey {
        mesh_id: 2,
        material_id: 2,
        flags: 0,
    };
    let batch_key_3 = BatchKey {
        mesh_id: 3,
        material_id: 3,
        flags: ALPHA_FLAG,
    };

    app.world_mut().spawn((
        TestRenderable {
            batch_key: batch_key_1,
            size_multiplier: 10,
        },
        GlobalTransform::default(),
    ));
    app.world_mut().spawn((
        TestRenderable {
            batch_key: batch_key_2,
            size_multiplier: 50,
        },
        GlobalTransform::default(),
    ));
    app.world_mut().spawn((
        TestRenderable {
            batch_key: batch_key_3,
            size_multiplier: 25,
        },
        GlobalTransform::default(),
    ));

    // Run simulation for 500 frames to detect memory leaks
    for _ in 0..500 {
        app.update();
    }

    // Analyze memory usage pattern
    let tracker = app.world().resource::<MemoryTracker>();
    let analysis = tracker.analyze_leak();

    match analysis {
        LeakAnalysis::Stable {
            plateau_memory_mb,
            peak_memory_mb,
        } => {
            info!(
                "✅ Memory stable: plateau {:.2}MB, peak {:.2}MB",
                plateau_memory_mb, peak_memory_mb
            );

            // Assert reasonable memory usage (should be < 50MB for this test)
            assert!(
                plateau_memory_mb < 50.0,
                "Memory usage too high: {:.2}MB",
                plateau_memory_mb
            );
        }
        LeakAnalysis::MemoryLeak {
            growth_rate_mb,
            peak_memory_mb,
        } => {
            panic!(
                "❌ Memory leak detected: {:.2}MB growth, {:.2}MB peak",
                growth_rate_mb, peak_memory_mb
            );
        }
        LeakAnalysis::InsufficientData => {
            panic!("❌ Insufficient data for leak analysis");
        }
    }
}

#[test]
fn test_buffer_pool_stats_tracking() {
    let mut pool = TransientBufferPool::default();

    // Create mock render device for testing
    // Note: This test focuses on the logic, not actual GPU operations

    // Test statistics tracking
    assert_eq!(pool.get_stats().total_allocated_bytes, 0);
    assert_eq!(pool.get_stats().allocations_this_frame, 0);
    assert_eq!(pool.get_stats().reuses_this_frame, 0);

    pool.clear_frame_stats();
    assert_eq!(pool.get_stats().allocations_this_frame, 0);
    assert_eq!(pool.get_stats().reuses_this_frame, 0);
}

#[test]
fn test_buffer_pool_cleanup() {
    let mut pool = TransientBufferPool::default();

    // Test cleanup logic (without actual GPU operations)
    pool.cleanup_unused_buffers(2);

    // Verify cleanup doesn't crash with empty pool
    assert_eq!(pool.get_stats().pooled_buffers, 0);
}
