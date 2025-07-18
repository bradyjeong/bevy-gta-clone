//! Unit tests for performance diagnostics system
//!
//! This module contains comprehensive unit tests for performance monitoring,
//! hard caps enforcement, and regression gate validation.

use super::*;
use crate::distance_cache::DistanceCachePlugin;
use bevy::app::App;
use bevy::diagnostic::{
    DiagnosticsPlugin, DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
};
// HierarchyPlugin is not needed for these tests
use bevy::prelude::*;
use bevy::time::TimePlugin;
use bevy::transform::TransformPlugin;
use bevy::utils::default;

/// Test that hard caps are properly enforced
#[test]
fn test_max_active_lights_hard_cap() {
    let mut app = create_test_app();

    // Spawn more point lights than the hard cap allows
    let lights_to_spawn = (PerformanceBudgets::MAX_ACTIVE_LIGHTS as usize) + 50;
    app.world_mut().spawn_batch((0..lights_to_spawn).map(|i| {
        (
            Transform::from_translation(Vec3::new(i as f32, 0.0, 0.0)),
            GlobalTransform::default(),
            PointLight {
                intensity: 1000.0,
                range: 10.0,
                ..default()
            },
        )
    }));

    // Run simulation
    for _ in 0..10 {
        app.update();
    }

    // Check that the hard cap is enforced
    let diagnostics_store = app.world().resource::<DiagnosticsStore>();
    if let Some(lights_diag) = diagnostics_store.get(&PerformanceDiagnosticIds::ACTIVE_POINT_LIGHTS)
    {
        if let Some(active_lights) = lights_diag.smoothed() {
            assert!(
                active_lights <= PerformanceBudgets::MAX_ACTIVE_LIGHTS,
                "Hard cap violated: {} active lights > {} max",
                active_lights,
                PerformanceBudgets::MAX_ACTIVE_LIGHTS
            );
        }
    }
}

/// Test that spawn queue length is properly limited
#[test]
fn test_max_spawns_per_frame_hard_cap() {
    let mut app = create_test_app();

    // Enable hard limits
    {
        let mut perf_diagnostics = app.world_mut().resource_mut::<PerformanceDiagnostics>();
        perf_diagnostics.hard_limits_enabled = true;
    }

    // Simulate excessive spawn requests
    let mut perf_diagnostics = app.world_mut().resource_mut::<PerformanceDiagnostics>();
    perf_diagnostics.current_frame.spawn_queue_length = 5000; // Excessive

    // Run simulation
    for _ in 0..5 {
        app.update();
    }

    // Check that spawn queue is limited
    let diagnostics_store = app.world().resource::<DiagnosticsStore>();
    if let Some(spawn_queue_diag) =
        diagnostics_store.get(&PerformanceDiagnosticIds::SPAWN_QUEUE_LENGTH)
    {
        if let Some(spawn_queue) = spawn_queue_diag.smoothed() {
            let hard_limit = PerformanceBudgets::MAX_SPAWN_QUEUE_LENGTH * 10.0; // 10x warning threshold
            assert!(
                spawn_queue <= hard_limit,
                "Hard cap violated: {} spawn queue > {} hard limit",
                spawn_queue,
                hard_limit
            );
        }
    }
}

/// Test performance budget validation
#[test]
fn test_performance_budget_validation() {
    let mut app = create_test_app();

    // Set up specific metrics for testing
    {
        let mut perf_diagnostics = app.world_mut().resource_mut::<PerformanceDiagnostics>();
        perf_diagnostics.current_frame.draw_calls = 800; // Within budget
        perf_diagnostics.current_frame.instance_count = 50000; // Within budget
        perf_diagnostics.current_frame.update_time_ms = 6.0; // Within budget
        perf_diagnostics.current_frame.active_point_lights = 150; // Within budget
    }

    // Run to update diagnostics
    for _ in 0..5 {
        app.update();
    }

    let perf_diagnostics = app.world().resource::<PerformanceDiagnostics>();

    // Test individual budget checks
    assert!(perf_diagnostics.is_within_budget("draw_calls"));
    assert!(perf_diagnostics.is_within_budget("instance_count"));
    assert!(perf_diagnostics.is_within_budget("update_time_ms"));
    assert!(perf_diagnostics.is_within_budget("active_point_lights"));

    // Test overall status
    assert_eq!(perf_diagnostics.get_status(), PerformanceStatus::Good);
}

/// Test performance budget violations
#[test]
fn test_performance_budget_violations() {
    let mut app = create_test_app();

    // Set up metrics that exceed budgets
    {
        let mut perf_diagnostics = app.world_mut().resource_mut::<PerformanceDiagnostics>();
        perf_diagnostics.current_frame.draw_calls = 1100; // Over budget
        perf_diagnostics.current_frame.instance_count = 80000; // Over budget
        perf_diagnostics.current_frame.update_time_ms = 10.0; // Over budget
        perf_diagnostics.current_frame.active_point_lights = 50; // Within budget
    }

    // Run to update diagnostics
    for _ in 0..5 {
        app.update();
    }

    let perf_diagnostics = app.world().resource::<PerformanceDiagnostics>();

    // Test individual budget violations
    assert!(!perf_diagnostics.is_within_budget("draw_calls"));
    assert!(!perf_diagnostics.is_within_budget("instance_count"));
    assert!(!perf_diagnostics.is_within_budget("update_time_ms"));
    assert!(perf_diagnostics.is_within_budget("active_point_lights"));

    // Test overall status
    assert_eq!(perf_diagnostics.get_status(), PerformanceStatus::Warning);
}

/// Test critical performance thresholds
#[test]
fn test_critical_performance_thresholds() {
    let mut app = create_test_app();

    // Set up metrics that trigger critical status
    {
        let mut perf_diagnostics = app.world_mut().resource_mut::<PerformanceDiagnostics>();
        perf_diagnostics.current_frame.draw_calls = 1300; // Critical
        perf_diagnostics.current_frame.active_point_lights = 250; // Critical
        perf_diagnostics.current_frame.update_time_ms = 20.0; // Critical
    }

    // Run to update diagnostics
    for _ in 0..5 {
        app.update();
    }

    let perf_diagnostics = app.world().resource::<PerformanceDiagnostics>();

    // Test critical status
    assert_eq!(perf_diagnostics.get_status(), PerformanceStatus::Critical);

    // Test that warnings are generated
    assert!(!perf_diagnostics.warnings.is_empty());
}

/// Test performance warning generation
#[test]
fn test_performance_warning_generation() {
    let mut app = create_test_app();

    // Set up metrics that should generate warnings
    {
        let mut perf_diagnostics = app.world_mut().resource_mut::<PerformanceDiagnostics>();
        perf_diagnostics.current_frame.draw_calls = 1100; // Over budget
        perf_diagnostics.current_frame.instance_count = 80000; // Over budget
        perf_diagnostics.current_frame.update_time_ms = 10.0; // Over budget
        perf_diagnostics.current_frame.gpu_culling_time_ms = 0.3; // Over budget
    }

    // Run to update diagnostics and generate warnings
    for _ in 0..5 {
        app.update();
    }

    let perf_diagnostics = app.world().resource::<PerformanceDiagnostics>();

    // Test that warnings are generated
    assert!(!perf_diagnostics.warnings.is_empty());

    // Test specific warning types
    let has_draw_calls_warning = perf_diagnostics
        .warnings
        .iter()
        .any(|w| matches!(w, PerformanceWarning::DrawCallsBudgetExceeded { .. }));

    let has_instance_count_warning = perf_diagnostics
        .warnings
        .iter()
        .any(|w| matches!(w, PerformanceWarning::InstanceCountBudgetExceeded { .. }));

    let has_update_time_warning = perf_diagnostics
        .warnings
        .iter()
        .any(|w| matches!(w, PerformanceWarning::UpdateTimeExceeded { .. }));

    let has_gpu_culling_warning = perf_diagnostics
        .warnings
        .iter()
        .any(|w| matches!(w, PerformanceWarning::GpuCullingTimeExceeded { .. }));

    assert!(has_draw_calls_warning);
    assert!(has_instance_count_warning);
    assert!(has_update_time_warning);
    assert!(has_gpu_culling_warning);
}

/// Test performance history tracking
#[test]
fn test_performance_history_tracking() {
    let mut app = create_test_app();

    // Run for multiple frames to build history
    for i in 0..70 {
        // Vary metrics over time
        {
            let mut perf_diagnostics = app.world_mut().resource_mut::<PerformanceDiagnostics>();
            perf_diagnostics.current_frame.draw_calls = 100 + (i % 20) as u32;
            perf_diagnostics.current_frame.instance_count = 10000 + (i % 1000) as u32;
            perf_diagnostics.current_frame.update_time_ms = 5.0 + (i % 10) as f32 * 0.1;
        }

        app.update();
    }

    let perf_diagnostics = app.world().resource::<PerformanceDiagnostics>();

    // Test that history is maintained
    assert_eq!(perf_diagnostics.history.len(), 60); // Max history length

    // Test that averages can be calculated
    let averages = perf_diagnostics.calculate_averages();
    assert!(averages.draw_calls > 0);
    assert!(averages.instance_count > 0);
    assert!(averages.update_time_ms > 0.0);
}

/// Test diagnostic ID uniqueness
#[test]
fn test_diagnostic_id_uniqueness() {
    let ids = vec![
        PerformanceDiagnosticIds::DRAW_CALLS,
        PerformanceDiagnosticIds::INSTANCE_COUNT,
        PerformanceDiagnosticIds::TOTAL_ENTITIES,
        PerformanceDiagnosticIds::ACTIVE_POINT_LIGHTS,
        PerformanceDiagnosticIds::SPAWN_QUEUE_LENGTH,
        PerformanceDiagnosticIds::DESPAWN_QUEUE_LENGTH,
        PerformanceDiagnosticIds::AVERAGE_UPDATE_TIME,
        PerformanceDiagnosticIds::GPU_CULLING_TIME,
        PerformanceDiagnosticIds::SECTORS_LOADED,
        PerformanceDiagnosticIds::MEMORY_USAGE_MB,
    ];

    // Test that all IDs are unique
    for (i, id1) in ids.iter().enumerate() {
        for (j, id2) in ids.iter().enumerate() {
            if i != j {
                assert_ne!(id1, id2, "Diagnostic IDs must be unique");
            }
        }
    }
}

/// Test performance budget constants
#[test]
fn test_performance_budget_constants() {
    // Test that budget constants are reasonable
    assert!(PerformanceBudgets::MAX_DRAW_CALLS > 0.0);
    assert!(PerformanceBudgets::MAX_INSTANCE_COUNT > 0.0);
    assert!(PerformanceBudgets::MAX_TOTAL_ENTITIES > 0.0);
    assert!(PerformanceBudgets::MAX_ACTIVE_LIGHTS > 0.0);
    assert!(PerformanceBudgets::MAX_SECTORS_LOADED > 0.0);
    assert!(PerformanceBudgets::MAX_AVERAGE_UPDATE_TIME_MS > 0.0);
    assert!(PerformanceBudgets::MAX_GPU_CULLING_TIME_MS > 0.0);

    // Test that alarm threshold is higher than target
    assert!(PerformanceBudgets::ALARM_DRAW_CALLS > PerformanceBudgets::MAX_DRAW_CALLS);

    // Test that budget values are within reasonable AAA-grade ranges
    assert!(PerformanceBudgets::MAX_DRAW_CALLS <= 10000.0); // Reasonable upper bound
    assert!(PerformanceBudgets::MAX_INSTANCE_COUNT <= 1000000.0); // Reasonable upper bound
    assert!(PerformanceBudgets::MAX_AVERAGE_UPDATE_TIME_MS <= 16.6); // 60 FPS minimum
    assert!(PerformanceBudgets::MAX_GPU_CULLING_TIME_MS <= 1.0); // Reasonable GPU time
    assert!(PerformanceBudgets::MAX_ACTIVE_LIGHTS <= 1000.0); // Reasonable light count
}

/// Test frame metrics default values
#[test]
fn test_frame_metrics_defaults() {
    let metrics = FrameMetrics::default();

    assert_eq!(metrics.draw_calls, 0);
    assert_eq!(metrics.instance_count, 0);
    assert_eq!(metrics.total_entities, 0);
    assert_eq!(metrics.active_point_lights, 0);
    assert_eq!(metrics.spawn_queue_length, 0);
    assert_eq!(metrics.despawn_queue_length, 0);
    assert_eq!(metrics.update_time_ms, 0.0);
    assert_eq!(metrics.gpu_culling_time_ms, 0.0);
    assert_eq!(metrics.sectors_loaded, 0);
    assert_eq!(metrics.memory_usage_mb, 0.0);
}

/// Test performance diagnostics resource initialization
#[test]
fn test_performance_diagnostics_initialization() {
    let diagnostics = PerformanceDiagnostics::default();

    assert_eq!(diagnostics.current_frame.draw_calls, 0);
    assert_eq!(diagnostics.history.len(), 0);
    assert_eq!(diagnostics.history.capacity(), 60);
    assert_eq!(diagnostics.warnings.len(), 0);
    assert_eq!(diagnostics.hard_limits_enabled, false);
}

/// Test warning limit enforcement
#[test]
fn test_warning_limit_enforcement() {
    let mut app = create_test_app();

    // Generate many warnings
    for _i in 0..150 {
        let mut perf_diagnostics = app.world_mut().resource_mut::<PerformanceDiagnostics>();
        perf_diagnostics.add_warning(PerformanceWarning::DrawCallsBudgetExceeded {
            current: 1100,
            budget: 1000,
        });

        // Run an update to trigger warning processing
        app.update();
    }

    let perf_diagnostics = app.world().resource::<PerformanceDiagnostics>();

    // Test that warnings are limited to prevent memory growth
    assert!(perf_diagnostics.warnings.len() <= 100);
}

/// Test color coding for UI metrics
#[test]
fn test_metric_color_coding() {
    let mut app = create_test_app();

    // Test good performance (green)
    {
        let mut perf_diagnostics = app.world_mut().resource_mut::<PerformanceDiagnostics>();
        perf_diagnostics.current_frame.draw_calls = 500; // Within budget
    }

    app.update();

    let perf_diagnostics = app.world().resource::<PerformanceDiagnostics>();
    let color = perf_diagnostics.get_metric_color("draw_calls");
    assert_eq!(color, (0.0, 1.0, 0.0)); // Green

    // Test warning performance (yellow)
    {
        let mut perf_diagnostics = app.world_mut().resource_mut::<PerformanceDiagnostics>();
        perf_diagnostics.current_frame.draw_calls = 1100; // Over budget
    }

    app.update();

    let perf_diagnostics = app.world().resource::<PerformanceDiagnostics>();
    let color = perf_diagnostics.get_metric_color("draw_calls");
    assert_eq!(color, (1.0, 1.0, 0.0)); // Yellow

    // Test critical performance (red)
    {
        let mut perf_diagnostics = app.world_mut().resource_mut::<PerformanceDiagnostics>();
        perf_diagnostics.current_frame.draw_calls = 1300; // Critical
    }

    app.update();

    let perf_diagnostics = app.world().resource::<PerformanceDiagnostics>();
    let color = perf_diagnostics.get_metric_color("draw_calls");
    assert_eq!(color, (1.0, 0.0, 0.0)); // Red
}

/// Test regression gate thresholds
#[test]
fn test_regression_gate_thresholds() {
    // Test that regression gate thresholds match Oracle's requirements
    assert_eq!(PerformanceBudgets::MAX_TOTAL_ENTITIES, 1200.0);
    assert_eq!(PerformanceBudgets::MAX_SECTORS_LOADED, 200.0);
    assert_eq!(PerformanceBudgets::MAX_DRAW_CALLS, 1000.0);
    assert_eq!(PerformanceBudgets::MAX_AVERAGE_UPDATE_TIME_MS, 8.0); // 125 FPS
    assert_eq!(PerformanceBudgets::MAX_ACTIVE_LIGHTS, 200.0);
    assert_eq!(PerformanceBudgets::MAX_GPU_CULLING_TIME_MS, 0.25);
}

/// Helper function to create a test app with performance monitoring
fn create_test_app() -> App {
    let mut app = App::new();

    app.add_plugins((
        TaskPoolPlugin::default(),
        TimePlugin::default(),
        TransformPlugin::default(),
        AssetPlugin::default(),
        // HierarchyPlugin::default(), // Not needed for these tests
        DiagnosticsPlugin::default(),
    ));

    app.add_plugins((
        FrameTimeDiagnosticsPlugin::default(),
        EntityCountDiagnosticsPlugin::default(),
        PerformanceDiagnosticsPlugin,
        DistanceCachePlugin,
    ));

    app
}
