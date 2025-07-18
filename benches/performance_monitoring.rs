//! Criterion benchmarks for performance monitoring system
//!
//! This module provides benchmarks for tracking performance trends
//! and ensuring that the performance monitoring system itself
//! doesn't introduce significant overhead.

use bevy::app::App;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use bevy::diagnostic::{
    DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
};
use bevy::prelude::*;
use bevy::time::TimePlugin;
use bevy::transform::TransformPlugin;
use bevy::utils::default;

use amp_render::diagnostics::{
    PerformanceDiagnosticPaths, PerformanceDiagnostics, PerformanceDiagnosticsPlugin,
};
use amp_render::BatchingPlugin;

/// Benchmark the performance monitoring system overhead
///
/// This benchmark measures the overhead introduced by the performance
/// monitoring system to ensure it doesn't significantly impact performance.
fn bench_performance_monitoring_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_monitoring_overhead");

    // Benchmark without performance monitoring
    group.bench_function("without_monitoring", |b| {
        b.iter(|| {
            let mut app = create_minimal_app();

            // Run for several frames
            for _ in 0..black_box(60) {
                app.update();
            }
        });
    });

    // Benchmark with performance monitoring
    group.bench_function("with_monitoring", |b| {
        b.iter(|| {
            let mut app = create_monitored_app();

            // Run for several frames
            for _ in 0..black_box(60) {
                app.update();
            }
        });
    });

    group.finish();
}

/// Benchmark performance monitoring with different entity counts
///
/// This benchmark measures how the performance monitoring system scales
/// with different numbers of entities in the world.
fn bench_performance_monitoring_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_monitoring_scaling");

    let entity_counts = [100, 500, 1000, 2000, 5000];

    for entity_count in entity_counts.iter() {
        group.bench_with_input(
            BenchmarkId::new("entity_count", entity_count),
            entity_count,
            |b, &entity_count| {
                b.iter(|| {
                    let mut app = create_monitored_app();

                    // Spawn entities
                    spawn_test_entities(&mut app, entity_count);

                    // Run for several frames
                    for _ in 0..black_box(30) {
                        app.update();
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark individual diagnostic measurements
///
/// This benchmark measures the cost of individual diagnostic operations
/// to identify performance bottlenecks.
fn bench_diagnostic_measurements(c: &mut Criterion) {
    let mut group = c.benchmark_group("diagnostic_measurements");

    let mut app = create_monitored_app();
    spawn_test_entities(&mut app, 1000);

    // Run a few frames to initialize
    for _ in 0..10 {
        app.update();
    }

    // Benchmark individual measurements
    group.bench_function("measure_total_entities", |b| {
        b.iter(|| {
            let entities = app.world().query::<Entity>();
            let count = entities.iter(&app.world).count();
            black_box(count);
        });
    });

    group.bench_function("measure_point_lights", |b| {
        b.iter(|| {
            let lights = app.world().query::<&PointLight>();
            let count = lights.iter(&app.world).count();
            black_box(count);
        });
    });

    group.bench_function("update_diagnostics", |b| {
        b.iter(|| {
            let mut diagnostics_store = app.world().resource_mut::<DiagnosticsStore>();
            diagnostics_store
                .add_measurement(&PerformanceDiagnosticPaths::DRAW_CALLS, || black_box(150.0));
            diagnostics_store.add_measurement(&PerformanceDiagnosticPaths::INSTANCE_COUNT, || {
                black_box(5000.0)
            });
        });
    });

    group.finish();
}

/// Benchmark performance status calculation
///
/// This benchmark measures the cost of calculating performance status
/// and checking against budgets.
fn bench_performance_status_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_status_calculation");

    let mut app = create_monitored_app();
    spawn_test_entities(&mut app, 1000);

    // Run to get some performance data
    for _ in 0..30 {
        app.update();
    }

    group.bench_function("get_status", |b| {
        b.iter(|| {
            let performance_diagnostics = app.world().resource::<PerformanceDiagnostics>();
            let status = performance_diagnostics.get_status();
            black_box(status);
        });
    });

    group.bench_function("check_budget", |b| {
        b.iter(|| {
            let performance_diagnostics = app.world().resource::<PerformanceDiagnostics>();
            let within_budget = performance_diagnostics.is_within_budget("draw_calls");
            black_box(within_budget);
        });
    });

    group.bench_function("calculate_averages", |b| {
        b.iter(|| {
            let performance_diagnostics = app.world().resource::<PerformanceDiagnostics>();
            let averages = performance_diagnostics.calculate_averages();
            black_box(averages);
        });
    });

    group.finish();
}

/// Benchmark memory usage tracking
///
/// This benchmark measures the memory overhead of the performance
/// monitoring system and its data structures.
fn bench_memory_usage_tracking(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage_tracking");

    // Measure memory usage with different history lengths
    let history_lengths = [10, 30, 60, 120, 300];

    for history_length in history_lengths.iter() {
        group.bench_with_input(
            BenchmarkId::new("history_length", history_length),
            history_length,
            |b, &history_length| {
                b.iter(|| {
                    let mut app = create_monitored_app();

                    // Simulate frames to fill history
                    for _ in 0..black_box(history_length) {
                        app.update();
                    }

                    // Measure memory impact
                    let performance_diagnostics = app.world().resource::<PerformanceDiagnostics>();
                    let history_len = performance_diagnostics.history.len();
                    black_box(history_len);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark regression gate checking
///
/// This benchmark measures the cost of running regression gates
/// to ensure they don't significantly impact CI performance.
fn bench_regression_gate_checking(c: &mut Criterion) {
    let mut group = c.benchmark_group("regression_gate_checking");

    let mut app = create_monitored_app();
    spawn_test_entities(&mut app, 1000);

    // Run simulation to get metrics
    for _ in 0..60 {
        app.update();
    }

    group.bench_function("check_all_gates", |b| {
        b.iter(|| {
            let performance_diagnostics = app.world().resource::<PerformanceDiagnostics>();
            let diagnostics_store = app.world().resource::<DiagnosticsStore>();

            // Simulate all regression gate checks
            let mut gate_results = Vec::new();

            // Check total entities
            if let Some(entities_diag) =
                diagnostics_store.get(&PerformanceDiagnosticPaths::TOTAL_ENTITIES)
            {
                if let Some(total_entities) = entities_diag.smoothed() {
                    gate_results.push(total_entities <= 1200.0);
                }
            }

            // Check sectors loaded
            if let Some(sectors_diag) =
                diagnostics_store.get(&PerformanceDiagnosticPaths::SECTORS_LOADED)
            {
                if let Some(sectors_loaded) = sectors_diag.smoothed() {
                    gate_results.push(sectors_loaded <= 200.0);
                }
            }

            // Check draw calls
            if let Some(draw_calls_diag) =
                diagnostics_store.get(&PerformanceDiagnosticPaths::DRAW_CALLS)
            {
                if let Some(draw_calls) = draw_calls_diag.smoothed() {
                    gate_results.push(draw_calls < 1000.0);
                }
            }

            // Check update time
            if let Some(update_time_diag) =
                diagnostics_store.get(&PerformanceDiagnosticPaths::AVERAGE_UPDATE_TIME)
            {
                if let Some(avg_update_time) = update_time_diag.smoothed() {
                    gate_results.push(avg_update_time < 8.0);
                }
            }

            // Check active lights
            if let Some(lights_diag) =
                diagnostics_store.get(&PerformanceDiagnosticPaths::ACTIVE_POINT_LIGHTS)
            {
                if let Some(active_lights) = lights_diag.smoothed() {
                    gate_results.push(active_lights <= 200.0);
                }
            }

            black_box(gate_results);
        });
    });

    group.finish();
}

/// Benchmark performance trends analysis
///
/// This benchmark measures the cost of analyzing performance trends
/// for baseline comparisons.
fn bench_performance_trends_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_trends_analysis");

    let mut app = create_monitored_app();
    spawn_test_entities(&mut app, 1000);

    // Run for a while to get trend data
    for _ in 0..120 {
        app.update();
    }

    group.bench_function("analyze_trends", |b| {
        b.iter(|| {
            let performance_diagnostics = app.world().resource::<PerformanceDiagnostics>();

            // Simulate trend analysis
            let averages = performance_diagnostics.calculate_averages();
            let current = &performance_diagnostics.current_frame;

            // Calculate trend indicators
            let frame_time_trend = current.update_time_ms - averages.update_time_ms;
            let draw_calls_trend = current.draw_calls as f32 - averages.draw_calls as f32;
            let instance_trend = current.instance_count as f32 - averages.instance_count as f32;

            black_box((frame_time_trend, draw_calls_trend, instance_trend));
        });
    });

    group.finish();
}

/// Create a minimal app without performance monitoring
fn create_minimal_app() -> App {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);

    app.add_plugins(BatchingPlugin);

    app
}

/// Create an app with performance monitoring enabled
fn create_monitored_app() -> App {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);

    // Add diagnostic plugins
    app.add_plugins((
        FrameTimeDiagnosticsPlugin::default(),
        EntityCountDiagnosticsPlugin::default(),
        PerformanceDiagnosticsPlugin,
    ));

    app.add_plugins(BatchingPlugin);

    app
}

/// Spawn test entities for benchmarking
fn spawn_test_entities(app: &mut App, count: usize) {
    // Spawn regular entities
    app.world_mut().spawn_batch((0..count).map(|i| {
        (
            Transform::from_translation(Vec3::new(i as f32, 0.0, 0.0)),
            GlobalTransform::default(),
        )
    }));

    // Spawn some point lights (10% of entities)
    app.world_mut().spawn_batch((0..count / 10).map(|i| {
        (
            Transform::from_translation(Vec3::new(i as f32 * 10.0, 5.0, 0.0)),
            GlobalTransform::default(),
            PointLight {
                intensity: 1000.0,
                range: 10.0,
                ..default()
            },
        )
    }));
}

criterion_group!(
    benches,
    bench_performance_monitoring_overhead,
    bench_performance_monitoring_scaling,
    bench_diagnostic_measurements,
    bench_performance_status_calculation,
    bench_memory_usage_tracking,
    bench_regression_gate_checking,
    bench_performance_trends_analysis,
);

criterion_main!(benches);
