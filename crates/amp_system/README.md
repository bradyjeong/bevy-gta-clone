# Amp System

[![Crates.io](https://img.shields.io/crates/v/amp_system.svg)](https://crates.io/crates/amp_system)
[![Documentation](https://docs.rs/amp_system/badge.svg)](https://docs.rs/amp_system)

Amp System is a facade crate providing physics and rendering systems for the Amp game engine. It integrates with Bevy to deliver high-performance, AAA-quality engine systems.

## Purpose

This facade crate serves as the system layer of the Amp ecosystem, providing:

- **Physics Systems**: Realistic physics simulation, vehicle dynamics, and collision detection
- **Rendering Pipeline**: Advanced rendering with GPU culling, LOD systems, and optimization
- **Engine Integration**: Complete Bevy plugin integration for core engine systems
- **Performance Focus**: Optimized systems targeting 60+ FPS gameplay

## Key Features

- ⚡ **High-Performance Physics**: Rapier3D integration with custom vehicle dynamics
- 🎨 **Advanced Rendering**: GPU culling, LOD systems, and batched rendering
- 🔧 **Engine Systems**: Core Bevy plugins for physics and rendering pipeline
- 📊 **Performance Diagnostics**: Built-in profiling and performance monitoring
- 🎮 **AAA Quality**: Professional-grade systems for commercial game development

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
amp_system = "0.1.0"
bevy = "0.16.1"
```

Basic usage:

```rust
use amp_system::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugin::default())
        .add_plugins(BatchingPlugin)
        .add_plugins(OptimizedCullingPlugin)
        .add_plugins(LodSystemPlugin)
        .run();
}
```

Advanced configuration:

```rust
use amp_system::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugin {
            gravity: Vec3::new(0.0, -19.62, 0.0), // Custom gravity
            ..default()
        })
        .add_plugins(SuspensionPlugin) // Vehicle suspension systems
        .add_plugins(BatchingPlugin)
        .add_plugins(CullingSystemPlugin)
        .add_plugins(PerformanceDiagnosticsPlugin)
        .run();
}
```

## Architecture

Amp System follows the facade pattern, providing a unified interface to:

- [`amp_physics`] - Physics simulation, vehicle dynamics, and collision detection
- [`amp_render`] - Rendering pipeline, GPU culling, and LOD systems

## Feature Flags

- `gpu_culling` - Enable GPU-based frustum culling for maximum performance
- `profile` - Enable detailed performance profiling and diagnostics
- `tracy` - Enable Tracy profiler integration for deep performance analysis
- `perf_trace` - Enable performance tracing and metrics collection
- `unstable_road_system` - Enable experimental road/surface system features
- `unstable_vegetation_lod` - Enable experimental vegetation LOD systems

## Systems Overview

### Physics Systems
- **Collision Detection**: Fast, accurate collision detection with Rapier3D
- **Vehicle Dynamics**: Realistic suspension, engine, and transmission physics
- **Interpolation**: Smooth visual interpolation for stable rendering
- **Debug Tools**: Visual physics debugging and diagnostic tools

### Rendering Systems
- **GPU Culling**: High-performance frustum and occlusion culling
- **LOD Management**: Automatic level-of-detail based on distance and performance
- **Batched Rendering**: Efficient rendering with automatic batching
- **Performance Monitoring**: Real-time performance metrics and diagnostics

## Performance Targets

Amp System is designed to meet AAA performance standards:

- **Target FPS**: 60+ FPS on desktop hardware
- **Culling Performance**: GPU culling ≤0.25ms per frame
- **Physics Step**: ≤2ms per physics update
- **Memory Efficiency**: Object pooling and minimal allocations

## Integration

For complete game development, combine with:

- [`amp_foundation`] - Core utilities and math (Bevy-free)
- [`amp_game`] - Gameplay systems and entity factories

## Examples

See the [examples directory](../../examples/) for complete system examples including:

- Physics simulation setup
- Vehicle dynamics demonstration
- Rendering pipeline configuration
- Performance optimization techniques

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
