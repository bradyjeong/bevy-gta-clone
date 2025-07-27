# Performance Profiling Guide

This document provides a comprehensive guide for profiling and optimizing the Amp game engine's performance.

## Overview

The Amp engine includes comprehensive performance monitoring and optimization systems:

- **Performance Diagnostics**: Real-time performance monitoring with hard caps
- **GPU Culling**: Compute shader-based instance culling for optimal rendering
- **Batch Processing**: Instance batching to reduce draw calls
- **LOD System**: Level-of-detail management for distant objects
- **Streaming**: Sector-based world streaming for large environments
- **Distance Cache**: Cached distance calculations for performance

## Performance Budgets

The engine enforces the following performance budgets:

### Frame Time Budgets
- **Target**: 60 FPS (16.67ms per frame)
- **Warning**: 33.3ms (30 FPS)
- **Critical**: 66.7ms (15 FPS)

### System Budgets
- **GPU Culling**: ≤0.25ms per frame
- **Batch Processing**: ≤2.5ms per frame
- **LOD Updates**: ≤1.0ms per frame
- **Streaming**: ≤1.5ms per frame

### Resource Limits
- **Max Active Lights**: 256
- **Max Spawns Per Frame**: 50
- **Max Batch Count**: 500
- **Max GPU Memory**: 2GB

## Profiling Tools

### Built-in Diagnostics

Enable performance diagnostics in your application:

```rust
use amp_render::prelude::*;

app.add_plugins(PerformanceDiagnosticsPlugin);
```

### Performance Metrics

Access performance metrics at runtime:

```rust
fn check_performance(diagnostics: Res<PerformanceDiagnostics>) {
    let frame_time = diagnostics.current_frame.frame_time;
    let gpu_culling_time = diagnostics.current_frame.gpu_culling_time;
    let batch_count = diagnostics.current_frame.batch_count;
    
    if frame_time > 16.67 {
        warn!("Frame time budget exceeded: {:.2}ms", frame_time);
    }
}
```

### External Profilers

#### Tracy Integration

Enable Tracy profiling:

```bash
cargo run --features profile --release
```

#### Criterion Benchmarks

Run performance benchmarks:

```bash
cargo bench
```

#### Custom Profiling

Use the built-in profiling macros:

```rust
use amp_render::prelude::*;

fn my_system() {
    profiling::scope!("my_system");
    
    // Your system code here
    
    profiling::scope!("expensive_operation");
    // Expensive operation
}
```

## Optimization Strategies

### GPU Culling Optimization

1. **Enable GPU Culling**: Use the `gpu_culling` feature
2. **Tune Work Group Size**: Adjust `gpu_culling.work_group_size` in config
3. **Hierarchical Culling**: Enable `gpu_culling.enable_hierarchical`

```rust
// Enable GPU culling
app.add_plugins(GpuCullingPlugin);

// Configure GPU culling
let mut tuning = app.world.resource_mut::<PerformanceTuning>();
tuning.gpu_culling.work_group_size = 64;
tuning.gpu_culling.enable_hierarchical = true;
```

### Batch Processing Optimization

1. **Optimize Batch Size**: Tune `batching.max_instances_per_batch`
2. **Reduce State Changes**: Group objects by material/mesh
3. **Update Frequency**: Adjust `batching.buffer_update_frequency`

```rust
// Optimize batching
let mut tuning = app.world.resource_mut::<PerformanceTuning>();
tuning.batching.max_instances_per_batch = 1000;
tuning.batching.batch_combine_threshold = 50;
```

### LOD System Optimization

1. **Tune Distances**: Adjust `lod.transition_distances`
2. **Hysteresis**: Configure `lod.hysteresis_factor` to prevent popping
3. **Cross-fade**: Use `lod.cross_fade_duration` for smooth transitions

```rust
// Optimize LOD
let mut tuning = app.world.resource_mut::<PerformanceTuning>();
tuning.lod.transition_distances = vec![50.0, 100.0, 200.0, 400.0];
tuning.lod.hysteresis_factor = 0.1;
```

### Streaming Optimization

1. **Sector Size**: Tune `streaming.sector_size` for your content
2. **Spawn Limits**: Adjust `streaming.max_spawns_per_frame`
3. **View Radius**: Configure `streaming.view_radius`

```rust
// Optimize streaming
let mut tuning = app.world.resource_mut::<PerformanceTuning>();
tuning.streaming.sector_size = 64.0;
tuning.streaming.max_spawns_per_frame = 50;
tuning.streaming.view_radius = 512.0;
```

## Performance Testing

### Benchmark Scenes

The engine includes benchmark scenes for performance testing:

1. **Empty Scene**: Baseline performance test
2. **Medium Scene**: 10k buildings, 2k lights
3. **Heavy Scene**: 34k buildings, 30k lights

### Running Benchmarks

```bash
# Run empty scene benchmark
cargo run --release --example empty_scene_benchmark

# Run medium scene benchmark  
cargo run --release --example medium_scene_benchmark

# Run heavy scene benchmark
cargo run --release --example heavy_scene_benchmark
```

### Headless Benchmarks

For CI/CD pipelines, use headless benchmarks:

```bash
# Run 1000-frame headless benchmark
./scripts/run_headless_benchmark.sh 1000
```

## Performance Gates

The engine includes performance gates that fail builds if performance regresses:

### CI Performance Gates

```yaml
# .github/workflows/ci.yml
- name: Performance Gate
  run: cargo run --release --example performance_gate
```

### Local Performance Checks

```bash
# Run performance checks locally
cargo xtask perf --format json
```

## Troubleshooting

### High Frame Times

1. **Check GPU Utilization**: Use GPU profilers
2. **Analyze Bottlenecks**: Use Tracy or other profilers
3. **Reduce Draw Calls**: Improve batching
4. **Optimize Shaders**: Profile GPU shaders

### Memory Issues

1. **Monitor Allocations**: Use allocation tracking
2. **Object Pools**: Implement object pooling
3. **Buffer Management**: Optimize buffer usage
4. **Garbage Collection**: Profile memory allocations

### Stuttering

1. **Frame Pacing**: Check frame time consistency
2. **Async Loading**: Use async streaming
3. **Reduce Spikes**: Spread work across frames
4. **Cache Warmup**: Pre-populate caches

## Configuration

### Performance Tuning File

Create `assets/config/performance_tuning.ron`:

```ron
(
    culling: (
        vehicle_max_distance: 150.0,
        building_max_distance: 300.0,
        npc_max_distance: 100.0,
        environment_max_distance: 200.0,
        frustum_margin: 10.0,
    ),
    lod: (
        transition_distances: [50.0, 100.0, 200.0, 400.0],
        hysteresis_factor: 0.1,
        cross_fade_duration: 0.5,
        max_lod_level: 4,
    ),
    streaming: (
        sector_size: 64.0,
        view_radius: 512.0,
        max_spawns_per_frame: 50,
        max_despawns_per_frame: 100,
        priority_threshold: 0.5,
    ),
    budgets: (
        max_active_lights: 256,
        max_spawn_queue_length: 1000,
        max_batch_count: 500,
        max_gpu_memory_mb: 2048,
        frame_time_budget_ms: 16.67,
    ),
    batching: (
        max_instances_per_batch: 1000,
        batch_combine_threshold: 50,
        buffer_update_frequency: 4,
        instance_buffer_size: 10000,
    ),
    gpu_culling: (
        work_group_size: 64,
        max_objects_per_pass: 100000,
        time_budget_ms: 0.25,
        enable_hierarchical: true,
    ),
)
```

### Loading Configuration

```rust
// Load performance tuning from file
let tuning = PerformanceTuning::load_from_file("assets/config/performance_tuning.ron")?;
app.insert_resource(tuning);
```

## Best Practices

1. **Profile Early**: Profile during development, not just at the end
2. **Measure First**: Always measure before optimizing
3. **Use Tools**: Leverage built-in profiling tools
4. **Test Regularly**: Run performance tests in CI/CD
5. **Document Changes**: Track performance impact of changes
6. **Monitor Production**: Use runtime diagnostics in production

## Performance Targets

### Desktop Targets (1080p)
- **Minimum**: 30 FPS stable
- **Target**: 60 FPS stable
- **Ideal**: 120 FPS capable

### Mobile Targets (720p)
- **Minimum**: 30 FPS stable
- **Target**: 60 FPS stable

### VR Targets (2160x1200)
- **Minimum**: 90 FPS stable
- **Target**: 120 FPS stable

## Conclusion

Performance optimization is an ongoing process. Use the tools and techniques in this guide to maintain optimal performance throughout development and in production.
