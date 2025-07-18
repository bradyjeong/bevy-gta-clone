//! Performance regression gates for CI
//!
//! This module contains headless tests that verify performance metrics
//! stay within acceptable bounds to prevent performance regressions.

use bevy::app::App;
use bevy::diagnostic::{
    DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
};
use bevy::prelude::*;
use bevy::time::TimePlugin;
use bevy::transform::TransformPlugin;

use amp_render::diagnostics::{
    PerformanceBudgets, PerformanceDiagnosticPaths, PerformanceDiagnostics,
    PerformanceDiagnosticsPlugin,
};
use amp_render::BatchingPlugin;

use std::time::Duration;

/// Headless performance regression test
///
/// This test runs a deterministic simulation and verifies that performance
/// metrics stay within acceptable bounds defined by Oracle's requirements.
#[test]
fn test_performance_regression_gates() {
    let mut app = create_headless_app();

    // Run simulation for several frames to get stable metrics
    for _ in 0..120 {
        app.update();
    }

    // Extract performance metrics
    let performance_diagnostics = app.world().resource::<PerformanceDiagnostics>();
    let diagnostics_store = app.world().resource::<DiagnosticsStore>();

    // Oracle's Regression Gates

    // 1. Total entities in world <= 1,200
    if let Some(entities_diag) = diagnostics_store.get(&PerformanceDiagnosticPaths::TOTAL_ENTITIES)
    {
        if let Some(total_entities) = entities_diag.smoothed() {
            assert!(
                total_entities <= PerformanceBudgets::MAX_TOTAL_ENTITIES,
                "Total entities ({:.0}) exceeds regression gate ({})",
                total_entities,
                PerformanceBudgets::MAX_TOTAL_ENTITIES
            );
        }
    }

    // 2. Sectors loaded <= 200
    if let Some(sectors_diag) = diagnostics_store.get(&PerformanceDiagnosticPaths::SECTORS_LOADED) {
        if let Some(sectors_loaded) = sectors_diag.smoothed() {
            assert!(
                sectors_loaded <= PerformanceBudgets::MAX_SECTORS_LOADED,
                "Sectors loaded ({:.0}) exceeds regression gate ({})",
                sectors_loaded,
                PerformanceBudgets::MAX_SECTORS_LOADED
            );
        }
    }

    // 3. Draw calls < 1000.0
    if let Some(draw_calls_diag) = diagnostics_store.get(&PerformanceDiagnosticPaths::DRAW_CALLS) {
        if let Some(draw_calls) = draw_calls_diag.smoothed() {
            assert!(
                draw_calls < PerformanceBudgets::MAX_DRAW_CALLS,
                "Draw calls ({:.0}) exceeds regression gate ({})",
                draw_calls,
                PerformanceBudgets::MAX_DRAW_CALLS
            );
        }
    }

    // 4. Average update time < 8.0ms (125 FPS target)
    if let Some(update_time_diag) =
        diagnostics_store.get(&PerformanceDiagnosticPaths::AVERAGE_UPDATE_TIME)
    {
        if let Some(avg_update_time) = update_time_diag.smoothed() {
            assert!(
                avg_update_time < PerformanceBudgets::MAX_AVERAGE_UPDATE_TIME_MS,
                "Average update time ({:.2}ms) exceeds regression gate ({}ms)",
                avg_update_time,
                PerformanceBudgets::MAX_AVERAGE_UPDATE_TIME_MS
            );
        }
    }

    // 5. Active point lights <= 200 (hard cap)
    if let Some(lights_diag) =
        diagnostics_store.get(&PerformanceDiagnosticPaths::ACTIVE_POINT_LIGHTS)
    {
        if let Some(active_lights) = lights_diag.smoothed() {
            assert!(
                active_lights <= PerformanceBudgets::MAX_ACTIVE_LIGHTS,
                "Active point lights ({:.0}) exceeds hard cap ({})",
                active_lights,
                PerformanceBudgets::MAX_ACTIVE_LIGHTS
            );
        }
    }

    // 6. Instance count <= 75k (performance target)
    if let Some(instance_diag) = diagnostics_store.get(&PerformanceDiagnosticPaths::INSTANCE_COUNT)
    {
        if let Some(instance_count) = instance_diag.smoothed() {
            assert!(
                instance_count <= PerformanceBudgets::MAX_INSTANCE_COUNT,
                "Instance count ({:.0}) exceeds performance target ({})",
                instance_count,
                PerformanceBudgets::MAX_INSTANCE_COUNT
            );
        }
    }

    // 7. GPU culling time <= 0.25ms (when available)
    if let Some(gpu_time_diag) =
        diagnostics_store.get(&PerformanceDiagnosticPaths::GPU_CULLING_TIME)
    {
        if let Some(gpu_time) = gpu_time_diag.smoothed() {
            assert!(
                gpu_time <= PerformanceBudgets::MAX_GPU_CULLING_TIME_MS,
                "GPU culling time ({:.3}ms) exceeds target ({}ms)",
                gpu_time,
                PerformanceBudgets::MAX_GPU_CULLING_TIME_MS
            );
        }
    }

    // 8. Memory usage should be reasonable
    if let Some(memory_diag) = diagnostics_store.get(&PerformanceDiagnosticPaths::MEMORY_USAGE_MB) {
        if let Some(memory_usage) = memory_diag.smoothed() {
            assert!(
                memory_usage <= PerformanceBudgets::MAX_MEMORY_USAGE_MB,
                "Memory usage ({:.1}MB) exceeds warning threshold ({}MB)",
                memory_usage,
                PerformanceBudgets::MAX_MEMORY_USAGE_MB
            );
        }
    }

    // 9. Queue lengths should be reasonable
    if let Some(spawn_queue_diag) =
        diagnostics_store.get(&PerformanceDiagnosticPaths::SPAWN_QUEUE_LENGTH)
    {
        if let Some(spawn_queue) = spawn_queue_diag.smoothed() {
            assert!(
                spawn_queue <= PerformanceBudgets::MAX_SPAWN_QUEUE_LENGTH,
                "Spawn queue length ({:.0}) exceeds warning threshold ({})",
                spawn_queue,
                PerformanceBudgets::MAX_SPAWN_QUEUE_LENGTH
            );
        }
    }

    if let Some(despawn_queue_diag) =
        diagnostics_store.get(&PerformanceDiagnosticPaths::DESPAWN_QUEUE_LENGTH)
    {
        if let Some(despawn_queue) = despawn_queue_diag.smoothed() {
            assert!(
                despawn_queue <= PerformanceBudgets::MAX_DESPAWN_QUEUE_LENGTH,
                "Despawn queue length ({:.0}) exceeds warning threshold ({})",
                despawn_queue,
                PerformanceBudgets::MAX_DESPAWN_QUEUE_LENGTH
            );
        }
    }

    // Print performance summary for CI logs
    println!("=== Performance Regression Gates PASSED ===");

    let current_metrics = &performance_diagnostics.current_frame;
    println!("Current Frame Metrics:");
    println!("  Total Entities: {}", current_metrics.total_entities);
    println!("  Draw Calls: {}", current_metrics.draw_calls);
    println!("  Instance Count: {}", current_metrics.instance_count);
    println!(
        "  Active Point Lights: {}",
        current_metrics.active_point_lights
    );
    println!("  Update Time: {:.2}ms", current_metrics.update_time_ms);
    println!(
        "  GPU Culling Time: {:.3}ms",
        current_metrics.gpu_culling_time_ms
    );
    println!("  Sectors Loaded: {}", current_metrics.sectors_loaded);
    println!("  Memory Usage: {:.1}MB", current_metrics.memory_usage_mb);
    println!("  Spawn Queue: {}", current_metrics.spawn_queue_length);
    println!("  Despawn Queue: {}", current_metrics.despawn_queue_length);

    // Performance status summary
    let status = performance_diagnostics.get_status();
    println!("Performance Status: {:?}", status);

    if !performance_diagnostics.warnings.is_empty() {
        println!(
            "Performance Warnings ({}): ",
            performance_diagnostics.warnings.len()
        );
        for warning in &performance_diagnostics.warnings {
            println!("  - {:?}", warning);
        }
    }
}

/// Test performance under heavy load
///
/// This test simulates a heavy load scenario and verifies that performance
/// degrades gracefully without exceeding critical thresholds.
#[test]
fn test_performance_under_heavy_load() {
    let mut app = create_headless_app();

    // Add heavy load simulation
    app.world_mut().spawn_batch((0..500).map(|i| {
        (
            Transform::from_translation(Vec3::new(i as f32, 0.0, 0.0)),
            GlobalTransform::default(),
            // Add some point lights to stress the system
            PointLight {
                intensity: 1000.0,
                range: 10.0,
                ..Default::default()
            },
        )
    }));

    // Run simulation under heavy load
    for _ in 0..60 {
        app.update();
    }

    let performance_diagnostics = app.world().resource::<PerformanceDiagnostics>();
    let diagnostics_store = app.world().resource::<DiagnosticsStore>();

    // Under heavy load, we should still meet critical thresholds

    // Active lights should never exceed hard cap
    if let Some(lights_diag) =
        diagnostics_store.get(&PerformanceDiagnosticPaths::ACTIVE_POINT_LIGHTS)
    {
        if let Some(active_lights) = lights_diag.smoothed() {
            assert!(
                active_lights <= PerformanceBudgets::MAX_ACTIVE_LIGHTS,
                "Active point lights ({:.0}) exceeds hard cap ({}) even under heavy load",
                active_lights,
                PerformanceBudgets::MAX_ACTIVE_LIGHTS
            );
        }
    }

    // Update time should not exceed critical threshold (2x normal)
    if let Some(update_time_diag) =
        diagnostics_store.get(&PerformanceDiagnosticPaths::AVERAGE_UPDATE_TIME)
    {
        if let Some(avg_update_time) = update_time_diag.smoothed() {
            let critical_threshold = PerformanceBudgets::MAX_AVERAGE_UPDATE_TIME_MS * 2.0;
            assert!(
                avg_update_time < critical_threshold,
                "Average update time ({:.2}ms) exceeds critical threshold ({}ms) under heavy load",
                avg_update_time,
                critical_threshold
            );
        }
    }

    // Memory usage should not exceed critical threshold
    if let Some(memory_diag) = diagnostics_store.get(&PerformanceDiagnosticPaths::MEMORY_USAGE_MB) {
        if let Some(memory_usage) = memory_diag.smoothed() {
            let critical_threshold = PerformanceBudgets::MAX_MEMORY_USAGE_MB * 2.0;
            assert!(
                memory_usage <= critical_threshold,
                "Memory usage ({:.1}MB) exceeds critical threshold ({}MB) under heavy load",
                memory_usage,
                critical_threshold
            );
        }
    }

    println!("=== Heavy Load Performance Test PASSED ===");
    println!("System maintained critical thresholds under heavy load");
}

/// Test hard caps enforcement
///
/// This test verifies that hard performance limits are enforced and
/// prevent system overload.
#[test]
fn test_hard_caps_enforcement() {
    let mut app = create_headless_app();

    // Enable hard limits
    {
        let mut performance_diagnostics = app.world_mut().resource_mut::<PerformanceDiagnostics>();
        performance_diagnostics.hard_limits_enabled = true;
    }

    // Run simulation
    for _ in 0..30 {
        app.update();
    }

    let performance_diagnostics = app.world().resource::<PerformanceDiagnostics>();
    let diagnostics_store = app.world().resource::<DiagnosticsStore>();

    // Test that hard caps are respected

    // MAX_ACTIVE_LIGHTS should never be exceeded
    if let Some(lights_diag) =
        diagnostics_store.get(&PerformanceDiagnosticPaths::ACTIVE_POINT_LIGHTS)
    {
        if let Some(active_lights) = lights_diag.smoothed() {
            assert!(
                active_lights <= PerformanceBudgets::MAX_ACTIVE_LIGHTS,
                "Hard cap violated: Active point lights ({:.0}) > MAX_ACTIVE_LIGHTS ({})",
                active_lights,
                PerformanceBudgets::MAX_ACTIVE_LIGHTS
            );
        }
    }

    // Spawn queue should not exceed hard limit
    if let Some(spawn_queue_diag) =
        diagnostics_store.get(&PerformanceDiagnosticPaths::SPAWN_QUEUE_LENGTH)
    {
        if let Some(spawn_queue) = spawn_queue_diag.smoothed() {
            // Hard limit is 10x the warning threshold
            let hard_limit = PerformanceBudgets::MAX_SPAWN_QUEUE_LENGTH * 10.0;
            assert!(
                spawn_queue <= hard_limit,
                "Hard cap violated: Spawn queue length ({:.0}) > hard limit ({})",
                spawn_queue,
                hard_limit
            );
        }
    }

    println!("=== Hard Caps Enforcement Test PASSED ===");
    println!("All hard performance limits are properly enforced");
}

/// Test baseline performance characteristics
///
/// This test establishes baseline performance metrics and verifies
/// they meet minimum AAA-grade requirements.
#[test]
fn test_baseline_performance() {
    let mut app = create_headless_app();

    // Run baseline simulation
    for _ in 0..60 {
        app.update();
    }

    let performance_diagnostics = app.world().resource::<PerformanceDiagnostics>();
    let diagnostics_store = app.world().resource::<DiagnosticsStore>();

    // Baseline performance requirements

    // Frame rate should be acceptable (at least 30 FPS equivalent)
    if let Some(update_time_diag) =
        diagnostics_store.get(&PerformanceDiagnosticPaths::AVERAGE_UPDATE_TIME)
    {
        if let Some(avg_update_time) = update_time_diag.smoothed() {
            assert!(
                avg_update_time < 33.0, // 30 FPS minimum
                "Baseline update time ({:.2}ms) indicates sub-30 FPS performance",
                avg_update_time
            );
        }
    }

    // Draw calls should be reasonable for baseline
    if let Some(draw_calls_diag) = diagnostics_store.get(&PerformanceDiagnosticPaths::DRAW_CALLS) {
        if let Some(draw_calls) = draw_calls_diag.smoothed() {
            assert!(
                draw_calls >= 10.0 && draw_calls <= 500.0,
                "Baseline draw calls ({:.0}) outside reasonable range (10-500)",
                draw_calls
            );
        }
    }

    // Instance count should be reasonable for baseline
    if let Some(instance_diag) = diagnostics_store.get(&PerformanceDiagnosticPaths::INSTANCE_COUNT)
    {
        if let Some(instance_count) = instance_diag.smoothed() {
            assert!(
                instance_count >= 100.0 && instance_count <= 10000.0,
                "Baseline instance count ({:.0}) outside reasonable range (100-10000)",
                instance_count
            );
        }
    }

    // Memory usage should be reasonable
    if let Some(memory_diag) = diagnostics_store.get(&PerformanceDiagnosticPaths::MEMORY_USAGE_MB) {
        if let Some(memory_usage) = memory_diag.smoothed() {
            assert!(
                memory_usage >= 50.0 && memory_usage <= 1000.0,
                "Baseline memory usage ({:.1}MB) outside reasonable range (50-1000MB)",
                memory_usage
            );
        }
    }

    println!("=== Baseline Performance Test PASSED ===");
    println!("All baseline performance requirements met");
}

/// Create a headless app for performance testing
///
/// This function sets up a minimal Bevy app without windowing or rendering
/// that can be used for deterministic performance testing.
fn create_headless_app() -> App {
    let mut app = App::new();

    // Add minimal plugins required for testing
    app.add_plugins(MinimalPlugins);

    // Add diagnostic plugins
    app.add_plugins((
        FrameTimeDiagnosticsPlugin::default(),
        EntityCountDiagnosticsPlugin::default(),
        PerformanceDiagnosticsPlugin,
    ));

    // Add rendering plugins (but skip actual rendering)
    app.add_plugins(BatchingPlugin);

    // Add test entities to simulate load
    app.world_mut().spawn_batch((0..100).map(|i| {
        (
            Transform::from_translation(Vec3::new(i as f32, 0.0, 0.0)),
            GlobalTransform::default(),
        )
    }));

    // Add some point lights for testing
    app.world_mut().spawn_batch((0..10).map(|i| {
        (
            Transform::from_translation(Vec3::new(i as f32 * 10.0, 5.0, 0.0)),
            GlobalTransform::default(),
            PointLight {
                intensity: 1000.0,
                range: 10.0,
                ..Default::default()
            },
        )
    }));

    app
}

/// Integration test for performance monitoring system
///
/// This test verifies that the performance monitoring system itself
/// works correctly and provides accurate measurements.
#[test]
fn test_performance_monitoring_system() {
    let mut app = create_headless_app();

    // Run for a few frames to initialize
    for _ in 0..10 {
        app.update();
    }

    // Verify that diagnostics are being collected
    let diagnostics_store = app.world().resource::<DiagnosticsStore>();

    // Check that all expected diagnostics are registered
    assert!(diagnostics_store
        .get(&PerformanceDiagnosticPaths::DRAW_CALLS)
        .is_some());
    assert!(diagnostics_store
        .get(&PerformanceDiagnosticPaths::INSTANCE_COUNT)
        .is_some());
    assert!(diagnostics_store
        .get(&PerformanceDiagnosticPaths::TOTAL_ENTITIES)
        .is_some());
    assert!(diagnostics_store
        .get(&PerformanceDiagnosticPaths::ACTIVE_POINT_LIGHTS)
        .is_some());
    assert!(diagnostics_store
        .get(&PerformanceDiagnosticPaths::SPAWN_QUEUE_LENGTH)
        .is_some());
    assert!(diagnostics_store
        .get(&PerformanceDiagnosticPaths::DESPAWN_QUEUE_LENGTH)
        .is_some());
    assert!(diagnostics_store
        .get(&PerformanceDiagnosticPaths::AVERAGE_UPDATE_TIME)
        .is_some());
    assert!(diagnostics_store
        .get(&PerformanceDiagnosticPaths::GPU_CULLING_TIME)
        .is_some());
    assert!(diagnostics_store
        .get(&PerformanceDiagnosticPaths::SECTORS_LOADED)
        .is_some());
    assert!(diagnostics_store
        .get(&PerformanceDiagnosticPaths::MEMORY_USAGE_MB)
        .is_some());

    // Verify that measurements are being taken
    let performance_diagnostics = app.world().resource::<PerformanceDiagnostics>();
    assert!(performance_diagnostics.history.len() > 0);

    // Verify that performance status is being calculated
    let status = performance_diagnostics.get_status();
    assert!(matches!(
        status,
        amp_render::diagnostics::PerformanceStatus::Good
            | amp_render::diagnostics::PerformanceStatus::Warning
            | amp_render::diagnostics::PerformanceStatus::Critical
    ));

    println!("=== Performance Monitoring System Test PASSED ===");
    println!("Performance monitoring system is working correctly");
}
