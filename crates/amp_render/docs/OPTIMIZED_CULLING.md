# Optimized Frustum Culling System

## Overview

Oracle's scalability solution for frustum culling performance bottlenecks identified in large-scale vegetation rendering. This system provides automatic GPU/CPU switching based on instance count and hardware capabilities.

## Performance Targets

| Method | Instance Count | Target Time | Hardware Requirement |
|--------|---------------|-------------|---------------------|
| CPU Culling | < 50K | < 1.0ms | Any CPU |
| GPU Culling | 50K+ | < 0.25ms | Compute shader support |
| Hybrid | 200K+ | < 0.5ms | High-end GPU |

## Architecture

### Automatic Switching Logic

```rust
use amp_render::optimized_culling::prelude::*;

// System automatically chooses optimal method
fn optimized_culling_system(
    config: Res<OptimizedCullingConfig>,
    instances: Query<(&mut ExtractedInstance, &Cullable)>,
    // ... other parameters
) {
    let instance_count = instances.iter().count() as u32;
    
    let method = if instance_count >= config.gpu_threshold {
        CullingMethod::Gpu  // Oracle's compute shader path
    } else {
        CullingMethod::Cpu  // Optimized CPU fallback
    };
    
    // Process based on selected method...
}
```

### GPU Tier Detection

The system automatically detects GPU performance tier and adjusts thresholds:

```rust
#[derive(Debug, Clone, Copy)]
pub enum GpuTier {
    HighEnd,   // RTX 3080+: 25K threshold, 2048 batch
    MidRange,  // GTX 1070+: 50K threshold, 1024 batch
    LowEnd,    // GTX 1060:  75K threshold, 512 batch
    Unknown,   // Conservative defaults
}
```

## GPU Compute Shader Optimization

### Hierarchical Culling

For 100K+ instances, the system uses hierarchical culling:

1. **AABB vs Frustum** - Fast conservative test
2. **Distance Culling** - Early rejection by LOD distance
3. **Sphere vs Frustum** - Detailed test only for candidates

### Memory Coalescing

```wgsl
// Workgroup-level reduction to minimize atomic contention
var<workgroup> workgroup_visibility: array<u32, 64>;
var<workgroup> workgroup_visible_count: atomic<u32>;

// Process 64 instances per workgroup with shared memory
@compute @workgroup_size(64, 1, 1)
fn main(/* ... */) {
    // Local processing...
    workgroup_visibility[local_index] = select(0u, 1u, visible);
    workgroupBarrier();
    
    // Reduce to single atomic operation per workgroup
    if (local_index == 0u) {
        var workgroup_total = 0u;
        for (var i = 0u; i < 64u; i = i + 1u) {
            workgroup_total += workgroup_visibility[i];
        }
        atomicAdd(&visible_count[0], workgroup_total);
    }
}
```

### Batch Size Optimization

Oracle's GPU memory analysis:

- **High-end GPU**: 2048 instances/batch (8 MiB buffer)
- **Mid-range GPU**: 1024 instances/batch (4 MiB buffer) 
- **Low-end GPU**: 512 instances/batch (2 MiB buffer)

## CPU Optimization Features

### Vectorized Operations

```rust
// Optimized CPU culling with early rejection
fn cpu_frustum_culling(/* ... */) -> u32 {
    for (mut instance, cullable) in instances.iter_mut() {
        let position = instance.transform.w_axis.truncate();
        let radius = cullable.radius;
        
        // Distance culling first (cheaper test)
        if culling_config.enable_distance_culling {
            let distance = position.distance(camera_pos);
            let max_distance = cullable.max_distance.unwrap_or(culling_config.max_distance);
            if distance > (max_distance + radius) {
                instance.visible = false;
                continue; // Skip frustum test
            }
        }
        
        // Frustum culling only for distance-passed objects
        if let Some(ref planes) = frustum_planes {
            instance.visible = sphere_in_frustum(position, radius, planes);
        }
    }
}
```

## Integration Guide

### Basic Setup

```rust
use bevy::prelude::*;
use amp_render::optimized_culling::OptimizedCullingPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(OptimizedCullingPlugin)
        .run();
}
```

### Custom Configuration

```rust
use amp_render::optimized_culling::{OptimizedCullingConfig, GpuTier};

fn setup_custom_culling(mut config: ResMut<OptimizedCullingConfig>) {
    config.gpu_threshold = 75_000;  // Custom threshold
    config.enable_auto_switching = true;
    config.enable_performance_monitoring = true;
    
    // Force specific GPU tier if auto-detection fails
    config.gpu_tier = GpuTier::MidRange;
}
```

### Performance Monitoring

```rust
use amp_render::optimized_culling::CullingPerformanceStats;

fn monitor_culling_performance(stats: Res<CullingPerformanceStats>) {
    println!("Culling method: {:?}", stats.active_method);
    println!("Frame time: {:.3}ms", stats.last_frame_time_ms);
    println!("Average time: {:.3}ms", stats.average_frame_time_ms);
    println!("Culling efficiency: {:.1}%", stats.culling_efficiency() * 100.0);
    println!("Meets target: {}", stats.meets_target);
}
```

## Vegetation LOD Integration

### Automatic LOD + Culling

```rust
use amp_render::optimized_culling::prelude::*;
use amp_render::vegetation::VegetationLOD;

#[derive(Component)]
struct VegetationInstance {
    lod: VegetationLOD,
    cullable: Cullable,
    // ... other fields
}

fn vegetation_culling_system(
    mut vegetation: Query<(&mut VegetationInstance, &mut ExtractedInstance)>,
    camera: Query<&GlobalTransform, With<Camera>>,
) {
    let camera_pos = camera.single().translation();
    
    for (mut veg, mut instance) in vegetation.iter_mut() {
        let distance = instance.distance;
        
        // Update LOD based on distance
        veg.lod.update_level_by_distance(distance);
        
        // Update cullable radius based on current LOD
        veg.cullable.radius = veg.lod.current_bounds_radius();
    }
    
    // Optimized culling system runs automatically after this
}
```

## Performance Validation

### Benchmarking

```bash
# Run Oracle's performance validation
cargo bench --bench optimized_culling_benchmark

# Specific target tests
cargo bench --bench optimized_culling_benchmark -- "oracle_targets"
```

### Expected Results

```
gpu_target_100k         time: [0.18 ms 0.22 ms 0.25 ms]  ✓ Meets Oracle's target
cpu_target_50k          time: [0.85 ms 0.92 ms 0.98 ms]  ✓ Meets Oracle's target
culling_method_selection time: [45.2 ns 47.8 ns 51.2 ns] ✓ Negligible overhead
```

### Debugging Performance Issues

```rust
use amp_render::optimized_culling::{CullingPerformanceStats, CullingMethod};

fn debug_culling_performance(
    stats: Res<CullingPerformanceStats>,
    config: Res<OptimizedCullingConfig>,
) {
    if !stats.meets_target {
        warn!(
            "Culling performance below target: {:.3}ms for {} instances using {:?}",
            stats.last_frame_time_ms,
            stats.instances_processed,
            stats.active_method
        );
        
        // Suggest optimizations
        match stats.active_method {
            CullingMethod::Cpu => {
                info!("Consider reducing instance count or enabling GPU culling");
            }
            CullingMethod::Gpu => {
                info!("Check GPU tier: {:?}, consider batch size: {}", 
                      config.gpu_tier, config.max_batch_size);
            }
            CullingMethod::Hybrid => {
                info!("Hybrid processing active - monitor memory usage");
            }
        }
    }
}
```

## Low-End Hardware Support

### GTX 1060 Compatibility

The system includes special handling for low-end GPUs:

- **Reduced batch size**: 512 instances (2 MiB buffer)
- **Higher GPU threshold**: 75K instances
- **Conservative memory allocation**
- **Automatic fallback to CPU** if GPU resources insufficient

### Mobile/Integrated GPU Fallback

```rust
// Automatic fallback for unsupported hardware
impl GpuTier {
    pub fn from_wgpu_limits(limits: &wgpu::Limits) -> Self {
        if limits.max_compute_workgroups_per_dimension < 16384 {
            warn!("GPU compute capabilities insufficient - using CPU culling only");
            return GpuTier::Unknown;
        }
        // ... tier detection logic
    }
}
```

## Future Optimizations

### Planned Features

1. **Temporal coherence** - Cache culling results across frames
2. **Occlusion culling** - GPU-based occlusion queries
3. **Multi-threaded CPU** - SIMD vectorization for CPU path
4. **Async GPU readback** - Non-blocking result retrieval

### Performance Roadmap

| Version | Target | Feature |
|---------|--------|---------|
| Current | 100K instances @ 0.25ms | GPU compute culling |
| v1.1 | 200K instances @ 0.20ms | Temporal coherence |
| v1.2 | 500K instances @ 0.15ms | Occlusion culling |
| v2.0 | 1M instances @ 0.10ms | Multi-GPU dispatch |

## Troubleshooting

### Common Issues

**GPU culling not activating:**
- Check feature flag: `cargo build --features gpu_culling`
- Verify GPU compute support in logs
- Instance count may be below threshold

**Performance below targets:**
- Monitor GPU/CPU method selection
- Check batch sizes for your GPU tier  
- Verify frustum plane calculation errors

**Memory usage growing:**
- Monitor batch count in diagnostics
- Check for instance leaks in cleanup
- Verify staging buffer reuse

### Debug Logging

```rust
// Enable detailed culling logs
RUST_LOG=amp_render::optimized_culling=debug cargo run
```

This provides comprehensive insight into method selection, timing, and performance characteristics for debugging production issues.
