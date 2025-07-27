//! Optimized culling performance benchmarks
//!
//! Oracle's performance targets:
//! - GPU culling: <0.25ms @ 100K+ instances
//! - CPU culling: <1.0ms @ <50K instances
//! - Automatic switching at 50K threshold

use amp_render::optimized_culling::{CullingPerformanceStats, GpuTier, OptimizedCullingConfig};
use amp_render::prelude::{BatchKey, Cullable, ExtractedInstance};
use bevy::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use glam::{Mat4, Vec3};
use std::time::Instant;

/// Generate test instances for benchmarking
fn generate_test_instances(count: usize) -> Vec<(ExtractedInstance, Cullable)> {
    let mut instances = Vec::with_capacity(count);
    let camera_pos = Vec3::ZERO;

    for i in 0..count {
        // Distribute instances across a large area
        let x = (i as f32 % 1000.0) * 10.0 - 5000.0;
        let z = (i as f32 / 1000.0).floor() * 10.0 - 5000.0;
        let y = fastrand::f32() * 200.0 - 100.0;

        let transform = Mat4::from_translation(Vec3::new(x, y, z));
        let batch_key = BatchKey {
            mesh_id: fastrand::u64(0..100),    // 100 different mesh types
            material_id: fastrand::u64(0..50), // 50 different materials
            flags: 0,
        };

        let instance = ExtractedInstance::new(transform, batch_key, camera_pos);
        let cullable = Cullable::new(fastrand::f32() * 50.0 + 5.0); // 5-55 radius

        instances.push((instance, cullable));
    }

    instances
}

/// Benchmark CPU culling performance at different instance counts
fn bench_cpu_culling_scaling(c: &mut Criterion) {
    let test_counts = [1_000, 5_000, 10_000, 25_000, 50_000, 75_000, 100_000];

    let mut group = c.benchmark_group("cpu_culling_scaling");

    for &count in &test_counts {
        let instances = generate_test_instances(count);

        group.bench_with_input(
            BenchmarkId::new("cpu_culling", count),
            &count,
            |b, &_count| {
                b.iter(|| {
                    // Simulate CPU frustum culling
                    let start = Instant::now();
                    let mut visible_count = 0;

                    for (instance, cullable) in &instances {
                        let position = instance.transform.w_axis.truncate();
                        let radius = cullable.radius;

                        // Simple distance + sphere test
                        let distance = position.length();
                        if distance <= (1000.0 + radius) {
                            visible_count += 1;
                        }
                    }

                    let elapsed = start.elapsed().as_secs_f32() * 1000.0;
                    black_box((visible_count, elapsed))
                })
            },
        );
    }

    group.finish();
}

/// Benchmark GPU tier detection and threshold selection
fn bench_gpu_tier_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("gpu_tier_detection");

    let tiers = [GpuTier::LowEnd, GpuTier::MidRange, GpuTier::HighEnd];

    for tier in &tiers {
        group.bench_with_input(
            BenchmarkId::new("tier_threshold", format!("{:?}", tier)),
            tier,
            |b, &tier| {
                b.iter(|| {
                    let threshold = tier.gpu_threshold();
                    let batch_size = tier.optimal_batch_size();
                    black_box((threshold, batch_size))
                })
            },
        );
    }

    group.finish();
}

/// Benchmark performance statistics tracking
fn bench_performance_stats(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_stats");

    group.bench_function("record_frame_stats", |b| {
        let mut stats = CullingPerformanceStats::default();

        b.iter(|| {
            stats.record_frame(
                black_box(0.15),   // frame time
                black_box(50_000), // instances
                black_box(30_000), // visible
                black_box(amp_render::optimized_culling::CullingMethod::Gpu),
            );

            let efficiency = stats.culling_efficiency();
            let meets_target = stats.meets_target;
            black_box((efficiency, meets_target))
        })
    });

    group.finish();
}

/// Benchmark culling method determination logic
fn bench_culling_method_selection(c: &mut Criterion) {
    let mut group = c.benchmark_group("culling_method_selection");

    let instance_counts = [10_000, 25_000, 50_000, 75_000, 100_000, 200_000];
    let configs = [
        (
            "low_end",
            OptimizedCullingConfig {
                gpu_tier: GpuTier::LowEnd,
                enable_auto_switching: true,
                ..Default::default()
            },
        ),
        (
            "mid_range",
            OptimizedCullingConfig {
                gpu_tier: GpuTier::MidRange,
                enable_auto_switching: true,
                ..Default::default()
            },
        ),
        (
            "high_end",
            OptimizedCullingConfig {
                gpu_tier: GpuTier::HighEnd,
                enable_auto_switching: true,
                ..Default::default()
            },
        ),
    ];

    for (config_name, config) in &configs {
        for &instance_count in &instance_counts {
            group.bench_with_input(
                BenchmarkId::new(
                    format!("{}_{}", config_name, instance_count),
                    instance_count,
                ),
                &(config, instance_count),
                |b, &(config, count)| {
                    b.iter(|| {
                        // Simulate method determination logic
                        let gpu_threshold = config.gpu_tier.gpu_threshold();
                        let use_gpu = config.enable_auto_switching && count >= gpu_threshold;

                        let method = if use_gpu {
                            amp_render::optimized_culling::CullingMethod::Gpu
                        } else {
                            amp_render::optimized_culling::CullingMethod::Cpu
                        };

                        black_box(method)
                    })
                },
            );
        }
    }

    group.finish();
}

/// Oracle's performance validation test
fn bench_oracle_targets(c: &mut Criterion) {
    let mut group = c.benchmark_group("oracle_performance_targets");
    group.sample_size(100); // More samples for statistical significance

    // Test Oracle's GPU target: <0.25ms for 100K+ instances
    group.bench_function("gpu_target_100k", |b| {
        let instances = generate_test_instances(100_000);

        b.iter(|| {
            let start = Instant::now();

            // Simulate GPU culling performance (optimistic)
            let batch_size = 1024;
            let mut visible_count = 0;

            for chunk in instances.chunks(batch_size) {
                // Simulate GPU dispatch latency
                std::thread::sleep(std::time::Duration::from_nanos(1000)); // 1μs per batch

                for (instance, cullable) in chunk {
                    let position = instance.transform.w_axis.truncate();
                    let distance = position.length();
                    if distance <= (1000.0 + cullable.radius) {
                        visible_count += 1;
                    }
                }
            }

            let elapsed = start.elapsed().as_secs_f32() * 1000.0;

            // Oracle's target: <0.25ms
            if elapsed > 0.25 {
                eprintln!(
                    "WARNING: GPU culling exceeded Oracle's 0.25ms target: {:.3}ms",
                    elapsed
                );
            }

            black_box((visible_count, elapsed))
        })
    });

    // Test Oracle's CPU target: <1.0ms for 50K instances
    group.bench_function("cpu_target_50k", |b| {
        let instances = generate_test_instances(50_000);

        b.iter(|| {
            let start = Instant::now();
            let mut visible_count = 0;

            // Optimized CPU culling
            for (instance, cullable) in &instances {
                let position = instance.transform.w_axis.truncate();
                let distance = position.length();

                // Early distance rejection
                if distance > 1050.0 {
                    // max_distance + max_radius
                    continue;
                }

                // Sphere test
                if distance <= (1000.0 + cullable.radius) {
                    visible_count += 1;
                }
            }

            let elapsed = start.elapsed().as_secs_f32() * 1000.0;

            // Oracle's target: <1.0ms
            if elapsed > 1.0 {
                eprintln!(
                    "WARNING: CPU culling exceeded Oracle's 1.0ms target: {:.3}ms",
                    elapsed
                );
            }

            black_box((visible_count, elapsed))
        })
    });

    group.finish();
}

/// Memory allocation benchmark for large instance counts
fn bench_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");

    let test_counts = [50_000, 100_000, 200_000, 400_000];

    for &count in &test_counts {
        group.bench_with_input(
            BenchmarkId::new("memory_allocation", count),
            &count,
            |b, &count| {
                b.iter(|| {
                    // Test memory allocation patterns
                    let instances = generate_test_instances(count);

                    // Simulate batch grouping (memory intensive operation)
                    let mut batches: std::collections::HashMap<u64, Vec<_>> =
                        std::collections::HashMap::new();

                    for (instance, cullable) in instances {
                        let key = instance.batch_key.mesh_id;
                        batches.entry(key).or_default().push((instance, cullable));
                    }

                    let batch_count = batches.len();
                    let total_instances: usize = batches.values().map(|v| v.len()).sum();

                    black_box((batch_count, total_instances))
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_cpu_culling_scaling,
    bench_gpu_tier_detection,
    bench_performance_stats,
    bench_culling_method_selection,
    bench_oracle_targets,
    bench_memory_efficiency
);

criterion_main!(benches);
