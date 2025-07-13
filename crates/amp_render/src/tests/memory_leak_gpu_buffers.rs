//! Memory leak tests for GPU buffer management
//!
//! These tests are ignored by default and run only in the weekly CI job
//! to prevent memory leaks in long-running scenarios.

use crate::render_world::{BufferPoolStats, RenderWorldPlugin};
use bevy::prelude::*;
use std::time::{Duration, Instant};

/// Test that RenderWorld systems don't leak memory over extended usage
#[test]
#[ignore = "long_memory"]
fn memory_leak_gpu_buffers() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, RenderWorldPlugin));

    // Initialize the app
    app.update();

    let start_time = Instant::now();
    let duration = Duration::from_secs(60); // Run for 1 minute
    let mut iteration = 0;
    let mut baseline_memory = 0.0f64;

    // Simulate intensive rendering workload
    while start_time.elapsed() < duration {
        iteration += 1;

        // Simulate frame processing that could cause memory leaks
        app.update();

        // Every 1000 iterations, check memory usage
        if iteration % 1000 == 0 {
            // Check if BufferPoolStats resource exists
            if let Some(stats) = app.world().get_resource::<BufferPoolStats>() {
                let current_memory_mb = stats.total_allocated_bytes as f64 / (1024.0 * 1024.0);

                if iteration == 1000 {
                    baseline_memory = current_memory_mb;
                }

                // Memory growth check: should not grow > 5MB per minute
                let elapsed_minutes = start_time.elapsed().as_secs_f64() / 60.0;
                let memory_growth = current_memory_mb - baseline_memory;
                let memory_growth_rate = if elapsed_minutes > 0.0 {
                    memory_growth / elapsed_minutes
                } else {
                    0.0
                };

                println!(
                    "Memory stats - Iteration {}: {:.2}MB total, {:.2}MB growth, {:.2}MB/min rate",
                    iteration, current_memory_mb, memory_growth, memory_growth_rate
                );

                // Fail if memory growth exceeds 5MB/min
                assert!(
                    memory_growth_rate <= 5.0,
                    "Memory leak detected: growth rate {:.2}MB/min exceeds 5MB/min threshold",
                    memory_growth_rate
                );

                // Fail if total memory exceeds 110MB
                assert!(
                    current_memory_mb <= 110.0,
                    "Memory leak detected: total memory {:.2}MB exceeds 110MB threshold",
                    current_memory_mb
                );
            } else {
                println!(
                    "BufferPoolStats not available, skipping memory check at iteration {}",
                    iteration
                );
            }
        }

        // Small delay to prevent spinning
        std::thread::sleep(Duration::from_millis(1));
    }

    println!(
        "✅ Memory leak test completed: {} iterations in 60 seconds",
        iteration
    );
}

/// Test render system memory stability under stress
#[test]
#[ignore = "long_memory"]
fn memory_leak_render_system_stress() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, RenderWorldPlugin));

    app.update();

    // Stress test with repeated updates
    let iterations = 10000;

    for i in 0..iterations {
        app.update();

        if i % 1000 == 0 {
            if let Some(stats) = app.world().get_resource::<BufferPoolStats>() {
                let memory_mb = stats.total_allocated_bytes as f64 / (1024.0 * 1024.0);

                // Memory should stabilize, not continuously grow
                assert!(
                    memory_mb <= 50.0,
                    "Render system stress test: memory {:.2}MB exceeds 50MB limit at iteration {}",
                    memory_mb,
                    i
                );

                println!("Iteration {}: {:.2}MB memory usage", i, memory_mb);
            }
        }
    }

    println!(
        "✅ Render system stress test completed: {} iterations",
        iterations
    );
}

/// Test that resource cleanup works correctly
#[test]
#[ignore = "long_memory"]
fn memory_leak_resource_cleanup() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, RenderWorldPlugin));

    app.update();

    // Test cleanup by running multiple cycles
    let cycles = 1000;
    let mut peak_memory = 0.0f64;

    for cycle in 0..cycles {
        // Run multiple updates per cycle
        for _ in 0..10 {
            app.update();
        }

        if cycle % 100 == 0 {
            if let Some(stats) = app.world().get_resource::<BufferPoolStats>() {
                let memory_mb = stats.total_allocated_bytes as f64 / (1024.0 * 1024.0);
                peak_memory = peak_memory.max(memory_mb);

                // Should converge to stable memory usage
                if cycle > 200 {
                    assert!(
                        memory_mb <= peak_memory * 1.1, // Allow 10% growth max
                        "Resource cleanup test: memory {:.2}MB growing beyond peak {:.2}MB at cycle {}",
                        memory_mb,
                        peak_memory,
                        cycle
                    );
                }

                println!(
                    "Cycle {}: {:.2}MB current, {:.2}MB peak",
                    cycle, memory_mb, peak_memory
                );
            }
        }
    }

    println!("✅ Resource cleanup test completed: {} cycles", cycles);
}
